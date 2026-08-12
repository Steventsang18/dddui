/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

import React from 'react';
import styles from './BrandMark.module.css';

export interface BrandMarkProps {
  /** 图标方块尺寸：sm=侧边栏/紧凑, md=默认, lg=向导/大屏 */
  size?: 'sm' | 'md' | 'lg';
  /** 是否显示 DDDUI 文字（collapsed 侧边栏传 false） */
  showName?: boolean;
  className?: string;
}

/**
 * 多 Agent 编排节点字标（产品统一视觉锚点）。
 * 与 Setup 门禁屏同源：中心节点 + 双叶节点 + 连线，呼应「多 Agent 编排」。
 * 主题感知：渐变用 --aou-* 色阶，随 light/dark 自动切换。
 */
const AgentNodeMark: React.FC = () => (
  <svg
    width="60%"
    height="60%"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <circle cx="12" cy="5" r="2.2" />
    <circle cx="5" cy="18" r="2.2" />
    <circle cx="19" cy="18" r="2.2" />
    <path d="M10.4 6.8 6.6 16M13.6 6.8 17.4 16M7.2 18h9.6" />
  </svg>
);

const BrandMark: React.FC<BrandMarkProps> = ({ size = 'md', showName = true, className }) => {
  return (
    <div className={`${styles.brand} ${styles[size]} ${className ?? ''}`}>
      <span className={styles.mark} aria-hidden="true">
        <AgentNodeMark />
      </span>
      {showName && <span className={styles.name}>DDDUI</span>}
    </div>
  );
};

export default BrandMark;
