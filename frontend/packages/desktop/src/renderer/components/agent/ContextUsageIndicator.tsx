/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

import { Popover } from '@arco-design/web-react';
import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import type { TokenUsageCost, TokenUsageData } from '@/common/config/storage';

interface ContextUsageIndicatorProps {
  tokenUsage: TokenUsageData | null;
  /**
   * Agent-reported context window size. Without it (<= 0) the popover shows the
   * raw token count instead of a percentage — never a percentage against a
   * guessed denominator.
   */
  context_limit: number;
  /**
   * Absolute start timestamp (ms epoch) of the in-flight turn, if any. When
   * set, the elapsed-time metric is shown and ticks live while `running`.
   */
  turnStartedAtMs?: number | null;
  /**
   * Whether a turn is currently streaming. Drives the live elapsed ticker.
   */
  running?: boolean;
  className?: string;
  /**
   * Whether the "压缩对话" action is available on this message (only the latest
   * assistant message of a compaction-capable platform shows it).
   */
  showCompact?: boolean;
  /**
   * Whether a compaction turn is currently in flight. Disables the button and
   * shows a spinner.
   */
  compacting?: boolean;
  /**
   * Triggers context compaction. Provided by the platform chat component.
   */
  onCompact?: () => void;
}

/**
 * 方案A：低饱和柔和配色 —— 安静、不喧宾夺主，深浅主题下均清晰可读。
 * 圆环（上下文状态）与四个指标各占一个独立低饱和色。
 */
const METRIC_COLORS = {
  ring: '#7c8db5', // 灰蓝 — 上下文状态圈
  tokens: '#7c8db5', // 灰蓝 — 总 token
  upload: '#6fa8a0', // 灰青 — 上行 (input) tokens
  download: '#9b8bb8', // 灰紫 — 下行 (output) tokens
  elapsed: '#c49a6c', // 灰棕 — 当轮耗时
} as const;

/* ── 上下文占比 → 颜色渐变（三色锚点线性插值） ── */

const RING_COLOR_STOPS = [
  { pct: 0, hex: '#7c8db5' }, // 充裕 — 蓝灰
  { pct: 65, hex: '#c49a6c' }, // 偏满 — 琥珀
  { pct: 85, hex: '#d96b6b' }, // 告急 — 红
] as const;

/* ── 方案B 仪表卡：状态三档 ── */

const STATUS_LEVELS = [
  { max: 65, label: '空间充足', color: '#7c8db5' },
  { max: 85, label: '即将告急', color: '#c49a6c' },
  { max: 101, label: '上下文已满', color: '#d96b6b' },
] as const;

function statusOf(pct: number): { label: string; color: string } {
  const p = Math.max(0, Math.min(100, pct));
  return STATUS_LEVELS.find((s) => p < s.max) ?? STATUS_LEVELS[STATUS_LEVELS.length - 1];
}

function hexToRgb(hex: string): [number, number, number] {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) & 0xff, (n >> 8) & 0xff, n & 0xff];
}

function lerp(a: number, b: number, t: number): number {
  return Math.round(a + (b - a) * t);
}

/** 按上下文占比在三色锚点间线性插值，返回圆环填充色。 */
function ringColor(pct: number): string {
  const p = Math.max(0, Math.min(100, pct));
  const from = p < RING_COLOR_STOPS[1].pct ? RING_COLOR_STOPS[0] : RING_COLOR_STOPS[1];
  const to = p < RING_COLOR_STOPS[1].pct ? RING_COLOR_STOPS[1] : RING_COLOR_STOPS[2];
  const range = to.pct - from.pct;
  const t = p < RING_COLOR_STOPS[1].pct ? p / range : Math.min(1, (p - from.pct) / range);
  const a = hexToRgb(from.hex);
  const b = hexToRgb(to.hex);
  return `rgb(${lerp(a[0], b[0], t)}, ${lerp(a[1], b[1], t)}, ${lerp(a[2], b[2], t)})`;
}

/* ── 14px stroke icons (currentColor) ── */

const iconBase = {
  width: 14,
  height: 14,
  viewBox: '0 0 16 16',
  fill: 'none',
  stroke: 'currentColor',
  strokeWidth: 1.5,
  strokeLinecap: 'round' as const,
  strokeLinejoin: 'round' as const,
};

/** 总 token 数量 — hash / counter glyph. */
const TokensIcon: React.FC = () => (
  <svg {...iconBase} aria-hidden='true'>
    <path d='M6 2.5v11M10 2.5v11M3.5 6h9M3.5 10h9' />
  </svg>
);

