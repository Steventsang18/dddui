/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

import React, { useCallback, useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Button, Empty, Input, Message, Popconfirm, Select, Spin, Tag } from '@arco-design/web-react';
import { ArrowLeft, Delete, Edit, Link, Plus } from '@icon-park/react';
import { ipcBridge } from '@/common';
import type { WikiEdgeType, WikiGraphNode, WikiPage } from '@/common/types/wiki';

const WikiDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const { t } = useTranslation();
  const navigate = useNavigate();

  const [page, setPage] = useState<WikiPage | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const [saving, setSaving] = useState(false);

  // Edit form state
  const [editTitle, setEditTitle] = useState('');
  const [editContent, setEditContent] = useState('');
  const [editTags, setEditTags] = useState('');
  const [editCategory, setEditCategory] = useState('');
  const [editDocType, setEditDocType] = useState('');
  const [editSourceRef, setEditSourceRef] = useState('');
  const [editStatus, setEditStatus] = useState('');
  const [editExtra, setEditExtra] = useState('');

  // Backlinks
  const [backlinks, setBacklinks] = useState<WikiPage[]>([]);
  const [backlinksLoading, setBacklinksLoading] = useState(false);

  // Unlinked mentions (known titles mentioned in text but not yet [[linked]])
  const [unlinked, setUnlinked] = useState<string[]>([]);
  const [unlinkedLoading, setUnlinkedLoading] = useState(false);

  // Link graph (typed edges)
  const [graph, setGraph] = useState<WikiGraphNode[]>([]);
  const [graphLoading, setGraphLoading] = useState(false);

  // Add-edge form state
  const [showAddEdge, setShowAddEdge] = useState(false);
  const [edgeTarget, setEdgeTarget] = useState<string | undefined>(undefined);
  const [edgeType, setEdgeType] = useState<WikiEdgeType>('relates');
  const [edgeOptions, setEdgeOptions] = useState<{ id: string; title: string }[]>([]);
  const [addingEdge, setAddingEdge] = useState(false);

  const fetchPage = useCallback(async () => {
    if (!id) return;
    setLoading(true);
    setError(null);
    try {
      const result = await ipcBridge.wiki.getPage.invoke({ page_id: id });
      setPage(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [id]);

  const fetchBacklinks = useCallback(async () => {
    if (!id) return;
    setBacklinksLoading(true);
    try {
      const result = await ipcBridge.wiki.getBacklinks.invoke({ page_id: id });
      setBacklinks(result);
    } catch {
      // Backlinks are non-critical
    } finally {
      setBacklinksLoading(false);
    }
  }, [id]);

  const fetchUnlinked = useCallback(async () => {
    if (!id) return;
    setUnlinkedLoading(true);
    try {
      const result = await ipcBridge.wiki.unlinked.invoke({ page_id: id });
      setUnlinked(result);
    } catch {
      // Unlinked mentions are non-critical
    } finally {
      setUnlinkedLoading(false);
    }
  }, [id]);

  const fetchGraph = useCallback(async () => {
    if (!id) return;
    setGraphLoading(true);
    try {
      const result = await ipcBridge.wiki.getLinkGraph.invoke({ page_id: id });
      setGraph(result);
    } catch {
      // Graph is non-critical
    } finally {
      setGraphLoading(false);
    }
  }, [id]);

  useEffect(() => {
    void fetchPage();
    void fetchBacklinks();
    void fetchGraph();
    void fetchUnlinked();
  }, [fetchPage, fetchBacklinks, fetchGraph, fetchUnlinked]);

  const handleStartEdit = useCallback(() => {
    if (!page) return;
    setEditTitle(page.title);
    setEditContent(page.content_md);
    setEditTags(page.tags.join(', '));
    setEditCategory(page.category ?? '');
    setEditDocType(page.doc_type ?? '');
    setEditSourceRef(page.source_ref ?? '');
    setEditStatus(page.status ?? '');
    setEditExtra(page.extra ? JSON.stringify(page.extra, null, 2) : '');
    setEditing(true);
  }, [page]);

  const handleCancelEdit = useCallback(() => {
    setEditing(false);
  }, []);

  const handleSave = useCallback(async () => {
    if (!id || !editTitle.trim()) {
      Message.warning(t('wiki.createTitleRequired'));
      return;
    }
    setSaving(true);
    try {
      const tagList = editTags
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean);
      // Parse the extra JSON field; ignore when empty.
      let extra: Record<string, unknown> | undefined;
      const trimmedExtra = editExtra.trim();
      if (trimmedExtra) {
        try {
          extra = JSON.parse(trimmedExtra) as Record<string, unknown>;
        } catch {
          Message.error(t('wiki.extraJsonInvalid'));
          setSaving(false);
          return;
        }
      }
      const updated = await ipcBridge.wiki.updatePage.invoke({
        page_id: id,
        updates: {
          title: editTitle.trim(),
          content_md: editContent,
          tags: tagList,
          category: editCategory.trim() || undefined,
          doc_type: editDocType.trim() || undefined,
          source_ref: editSourceRef.trim() || undefined,
          status: editStatus.trim() || undefined,
          extra,
        },
      });
      setPage(updated);
      setEditing(false);
      Message.success(t('wiki.updateSuccess'));
      // Refresh graph since edges/titles may have changed.
      void fetchGraph();
    } catch (e) {
      Message.error(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  }, [id, editTitle, editContent, editTags, editCategory, editDocType, editSourceRef, editStatus, editExtra, t, fetchGraph]);

  const handleDelete = useCallback(async () => {
    if (!id) return;
    try {
      await ipcBridge.wiki.deletePage.invoke({ page_id: id });
      Message.success(t('wiki.deleteSuccess'));
      navigate('/wiki', { replace: true });
    } catch (e) {
      Message.error(e instanceof Error ? e.message : String(e));
    }
  }, [id, navigate, t]);

  const handleCopyCite = useCallback(async () => {
    if (!id) return;
    try {
      const citation = await ipcBridge.wiki.cite.invoke({ page_id: id });
      await navigator.clipboard.writeText(citation);
      Message.success(t('wiki.citeCopied'));
    } catch (e) {
      Message.error(e instanceof Error ? e.message : String(e));
    }
  }, [id, t]);

  const handleSearchEdgeTarget = useCallback(async (keyword: string) => {
    const q = keyword.trim();
    if (!q) {
      setEdgeOptions([]);
      return;
    }
    try {
      const result = await ipcBridge.wiki.search.invoke({ q, limit: 20 });
      // Exclude the current page from candidates.
      setEdgeOptions(
        result.hits
          .filter((p) => p.id !== id)
          .map((p) => ({ id: p.id, title: p.title }))
      );
    } catch {
      setEdgeOptions([]);
    }
  }, [id]);

  const handleAddEdge = useCallback(async () => {
    if (!id || !edgeTarget) {
      Message.warning(t('wiki.edgeTargetPlaceholder'));
      return;
    }
    setAddingEdge(true);
    try {
      await ipcBridge.wiki.putEdge.invoke({
        page_id: id,
        to_page_id: edgeTarget,
        edge_type: edgeType,
      });
      Message.success(t('wiki.edgeAdded'));
      setShowAddEdge(false);
      setEdgeTarget(undefined);
      setEdgeType('relates');
      void fetchGraph();
    } catch (e) {
      Message.error(e instanceof Error ? e.message : String(e));
    } finally {
      setAddingEdge(false);
    }
  }, [id, edgeTarget, edgeType, t, fetchGraph]);

  const handleDeleteEdge = useCallback(
    async (node: WikiGraphNode) => {
      if (!id) return;
      try {
        await ipcBridge.wiki.deleteEdge.invoke({
          page_id: id,
          to_page_id: node.page_id,
          edge_type: node.edge_type,
        });
        Message.success(t('wiki.edgeDeleted'));
        void fetchGraph();
      } catch (e) {
        Message.error(e instanceof Error ? e.message : String(e));
      }
    },
    [id, t, fetchGraph]
  );

  const handleBacklinkClick = useCallback(
    (backlinkId: string) => {
      navigate(`/wiki/${backlinkId}`);
    },
    [navigate]
  );

  // Turn an unlinked mention into a [[link]] by entering edit mode with the
  // first occurrence wrapped. The user can save to persist.
  const handleLinkMention = useCallback(
    (mention: string) => {
      if (!page) return;
      const wrapped = page.content_md.replace(mention, `[[${mention}]]`);
      setEditTitle(page.title);
      setEditContent(wrapped);
      setEditTags(page.tags.join(', '));
      setEditCategory(page.category ?? '');
      setEditDocType(page.doc_type ?? '');
      setEditSourceRef(page.source_ref ?? '');
      setEditStatus(page.status ?? '');
      setEditExtra(page.extra ? JSON.stringify(page.extra, null, 2) : '');
      setEditing(true);
    },
    [page]
  );

  const handleGraphNodeClick = useCallback(
    (nodeId: string) => {
      navigate(`/wiki/${nodeId}`);
    },
    [navigate]
  );

  const handleTagClick = useCallback(
    (tag: string) => {
      navigate(`/wiki?tag=${encodeURIComponent(tag)}`);
    },
    [navigate]
  );

  const formatDate = (ts: number) => {
    const d = new Date(ts * 1000);
    return d.toLocaleString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  // Map a typed-edge enum to its localized label.
  const edgeLabel = (edgeType: WikiEdgeType): string => {
    switch (edgeType) {
      case 'cites':
        return t('wiki.edgeCites');
      case 'supersedes':
        return t('wiki.edgeSupersedes');
      case 'conflicts':
        return t('wiki.edgeConflicts');
      case 'exemplifies':
        return t('wiki.edgeExemplifies');
      case 'relates':
      default:
        return t('wiki.edgeRelates');
    }
  };

  const renderMarkdown = (content: string) => {
    // Simple markdown rendering: convert [[links]] and basic formatting
    // In a full implementation, this would use a proper markdown renderer
    const lines = content.split('\n');
    return lines.map((line, i) => {
      // Handle headings
      if (line.startsWith('# ')) return <h1 key={i} className='text-24px font-700 mt-16px mb-8px'>{line.slice(2)}</h1>;
      if (line.startsWith('## ')) return <h2 key={i} className='text-20px font-600 mt-14px mb-6px'>{line.slice(3)}</h2>;
      if (line.startsWith('### ')) return <h3 key={i} className='text-18px font-600 mt-12px mb-4px'>{line.slice(4)}</h3>;
      // Handle code blocks
      if (line.startsWith('```')) return <div key={i} className='bg-fill-2 rd-4px p-8px my-4px text-13px font-mono'>{line.slice(3)}</div>;
      // Handle [[wiki links]]
      const linkRegex = /\[\[([^\]]+)\]\]/g;
      const parts = line.split(linkRegex);
      if (parts.length > 1) {
        return (
          <p key={i} className='my-4px text-14px leading-relaxed'>
            {parts.map((part, j) =>
              j % 2 === 1 ? (
                <span key={j} className='text-primary cursor-pointer underline'>{part}</span>
              ) : (
                part
              )
            )}
          </p>
        );
      }
      // Empty line
      if (line.trim() === '') return <div key={i} className='h-8px' />;
      // Regular paragraph
      return <p key={i} className='my-4px text-14px leading-relaxed'>{line}</p>;
    });
  };

  if (loading) {
    return (
      <div className='flex items-center justify-center h-full'>
        <Spin />
      </div>
    );
  }

  if (error || !page) {
    return (
      <div className='flex flex-col items-center justify-center h-full gap-12px p-20px'>
        <Empty description={error || t('wiki.pageNotFound')} />
        <Button icon={<ArrowLeft theme='outline' size='16' />} onClick={() => navigate('/wiki')}>
          {t('wiki.backToList')}
        </Button>
      </div>
    );
  }

  // Frontmatter values worth showing in the meta strip.
  const metaItems: Array<{ label: string; value: string }> = [];
  if (page.category) metaItems.push({ label: t('wiki.category'), value: page.category });
  if (page.doc_type) metaItems.push({ label: t('wiki.docType'), value: page.doc_type });
  if (page.source_ref) metaItems.push({ label: t('wiki.sourceRef'), value: page.source_ref });
  if (page.status) metaItems.push({ label: t('wiki.status'), value: page.status });

  return (
    <div className='flex flex-col h-full bg-base'>
      {/* Top bar */}
      <div className='shrink-0 px-20px pt-16px pb-12px border-b border-fill-3 flex items-center gap-12px'>
        <Button
          type='text'
          icon={<ArrowLeft theme='outline' size='18' />}
          onClick={() => navigate('/wiki')}
        />
        <div className='flex-1' />
        {!editing && (
          <>
            <Button type='outline' icon={<Link theme='outline' size='16' />} onClick={() => void handleCopyCite()}>
              {t('wiki.copyCite')}
            </Button>
            <Button type='outline' icon={<Edit theme='outline' size='16' />} onClick={handleStartEdit}>
              {t('common.edit')}
            </Button>
            <Popconfirm title={t('wiki.deleteConfirm')} onOk={() => void handleDelete()}>
              <Button type='outline' status='danger' icon={<Delete theme='outline' size='16' />}>
                {t('common.delete')}
              </Button>
            </Popconfirm>
          </>
        )}
      </div>

      {/* Content area */}
      <div className='flex-1 min-h-0 overflow-y-auto'>
        {editing ? (
          /* Edit mode */
          <div className='px-20px py-16px flex flex-col gap-12px max-w-900px'>
            <div>
              <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.pageTitle')}</label>
              <Input value={editTitle} onChange={(v) => setEditTitle(v)} />
            </div>
            <div>
              <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.pageContent')}</label>
              <Input.TextArea
                value={editContent}
                onChange={(v) => setEditContent(v)}
                rows={14}
                className='font-mono text-13px'
              />
            </div>
            <div className='grid grid-cols-2 gap-12px'>
              <div>
                <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.category')}</label>
                <Input value={editCategory} onChange={(v) => setEditCategory(v)} placeholder='e.g. law' />
              </div>
              <div>
                <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.docType')}</label>
                <Input value={editDocType} onChange={(v) => setEditDocType(v)} placeholder='e.g. statute' />
              </div>
              <div>
                <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.sourceRef')}</label>
                <Input value={editSourceRef} onChange={(v) => setEditSourceRef(v)} placeholder='e.g. CN-Civil-Code#523' />
              </div>
              <div>
                <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.status')}</label>
                <Input value={editStatus} onChange={(v) => setEditStatus(v)} placeholder='e.g. active' />
              </div>
            </div>
            <div>
              <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.extra')}</label>
              <Input.TextArea
                value={editExtra}
                onChange={(v) => setEditExtra(v)}
                rows={3}
                className='font-mono text-13px'
                placeholder='{ "jurisdiction": "CN" }'
              />
            </div>
            <div>
              <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.tags')}</label>
              <Input
                value={editTags}
                onChange={(v) => setEditTags(v)}
                placeholder={t('wiki.tagsPlaceholder')}
              />
            </div>
            <div className='flex gap-8px justify-end mt-4px'>
              <Button onClick={handleCancelEdit}>{t('common.cancel')}</Button>
              <Button type='primary' loading={saving} onClick={() => void handleSave()}>
                {t('common.save')}
              </Button>
            </div>
          </div>
        ) : (
          /* View mode */
          <div className='px-20px py-20px max-w-900px'>
            {/* Title */}
            <h1 className='text-24px font-700 text-t-primary m-0 mb-4px'>{page.title}</h1>

            {/* Meta */}
            <div className='flex items-center gap-8px text-12px text-t-tertiary mb-8px'>
              <span>{t('wiki.created')}: {formatDate(page.created_at)}</span>
              <span>·</span>
              <span>{t('wiki.updated')}: {formatDate(page.updated_at)}</span>
            </div>

            {/* Frontmatter meta strip */}
            {metaItems.length > 0 && (
              <div className='flex flex-wrap gap-8px mb-16px'>
                {metaItems.map((item) => (
                  <Tag key={item.label} color='arcoblue' size='small'>
                    {item.label}: {item.value}
                  </Tag>
                ))}
              </div>
            )}

            {/* Tags */}
            {page.tags.length > 0 && (
              <div className='flex flex-wrap gap-4px mb-16px'>
                {page.tags.map((tag) => (
                  <Tag key={tag} onClick={() => handleTagClick(tag)} className='cursor-pointer'>
                    {tag}
                  </Tag>
                ))}
              </div>
            )}

            {/* Content */}
            <div className='prose prose-sm max-w-none text-t-primary'>
              {page.content_md ? renderMarkdown(page.content_md) : (
                <p className='text-t-tertiary italic'>{t('wiki.noContent')}</p>
              )}
            </div>

            {/* Link graph (typed edges) */}
            <div className='mt-32px pt-20px border-t border-fill-3'>
              <div className='flex items-center justify-between mb-8px'>
                <h3 className='text-16px font-600 text-t-primary m-0'>{t('wiki.linkGraph')}</h3>
                {!showAddEdge && (
                  <Button
                    type='text'
                    size='mini'
                    icon={<Plus theme='outline' size='14' />}
                    onClick={() => {
                      setShowAddEdge(true);
                      setEdgeTarget(undefined);
                      setEdgeType('relates');
                      setEdgeOptions([]);
                    }}
                  >
                    {t('wiki.addEdge')}
                  </Button>
                )}
              </div>

              {showAddEdge && (
                <div className='flex items-center gap-8px mb-12px p-8px rd-6px bg-fill-2'>
                  <Select
                    showSearch
                    allowClear
                    placeholder={t('wiki.edgeTargetPlaceholder')}
                    style={{ flex: 1 }}
                    value={edgeTarget}
                    onChange={(v) => setEdgeTarget(v)}
                    onSearch={handleSearchEdgeTarget}
                    filterOption={false}
                    notFoundContent={null}
                  >
                    {edgeOptions.map((opt) => (
                      <Select.Option key={opt.id} value={opt.id}>
                        {opt.title}
                      </Select.Option>
                    ))}
                  </Select>
                  <Select
                    style={{ width: 120 }}
                    value={edgeType}
                    onChange={(v) => setEdgeType(v as WikiEdgeType)}
                  >
                    <Select.Option value='cites'>{t('wiki.edgeCites')}</Select.Option>
                    <Select.Option value='supersedes'>{t('wiki.edgeSupersedes')}</Select.Option>
                    <Select.Option value='conflicts'>{t('wiki.edgeConflicts')}</Select.Option>
                    <Select.Option value='exemplifies'>{t('wiki.edgeExemplifies')}</Select.Option>
                    <Select.Option value='relates'>{t('wiki.edgeRelates')}</Select.Option>
                  </Select>
                  <Button type='primary' size='mini' loading={addingEdge} onClick={() => void handleAddEdge()}>
                    {t('common.confirm')}
                  </Button>
                  <Button size='mini' onClick={() => setShowAddEdge(false)}>
                    {t('common.cancel')}
                  </Button>
                </div>
              )}

              {graphLoading ? (
                <Spin />
              ) : graph.length === 0 ? (
                <p className='text-13px text-t-tertiary'>{t('wiki.edgesEmpty')}</p>
              ) : (
                <div className='flex flex-col gap-4px'>
                  {graph.map((node) => (
                    <div
                      key={`${node.direction}-${node.page_id}-${node.edge_type}`}
                      className='flex items-center gap-8px p-8px rd-6px cursor-pointer hover:bg-fill-2 transition-colors'
                      onClick={() => handleGraphNodeClick(node.page_id)}
                    >
                      <Tag size='small' color={node.direction === 'incoming' ? 'red' : 'green'}>
                        {node.direction === 'incoming' ? t('wiki.incoming') : t('wiki.outgoing')}
                      </Tag>
                      <Tag size='small' color='arcoblue'>{edgeLabel(node.edge_type)}</Tag>
                      <span className='text-14px text-primary font-500'>{node.title}</span>
                      <Popconfirm
                        title={t('wiki.edgeDeleteConfirm')}
                        onOk={() => void handleDeleteEdge(node)}
                      >
                        <Button
                          type='text'
                          size='mini'
                          status='danger'
                          icon={<Delete theme='outline' size='14' />}
                          onClick={(e) => e.stopPropagation()}
                        />
                      </Popconfirm>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Backlinks */}
            {backlinks.length > 0 && (
              <div className='mt-32px pt-20px border-t border-fill-3'>
                <h3 className='text-16px font-600 text-t-primary mb-8px'>{t('wiki.backlinks')}</h3>
                <div className='flex flex-col gap-4px'>
                  {backlinks.map((bl) => (
                    <div
                      key={bl.id}
                      className='flex items-center gap-8px p-8px rd-6px cursor-pointer hover:bg-fill-2 transition-colors'
                      onClick={() => handleBacklinkClick(bl.id)}
                    >
                      <span className='text-14px text-primary font-500'>{bl.title}</span>
                      {bl.tags.map((tag) => (
                        <Tag key={tag} size='small'>{tag}</Tag>
                      ))}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Unlinked mentions — known page titles referenced in text but not yet linked */}
            {unlinked.length > 0 && (
              <div className='mt-32px pt-20px border-t border-fill-3'>
                <h3 className='text-16px font-600 text-t-primary mb-4px'>{t('wiki.unlinked')}</h3>
                <p className='text-12px text-t-tertiary mb-8px'>{t('wiki.unlinkedHint')}</p>
                <div className='flex flex-wrap gap-4px'>
                  {unlinked.map((title) => (
                    <Tag
                      key={title}
                      size='small'
                      className='cursor-pointer'
                      onClick={() => handleLinkMention(title)}
                    >
                      {title} ＋
                    </Tag>
                  ))}
                </div>
              </div>
            )}

            {/* Backlinks loading */}
            {backlinksLoading && (
              <div className='mt-32px pt-20px border-t border-fill-3'>
                <Spin />
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default WikiDetailPage;
