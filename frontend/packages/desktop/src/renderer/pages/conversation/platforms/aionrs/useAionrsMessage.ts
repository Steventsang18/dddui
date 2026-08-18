/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

import { Message, Modal } from '@arco-design/web-react';
import { ipcBridge } from '@/common';
import { isErrorTipMessage, transformMessage } from '@/common/chat/chatLib';
import type { IResponseMessage } from '@/common/adapter/ipcBridge';
import type { TChatConversation, TokenUsageData, TurnUsageStats } from '@/common/config/storage';
import { uuid } from '@/common/utils';
import type { ThoughtData } from '@/renderer/components/chat/ThoughtDisplay';
import { useMergeLiveMessage } from '@/renderer/pages/conversation/Messages/hooks';
import { logStreamTerminalObserved } from '@/renderer/pages/conversation/runtime/useConversationRuntimeView';
import { getConversationOrNull } from '@/renderer/pages/conversation/utils/conversationCache';
import { isConversationProcessing } from '@/renderer/pages/conversation/utils/conversationRuntime';
import {
  beginConversationTurn,
  endConversationTurn,
  getConversationTurnStart,
} from '@/renderer/pages/conversation/utils/conversationTurnClock';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { processLocalCronResponse } from './localCronCommands';

type TokenUsage = {
  input_tokens?: number;
  output_tokens?: number;
  /** Wall-clock duration (ms) of the just-finished turn, from the backend Finish event. */
  elapsed_ms?: number;
};

