/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

import React from 'react';
import styles from './Skeleton.module.css';

export type SkeletonVariant = 'text' | 'rect' | 'circle';

export interface SkeletonProps {
  variant?: SkeletonVariant;
  /** px number or any CSS length. Defaults differ per variant. */
  width?: number | string;
  height?: number | string;
  /** Extra class (e.g. UnoCSS utilities) merged after module classes. */
  className?: string;
  style?: React.CSSProperties;
  /** Shimmer sweep. Defaults to true. */
  active?: boolean;
  /** Larger radius for cards/blocks. */
  rounded?: boolean;
}

/**
 * Primitive placeholder block. Theme-aware: uses Arco's --color-fill-2 base and
 * a color-mix highlight so it tracks light/dark automatically. Replaces bare
 * <Spin loading /> page blockers with a content-shaped placeholder.
 */
export const Skeleton: React.FC<SkeletonProps> = ({
  variant = 'rect',
  width,
  height,
  className = '',
  style,
  active = true,
  rounded = false,
}) => {
  const cls = [
    styles.skeleton,
    styles[variant],
    active ? styles.shimmer : '',
    rounded ? styles.rounded : '',
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <span
      className={cls}
      style={{ width, height, ...style }}
      aria-hidden="true"
    />
  );
};

export interface SkeletonTextProps {
  lines?: number;
  /** Per-line width; falls back to 100% then 80% for the last line. */
  lineWidths?: (number | string)[];
  className?: string;
  gap?: number;
}

/** Vertical stack of text-line skeletons, mimicking a paragraph. */
export const SkeletonText: React.FC<SkeletonTextProps> = ({
  lines = 3,
  lineWidths,
  className = '',
  gap = 10,
}) => (
  <div
    className={`${styles.lines} ${className}`}
    style={{ gap }}
    aria-hidden="true"
  >
    {Array.from({ length: lines }).map((_, i) => (
      <Skeleton
        key={i}
        variant="text"
        width={lineWidths?.[i] ?? (i === lines - 1 ? '80%' : '100%')}
        active
      />
    ))}
  </div>
);

export interface PageSkeletonProps {
  className?: string;
  /** Approximation of the page: header bar + stacked content blocks. */
  titleWidth?: number | string;
  blocks?: number;
}

/**
 * Full-area loading placeholder for route-level blockers (e.g. conversation /
 * team index). Mirrors a header + content layout so the page doesn't "pop" in.
 */
export const PageSkeleton: React.FC<PageSkeletonProps> = ({
  className = '',
  titleWidth = 220,
  blocks = 2,
}) => (
  <div
    className={`${styles.page} ${className}`}
    role="status"
    aria-busy="true"
    aria-label="加载中"
  >
    <Skeleton variant="rect" height={28} width={titleWidth} rounded active />
    <div className={styles.pageBody}>
      <Skeleton variant="rect" height={160} rounded active />
      <SkeletonText lines={4} />
      {blocks > 0 && (
        <Skeleton variant="rect" height={120} rounded active />
      )}
    </div>
  </div>
);

export default Skeleton;
