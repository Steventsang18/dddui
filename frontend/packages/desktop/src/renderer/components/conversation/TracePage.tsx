/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 *
 * 对话轨迹完整全页面（/conversation/:id/trace）。
 * GitHub contributions 式迷你矩阵：色相定类型（Input 蓝绿 / Model 紫 / Tools 琥珀），
 * 同色相深浅分级表示列内密度；方块与下方事件列表双向联动。
 * 后端：GET /api/conversations/{id}/trace （event_kind / model / from_ts / to_ts 过滤）
 */

import { ArrowLeft, History, Down, Right } from '@icon-park/react';
import { Button, Input, Select } from '@arco-design/web-react';
import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { getBaseUrl } from '@/common/adapter/httpBridge';
import './TracePage.css';

type TraceEvent = {
  id: string;
  turn_seq: number;
  event_kind: 'text' | 'thinking' | 'tool_call' | 'model_call' | string;
  role?: string | null;
  model?: string | null;
  input: Record<string, unknown>;
  output: Record<string, unknown>;
  token_usage: Record<string, unknown>;
  status?: string | null;
  created_at: number;
};

type TraceKind = '' | 'model_call' | 'tool_call' | 'text' | 'thinking';

/** 后端聚合出的「轮次步骤」时间线节点 */
type TraceStep = {
  kind: 'answer' | 'tool_call' | 'model_call' | 'thinking';
  time: number;
  ids: string[];
  content?: string;
  parts?: number;
  name?: string;
  cn?: string;
  icon?: string;
  args?: Record<string, unknown>;
  output?: string;
  status?: string;
  status_text?: string;
  status_cls?: string;
  summary?: string;
  model?: string;
  duration_ms?: number;
  prompt?: string;
  raw_input?: Record<string, unknown>;
  raw_output?: Record<string, unknown>;
};

/** 一轮对话：以用户提问为锚 */
type TraceTurn = {
  turn: number;
  time: number;
  question?: string | null;
  question_id?: string | null;
  steps: TraceStep[];
};

/** 矩阵行：把 4 种 event_kind 归并到 3 行（thinking 归入 Model 行） */
type RowKey = 'text' | 'model' | 'tool';
const ROW_META: Record<RowKey, { label: string; dot: string; badge: string }> = {
  text: { label: 'Input', dot: '#63b2ab', badge: 'TEXT' },
  model: { label: 'Model', dot: '#a08fdd', badge: 'MODEL' },
  tool: { label: 'Tools', dot: '#e0ae6e', badge: 'TOOL' },
};
const rowOf = (kind: string): RowKey =>
  kind === 'tool_call' ? 'tool' : kind === 'text' ? 'text' : 'model';

/** C 样式（毛玻璃卡）顶部色条渐变的次色：三色循环 */
const TIP_GRAD: Record<RowKey, string> = { text: '#a08fdd', model: '#e0ae6e', tool: '#a08fdd' };

const KIND_OPTIONS: { label: string; value: TraceKind }[] = [
  { label: '全部', value: '' },
  { label: '模型', value: 'model_call' },
  { label: '工具', value: 'tool_call' },
  { label: '文本', value: 'text' },
  { label: '思考', value: 'thinking' },
];

function maskSecrets(value: unknown): unknown {
  if (typeof value === 'string') {
    const t = value.trim();
    if (/^(sk-|sk_|ghp_|github_pat_|xoxb-|xoxp-|AKIA)/.test(t)) return t.slice(0, 3) + '***';
    return value;
  }
  if (Array.isArray(value)) return value.map(maskSecrets);
  if (value && typeof value === 'object') {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
      if (/password|passwd|pwd|secret|token|authorization|cookie|credential|api_?key|access_key|private_key/i.test(k)) {
        out[k] = '***';
      } else {
        out[k] = maskSecrets(v);
      }
    }
    return out;
  }
  return value;
}

