/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

import type { IConversationMcpStatus } from '@/common/config/storage';
import type { ChatFileRef } from '@/common/types/chatFile';
import type { ConversationContextValue } from '@/renderer/hooks/context/ConversationContext';
import { ConversationProvider } from '@/renderer/hooks/context/ConversationContext';
import { CHAT_SURFACE_CONTAINER_CLASS } from '@/renderer/pages/conversation/utils/chatSurfaceWidth';
import FlexFullContainer from '@renderer/components/layout/FlexFullContainer';
import MessageList from '@renderer/pages/conversation/Messages/MessageList';
import { ConversationArtifactProvider } from '@renderer/pages/conversation/Messages/artifacts';
import {
  MessageListLoadingProvider,
  MessageListProvider,
  MessagePaginationProvider,
  useMessageLstCache,
} from '@renderer/pages/conversation/Messages/hooks';
import { usePendingConfirmationsRecovery } from '@renderer/pages/conversation/Messages/usePendingConfirmationsRecovery';
import HOC from '@renderer/utils/ui/HOC';
import React, { useEffect, useMemo, useState } from 'react';
import LocalImageView from '@renderer/components/media/LocalImageView';
import type { TeamSendBoxRuntime } from '@/renderer/pages/team/components/teamSendRuntime';
import AionrsSendBox from './AionrsSendBox';
import type { AionrsModelSelection } from './useAionrsModelSelection';
import { useAionrsMessage } from './useAionrsMessage';
import { modeOptionsFromCapabilities } from './useAionrsModelSelection';
import type { AgentModeOption } from '@/renderer/utils/model/agentTypes';

const AionrsChat: React.FC<{
  conversation_id: string;
  workspace: string;
  modelSelection: AionrsModelSelection;
  session_mode?: string;
  cron_job_id?: string;
  emptySlot?: React.ReactNode;
  loadedSkills?: string[];
  loadedMcpServers?: string[];
  loadedMcpStatuses?: IConversationMcpStatus[];
  agent_name?: string;
  teamSendMessage?: (payload: { input: string; files: ChatFileRef[] }) => Promise<void>;
  teamRuntime?: TeamSendBoxRuntime;
  assistantId?: string;
}> = ({
  conversation_id,
  workspace,
  modelSelection,
  session_mode,
  cron_job_id,
  emptySlot,
  loadedSkills,
  loadedMcpServers,
  loadedMcpStatuses,
  agent_name,
  teamSendMessage,
  teamRuntime,
  assistantId,
}) => {
  useMessageLstCache(conversation_id);
  usePendingConfirmationsRecovery(conversation_id);
  const updateLocalImage = LocalImageView.useUpdateLocalImage();
  useEffect(() => {
    updateLocalImage({ root: workspace });
  }, [workspace]);

  // 单实例：Chat 持有 useAionrsMessage（tokenUsage/context_limit + 模式下发），
  // 通过 prop 共享给 SendBox，避免双 hook 实例重复订阅流。
  const [dynamicModes, setDynamicModes] = useState<AgentModeOption[]>([]);
  const aionrsMessage = useAionrsMessage(conversation_id, {
    model: modelSelection.current_model?.use_model,
    onConfigChanged: (capabilities) => {
      const modes = (capabilities as { modes?: string[] })?.modes;
      if (modes && modes.length > 0) {
        setDynamicModes(modeOptionsFromCapabilities(modes));
      }
    },
  });

  const conversationValue = useMemo<ConversationContextValue>(() => {
    return {
      conversation_id: conversation_id,
      workspace,
      type: 'aionrs',
      cron_job_id,
      loadedSkills,
      loadedMcpServers,
      loadedMcpStatuses,
      assistantId,
      tokenUsage: aionrsMessage.tokenUsage,
      context_limit: aionrsMessage.context_limit,
      turnStartedAtMs: aionrsMessage.turnStartedAtMs,
      running: aionrsMessage.running,
      activeMsgId: aionrsMessage.activeMsgId,
      msgUsageStats: aionrsMessage.msgUsageStats,
      canCompact: true,
      compacting: aionrsMessage.compacting,
      onCompact: aionrsMessage.onCompact,
    };
  }, [conversation_id, workspace, cron_job_id, loadedSkills, loadedMcpServers, loadedMcpStatuses, assistantId, aionrsMessage.tokenUsage, aionrsMessage.context_limit, aionrsMessage.turnStartedAtMs, aionrsMessage.running, aionrsMessage.activeMsgId, aionrsMessage.msgUsageStats, aionrsMessage.compacting, aionrsMessage.onCompact]);

  return (
    <ConversationProvider value={conversationValue}>
      <ConversationArtifactProvider conversation_id={conversation_id}>
        <div className={`${CHAT_SURFACE_CONTAINER_CLASS} flex-1 flex flex-col px-20px min-h-0`}>
          <FlexFullContainer>
            <MessageList className='flex-1' emptySlot={emptySlot} />
          </FlexFullContainer>
          <AionrsSendBox
            conversation_id={conversation_id}
            modelSelection={modelSelection}
            session_mode={session_mode}
            agent_name={agent_name}
            teamSendMessage={teamSendMessage}
            teamRuntime={teamRuntime}
            messageState={aionrsMessage}
            dynamicModes={dynamicModes}
          />
        </div>
      </ConversationArtifactProvider>
    </ConversationProvider>
  );
};

export default HOC.Wrapper(
  MessageListProvider,
  MessageListLoadingProvider,
  MessagePaginationProvider,
  LocalImageView.Provider
)(AionrsChat);