export const useAionrsMessage = (
  conversation_id: string,
  options?: {
    onError?: (message: IResponseMessage) => void;
    onConfigChanged?: (capabilities: Record<string, unknown>) => void;
    /** 当前使用的模型名（供轨迹 model_call 上报） */
    model?: string;
  }
) => {
  const onError = options?.onError;
  const onConfigChanged = options?.onConfigChanged;
  const onConfigChangedRef = useRef(onConfigChanged);
  // Model 名经 ref 传给流订阅回调，避免订阅随模型切换反复重建。
  const modelRef = useRef(options?.model);
  useEffect(() => {
    modelRef.current = options?.model;
  }, [options?.model]);
  const mergeLiveMessage = useMergeLiveMessage();
  const [streamRunning, setStreamRunning] = useState(false);
  const [hasActiveTools, setHasActiveTools] = useState(false);
  const [waitingResponse, setWaitingResponse] = useState(false);
  const [hasHydratedRunningState, setHasHydratedRunningState] = useState(false);
  const [thought, setThought] = useState<ThoughtData>({
    description: '',
    subject: '',
  });
  const [tokenUsage, setTokenUsage] = useState<TokenUsageData | null>(null);
  // Live cumulative-token snapshot mirrored into a ref so the (once-subscribed)
  // stream listener can compute per-reply frozen stats without stale closures.
  const tokenUsageRef = useRef<TokenUsageData | null>(null);
  // msg_id of the assistant reply currently in flight. Only this message
  // renders live (ticking) metrics; historical replies use their own frozen
  // stats from `msgUsageStats`.
  const [activeMsgId, setActiveMsgIdState] = useState<string | null>(null);
  // Per-reply frozen usage snapshots: msg_id → stats. Persisted to
  // conversation.extra.usage_by_msg so a reload restores every historical
  // reply's own values instead of inheriting the latest turn's.
  const [msgUsageStats, setMsgUsageStats] = useState<Record<string, TurnUsageStats>>({});
  const msgUsageStatsRef = useRef<Record<string, TurnUsageStats>>({});
  const [context_limit, setContextLimit] = useState<number>(0);
  // Whether a context-compaction turn is in flight (drives the "压缩对话" button).
  const [compacting, setCompacting] = useState(false);
  const compactingRef = useRef(false);
  // Turn start origin for the elapsed indicator; backed by the module-level
  // conversation turn clock so it survives unmount on conversation switches.
  const [turnStartedAtMs, setTurnStartedAtMs] = useState<number | null>(null);
  // Conversation whose running state has been hydrated from the backend. Guards
  // the turn-clock cleanup below: a pre-hydration `running === false` (stale
  // state from the previous conversation) must not delete the persisted origin.
  const hydratedConversationRef = useRef<string | null>(null);
  // Current active message ID to filter out events from old requests (prevents aborted request events from interfering with new ones)
  const activeMsgIdRef = useRef<string | null>(null);
  const messageBufferRef = useRef(new Map<string, string>());
  const processedCronMsgIdsRef = useRef(new Set<string>());

  // Use refs to avoid useEffect re-subscription when these states change
  const hasActiveToolsRef = useRef(hasActiveTools);
  const streamRunningRef = useRef(streamRunning);
  const waitingResponseRef = useRef(waitingResponse);

  // Track whether current turn has content output
  // Only reset waitingResponse when finish arrives after content (not after tool calls)
  const hasContentInTurnRef = useRef(false);

  useEffect(() => {
    onConfigChangedRef.current = onConfigChanged;
  }, [onConfigChanged]);
  useEffect(() => {
    hasActiveToolsRef.current = hasActiveTools;
  }, [hasActiveTools]);
  useEffect(() => {
    streamRunningRef.current = streamRunning;
  }, [streamRunning]);

  // Throttle thought updates to reduce render frequency
  const thoughtThrottleRef = useRef<{
    lastUpdate: number;
    pending: ThoughtData | null;
    timer: ReturnType<typeof setTimeout> | null;
  }>({ lastUpdate: 0, pending: null, timer: null });

  const throttledSetThought = useMemo(() => {
    const THROTTLE_MS = 50; // 50ms throttle interval
    return (data: ThoughtData) => {
      const now = Date.now();
      const ref = thoughtThrottleRef.current;

      if (now - ref.lastUpdate >= THROTTLE_MS) {
        ref.lastUpdate = now;
        ref.pending = null;
        if (ref.timer) {
          clearTimeout(ref.timer);
          ref.timer = null;
        }
        setThought(data);
      } else {
        ref.pending = data;
        if (!ref.timer) {
          ref.timer = setTimeout(
            () => {
              ref.lastUpdate = Date.now();
              ref.timer = null;
              if (ref.pending) {
                setThought(ref.pending);
                ref.pending = null;
              }
            },
            THROTTLE_MS - (now - ref.lastUpdate)
          );
        }
      }
    };
  }, []);

  // Cleanup throttle timer
  useEffect(() => {
    return () => {
      if (thoughtThrottleRef.current.timer) {
        clearTimeout(thoughtThrottleRef.current.timer);
      }
    };
  }, []);

  // Combined running state: waiting for response OR stream is running OR tools are active
  const running = waitingResponse || streamRunning || hasActiveTools;

  // Keep the persisted turn origin in sync with the running state so the
  // elapsed indicator does not restart from zero after a conversation switch.
  useEffect(() => {
    if (running) {
      // begin keeps an already-recorded origin, so re-entering a conversation
      // mid-turn restores the original start time instead of resetting it.
      setTurnStartedAtMs(beginConversationTurn(conversation_id));
      return;
    }
    // Only drop the origin once hydration confirmed the idle state belongs to
    // THIS conversation; transient falses during a switch must keep it alive.
    if (hydratedConversationRef.current === conversation_id) {
      endConversationTurn(conversation_id);
    }
    // Keep the turn origin pinned after the reply converges so the elapsed
    // metric still shows the completed turn's duration. The conversation_id
    // effect below clears it when switching conversations.
  }, [running, conversation_id]);

  // Set current active message ID
  const setActiveMsgId = useCallback((msgId: string | null) => {
    activeMsgIdRef.current = msgId;
    // Also publish to state so MessageText can tell which single message is
    // the live one and render ticking metrics only for it.
    setActiveMsgIdState(msgId);
  }, []);

  const processCompletedAssistantMessage = useCallback(
    async (msgId: string) => {
      if (!msgId || processedCronMsgIdsRef.current.has(msgId)) {
        return;
      }

      const rawContent = messageBufferRef.current.get(msgId) ?? '';
      if (!rawContent.trim()) {
        return;
      }

      processedCronMsgIdsRef.current.add(msgId);

      try {
        const result = await processLocalCronResponse(conversation_id, rawContent);
        if (result.displayContent !== undefined && result.displayContent !== rawContent) {
          mergeLiveMessage({
            id: uuid(),
            msg_id: msgId,
            type: 'text',
            position: 'left',
            conversation_id,
            created_at: Date.now(),
            content: {
              content: result.displayContent,
              replace: true,
            },
          });
        }

        for (const response of result.systemResponses) {
          mergeLiveMessage(
            {
              id: uuid(),
              msg_id: `cron-local-${uuid()}`,
              type: 'tips',
              position: 'center',
              conversation_id,
              created_at: Date.now(),
              content: {
                content: response,
                type: response.startsWith('❌') ? 'error' : 'success',
              },
            },
            true
          );
        }
      } catch {
        processedCronMsgIdsRef.current.delete(msgId);
      }
    },
    [mergeLiveMessage, conversation_id]
  );

  useEffect(() => {
    return ipcBridge.conversation.responseStream.on((message) => {
      if (conversation_id !== message.conversation_id) {
        return;
      }

      if (isErrorTipMessage(message)) {
        setStreamRunning(false);
        streamRunningRef.current = false;
        setWaitingResponse(false);
        waitingResponseRef.current = false;
        setHasActiveTools(false);
        hasActiveToolsRef.current = false;
        setThought({ subject: '', description: '' });
        hasContentInTurnRef.current = false;
        const transformedMessage = transformMessage(message);
        if (transformedMessage) {
          mergeLiveMessage(transformedMessage);
        }
        return;
      }

      // Filter out events not belonging to current active request (prevents aborted events from interfering)
      // Note: only filter out thought and start messages, other messages must be rendered
      if (activeMsgIdRef.current && message.msg_id && message.msg_id !== activeMsgIdRef.current) {
        if (message.type === 'thought') {
          return;
        }
      }

      if ((message.type === 'content' || message.type === 'text') && message.msg_id) {
        const payload = message.data;
        const chunk =
          typeof payload === 'string'
            ? payload
            : typeof payload === 'object' &&
                payload !== null &&
                'content' in payload &&
                typeof (payload as { content?: unknown }).content === 'string'
              ? ((payload as { content: string }).content ?? '')
              : '';

        if (chunk) {
          const previous = messageBufferRef.current.get(message.msg_id) ?? '';
          messageBufferRef.current.set(message.msg_id, previous + chunk);
        }
      }

      switch (message.type) {
        case 'thought':
          // Auto-recover streamRunning if thought arrives after finish
          if (!streamRunningRef.current) {
            setStreamRunning(true);
            streamRunningRef.current = true;
          }
          throttledSetThought(message.data as ThoughtData);
          break;
        case 'start':
          setStreamRunning(true);
          streamRunningRef.current = true;
          // Don't reset waitingResponse here - let tool completion flow handle it
          break;
        case 'finish':
          {
            logStreamTerminalObserved(conversation_id, message.turn_id, 'aionrs', message.type);
            // aionrs stream_end carries usage in data field
            const usageData = message.data as TokenUsage | undefined;
            if (usageData && typeof usageData === 'object' && 'input_tokens' in usageData) {
              const input = usageData.input_tokens || 0;
              const output = usageData.output_tokens || 0;
              // Wall-clock of this turn. Prefer the backend-provided value;
              // when it is missing, derive it locally from the persisted turn
              // origin so the Clock metric never settles on 0.
              const elapsedMs =
                typeof usageData.elapsed_ms === 'number'
                  ? usageData.elapsed_ms
                  : Math.max(Date.now() - (getConversationTurnStart(conversation_id) ?? Date.now()), 0);
              const perTurnTotal = input + output;
              const cumulativeTotal = tokenUsageRef.current?.total_tokens ?? perTurnTotal;
              const nextUsage: TokenUsageData = {
                // `total_tokens` is the cumulative conversation count (the live
                // /usage snapshot below is authoritative); only fall back to this
                // turn's sum while that snapshot is still pending. `breakdown`
                // holds the per-reply 上行/下行 token counts.
                total_tokens: cumulativeTotal,
                breakdown: {
                  ...(tokenUsageRef.current?.breakdown ?? {}),
                  input_tokens: input,
                  output_tokens: output,
                },
                // Persist the just-finished turn's wall-clock duration so the
                // Clock metric stays visible on every historical reply.
                elapsed_ms: elapsedMs,
                // Keep the last known real context footprint; the /usage
                // snapshot below refreshes it with the just-finished turn's
                // value as soon as it arrives.
                context_used: tokenUsageRef.current?.context_used,
              };
              tokenUsageRef.current = nextUsage;
              setTokenUsage(nextUsage);
              // Freeze THIS reply's own stats (frozen total = cumulative at
              // finish). The full map is persisted so a reload restores every
              // historical reply's snapshot instead of inheriting the latest
              // turn's values.
              if (message.msg_id) {
                msgUsageStatsRef.current = {
                  ...msgUsageStatsRef.current,
                  [message.msg_id]: {
                    total_tokens: cumulativeTotal,
                    input_tokens: input,
                    output_tokens: output,
                    elapsed_ms: elapsedMs,
                  },
                };
                setMsgUsageStats(msgUsageStatsRef.current);
              }
              void ipcBridge.conversation.update.invoke({
                id: conversation_id,
                updates: {
                  extra: {
                    last_token_usage: {
                      total_tokens: cumulativeTotal,
                      breakdown: { input_tokens: input, output_tokens: output },
                      elapsed_ms: elapsedMs,
                    },
                    usage_by_msg: msgUsageStatsRef.current,
                  } as TChatConversation['extra'],
                },
                merge_extra: true,
              });
            }
            // Re-fetch the live /usage snapshot once a turn converges so the
            // cumulative "total tokens" metric stays current. A fresh conversation
            // reports 404 until the first turn activates the agent (the mount-time
            // hydrate can therefore miss it). Never shrink an already-known window.
            void ipcBridge.conversation.getUsage
              .invoke({ conversation_id })
              .then((usage) => {
                if (!usage || typeof usage.used !== 'number' || usage.used <= 0) return;
                setTokenUsage((prev) => {
                  const next: TokenUsageData = {
                    total_tokens: Math.max(usage.used, prev?.total_tokens ?? 0),
                    // Keep the frozen elapsed value: the previous code dropped
                    // `elapsed_ms` here, which is why the Clock metric could
                    // collapse to 0 right after a reply converged.
                    breakdown: prev?.breakdown,
                    elapsed_ms: prev?.elapsed_ms,
                    // Real context-window footprint. The context ring uses this
                    // (not the cumulative `total_tokens`) for its percentage, so
                    // it can never blow past the window size like "592.1%".
                    context_used: usage.used,
                  };
                  tokenUsageRef.current = next;
                  return next;
                });
                if (usage.size > 0) {
                  setContextLimit((prev) => (prev > 0 ? prev : usage.size));
                }
                // Write the true context footprint back into THIS reply's frozen
                // snapshot and persist it, so a historical reply's ring shows the
                // real percentage instead of the cumulative counter overflowing.
                const finishedMsgId = message.msg_id;
                const frozen = finishedMsgId ? msgUsageStatsRef.current[finishedMsgId] : undefined;
                if (frozen) {
                  msgUsageStatsRef.current = {
                    ...msgUsageStatsRef.current,
                    [finishedMsgId]: { ...frozen, context_used: usage.used },
                  };
                  setMsgUsageStats(msgUsageStatsRef.current);
                  const persisted = tokenUsageRef.current;
                  void ipcBridge.conversation.update.invoke({
                    id: conversation_id,
                    updates: {
                      extra: {
                        last_token_usage: persisted
                          ? {
                              total_tokens: persisted.total_tokens,
                              breakdown: persisted.breakdown,
                              elapsed_ms: persisted.elapsed_ms,
                              context_used: persisted.context_used,
                            }
                          : undefined,
                        usage_by_msg: msgUsageStatsRef.current,
                      } as TChatConversation['extra'],
                    },
                    merge_extra: true,
                  });
                }
              })
              .catch(() => {});
            // 上报 model_call 轨迹事件（方案 B：补齐 aionrs 路径，供轨迹页时间线展示）。
            // 同源 POST，Owner 模式下无需额外鉴权头。
            const traceModel = modelRef.current;
            if (traceModel) {
              const turnStart = getConversationTurnStart(conversation_id) ?? Date.now();
              const durationMs = Math.max(Date.now() - turnStart, 0);
              fetch(`/api/conversations/${conversation_id}/trace-event`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                  event_kind: 'model_call',
                  model: traceModel,
                  role: 'assistant',
                  input: { backend: 'aionrs' },
                  output: { ok: true },
                  token_usage: { duration_ms: durationMs },
                  status: 'finish',
                }),
                keepalive: true,
              }).catch((err) => {
                console.warn('[Trace] failed to record aionrs model_call:', err);
              });
            }
            setStreamRunning(false);
            setWaitingResponse(false);
            setThought({ subject: '', description: '' });
            if (message.msg_id) {
              void processCompletedAssistantMessage(message.msg_id);
            }
          }
          break;
        case 'tool_group':
          {
            // Mark that current turn has content output
            hasContentInTurnRef.current = true;

            // Auto-recover streamRunning if tool_group arrives after finish
            if (!streamRunningRef.current) {
              setStreamRunning(true);
              streamRunningRef.current = true;
            }

            // Check if any tools are executing or awaiting confirmation
            const tools = message.data as Array<{ status: string; name?: string }>;
            const activeStatuses = new Set(['Executing', 'Confirming', 'Pending']);
            const hasActive = tools.some((tool) => activeStatuses.has(tool.status));
            const wasActive = hasActiveToolsRef.current;

            setHasActiveTools(hasActive);
            hasActiveToolsRef.current = hasActive; // Sync update ref immediately

            // When tools transition from active to inactive, set waitingResponse=true
            // because backend needs to continue sending requests to model
            if (wasActive && !hasActive && tools.length > 0) {
              setWaitingResponse(true);
              waitingResponseRef.current = true;
            }

            // If tools are awaiting confirmation, update thought hint
            const confirmingTool = tools.find((tool) => tool.status === 'Confirming');
            if (confirmingTool) {
              setThought({
                subject: 'Awaiting Confirmation',
                description: confirmingTool.name || 'Tool execution',
              });
            } else if (hasActive) {
              const executingTool = tools.find((tool) => tool.status === 'Executing');
              if (executingTool) {
                setThought({
                  subject: 'Executing',
                  description: executingTool.name || 'Tool',
                });
              }
            } else if (!streamRunningRef.current) {
              // All tools completed and stream stopped, clear thought
              setThought({ subject: '', description: '' });
            }

            // Continue passing message to message list update
            mergeLiveMessage(transformMessage(message));
          }
          break;
        case 'permission':
        case 'acp_permission':
          if (!streamRunningRef.current) {
            setStreamRunning(true);
            streamRunningRef.current = true;
          }
          // Backend aionrs emits wire type 'acp_permission' but the payload is
          // Confirmation-shaped (legacy), which matches MessagePermission, not
          // MessageAcpPermission. Re-tag so transformMessage routes it correctly.
          mergeLiveMessage(transformMessage({ ...message, type: 'permission' }));
          break;
        case 'config_changed':
          onConfigChangedRef.current?.(message.data as Record<string, unknown>);
          break;
        default: {
          if (message.type === 'error') {
            logStreamTerminalObserved(conversation_id, message.turn_id, 'aionrs', message.type);
            setStreamRunning(false);
            streamRunningRef.current = false;
            setWaitingResponse(false);
            waitingResponseRef.current = false;
            setThought({ subject: '', description: '' });
            onError?.(message as IResponseMessage);
          } else {
            // Mark that current turn has content output (exclude error type)
            hasContentInTurnRef.current = true;
            // Reset waitingResponse when actual content arrives
            if (message.type === 'content') {
              setWaitingResponse(false);
              waitingResponseRef.current = false;
            }
            // Auto-recover streamRunning if content arrives after finish
            if (!streamRunningRef.current) {
              setStreamRunning(true);
              streamRunningRef.current = true;
            }
          }
          // Backend handles persistence, Frontend only updates UI
          mergeLiveMessage(transformMessage(message));
          break;
        }
      }
    });
    // Note: hasActiveTools and streamRunning are accessed via refs to avoid re-subscription
  }, [conversation_id, mergeLiveMessage, onError, processCompletedAssistantMessage]);

  useEffect(() => {
    let cancelled = false;

    setThought({ subject: '', description: '' });
    setTokenUsage(null);
    tokenUsageRef.current = null;
    setContextLimit(0);
    // Reset per-message frozen stats when switching conversations.
    setMsgUsageStats({});
    msgUsageStatsRef.current = {};
    // Reset the pinned elapsed origin when switching conversations so a stale
    // duration from the previous conversation is never shown.
    setTurnStartedAtMs(null);
    hasContentInTurnRef.current = false;
    setHasHydratedRunningState(false);

    // Check actual conversation status from backend before resetting all running states
    // to avoid flicker when switching to a running conversation
    void getConversationOrNull(conversation_id).then((res) => {
      if (cancelled) {
        return;
      }

      if (!res) {
        hydratedConversationRef.current = conversation_id;
        endConversationTurn(conversation_id);
        setStreamRunning(false);
        streamRunningRef.current = false;
        setHasActiveTools(false);
        hasActiveToolsRef.current = false;
        setWaitingResponse(false);
        waitingResponseRef.current = false;
        setHasHydratedRunningState(true);
        return;
      }
      const isRunning = isConversationProcessing(res);
      hydratedConversationRef.current = conversation_id;
      if (!isRunning) {
        // Turn ended while this conversation was in the background — drop the
        // stale origin so the next turn starts from its own send time. (The
        // sync effect above cannot cover this: running may already be false,
        // so it never re-runs after hydration.)
        endConversationTurn(conversation_id);
      }
      setStreamRunning(isRunning);
      streamRunningRef.current = isRunning;
      // Reset tool states - they will be restored by incoming messages if still active
      setHasActiveTools(false);
      hasActiveToolsRef.current = false;
      setWaitingResponse(isRunning);
      waitingResponseRef.current = isRunning;
      // Load persisted token usage stats
      if (res.type === 'aionrs' && res.extra?.last_token_usage) {
        const { last_token_usage } = res.extra;
        if (last_token_usage.total_tokens > 0) {
          tokenUsageRef.current = last_token_usage;
          setTokenUsage(last_token_usage);
        }
      }
      // Restore per-reply frozen stats so every historical reply keeps its
      // own elapsed/token snapshot instead of inheriting the latest turn's.
      if (res.type === 'aionrs' && res.extra?.usage_by_msg) {
        msgUsageStatsRef.current = res.extra.usage_by_msg;
        setMsgUsageStats(res.extra.usage_by_msg);
      }
      // Hydrate the context-usage indicator from the live backend snapshot
      // (the #14 consumption ring). A streaming finish event may set it first,
      // so never overwrite a value that is already present.
      void ipcBridge.conversation.getUsage
        .invoke({ conversation_id })
        .then((usage) => {
          if (cancelled || !usage || typeof usage.used !== 'number' || usage.used <= 0) return;
          setTokenUsage((prev) => {
            const next: TokenUsageData = {
              total_tokens: Math.max(usage.used, prev?.total_tokens ?? 0),
              breakdown: prev?.breakdown,
              elapsed_ms: prev?.elapsed_ms,
              context_used: usage.used,
            };
            tokenUsageRef.current = next;
            return next;
          });
          if (usage.size > 0) {
            setContextLimit((prev) => (prev > 0 ? prev : usage.size));
          }
        })
        .catch(() => {});
      setHasHydratedRunningState(true);
    });

    return () => {
      cancelled = true;
    };
  }, [conversation_id]);

  const resetState = useCallback(() => {
    setWaitingResponse(false);
    waitingResponseRef.current = false;
    setStreamRunning(false);
    streamRunningRef.current = false;
    setHasActiveTools(false);
    hasActiveToolsRef.current = false;
    setThought({ subject: '', description: '' });
    hasContentInTurnRef.current = false;
    // Clear active message ID to prevent filtering events from new messages after stop
    activeMsgIdRef.current = null;
    setActiveMsgIdState(null);
  }, []);

  // Compact the conversation context: confirm → POST /compact (hidden turn) →
  // refresh the live usage snapshot (context_used shrinks).
  const onCompact = useCallback(() => {
    if (compactingRef.current || streamRunningRef.current) return;
    Modal.confirm({
      title: '压缩对话',
      content: '将调用模型生成摘要并替换会话上下文（约 10–30s），期间建议不要发送新消息。此操作不可撤销。',
      okText: '确认压缩',
      cancelText: '取消',
      onOk: async () => {
        compactingRef.current = true;
        setCompacting(true);
        try {
          const res = await ipcBridge.conversation.compact.invoke({ conversation_id });
          if (res?.status !== 'completed') {
            throw new Error('compaction failed');
          }
          // Refresh the live context footprint after the hidden turn finished.
          void ipcBridge.conversation.getUsage
            .invoke({ conversation_id })
            .then((usage) => {
              if (!usage || typeof usage.used !== 'number' || usage.used <= 0) return;
              setTokenUsage((prev) => {
                const next: TokenUsageData = {
                  total_tokens: Math.max(usage.used, prev?.total_tokens ?? 0),
                  breakdown: prev?.breakdown,
                  elapsed_ms: prev?.elapsed_ms,
                  context_used: usage.used,
                };
                tokenUsageRef.current = next;
                return next;
              });
              if (usage.size > 0) {
                setContextLimit((prev) => (prev > 0 ? prev : usage.size));
              }
            })
            .catch(() => {});
          Message.success('上下文已压缩');
        } catch (err) {
          Message.error('压缩失败，请稍后重试');
        } finally {
          compactingRef.current = false;
          setCompacting(false);
        }
      },
    });
  }, [conversation_id]);

  return {
    thought,
    setThought,
    running,
    hasHydratedRunningState,
    turnStartedAtMs,
    tokenUsage,
    context_limit,
    activeMsgId,
    msgUsageStats,
    setActiveMsgId,
    setWaitingResponse,
    resetState,
    compacting,
    onCompact,
  };
};
