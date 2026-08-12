import { Button, Message, Steps } from '@arco-design/web-react';
import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { ipcBridge } from '@/common';
import type { IProvider } from '@/common/config/storage';
import AddPlatformModal from '@renderer/pages/settings/components/AddPlatformModal';
import BrandMark from '@renderer/components/common/BrandMark';
import './Setup.css';

/**
 * 首启 Setup 向导（纯 UI 层）。
 * 流程：欢迎 → 配置模型（复用现有 AddPlatformModal，产出 IProvider）→ 完成。
 * 模型配置落库调用已部署接口 ipcBridge.mode.createProvider（POST /api/providers），
 * 不新增任何底层接口、不改端口与认证逻辑。
 */
const Setup: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [step, setStep] = useState(0);
  const [saving, setSaving] = useState(false);

  const [platformModal, platformModalNode] = AddPlatformModal.useModal({
    onSubmit: async (provider: IProvider) => {
      try {
        setSaving(true);
        await ipcBridge.mode.createProvider.invoke(provider);
        Message.success(t('settings.modelAdded', { defaultValue: '模型已添加' }));
        setStep(2);
      } catch (e: unknown) {
        Message.error(e instanceof Error ? e.message : '配置失败');
      } finally {
        setSaving(false);
      }
    },
  });

  return (
    <div className="setup-page">
      <div className="setup-card">
        <div className="setup-brand">
          <BrandMark size="lg" />
        </div>

        <h1 className="setup-title">{t('setup.welcome', { defaultValue: '欢迎使用 DDDUI' })}</h1>
        <p className="setup-subtitle">
          {t('setup.intro', {
            defaultValue: '本地优先的多 Agent 编排平台。开始对话前，请先配置一个模型（如 DeepSeek）并粘贴你的 API Key。',
          })}
        </p>

        <div className="setup-steps">
          <Steps type="dot" current={step}>
            <Steps.Step title={t('setup.stepWelcome', { defaultValue: '欢迎' })} />
            <Steps.Step title={t('setup.stepConfig', { defaultValue: '配置模型' })} />
            <Steps.Step title={t('setup.stepDone', { defaultValue: '完成' })} />
          </Steps>
        </div>

        {/* key={step} 让每次步骤切换重挂载，重放淡入动效 */}
        <div className="setup-body" key={step}>
          {step === 0 && (
            <div className="setup-actions">
              <Button type="primary" long onClick={() => setStep(1)}>
                {t('setup.start', { defaultValue: '开始配置' })}
              </Button>
            </div>
          )}

          {step === 1 && (
            <div className="setup-actions">
              <Button type="primary" long loading={saving} onClick={() => platformModal.open()}>
                {t('setup.addModel', { defaultValue: '添加模型 / API Key' })}
              </Button>
              <Button type="text" long onClick={() => setStep(2)}>
                {t('setup.skip', { defaultValue: '稍后再说（可在「设置 → 模型」中配置）' })}
              </Button>
            </div>
          )}

          {step === 2 && (
            <div className="setup-done">
              <span className="setup-done-icon" aria-hidden="true">
                <svg
                  width="28"
                  height="28"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2.4"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M20 6 9 17l-5-5" />
                </svg>
              </span>
              <p className="setup-done-text">
                {t('setup.done', { defaultValue: '配置完成，开始你的第一次对话吧。' })}
              </p>
              <Button type="primary" long onClick={() => navigate('/guid')}>
                {t('setup.enter', { defaultValue: '进入工作台' })}
              </Button>
            </div>
          )}
        </div>

        <div className="setup-footer-note">
          <svg
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
          </svg>
          {t('setup.localFirst', { defaultValue: '本地优先 · 数据不出端' })}
        </div>
      </div>
      {platformModalNode}
    </div>
  );
};

export default Setup;
