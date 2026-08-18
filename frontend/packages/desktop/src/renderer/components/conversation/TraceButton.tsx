/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 *
 * 对话轨迹入口按钮（图标徽章，hover 显示"轨迹"提示）。
 * 点击导航到完整全页面 /conversation/:id/trace，不弹出抽屉。
 * 仅查询当前会话的轨迹（监控录像），不跨会话。
 * 后端：GET /api/conversations/{id}/trace （event_kind / model / from_ts / to_ts 过滤）
 */

import { History } from '@icon-park/react';
import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Tooltip } from '@arco-design/web-react';
import { useTranslation } from 'react-i18next';
import { iconColors } from '@/renderer/styles/colors';
import './TraceButton.css';

const TraceButton: React.FC<{ conversationId: string }> = ({ conversationId }) => {
  const { t } = useTranslation();
  const navigate = useNavigate();

  return (
    <Tooltip content={t('conversation.trace.button')} position='top'>
      <button
        type='button'
        aria-label={t('conversation.trace.button')}
        onClick={() => navigate(`/conversation/${conversationId}/trace`)}
        className='trace-btn-icon'
      >
        <History theme='outline' size={16} fill={iconColors.primary} />
      </button>
    </Tooltip>
  );
};

export default TraceButton;