/** 上行数据量 — up arrow. */
const UploadIcon: React.FC = () => (
  <svg {...iconBase} aria-hidden='true'>
    <path d='M8 13.5v-10M4.5 7 8 3.5 11.5 7' />
  </svg>
);

/** 下行数据量 — down arrow. */
const DownloadIcon: React.FC = () => (
  <svg {...iconBase} aria-hidden='true'>
    <path d='M8 2.5v10M4.5 9l3.5 3.5L11.5 9' />
  </svg>
);

/** 运行耗时 — clock. */
const ClockIcon: React.FC = () => (
  <svg {...iconBase} aria-hidden='true'>
    <circle cx='8' cy='8' r='5.5' />
    <path d='M8 5.5V8l2 1.5' />
  </svg>
);

/**
 * 上下文状态圈 —— 用 SVG 圆环图形化展示「已用 token / 上下文窗口」占比。
 * 五个元素中唯一用图形（而非数字）呈现的一项：外圈为轨道、内圈按占比填充。
 */
const ContextRing: React.FC<{ percentage: number; hasWindow: boolean }> = ({ percentage, hasWindow }) => {
  const size = 14;
  const strokeWidth = 2;
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const clamped = hasWindow ? Math.max(0, Math.min(100, percentage)) : 0;
  const dashOffset = circumference * (1 - clamped / 100);
  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden='true' className='shrink-0'>
      {/* 轨道 */}
      <circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill='none'
        stroke='var(--color-border-2)'
        strokeWidth={strokeWidth}
      />
      {/* 已用占比（仅在有窗口信息时绘制填充弧） */}
      {hasWindow && (
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill='none'
          stroke={ringColor(clamped)}
          strokeWidth={strokeWidth}
          strokeLinecap='round'
          strokeDasharray={circumference}
          strokeDashoffset={dashOffset}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
        />
      )}
    </svg>
  );
};

