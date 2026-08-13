/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 *
 * Industry solution template settings page.
 *
 * Lets the user pick a vertical template (legal / education / medical / finance)
 * and optionally supply a company override (JSON) that further scopes tools and
 * safety. Selection is persisted via the `/api/industry-template` endpoints and
 * consumed by the Rupoo engine at agent build time.
 */

import React, { useEffect, useMemo, useState } from 'react';
import {
  Alert,
  Button,
  Card,
  Empty,
  Input,
  Message,
  Modal,
  Spin,
  Tag,
  Typography,
} from '@arco-design/web-react';
import { BookOpen, CheckOne, Delete, Save } from '@icon-park/react';
import { useTranslation } from 'react-i18next';
import SettingsPageWrapper from '../components/SettingsPageWrapper';
import MarkdownView from '@/renderer/components/Markdown';
import {
  deleteIndustryTemplate,
  getIndustryTemplate,
  listIndustryTemplates,
  updateIndustryTemplate,
  type IndustryTemplateMeta,
} from '@/common/api/industryTemplate';

const { Title, Paragraph, Text } = Typography;

const INDUSTRY_TEMPLATE_GUIDE = `## 行业模板是什么

行业模板是一组预设规则，用于约束助手在该行业场景下的行为：

- **系统提示词**：定义助手的角色与行业合规立场（如遵循《个人信息保护法》）。
- **可用工具**：限定助手能调用的能力（文件读写、搜索、命令执行等）。
- **安全策略**：决定哪些操作需要你手动审批，哪些被永久禁止。

套用模板后，助手在**每次新对话**中都会按该模板的规则运行。

## 内置模板一览

| 模板 | 角色定位 | 默认约束要点 |
| --- | --- | --- |
| 法律合规助手 | 律所合规审查 | 只读分析，禁止修改/删除文件、禁止执行系统命令。 |
| 教学辅导助手 | 作业批改与讲解 | 可读写作业文件，禁止执行系统命令。 |
| 医疗数据助手 | 敏感健康信息处理 | 数据严禁出本机；只读分析，禁止上传网络、禁止执行命令、禁止改文件。 |
| 金融风控助手 | 交易与审计分析 | 可运行只读审计命令，严禁修改/删除文件或执行危险命令。 |

## 如何操作

1. 在模板卡片中选择一个（高亮即已选中）。
2. （可选）在「公司覆盖（JSON）」中填写额外约束，进一步收紧规则。
3. 点击「保存」。新对话即生效；已在进行的对话不受影响。
4. 若要恢复「不使用模板」，点击「清除选择」。

## 公司覆盖（JSON）怎么填

可选的一段 JSON，会**合并**到模板基线上，用于进一步收紧约束。

支持的字段：

- \`excluded_tools\`（string[]）：在模板基础上**额外禁用**的工具。
- \`approval_policy\`（"always" | "dangerous"）：\`always\` = 所有工具需审批；\`dangerous\` = 仅危险操作需审批（模板默认）。

示例（禁用命令执行并要求全部审批）：

\`\`\`json
{
  "excluded_tools": ["shell_exec"],
  "approval_policy": "always"
}
\`\`\`

不填（留空 \`{}\`）即直接使用模板默认配置。

## ⚠️ 重要红线：只能更严，不能更松

无论怎么写，**模板锁定的安全底线都无法被放宽或关闭**，例如：

- 禁止删除/破坏系统或用户文件；
- 禁止将敏感数据（医疗、金融、个人身份信息）传出本机；
- 禁止绕过审批直接执行高危命令。

\`excluded_tools\` 只能往里加，不能把模板已禁用的工具「放出来」；\`approval_policy\` 只能从「仅危险」升级到「全部」，不能降级到「全免审」。非法 JSON 会在保存时提示错误。

## 常见问题

- **选了模板还要配置模型吗？** 要。模板只管「规则」，模型 API Key 仍需在「模型」设置中配置。
- **会影响已有对话吗？** 不会，仅作用于保存后新发起的对话。
`;

