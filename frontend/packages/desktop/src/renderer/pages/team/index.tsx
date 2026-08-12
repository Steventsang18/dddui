import { ipcBridge } from '@/common';
import React from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import useSWR from 'swr';
import EmptyState from '@renderer/components/common/EmptyState';
import { PageSkeleton } from '@renderer/components/common/Skeleton';
import TeamPage from './TeamPage';

const TeamIndex: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { data: team, isLoading } = useSWR(id ? `team/${id}` : null, () => ipcBridge.team.get.invoke({ id: id! }));

  if (isLoading) return <PageSkeleton titleWidth={180} blocks={1} />;
  if (!team) {
    return (
      <EmptyState
        title="团队不存在或已被删除"
        description="该团队可能已被移除，或链接已失效。"
        actionText="返回工作台"
        onAction={() => navigate('/guid')}
      />
    );
  }
  return <TeamPage key={team.id} team={team} />;
};

export default TeamIndex;