const ContextUsageIndicator: React.FC<ContextUsageIndicatorProps> = ({
  tokenUsage,
  context_limit,
  turnStartedAtMs = null,
  running = false,
  className = '',
  showCompact = false,
  compacting = false,
  onCompact,
}) => {
  const { t } = useTranslation();

  const hasWindow = context_limit > 0;

  // Live clock for the turn elapsed metric. It ticks every 500ms while a turn
  // is streaming so the "运行耗时" value stays current without re-rendering
  // the whole transcript.
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!running || turnStartedAtMs == null) {
      return;
    }
    setNow(Date.now());
    const id = window.setInterval(() => setNow(Date.now()), 500);
    return () => window.clearInterval(id);
  }, [running, turnStartedAtMs]);

  if (!tokenUsage) {
    return null;
  }

  const total = tokenUsage.total_tokens;
  const breakdown = tokenUsage.breakdown;

  // Context-ring numerator: prefer the real context-window footprint
  // (`context_used`, from the live /usage snapshot). `total_tokens` is a
  // cumulative session counter that grows every turn, so using it here made
  // the ring blow past 100% (e.g. "上下文达到592.1%") on long conversations.
  const contextUsed =
    typeof tokenUsage.context_used === 'number' && tokenUsage.context_used > 0 ? tokenUsage.context_used : 0;
  const percentage =
    hasWindow && contextUsed > 0
      ? (contextUsed / context_limit) * 100
      : hasWindow && total > 0
        ? (total / context_limit) * 100
        : 0;
  // Display value inside the ring popover: the real context footprint when we
  // have one, otherwise the reported total (platforms without `context_used`).
  const displayUsed = formatTokenCount(contextUsed > 0 ? contextUsed : total);
  const displayTotal = formatTokenCount(total);
  const displayLimit = hasWindow ? formatTokenCount(context_limit, true) : '—';

  // Per-metric real values. 图标始终以图形方式渲染；数值在无真实值时用
  // 诚实的下界（0 token / 0s 耗时），避免「元素消失」或写死占位。
  const uploadValue =
    breakdown && typeof breakdown.input_tokens === 'number'
      ? formatTokenCount(breakdown.input_tokens)
      : '0';
  const downloadValue =
    breakdown && typeof breakdown.output_tokens === 'number'
      ? formatTokenCount(breakdown.output_tokens)
      : '0';
  // 当轮耗时：流式期间用实时 ticker（turnStartedAtMs），历史回复回退到后端
  // 持久化的 elapsed_ms，确保 Clock 在每条历史消息上都能稳定显示。
  const liveElapsedMs =
    running && turnStartedAtMs != null ? Math.max(0, now - turnStartedAtMs) : null;
  const persistedElapsedMs = typeof tokenUsage.elapsed_ms === 'number' ? tokenUsage.elapsed_ms : null;
  const elapsedMs = liveElapsedMs ?? persistedElapsedMs;
  const elapsedValue = elapsedMs != null ? formatElapsed(elapsedMs) : '0秒';

  // 五元素从左到右：Clock / Tokens / Upload / Download / 上下文圆环
  const metrics = [
    { key: 'elapsed', label: t('conversation.contextUsage.elapsed', 'Elapsed'), value: elapsedValue, color: METRIC_COLORS.elapsed, icon: <ClockIcon /> },
    { key: 'tokens', label: t('conversation.contextUsage.tokens', 'tokens'), value: displayTotal, color: METRIC_COLORS.tokens, icon: <TokensIcon /> },
    { key: 'upload', label: t('conversation.contextUsage.upload', 'Upload'), value: uploadValue, color: METRIC_COLORS.upload, icon: <UploadIcon /> },
    { key: 'download', label: t('conversation.contextUsage.download', 'Download'), value: downloadValue, color: METRIC_COLORS.download, icon: <DownloadIcon /> },
  ];

  // 圆环独立悬停：方案B 仪表卡（紧凑版）—— 大号百分比 + 状态胶囊 + 压缩按钮 /
  // 状态色渐变进度条 / 已用 X / Y context。与其他指标彻底分开。
  const status = statusOf(percentage);
  const barColor =
    hasWindow && contextUsed > 0
      ? `linear-gradient(90deg, ${status.color}, ${ringColor(Math.min(100, percentage))})`
      : 'var(--color-fill-3)';
  const ringPopoverContent = (
    <div className='text-left context-compact-card' style={{ width: 168 }}>
      {/* row1：大号百分比 + 状态胶囊 + 压缩对话按钮 */}
      <div className='flex items-center justify-between gap-6px'>
        <div className='flex items-baseline gap-5px min-w-0'>
          <span
            className='pct font-bold tabular-nums whitespace-nowrap text-t-primary'
            style={{ fontSize: 11, lineHeight: '14px', letterSpacing: '-0.2px' }}
          >
            {hasWindow && percentage > 0 ? `${percentage.toFixed(1)}%` : '—'}
          </span>
          <span
            className='status inline-flex items-center font-semibold'
            style={{
              fontSize: 8,
              gap: 3,
              padding: '1px 5px',
              borderRadius: 999,
              color: status.color,
              background: `color-mix(in srgb, ${status.color} 12%, transparent)`,
              border: `1px solid color-mix(in srgb, ${status.color} 35%, transparent)`,
            }}
          >
            <span style={{ width: 3, height: 3, borderRadius: '50%', background: 'currentColor' }} />
            {t(`conversation.contextUsage.status.${status.label}`, status.label)}
          </span>
        </div>
        {showCompact && onCompact && (
          <button
            type='button'
            onClick={(e) => {
              e.stopPropagation();
              onCompact();
            }}
            disabled={compacting}
            className='compact-btn inline-flex items-center shrink-0'
            style={{
              gap: 2,
              fontSize: 8,
              color: '#b9c2d4',
              background: 'rgba(124, 141, 181, 0.12)',
              border: '1px solid rgba(124, 141, 181, 0.35)',
              borderRadius: 4,
              padding: '1px 5px',
              cursor: compacting ? 'default' : 'pointer',
              opacity: compacting ? 0.7 : 1,
            }}
          >
            {compacting ? (
              <svg
                className='animate-spin'
                width='8'
                height='8'
                viewBox='0 0 16 16'
                fill='none'
                stroke='currentColor'
                strokeWidth='1.8'
                strokeLinecap='round'
              >
                <path d='M13.5 8a5.5 5.5 0 1 1-1.6-3.9' />
              </svg>
            ) : (
              <svg
                width='8'
                height='8'
                viewBox='0 0 16 16'
                fill='none'
                stroke='currentColor'
                strokeWidth='1.6'
                strokeLinecap='round'
                strokeLinejoin='round'
              >
                <path d='M6.5 9.5 3.5 12.5M3.5 12.5h3.2M3.5 12.5V9.3M9.5 6.5l3-3M12.5 3.5v3.2M12.5 3.5H9.3' />
              </svg>
            )}
            <span>{compacting ? t('conversation.contextUsage.compacting', '压缩中…') : t('conversation.contextUsage.compact', '压缩对话')}</span>
          </button>
        )}
      </div>
      {/* row2：状态色渐变进度条 */}
      <div
        className='overflow-hidden'
        style={{ height: 2.5, borderRadius: 999, background: 'rgba(255,255,255,0.07)', margin: '5px 0 4px' }}
      >
        <div
          style={{
            height: '100%',
            width: hasWindow && percentage > 0 ? `${Math.max(2, Math.min(100, percentage))}%` : 0,
            borderRadius: 999,
            background: barColor,
            transition: 'width 0.4s ease',
          }}
        />
      </div>
      {/* row3：已用 / 窗口 */}
      <div
        className='flex items-center justify-between tabular-nums text-t-secondary whitespace-nowrap'
        style={{ fontSize: 8 }}
      >
        {hasWindow
          ? t('conversation.contextUsage.usedWindow', '已用 {{used}} / {{limit}}', { used: displayUsed, limit: displayLimit })
          : t('conversation.contextUsage.tokensUsed', '{{tokens}} tokens used', { tokens: displayUsed })}
      </div>
    </div>
  );

  return (
    <div
      className={`context-usage-indicator cursor-default flex items-center gap-12px select-none ${className}`}
      aria-label={t('conversation.contextUsage.contextUsed', 'Context usage')}
    >
      {/* 元素1-4：Clock / Tokens / Upload / Download（图标 + 数值直显，无悬停提示） */}
      <div className='flex items-center gap-12px'>
        {metrics.map((m) => (
          <span key={m.key} className='flex items-center gap-4px' aria-label={m.label}>
            <span className='flex items-center justify-center shrink-0' style={{ color: m.color }}>
              {m.icon}
            </span>
            <span className='text-t-primary font-medium tabular-nums text-11px leading-14px whitespace-nowrap'>
              {m.value}
            </span>
          </span>
        ))}
      </div>
      {/* 元素5：上下文状态圈（图形化占比，独立悬停只显示上下文使用情况） */}
      <Popover content={ringPopoverContent} position='right' trigger='hover' className='context-usage-popover'>
        <span
          className='flex items-center justify-center shrink-0'
          aria-label={t('conversation.contextUsage.contextRing', 'Context usage percentage')}
        >
          <ContextRing percentage={percentage} hasWindow={hasWindow} />
        </span>
      </Popover>
    </div>
  );
};

