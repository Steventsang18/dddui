/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

import type {
  IConversationMcpStatus,
  TokenUsageData,
  TurnUsageStats,
} from '@/common/config/storage';
import React, { createContext, useContext } from 'react';

/**
 * Conversation context interface
 * 会话上下文接口
 */
export interface ConversationContextValue {
  /**
   * Conversation ID
   * 会话 ID
   */
  conversation_id: string;

  /**
   * Workspace directory path
   * 工作空间目录路径
   */
  workspace?: string;

  /**
   * Conversation type
   * 会话类型
   */
  type: 'acp' | 'codex' | 'aionrs';

  /**
   * Cron job ID (if this conversation was created by a scheduled task)
   */
  cron_job_id?: string;

  /**
   * When true, platform chat components should hide the SendBox (e.g. sub-agents in team mode)
   */
  hideSendBox?: boolean;

  /**
   * Loaded skill names for this conversation (snapshot from conversation.extra.skills).
   * Surfaced inside the SendBox `+` menu so users can review/jump to active skills.
   */
  loadedSkills?: string[];

  /**
   * Loaded MCP server names for this conversation (snapshot from
   * conversation.extra.mcp_servers).
   */
  loadedMcpServers?: string[];

  /**
   * Structured MCP status snapshot for this conversation (from
   * conversation.extra.mcp_statuses).
   */
  loadedMcpStatuses?: IConversationMcpStatus[];

  /**
   * Assistant id bound to this conversation snapshot, if any.
   */
  assistantId?: string;

  /**
   * Live session-level token usage (cumulative context window occupancy).
   * Surfaced under each assistant message so the user sees the current
   * context-water-level ring without leaving the transcript.
   */
  tokenUsage?: TokenUsageData | null;

  /**
   * Agent-reported context window size (denominator for the occupancy ring).
   * 0 / undefined when the agent does not report a window.
   */
  context_limit?: number;

  /**
   * Absolute start timestamp (ms epoch) of the in-flight turn, if any. Drives
   * the live "运行耗时" metric in the context ring. Null when no turn runs.
   */
  turnStartedAtMs?: number | null;

  /**
   * Whether a turn is currently streaming. True drives the live elapsed
   * ticker in the context ring.
   */
  running?: boolean;

  /**
   * msg_id of the assistant message whose turn is currently in flight. Only
   * that message renders live (ticking) metrics; historical replies fall back
   * to their own frozen `msgUsageStats` and stay untouched.
   */
  activeMsgId?: string | null;

  /**
   * Per-message frozen usage snapshots (msg_id → turn stats). Historical
   * replies render these instead of the live session state, so a completed
   * reply's elapsed/tokens never change when a new turn starts.
   */
  msgUsageStats?: Record<string, TurnUsageStats>;

  /**
   * Whether the platform supports context compaction (`/compact`) for this
   * conversation. When true, the context-ring popover offers a "压缩对话"
   * action (only on the latest assistant message).
   */
  canCompact?: boolean;

  /**
   * Whether a compaction turn is currently in flight. Drives the disabled /
   * spinner state of the "压缩对话" button.
   */
  compacting?: boolean;

  /**
   * Triggers context compaction for the conversation. Provided by the aionrs
   * platform chat component; resolves once the compaction turn completes.
   */
  onCompact?: () => Promise<void>;
}

/**
 * Conversation context
 * 会话上下文 - 提供会话级别的信息，如工作空间路径
 */
const ConversationContext = createContext<ConversationContextValue | null>(null);

/**
 * Conversation context provider
 * 会话上下文提供者
 */
export const ConversationProvider: React.FC<{
  children: React.ReactNode;
  value: ConversationContextValue;
}> = ({ children, value }) => {
  return <ConversationContext.Provider value={value}>{children}</ConversationContext.Provider>;
};

/**
 * Hook to use conversation context
 * 使用会话上下文的 Hook
 */
export const useConversationContext = (): ConversationContextValue => {
  const context = useContext(ConversationContext);
  if (!context) {
    throw new Error('useConversationContext must be used within ConversationProvider');
  }
  return context;
};

/**
 * Hook to safely use conversation context (returns null if not in provider)
 * 安全使用会话上下文的 Hook（如果不在 provider 中则返回 null）
 */
export const useConversationContextSafe = (): ConversationContextValue | null => {
  return useContext(ConversationContext);
};