const fmtTime = (ms: number) => {
  const d = new Date(ms);
  const p = (n: number) => String(n).padStart(2, '0');
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
};
const fmtDate = (ms: number) => {
  const d = new Date(ms);
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
};
const fmtTick = (ms: number) => {
  const d = new Date(ms);
  const p = (n: number) => String(n).padStart(2, '0');
  return `${p(d.getHours())}:${p(d.getMinutes())}`;
};
const fmtFull = (ms: number) => {
  const d = new Date(ms);
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
};

function summarizeJson(obj: Record<string, unknown>): string {
  const keys = ['description', 'content', 'command', 'name', 'prompt', 'query', 'message', 'text'];
  for (const k of keys) {
    const v = obj[k];
    if (v == null) continue;
    if (typeof v === 'string') return v.slice(0, 200);
    if (typeof v === 'object') {
      const s = JSON.stringify(v).slice(0, 200);
      return s.length > 200 ? s.slice(0, 200) + '…' : s;
    }
  }
  const s = JSON.stringify(obj).slice(0, 200);
  return s.length > 200 ? s.slice(0, 200) + '…' : s;
}

const TracePage: React.FC = () => {
  const { id: conversationId } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { t } = useTranslation();

  const [events, setEvents] = useState<TraceEvent[]>([]);
  const [timeline, setTimeline] = useState<TraceTurn[]>([]);
  const [loading, setLoading] = useState(false);
  const [kind, setKind] = useState<TraceKind>('');
  const [model, setModel] = useState('');
  const [from, setFrom] = useState('');
  const [to, setTo] = useState('');
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [hover, setHover] = useState<{ row: RowKey; time: string; model: string; summary: string; x: number; y: number } | null>(null);
  const abortRef = useRef<AbortController | null>(null);
  const listRef = useRef<HTMLDivElement | null>(null);

  const load = useCallback(async () => {
    if (!conversationId) return;
    abortRef.current?.abort();
    const ctrl = new AbortController();
    abortRef.current = ctrl;
    setLoading(true);
    setSelectedId(null);
    try {
      const params = new URLSearchParams();
      if (kind) params.set('event_kind', kind);
      if (model.trim()) params.set('model', model.trim());
      if (from) params.set('from_ts', String(new Date(from).getTime()));
      if (to) params.set('to_ts', String(new Date(to).getTime()));
      const url = `${getBaseUrl()}/api/conversations/${conversationId}/trace${params.toString() ? `?${params}` : ''}`;
      const resp = await fetch(url, { signal: ctrl.signal });
      if (!resp.ok) {
        setEvents([]);
        setTimeline([]);
        return;
      }
      const json = (await resp.json()) as { success: boolean; data?: TraceEvent[]; timeline?: TraceTurn[] };
      setEvents(json.data ?? []);
      setTimeline(json.timeline ?? []);
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  }, [conversationId, kind, model, from, to]);

  useEffect(() => {
    void load();
    return () => abortRef.current?.abort();
  }, [load]);

  const resetFilters = () => {
    setKind('');
    setModel('');
    setFrom('');
    setTo('');
  };

  const back = () => {
    if (conversationId) navigate(`/conversation/${conversationId}`);
    else navigate(-1);
  };

  /** 导出当前过滤条件下的轨迹为 Markdown / JSON 文件（后端生成，前端触发下载） */
  const exportTrace = useCallback(
    async (fmt: 'markdown' | 'json') => {
      if (!conversationId) return;
      const params = new URLSearchParams();
      if (kind) params.set('event_kind', kind);
      if (model.trim()) params.set('model', model.trim());
      if (from) params.set('from_ts', String(new Date(from).getTime()));
      if (to) params.set('to_ts', String(new Date(to).getTime()));
      params.set('fmt', fmt);
      const url = `${getBaseUrl()}/api/conversations/${conversationId}/trace/export${params.toString() ? `?${params}` : ''}`;
      const resp = await fetch(url);
      if (!resp.ok) return;
      const blob = await resp.blob();
      const disp = resp.headers.get('content-disposition') ?? '';
      const m = disp.match(/filename="?([^"]+)"?/);
      const fname = m?.[1] ?? `trace-${conversationId}.${fmt === 'json' ? 'json' : 'md'}`;
      const link = document.createElement('a');
      link.href = URL.createObjectURL(blob);
      link.download = fname;
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(link.href);
    },
    [conversationId, kind, model, from, to],
  );

  const toggleExpand = (id: string) => {
    setExpandedIds((prev) => {
      const n = new Set(prev);
      if (n.has(id)) n.delete(id);
      else n.add(id);
      return n;
    });
  };

  /** 双向联动：选中事件，高亮色块 + 滚动列表到可见（data-ids 覆盖被合并进同一步骤的事件） */
  const selectEvent = (id: string) => {
    setSelectedId(id);
    requestAnimationFrame(() => {
      const item = listRef.current?.querySelector<HTMLElement>(
        `.trace-event-item[data-id="${id}"], .trace-event-item[data-ids*="${id}"]`
      );
      item?.scrollIntoView({ behavior: 'smooth', block: 'center' });
      const cell = document.querySelector<HTMLElement>(`.trace-matrix__cell[data-id="${id}"]`);
      cell?.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' });
    });
  };

  /** 时间桶：把事件按时间切列，返回列索引 → 事件 id 列表（每行独立） */
  const EMPTY_MAP: Record<RowKey, string[][]> = { text: [], model: [], tool: [] };
  const buckets = useMemo(() => {
    if (events.length === 0) return { start: 0, bucketSec: 1, nCols: 0, map: EMPTY_MAP };
    let min = Infinity;
    let max = -Infinity;
    for (const e of events) {
      if (e.created_at < min) min = e.created_at;
      if (e.created_at > max) max = e.created_at;
    }
    const range = Math.max(max - min, 1);
    const nCols = Math.min(90, Math.max(30, Math.ceil(events.length / 2)));
    const bucketSec = Math.max(Math.ceil(range / 1000 / nCols), 1);
    const map: Record<RowKey, string[][]> = { text: [], model: [], tool: [] };
    for (const k of ['text', 'model', 'tool'] as RowKey[]) {
      map[k] = Array.from({ length: nCols }, () => [] as string[]);
    }
    for (const e of events) {
      const row = rowOf(e.event_kind);
      let col = Math.floor((e.created_at - min) / 1000 / bucketSec);
      if (col < 0) col = 0;
      if (col >= nCols) col = nCols - 1;
      map[row][col].push(e.id);
    }
    return { start: min, bucketSec, nCols, map };
  }, [events]);

  const byId = useMemo(() => {
    const m = new Map<string, TraceEvent>();
    for (const e of events) m.set(e.id, e);
    return m;
  }, [events]);

  /** 本会话使用过的模型列表（去重），用于模型筛选下拉 + 模型标签云 */
  const modelOptions = useMemo(() => {
    const set = new Set<string>();
    for (const e of events) if (e.model) set.add(e.model);
    return Array.from(set).map((m) => ({ label: m, value: m }));
  }, [events]);

  const rows: RowKey[] = ['text', 'model', 'tool'];

  /** 渲染一条时间线步骤：人性化卡片（回答/工具/模型/思考），可折叠原始 JSON */
  const renderStep = (step: TraceStep, idx: number) => {
    const key = step.ids[0] ?? `${step.kind}-${step.time}-${idx}`;
    const isExpanded = expandedIds.has(key);
    const active = selectedId === key;
    const timeText = fmtTime(step.time);
    const dataAttrs = { 'data-id': step.ids[0] ?? '', 'data-ids': step.ids.join(' ') };
    const titleClick = (ev: React.MouseEvent) => {
      ev.stopPropagation();
      toggleExpand(key);
    };
    const stepClick = () => selectEvent(key);
    const chevron = (
      <span className='trace-step__expandicon'>
        {isExpanded ? <Down theme='outline' size={12} /> : <Right theme='outline' size={12} />}
      </span>
    );
    const maskedRaw = (v?: Record<string, unknown>) =>
      v ? (maskSecrets(v) as Record<string, unknown>) : {};
    const jsonOf = (v: Record<string, unknown>) => JSON.stringify(v, null, 2);

    switch (step.kind) {
      case 'answer':
        return (
          <div key={key} className={`trace-event-item trace-step trace-step--answer ${active ? 'active' : ''}`} {...dataAttrs} onClick={stepClick}>
            <span className='trace-step__icon'>💬</span>
            <div className='trace-step__body'>
              <div className='trace-step__title' onClick={titleClick}>
                {chevron}
                <span>回答</span>
                {step.parts != null && step.parts > 1 && <span className='trace-step__hint'>共 {step.parts} 段</span>}
              </div>
              <div className={`trace-step__content ${isExpanded ? '' : 'trace-step__content--clamp'}`}>{step.content}</div>
            </div>
            <span className='trace-step__meta'>{timeText}</span>
          </div>
        );
      case 'tool_call': {
        const stCls = step.status_cls ?? 'run';
        const stText =
          step.status_text ?? (stCls === 'done' ? '完成' : stCls === 'err' ? '出错' : '进行中');
        const argsJson = jsonOf(maskedRaw(step.args));
        const inJson = jsonOf(maskedRaw(step.raw_input));
        const outJson = jsonOf(maskedRaw(step.raw_output));
        const hasDetail = argsJson !== '{}' || inJson !== '{}' || outJson !== '{}';
        return (
          <div key={key} className={`trace-event-item trace-step trace-step--tool ${active ? 'active' : ''}`} {...dataAttrs} onClick={stepClick}>
            <span className='trace-step__icon'>{step.icon ?? '🔧'}</span>
            <div className='trace-step__body'>
              <div className='trace-step__title' onClick={titleClick}>
                {chevron}
                <span>{step.cn ?? '调用工具'}</span>
                {step.name && <code className='trace-step__code'>{step.name}</code>}
                <span className={`trace-step__status st-${stCls}`}>{stText}</span>
              </div>
              {step.summary && <div className='trace-step__summary'>{step.summary}</div>}
              {step.output && <div className='trace-step__output'>{step.output}</div>}
              {isExpanded && hasDetail && (
                <div className='trace-row__detail'>
                  {argsJson !== '{}' && (
                    <div className='trace-io-block'>
                      <div className='trace-io-label'>ARGS</div>
                      <pre className='trace-io-pre'>{argsJson}</pre>
                    </div>
                  )}
                  {inJson !== '{}' && (
                    <div className='trace-io-block'>
                      <div className='trace-io-label'>INPUT</div>
                      <pre className='trace-io-pre'>{inJson}</pre>
                    </div>
                  )}
                  {outJson !== '{}' && (
                    <div className='trace-io-block'>
                      <div className='trace-io-label'>OUTPUT</div>
                      <pre className='trace-io-pre'>{outJson}</pre>
                    </div>
                  )}
                </div>
              )}
            </div>
            <span className='trace-step__meta'>{timeText}</span>
          </div>
        );
      }
      case 'model_call': {
        const inJson = jsonOf(maskedRaw(step.raw_input));
        const outJson = jsonOf(maskedRaw(step.raw_output));
        const hasDetail = inJson !== '{}' || outJson !== '{}';
        return (
          <div key={key} className={`trace-event-item trace-step trace-step--model ${active ? 'active' : ''}`} {...dataAttrs} onClick={stepClick}>
            <span className='trace-step__icon'>🤖</span>
            <div className='trace-step__body'>
              <div className='trace-step__title' onClick={titleClick}>
                {chevron}
                <span>调用模型</span>
                {step.model && <code className='trace-step__code'>{step.model}</code>}
              </div>
              {step.summary && <div className='trace-step__summary'>{step.summary}</div>}
              {isExpanded && hasDetail && (
                <div className='trace-row__detail'>
                  {inJson !== '{}' && (
                    <div className='trace-io-block'>
                      <div className='trace-io-label'>INPUT</div>
                      <pre className='trace-io-pre'>{inJson}</pre>
                    </div>
                  )}
                  {outJson !== '{}' && (
                    <div className='trace-io-block'>
                      <div className='trace-io-label'>OUTPUT</div>
                      <pre className='trace-io-pre'>{outJson}</pre>
                    </div>
                  )}
                </div>
              )}
            </div>
            <span className='trace-step__meta'>
              {step.duration_ms != null && step.duration_ms > 0 ? `⏱ ${step.duration_ms}ms · ` : ''}
              {timeText}
            </span>
          </div>
        );
      }
      case 'thinking':
        return (
          <div key={key} className={`trace-event-item trace-step trace-step--think ${active ? 'active' : ''}`} {...dataAttrs} onClick={stepClick}>
            <span className='trace-step__icon'>🧠</span>
            <div className='trace-step__body'>
              <div className='trace-step__title' onClick={titleClick}>
                {chevron}
                <span>思考过程</span>
              </div>
              {step.content && (
                <div className={`trace-step__content ${isExpanded ? '' : 'trace-step__content--clamp'}`}>{step.content}</div>
              )}
            </div>
            <span className='trace-step__meta'>{timeText}</span>
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className='trace-page'>
      <header className='trace-page__bar'>
        <button type='button' className='trace-page__back' onClick={back} aria-label={t('common.back')}>
          <ArrowLeft theme='outline' size={18} />
          <span>{t('common.back', { defaultValue: 'Back' })}</span>
        </button>
        <div className='trace-page__title'>
          <History theme='outline' size={18} />
          <span>{t('conversation.trace.title')}</span>
        </div>
        <div className='trace-page__count'>
          {loading ? t('conversation.trace.loading') : `${events.length} events`}
        </div>
      </header>

      <div className='trace-page__filters'>
        <Select
          size='small'
          style={{ width: 120 }}
          value={kind}
          onChange={(v) => setKind(v as TraceKind)}
          options={KIND_OPTIONS}
        />
        <Select
          size='small'
          style={{ width: 180 }}
          placeholder={t('conversation.trace.filterModel')}
          value={model || undefined}
          onChange={(v) => setModel(v ?? '')}
          options={modelOptions}
          allowClear
          notFoundContent={t('conversation.trace.empty')}
        />
        <Input size='small' style={{ width: 170 }} type='datetime-local' value={from} onChange={setFrom} />
        <Input size='small' style={{ width: 170 }} type='datetime-local' value={to} onChange={setTo} />
        <Button size='small' onClick={resetFilters}>
          {t('conversation.trace.clear')}
        </Button>
        <Button size='small' type='primary' onClick={() => void load()}>
          {t('common.confirm', { defaultValue: 'Apply' })}
        </Button>
        <span style={{ flex: 1 }} />
        <Button size='small' onClick={() => void exportTrace('markdown')}>
          {t('conversation.trace.exportMarkdown', { defaultValue: '导出 MD' })}
        </Button>
        <Button size='small' onClick={() => void exportTrace('json')}>
          {t('conversation.trace.exportJson', { defaultValue: '导出 JSON' })}
        </Button>
      </div>

      {/* GitHub 式迷你矩阵 */}
      <div className='trace-matrix-card'>
        {/* 模型标签云：点击即按该模型筛选 */}
        {modelOptions.length > 0 && (
          <div className='trace-matrix__models'>
            <span className='trace-matrix__models-label'>模型：</span>
            {modelOptions.map((opt) => (
              <button
                key={opt.value}
                type='button'
                className={`trace-model-chip ${model === opt.value ? 'trace-model-chip--active' : ''}`}
                onClick={() => setModel(model === opt.value ? '' : opt.value)}
              >
                {opt.value}
              </button>
            ))}
          </div>
        )}
        <div className='trace-matrix-scroll'>
          <div className='trace-matrix__grid'>
            {buckets.nCols > 0 &&
              rows.map((rk) => {
                const meta = ROW_META[rk];
                const rowCols = buckets.map[rk] ?? [];
                const count = rowCols.reduce((a, b) => a + b.length, 0);
                return (
                  <div className='trace-matrix__row' key={rk}>
                    <div className='trace-matrix__label'>
                      <span className='trace-matrix__dot' style={{ background: meta.dot }} />
                      {meta.label}
                      <span className='trace-matrix__count'>{count}</span>
                    </div>
                    <div className='trace-matrix__track'>
                      {rowCols.map((ids, c) => {
                        const count = ids.length;
                        const firstId = count > 0 ? ids[0] : undefined;
                        const e = firstId ? byId.get(firstId) : undefined;
                        const dim = !!(model && e && e.model !== model);
                        const summary = e ? summarizeJson(maskSecrets(e.input) as Record<string, unknown>) : '';
                        const lvl = count >= 4 ? 4 : count; // 深浅 1..4 表示该桶内事件密度
                        return (
                          <div className='trace-matrix__col' key={c}>
                            {count === 0 || !e ? (
                              <div className='trace-matrix__cell empty' />
                            ) : (
                              <div
                                className={`trace-matrix__cell t-${rk} l${lvl} ${selectedId === firstId ? 'hit' : ''} ${dim ? 'is-dim' : ''}`}
                                data-id={firstId}
                                data-model={e.model ?? ''}
                                onMouseEnter={(ev) => setHover({ row: rk, time: fmtFull(e.created_at), model: e.model ?? '', summary, x: ev.clientX, y: ev.clientY })}
                                onMouseMove={(ev) => setHover((h) => (h ? { ...h, x: ev.clientX, y: ev.clientY } : h))}
                                onMouseLeave={() => setHover(null)}
                                onClick={() => selectEvent(firstId)}
                              />
                            )}
                          </div>
                        );
                      })}
                    </div>
                  </div>
                );
              })}
          </div>
        </div>
        {hover &&
          (() => {
            const meta = ROW_META[hover.row];
            return (
              <div
                className='trace-matrix__celltip'
                style={{
                  left: hover.x,
                  top: hover.y,
                  transform:
                    hover.x > (typeof window !== 'undefined' ? window.innerWidth : 0) - 300
                      ? 'translate(calc(-100% - 14px), 16px)'
                      : 'translate(14px, 16px)',
                  ['--tc' as string]: meta.dot,
                  ['--tc2' as string]: TIP_GRAD[hover.row],
                }}
              >
                <span className='trace-matrix__tip-kind'>
                  <i style={{ background: meta.dot }} />
                  {meta.badge}
                </span>
                <span className='trace-matrix__tip-time'>{hover.time}</span>
                {hover.model && <span className='trace-matrix__tip-model'>· {hover.model}</span>}
                {hover.summary && <span className='trace-matrix__tip-sum'>{hover.summary}</span>}
              </div>
            );
          })()}
      </div>

      {/* 人性化时间线（方案 A；矩阵区块保持原样） */}
      <div className='trace-list-head'>
        <h3>{t('conversation.trace.title')} · 时间线</h3>
        <span className='trace-list-hint'>点击可展开详情 · 与上方色块双向定位</span>
      </div>
      <div className='trace-page__body' ref={listRef}>
        {loading && <div className='trace-empty'>{t('conversation.trace.loading')}</div>}
        {!loading && timeline.length === 0 && events.length === 0 && (
          <div className='trace-empty'>{t('conversation.trace.empty')}</div>
        )}
        {!loading && timeline.length === 0 && events.length > 0 && (
          <div className='trace-empty'>时间线暂不可用（服务端版本过旧），请升级后端后重试</div>
        )}

        {!loading &&
          timeline.map((turn) => (
            <div key={turn.turn} className='trace-turn'>
              <div className='trace-turn__head'>
                <span className='trace-turn__num'>第 {turn.turn} 轮</span>
                <span className='trace-turn__time'>{fmtTime(turn.time)}</span>
                {turn.question && <span className='trace-turn__q' title={turn.question}>{turn.question}</span>}
              </div>
              <div className='trace-turn__steps'>{turn.steps.map((step, idx) => renderStep(step, idx))}</div>
            </div>
          ))}
      </div>
    </div>
  );
};

export default TracePage;