/**
 * Format an agent-reported cumulative session cost, e.g. "$0.42".
 * Falls back to "0.42 USD" when the currency code is not renderable.
 */
export function formatCostAmount(cost: TokenUsageCost): string {
  try {
    return new Intl.NumberFormat(undefined, {
      style: 'currency',
      currency: cost.currency,
      maximumFractionDigits: 4,
    }).format(cost.amount);
  } catch {
    return `${cost.amount.toFixed(4)} ${cost.currency}`;
  }
}

/**
 * 格式化 token 数量显示
 * @param count token 数量
 * @param hideZeroDecimals 是否隐藏小数点为0的情况（如 1.0M 显示为 1M），默认为 false
 * @returns 格式化后的字符串，如 "37.0K" 或 "1.2M"，当 hideZeroDecimals 为 true 时 "1.0M" 显示为 "1M"
 */
export function formatTokenCount(count: number, hideZeroDecimals = false): string {
  if (count >= 1_000_000) {
    const value = count / 1_000_000;
    const formatted = value.toFixed(1);
    return hideZeroDecimals && formatted.endsWith('.0') ? `${Math.floor(value)}M` : `${formatted}M`;
  }
  if (count >= 1_000) {
    const value = count / 1_000;
    const formatted = value.toFixed(1);
    return hideZeroDecimals && formatted.endsWith('.0') ? `${Math.floor(value)}K` : `${formatted}K`;
  }
  return count.toString();
}

/**
 * 格式化运行耗时，按量级呈现：
 * - < 1 秒 → 毫秒级："623毫秒"
 * - 1 秒 ~ 1 分钟 → 秒级，保留一位小数："12.5秒"
 * - ≥ 1 分钟 → 分钟级："3分钟15秒"，超 1 小时为 "1小时2分钟3秒"
 * 为 0 的段自动省略（如 60s → "1分钟"，3600s → "1小时"）。
 */
export function formatElapsed(ms: number): string {
  const totalMs = Math.max(0, ms);
  if (totalMs < 1000) {
    return `${Math.round(totalMs)}毫秒`;
  }
  if (totalMs < 60_000) {
    const sec = totalMs / 1000;
    return `${sec.toFixed(1)}秒`;
  }
  const totalSec = Math.round(totalMs / 1000);
  const h = Math.floor(totalSec / 3600);
  const m = Math.floor((totalSec % 3600) / 60);
  const s = totalSec % 60;
  const parts: string[] = [];
  if (h > 0) parts.push(`${h}小时`);
  if (m > 0) parts.push(`${m}分钟`);
  if (s > 0) parts.push(`${s}秒`);
  return parts.join('');
}

export default ContextUsageIndicator;
