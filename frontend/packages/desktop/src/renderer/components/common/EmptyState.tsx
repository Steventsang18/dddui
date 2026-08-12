import { Button, Empty } from '@arco-design/web-react';
import React from 'react';
import './EmptyState.css';

export interface EmptyStateProps {
  /** 主提示文案（加粗显示） */
  title?: string;
  /** 次级说明文案 */
  description?: string;
  /** 操作按钮文案；不传则不显示按钮 */
  actionText?: string;
  /** 点击操作按钮的回调 */
  onAction?: () => void;
  /** 附加自定义类名（可选） */
  className?: string;
}

/**
 * 统一空状态组件。用于替代散落在各页面里的 `return null` 或裸 <Empty/>，
 * 保证所有「无数据 / 不存在」场景有一致的视觉与可恢复的出口。
 * 视觉语言与 Setup 门禁屏共用同一套主题令牌与间距尺度。
 */
const EmptyState: React.FC<EmptyStateProps> = ({
  title,
  description,
  actionText,
  onAction,
  className,
}) => (
  <div className={`empty-state${className ? ` ${className}` : ''}`}>
    <Empty description={title} />
    {description && <div className="empty-state__desc">{description}</div>}
    {actionText && onAction && (
      <Button type="primary" onClick={onAction}>
        {actionText}
      </Button>
    )}
  </div>
);

export default EmptyState;
