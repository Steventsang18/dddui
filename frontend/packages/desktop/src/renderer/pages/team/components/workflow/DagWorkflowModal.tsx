import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { Button, Input, Message, Select } from '@arco-design/web-react';
import { ArrowDown, ArrowUp, PlayOne } from '@icon-park/react';
import { useTranslation } from 'react-i18next';
import { ipcBridge } from '@/common';
import type { IStartWorkflowResponse, TTeam, TeamAssistant } from '@/common/types/team/teamTypes';
import RoseModal from '@renderer/components/base/RoseModal';

type Step = {
  slot_id: string;
  assistant_name: string;
  prompt: string;
  depends_on: string[];
};

type Props = {
  team: TTeam;
  visible: boolean;
  onClose: () => void;
};

/** Build the default linear pipeline: every teammate in roster order, each
 *  depending on the previous one. Root (first) step has empty depends_on. */
function buildDefaultSteps(assistants: TeamAssistant[]): Step[] {
  return assistants.map((assistant, index) => ({
    slot_id: assistant.slot_id,
    assistant_name: assistant.assistant_name,
    prompt: '',
    depends_on: index === 0 ? [] : [assistants[index - 1].slot_id],
  }));
}

const DagWorkflowModal: React.FC<Props> = ({ team, visible, onClose }) => {
  const { t } = useTranslation();
  const [steps, setSteps] = useState<Step[]>([]);
  const [loading, setLoading] = useState(false);
  const [runId, setRunId] = useState<string | null>(null);

  // Reset to a fresh linear pipeline whenever the modal opens for this team.
  useEffect(() => {
    if (visible) {
      setSteps(buildDefaultSteps(team.assistants));
      setRunId(null);
      setLoading(false);
    }
  }, [visible, team]);

  const updateStep = useCallback((index: number, patch: Partial<Step>) => {
    setSteps((prev) => prev.map((step, i) => (i === index ? { ...step, ...patch } : step)));
  }, []);

  const move = useCallback((index: number, direction: -1 | 1) => {
    setSteps((prev) => {
      const target = index + direction;
      if (target < 0 || target >= prev.length) return prev;
      const next = [...prev];
      [next[index], next[target]] = [next[target], next[index]];
      // Rebuild depends_on to stay consistent with the new order: a step may
      // only depend on earlier steps, so drop any dangling predecessor.
      const validIds = new Set(next.slice(0, target < index ? target : index).map((s) => s.slot_id));
      const fixed = next.map((step, i) => ({
        ...step,
        depends_on: i === 0 ? [] : step.depends_on.filter((id) => validIds.has(id) || i > 0),
      }));
      return fixed;
    });
  }, []);

  // Candidate predecessors for a step at `index` = every step earlier in the
  // ordered list (steps after it cannot run before it).
  const predecessorsOf = useCallback(
    (index: number): Step[] => steps.slice(0, index),
    [steps]
  );

  const handleRun = useCallback(async () => {
    if (steps.length === 0) {
      Message.warning(t('team.workflow.empty', { defaultValue: 'Add at least one step to run the pipeline.' }));
      return;
    }
    setLoading(true);
    try {
      const res = (await ipcBridge.team.startWorkflow.invoke({
        team_id: team.id,
        nodes: steps.map((step) => ({
          id: step.slot_id,
          slot_id: step.slot_id,
          prompt: step.prompt,
          depends_on: step.depends_on,
        })),
      })) as IStartWorkflowResponse;
      setRunId(res.run_id);
      Message.success(
        t('team.workflow.started', { defaultValue: 'Workflow started', runId: res.run_id })
      );
    } catch (error) {
      Message.error(String(error));
    } finally {
      setLoading(false);
    }
  }, [steps, team.id, t]);

  const orderedLabel = useMemo(
    () => steps.map((s) => s.assistant_name || s.slot_id).join(' → '),
    [steps]
  );

  return (
    <RoseModal
      visible={visible}
      onCancel={onClose}
      title={t('team.workflow.title', { defaultValue: 'Workflow Pipeline' })}
      footer={
        <div className='flex items-center justify-between w-full'>
          <span className='text-12px text-[color:var(--color-text-3)] truncate max-w-60%'>
            {steps.length > 0 ? orderedLabel : ''}
          </span>
          <div className='flex gap-8px'>
            <Button onClick={onClose}>{t('common.cancel', { defaultValue: 'Cancel' })}</Button>
            <Button type='primary' loading={loading} icon={<PlayOne />} onClick={handleRun}>
              {t('team.workflow.run', { defaultValue: 'Run pipeline' })}
            </Button>
          </div>
        </div>
      }
    >
      <div className='flex flex-col gap-8px max-h-60vh overflow-y-auto'>
        {steps.length === 0 ? (
          <div className='text-13px text-[color:var(--color-text-3)] py-24px text-center'>
            {t('team.workflow.noMembers', { defaultValue: 'This team has no members to orchestrate.' })}
          </div>
        ) : (
          steps.map((step, index) => {
            const predecessors = predecessorsOf(index);
            return (
              <div
                key={step.slot_id}
                className='flex flex-col gap-6px p-12px rd-8px border border-solid border-[color:var(--border-base)] bg-1'
              >
                <div className='flex items-center justify-between gap-8px'>
                  <div className='flex items-center gap-8px min-w-0'>
                    <span className='shrink-0 w-20px h-20px rd-full bg-[var(--primary-1)] text-primary text-12px font-600 flex items-center justify-center'>
                      {index + 1}
                    </span>
                    <span className='text-13px font-600 text-t-primary truncate'>
                      {step.assistant_name || step.slot_id}
                    </span>
                  </div>
                  <div className='flex items-center gap-4px shrink-0'>
                    <button
                      type='button'
                      className='w-24px h-24px rd-4px flex items-center justify-center text-[color:var(--color-text-3)] hover:bg-[var(--fill-3)] disabled:opacity-30 cursor-pointer transition-colors'
                      disabled={index === 0}
                      onClick={() => move(index, -1)}
                      aria-label='move up'
                    >
                      <ArrowUp size='14' />
                    </button>
                    <button
                      type='button'
                      className='w-24px h-24px rd-4px flex items-center justify-center text-[color:var(--color-text-3)] hover:bg-[var(--fill-3)] disabled:opacity-30 cursor-pointer transition-colors'
                      disabled={index === steps.length - 1}
                      onClick={() => move(index, 1)}
                      aria-label='move down'
                    >
                      <ArrowDown size='14' />
                    </button>
                  </div>
                </div>
                <Input.TextArea
                  placeholder={t('team.workflow.promptPlaceholder', {
                    defaultValue: 'Step prompt (optional, overrides the teammate default)',
                  })}
                  value={step.prompt}
                  onChange={(v: string) => updateStep(index, { prompt: v })}
                  autoSize={{ minRows: 1, maxRows: 3 }}
                  allowClear
                />
                {predecessors.length > 0 && (
                  <div className='flex items-center gap-8px'>
                    <span className='text-12px text-[color:var(--color-text-3)] shrink-0'>
                      {t('team.workflow.dependsOn', { defaultValue: 'After' })}
                    </span>
                    <Select
                      mode='multiple'
                      size='mini'
                      placeholder={t('team.workflow.dependsOnPlaceholder', { defaultValue: 'select predecessors' })}
                      value={step.depends_on}
                      onChange={(v) => updateStep(index, { depends_on: v as string[] })}
                      className='flex-1'
                      options={predecessors.map((p) => ({
                        label: p.assistant_name || p.slot_id,
                        value: p.slot_id,
                      }))}
                    />
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>
      {runId && (
        <div className='mt-8px text-12px text-[color:var(--color-text-3)]'>
          {t('team.workflow.runId', { defaultValue: 'Run ID', runId })}
        </div>
      )}
    </RoseModal>
  );
};

export default DagWorkflowModal;