const IndustryTemplateSettings: React.FC = () => {
  const { t } = useTranslation();
  const [metas, setMetas] = useState<IndustryTemplateMeta[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [overrideJson, setOverrideJson] = useState<string>('{}');
  const [saving, setSaving] = useState(false);
  const [jsonError, setJsonError] = useState<string | null>(null);
  const [docOpen, setDocOpen] = useState(false);

  const load = async () => {
    try {
      const [list, current] = await Promise.all([listIndustryTemplates(), getIndustryTemplate()]);
      setMetas(list);
      if (current) {
        setSelectedId(current.template_id);
        setOverrideJson(current.override_json || '{}');
      }
    } catch (e) {
      Message.error((e as Error).message || 'Failed to load industry templates');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const selectedMeta = useMemo(
    () => metas.find((m) => m.id === selectedId) ?? null,
    [metas, selectedId]
  );

  const onOverrideChange = (value: string) => {
    setOverrideJson(value);
    if (!value.trim()) {
      setJsonError(null);
      return;
    }
    try {
      JSON.parse(value);
      setJsonError(null);
    } catch (e) {
      setJsonError((e as Error).message);
    }
  };

  const onSave = async () => {
    if (!selectedId) {
      Message.warning(t('industryTemplate.pleaseSelect', { defaultValue: 'Please select a template' }));
      return;
    }
    if (jsonError) {
      Message.error(t('industryTemplate.invalidJson', { defaultValue: 'Company override is not valid JSON' }));
      return;
    }
    setSaving(true);
    try {
      await updateIndustryTemplate(selectedId, overrideJson);
      Message.success(t('industryTemplate.saved', { defaultValue: 'Industry template saved' }));
    } catch (e) {
      Message.error((e as Error).message || 'Failed to save');
    } finally {
      setSaving(false);
    }
  };

  const onClear = async () => {
    setSaving(true);
    try {
      await deleteIndustryTemplate();
      setSelectedId(null);
      setOverrideJson('{}');
      Message.success(t('industryTemplate.cleared', { defaultValue: 'Industry template cleared' }));
    } catch (e) {
      Message.error((e as Error).message || 'Failed to clear');
    } finally {
      setSaving(false);
    }
  };

  return (
    <SettingsPageWrapper>
      <div className='industry-template-settings'>
        <div className='flex items-center justify-between gap-12px'>
          <Title heading={4} className='!mb-0'>
            {t('industryTemplate.title', { defaultValue: '行业解决方案模板' })}
          </Title>
          <Button
            icon={<BookOpen />}
            onClick={() => setDocOpen(true)}
          >
            {t('industryTemplate.docButton', { defaultValue: '查看操作指引' })}
          </Button>
        </div>
        <Paragraph type='secondary' className='mt-8px'>
          {t('industryTemplate.subtitle', {
            defaultValue:
              '选择垂直行业模板，定制助手的系统提示词、可用工具与安全策略。公司覆盖可进一步收紧约束。',
          })}
        </Paragraph>

        {loading ? (
          <div className='flex justify-center py-40px'>
            <Spin />
          </div>
        ) : metas.length === 0 ? (
          <Empty description={t('industryTemplate.none', { defaultValue: 'No industry templates available' })} />
        ) : (
          <div className='flex flex-col gap-16px'>
            <div className='grid grid-cols-1 md:grid-cols-2 gap-12px'>
              {metas.map((meta) => {
                const active = meta.id === selectedId;
                return (
                  <Card
                    key={meta.id}
                    hoverable
                    className={active ? 'industry-template-card industry-template-card--active' : 'industry-template-card'}
                    onClick={() => setSelectedId(meta.id)}
                  >
                    <div className='flex items-center justify-between'>
                      <Text bold>{meta.label}</Text>
                      {active && <CheckOne theme='filled' size='18' className='text-green-500' />}
                    </div>
                    <Paragraph type='secondary' className='mt-8px text-13px'>
                      {meta.description}
                    </Paragraph>
                    <div className='flex flex-wrap gap-4px mt-8px'>
                      {meta.excluded_tools.length > 0 && (
                        <Tag color='red'>
                          {t('industryTemplate.disabledTools', { defaultValue: 'Disabled' })}:{' '}
                          {meta.excluded_tools.join(', ')}
                        </Tag>
                      )}
                      <Tag color={meta.approval_policy === 'always' ? 'orange' : 'green'}>
                        {meta.approval_policy === 'always'
                          ? t('industryTemplate.approveAll', { defaultValue: 'Approve all tools' })
                          : t('industryTemplate.approveDangerous', { defaultValue: 'Approve dangerous only' })}
                      </Tag>
                    </div>
                  </Card>
                );
              })}
            </div>

            {selectedMeta && (
              <Card title={t('industryTemplate.companyOverride', { defaultValue: 'Company Override (JSON)' })}>
                <Alert
                  className='mb-12px'
                  type='info'
                  content={t('industryTemplate.overrideHint', {
                    defaultValue:
                      'Optional. JSON merged over the template baseline — e.g. {"excluded_tools":["shell_exec"],"approval_policy":"always"}. Locked safety bottom-lines cannot be relaxed.',
                  })}
                />
                <Input.TextArea
                  value={overrideJson}
                  onChange={onOverrideChange}
                  autoSize={{ minRows: 6, maxRows: 16 }}
                  placeholder='{}'
                  status={jsonError ? 'error' : undefined}
                />
                {jsonError && (
                  <Text type='error' className='text-12px mt-4px'>
                    {jsonError}
                  </Text>
                )}
              </Card>
            )}

            <div className='flex gap-12px'>
              <Button type='primary' loading={saving} icon={<Save />} onClick={onSave}>
                {t('common.save', { defaultValue: 'Save' })}
              </Button>
              <Button loading={saving} icon={<Delete />} onClick={onClear} disabled={!selectedId}>
                {t('industryTemplate.clear', { defaultValue: 'Clear selection' })}
              </Button>
            </div>
          </div>
        )}
      </div>

      <Modal
        title={t('industryTemplate.docTitle', { defaultValue: '行业模板操作指引' })}
        visible={docOpen}
        onCancel={() => setDocOpen(false)}
        onOk={() => setDocOpen(false)}
        footer={null}
        width={720}
        style={{ maxHeight: '82vh' }}
      >
        <div style={{ maxHeight: '72vh', overflowY: 'auto' }}>
          <MarkdownView>{INDUSTRY_TEMPLATE_GUIDE}</MarkdownView>
        </div>
      </Modal>
    </SettingsPageWrapper>
  );
};

export default IndustryTemplateSettings;
