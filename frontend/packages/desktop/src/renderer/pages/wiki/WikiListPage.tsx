/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

import React, { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Button, Card, Empty, Input, Message, Modal, Pagination, Spin, Tag } from '@arco-design/web-react';
import { Plus, Search } from '@icon-park/react';
import { ipcBridge } from '@/common';
import type { CreateWikiPageRequest, RawEntryView, WikiPage, WikiTag } from '@/common/types/wiki';

const PAGE_SIZE = 20;

const WikiListPage: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const [pages, setPages] = useState<WikiPage[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<WikiPage[] | null>(null);
  const [searching, setSearching] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(0);
  const [allTags, setAllTags] = useState<WikiTag[]>([]);

  // Create dialog
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [newTitle, setNewTitle] = useState('');
  const [newContent, setNewContent] = useState('');
  const [newTags, setNewTags] = useState('');
  const [newCategory, setNewCategory] = useState('');
  const [newDocType, setNewDocType] = useState('');
  const [newSourceRef, setNewSourceRef] = useState('');
  const [newStatus, setNewStatus] = useState('');
  const [creating, setCreating] = useState(false);

  // Template chooser
  const [showTemplateDialog, setShowTemplateDialog] = useState(false);
  const [templates, setTemplates] = useState<string[]>([]);
  const [applyingTemplate, setApplyingTemplate] = useState(false);

  // Source library (raw ingest)
  const [activeTab, setActiveTab] = useState<'pages' | 'raw'>('pages');
  const [rawFiles, setRawFiles] = useState<RawEntryView[]>([]);
  const [rawLoading, setRawLoading] = useState(false);
  const [rawUploading, setRawUploading] = useState(false);
  const [ingestingPath, setIngestingPath] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<RawEntryView | null>(null);
  const [deleting, setDeleting] = useState(false);

  const isSearching = searchResults !== null;

  const fetchPages = useCallback(async (page: number) => {
    setLoading(true);
    setError(null);
    try {
      const offset = (page - 1) * PAGE_SIZE;
      const result = await ipcBridge.wiki.listPages.invoke({ limit: PAGE_SIZE + 1, offset });
      // Since backend doesn't return total count, we use +1 trick
      setTotalPages(result.length === PAGE_SIZE + 1 ? page + 1 : page);
      setPages(result.slice(0, PAGE_SIZE));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setPages([]);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchTags = useCallback(async () => {
    try {
      const tags = await ipcBridge.wiki.listTags.invoke();
      setAllTags(tags);
    } catch {
      // Tags are non-critical
    }
  }, []);

  useEffect(() => {
    void fetchPages(1);
    void fetchTags();
  }, [fetchPages, fetchTags]);

  const handleSearch = useCallback(async () => {
    const q = searchQuery.trim();
    if (!q) {
      setSearchResults(null);
      return;
    }
    setSearching(true);
    try {
      const result = await ipcBridge.wiki.search.invoke({ q });
      setSearchResults(result.hits);
    } catch (e) {
      Message.error(e instanceof Error ? e.message : String(e));
    } finally {
      setSearching(false);
    }
  }, [searchQuery]);

  const handleClearSearch = useCallback(() => {
    setSearchQuery('');
    setSearchResults(null);
  }, []);

  const handleCreate = useCallback(async () => {
    if (!newTitle.trim()) {
      Message.warning(t('wiki.createTitleRequired'));
      return;
    }
    setCreating(true);
    try {
      const payload: CreateWikiPageRequest = {
        title: newTitle.trim(),
        content_md: newContent,
      };
      const tagList = newTags
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean);
      if (tagList.length > 0) {
        payload.tags = tagList;
      }
      if (newCategory.trim()) payload.category = newCategory.trim();
      if (newDocType.trim()) payload.doc_type = newDocType.trim();
      if (newSourceRef.trim()) payload.source_ref = newSourceRef.trim();
      if (newStatus.trim()) payload.status = newStatus.trim();
      const created = await ipcBridge.wiki.createPage.invoke(payload);
      Message.success(t('wiki.createSuccess'));
      setShowCreateDialog(false);
      setNewTitle('');
      setNewContent('');
      setNewTags('');
      setNewCategory('');
      setNewDocType('');
      setNewSourceRef('');
      setNewStatus('');
      navigate(`/wiki/${created.id}`);
    } catch (e) {
      Message.error(e instanceof Error ? e.message : String(e));
    } finally {
      setCreating(false);
    }
  }, [newTitle, newContent, newTags, navigate, t]);

  const handlePageClick = useCallback(
    (id: string) => {
      navigate(`/wiki/${id}`);
    },
    [navigate]
  );

  const handleOpenTemplateDialog = useCallback(async () => {
    setShowTemplateDialog(true);
    try {
      const result = await ipcBridge.wiki.listTemplates.invoke();
      setTemplates(result.templates);
    } catch {
      setTemplates(['legal', 'blank']);
    }
  }, []);

  const handleApplyTemplate = useCallback(
    async (template: string) => {
      setApplyingTemplate(true);
      try {
        const count = await ipcBridge.wiki.initTemplate.invoke({ template });
        Message.success(t('wiki.templateApplied', { count }));
        setShowTemplateDialog(false);
        void fetchPages(1);
      } catch (e) {
        Message.error(e instanceof Error ? e.message : String(e));
      } finally {
        setApplyingTemplate(false);
      }
    },
    [t, fetchPages]
  );

  // --- Source library (raw ingest) -----------------------------------------

  const fetchRaw = useCallback(async () => {
    setRawLoading(true);
    try {
      const list = await ipcBridge.wiki.listRaw.invoke();
      setRawFiles(list);
    } catch (e) {
      Message.error(e instanceof Error ? e.message : String(e));
    } finally {
      setRawLoading(false);
    }
  }, []);

  const handleTabChange = useCallback(
    (tab: 'pages' | 'raw') => {
      setActiveTab(tab);
      if (tab === 'raw') {
        void fetchRaw();
      }
    },
    [fetchRaw]
  );

  const handleRawUpload = useCallback(
    async (files: FileList | null) => {
      if (!files || files.length === 0) return;
      setRawUploading(true);
      try {
        for (const file of Array.from(files)) {
          await ipcBridge.wiki.uploadRawFile.invoke(file);
        }
        Message.success(t('wiki.rawUploadSuccess'));
        await fetchRaw();
      } catch (e) {
        Message.error(e instanceof Error ? e.message : String(e));
      } finally {
        setRawUploading(false);
      }
    },
    [fetchRaw, t]
  );

  const handleIngest = useCallback(
    async (entry: RawEntryView) => {
      setIngestingPath(entry.relative);
      try {
        const result = await ipcBridge.wiki.ingestRaw.invoke({ raw_path: entry.relative });
        Message.success(t('wiki.rawIngestSuccess', { count: result.slice_page_ids.length }));
        await fetchRaw();
      } catch (e) {
        Message.error(e instanceof Error ? e.message : String(e));
      } finally {
        setIngestingPath(null);
      }
    },
    [fetchRaw, t]
  );

  const handleConfirmDelete = useCallback(async () => {
    if (!deleteTarget) return;
    setDeleting(true);
    try {
      await ipcBridge.wiki.deleteRaw.invoke({ raw_path: deleteTarget.relative });
      Message.success(t('wiki.rawDeleteSuccess'));
      setDeleteTarget(null);
      await fetchRaw();
    } catch (e) {
      Message.error(e instanceof Error ? e.message : String(e));
    } finally {
      setDeleting(false);
    }
  }, [deleteTarget, fetchRaw, t]);

  const handlePageChange = useCallback(
    (page: number) => {
      setCurrentPage(page);
      void fetchPages(page);
    },
    [fetchPages]
  );

  const formatDate = (ts: number) => {
    const d = new Date(ts * 1000);
    return d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
  };

  const displayPages = isSearching ? searchResults : pages;

  return (
    <div className='flex flex-col h-full bg-1'>
      {/* Header */}
      <div className='shrink-0 px-20px pt-20px pb-12px border-b border-fill-3'>
        <div className='flex items-center justify-between mb-12px'>
          <div className='flex items-center gap-16px'>
            <h1 className='text-20px font-600 text-t-primary m-0'>{t('wiki.title')}</h1>
            <div className='flex bg-fill-2 rd-8px p-2px'>
              <button
                className={`px-12px py-4px rd-6px text-14px border-none cursor-pointer transition-colors ${
                  activeTab === 'pages'
                    ? 'bg-bg-1 text-t-primary font-500 shadow-1'
                    : 'bg-transparent text-t-secondary'
                }`}
                onClick={() => handleTabChange('pages')}
              >
                {t('wiki.tabPages')}
              </button>
              <button
                className={`px-12px py-4px rd-6px text-14px border-none cursor-pointer transition-colors ${
                  activeTab === 'raw'
                    ? 'bg-bg-1 text-t-primary font-500 shadow-1'
                    : 'bg-transparent text-t-secondary'
                }`}
                onClick={() => handleTabChange('raw')}
              >
                {t('wiki.tabRaw')}
              </button>
            </div>
          </div>
          {activeTab === 'pages' && (
            <div className='flex gap-8px'>
              <Button
                type='primary'
                icon={<Plus theme='outline' size='16' fill='currentColor' />}
                onClick={() => setShowCreateDialog(true)}
              >
                {t('wiki.createPage')}
              </Button>
              <Button icon={<Plus theme='outline' size='16' />} onClick={() => void handleOpenTemplateDialog()}>
                {t('wiki.template')}
              </Button>
            </div>
          )}
        </div>
        {/* Search bar */}
        <div className='flex gap-8px'>
          <Input
            placeholder={t('wiki.searchPlaceholder')}
            value={searchQuery}
            onChange={(v) => setSearchQuery(v)}
            onPressEnter={() => void handleSearch()}
            prefix={<Search theme='outline' size='16' fill='var(--color-text-3)' />}
            allowClear
            onClear={handleClearSearch}
            className='flex-1'
          />
          <Button onClick={() => void handleSearch()} loading={searching}>
            {t('wiki.search')}
          </Button>
        </div>
        {/* Tag cloud */}
        {allTags.length > 0 && (
          <div className='flex flex-wrap gap-4px mt-8px'>
            {allTags.map((tag) => (
              <Tag
                key={tag.name}
                color='arcoblue'
                className='cursor-pointer'
                onClick={() => {
                  setSearchQuery(`#${tag.name}`);
                  void ipcBridge.wiki.search.invoke({ q: `#${tag.name}` }).then((result) => {
                    setSearchResults(result.hits);
                  });
                }}
              >
                {tag.name} ({tag.count})
              </Tag>
            ))}
          </div>
        )}
      </div>

      {/* Content */}
      <div className='flex-1 min-h-0 overflow-y-auto px-20px py-16px'>
        {error && (
          <div className='mb-16px p-12px bg-danger-1 rd-8px text-danger-6 text-14px'>
            {error}
          </div>
        )}

        {activeTab === 'raw' ? (
          <div className='flex flex-col gap-12px'>
            <p className='text-13px text-t-secondary m-0'>{t('wiki.rawHint')}</p>
            {/* Drop zone */}
            <label
              className='flex flex-col items-center justify-center gap-8px p-24px rd-12px border-2 border-dashed border-fill-3 cursor-pointer hover:border-primary transition-colors'
              onDragOver={(e) => e.preventDefault()}
              onDrop={(e) => {
                e.preventDefault();
                void handleRawUpload(e.dataTransfer.files);
              }}
            >
              <input
                type='file'
                multiple
                accept='.pdf,.docx,.md,.markdown,.txt'
                className='hidden'
                onChange={(e) => void handleRawUpload(e.target.files)}
              />
              <span className='text-14px text-t-secondary'>{t('wiki.rawDrop')}</span>
              {rawUploading && <Spin />}
            </label>

            {rawLoading ? (
              <div className='flex items-center justify-center h-120px'>
                <Spin />
              </div>
            ) : rawFiles.length === 0 ? (
              <Empty description={t('wiki.rawEmpty')} className='mt-30px' />
            ) : (
              rawFiles.map((entry) => (
                <Card key={entry.relative} hoverable className='overflow-hidden'>
                  <div className='flex items-center justify-between gap-12px'>
                    <div className='flex-1 min-w-0'>
                      <div className='text-15px font-500 text-t-primary truncate'>{entry.relative}</div>
                      <div className='text-12px text-t-tertiary mt-2px'>
                        {entry.ext.toUpperCase()} · {(entry.size / 1024).toFixed(1)} KB
                      </div>
                    </div>
                    <span
                      className={`text-12px rd-4px px-8px py-2px ${
                        entry.status === 'done'
                          ? 'bg-success-1 text-success-6'
                          : entry.status === 'failed'
                          ? 'bg-danger-1 text-danger-6'
                          : 'bg-fill-2 text-t-tertiary'
                      }`}
                    >
                      {t(
                        entry.status === 'done'
                          ? 'wiki.rawStatusDone'
                          : entry.status === 'failed'
                          ? 'wiki.rawStatusFailed'
                          : entry.status === 'deleted'
                          ? 'wiki.rawStatusDeleted'
                          : 'wiki.rawStatusPending'
                      )}
                    </span>
                    <div className='flex gap-8px shrink-0'>
                      {entry.status !== 'done' && (
                        <Button
                          type='outline'
                          size='small'
                          loading={ingestingPath === entry.relative}
                          onClick={() => void handleIngest(entry)}
                        >
                          {t('wiki.rawIngest')}
                        </Button>
                      )}
                      {entry.status === 'done' && entry.relative && (
                        <Button
                          size='small'
                          onClick={() => {
                            // Navigate to the summary page produced by ingest (best-effort: open wiki list)
                            void ipcBridge.wiki.listPages.invoke({ limit: 1, offset: 0 });
                          }}
                        >
                          {t('wiki.rawIngested')}
                        </Button>
                      )}
                      <Button
                        size='small'
                        status='danger'
                        onClick={() => setDeleteTarget(entry)}
                      >
                        {t('wiki.rawDelete')}
                      </Button>
                    </div>
                  </div>
                </Card>
              ))
            )}
          </div>
        ) : loading || searching ? (
          <div className='flex items-center justify-center h-200px'>
            <Spin />
          </div>
        ) : displayPages.length === 0 ? (
          <Empty
            description={isSearching ? t('wiki.noSearchResults') : t('wiki.noPages')}
            className='mt-60px'
          />
        ) : (
          <div className='flex flex-col gap-12px'>
            {displayPages.map((page) => (
              <Card
                key={page.id}
                hoverable
                className='cursor-pointer'
                onClick={() => handlePageClick(page.id)}
              >
                <div className='flex items-start justify-between'>
                  <div className='flex-1 min-w-0'>
                    <h3 className='text-16px font-500 text-t-primary m-0 mb-4px truncate'>
                      {page.title}
                    </h3>
                    <div className='text-13px text-t-secondary line-clamp-2 mb-6px'>
                      {page.content_md.slice(0, 200) || t('wiki.noContent')}
                    </div>
                    <div className='flex items-center gap-8px flex-wrap'>
                      {page.category && (
                        <span className='text-12px text-primary bg-fill-2 rd-4px px-6px py-1px'>{page.category}</span>
                      )}
                      {page.doc_type && (
                        <Tag size='small'>{page.doc_type}</Tag>
                      )}
                      {page.tags.map((tag) => {
                        return (
                          <Tag key={tag} size='small'>
                          {tag}
                        </Tag>
                        );
                      })}
                      <span className='text-12px text-t-tertiary ml-auto'>
                        {formatDate(page.updated_at)}
                      </span>
                    </div>
                  </div>
                </div>
              </Card>
            ))}

            {/* Pagination — only show when not searching */}
            {!isSearching && totalPages > 1 && (
              <div className='flex justify-center mt-8px'>
                <Pagination
                  current={currentPage}
                  total={totalPages * PAGE_SIZE}
                  pageSize={PAGE_SIZE}
                  onChange={handlePageChange}
                  simple
                />
              </div>
            )}
          </div>
        )}
      </div>

      {/* Create dialog */}
      <Modal
        title={t('wiki.createPage')}
        visible={showCreateDialog}
        onCancel={() => setShowCreateDialog(false)}
        onOk={() => void handleCreate()}
        confirmLoading={creating}
        okText={t('common.create')}
      >
        <div className='flex flex-col gap-12px py-8px'>
          <div>
            <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.pageTitle')}</label>
            <Input
              placeholder={t('wiki.titlePlaceholder')}
              value={newTitle}
              onChange={(v) => setNewTitle(v)}
            />
          </div>
          <div>
            <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.pageContent')}</label>
            <Input.TextArea
              placeholder={t('wiki.contentPlaceholder')}
              value={newContent}
              onChange={(v) => setNewContent(v)}
              rows={6}
            />
          </div>
          <div>
            <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.tags')}</label>
            <Input
              placeholder={t('wiki.tagsPlaceholder')}
              value={newTags}
              onChange={(v) => setNewTags(v)}
            />
          </div>
          <div className='grid grid-cols-2 gap-12px'>
            <div>
              <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.category')}</label>
              <Input placeholder='e.g. law' value={newCategory} onChange={(v) => setNewCategory(v)} />
            </div>
            <div>
              <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.docType')}</label>
              <Input placeholder='e.g. statute' value={newDocType} onChange={(v) => setNewDocType(v)} />
            </div>
            <div>
              <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.sourceRef')}</label>
              <Input placeholder='e.g. CN-Civil-Code#523' value={newSourceRef} onChange={(v) => setNewSourceRef(v)} />
            </div>
            <div>
              <label className='text-14px text-t-secondary mb-4px block'>{t('wiki.status')}</label>
              <Input placeholder='e.g. active' value={newStatus} onChange={(v) => setNewStatus(v)} />
            </div>
          </div>
        </div>
      </Modal>

      {/* Template chooser */}
      <Modal
        title={t('wiki.template')}
        visible={showTemplateDialog}
        footer={null}
        onCancel={() => setShowTemplateDialog(false)}
      >
        <div className='flex flex-col gap-12px py-8px'>
          <p className='text-13px text-t-secondary m-0'>{t('wiki.templateHint')}</p>
          {templates.map((tpl) => (
            <div
              key={tpl}
              className='flex items-center justify-between p-12px rd-8px border border-fill-3 hover:bg-fill-2 transition-colors'
            >
              <div>
                <div className='text-15px font-500 text-t-primary'>{t(`wiki.template_${tpl}`)}</div>
                <div className='text-12px text-t-tertiary'>{t(`wiki.template_${tpl}_desc`)}</div>
              </div>
              <Button
                type='outline'
                loading={applyingTemplate}
                onClick={() => void handleApplyTemplate(tpl)}
              >
                {t('wiki.templateUse')}
              </Button>
            </div>
          ))}
        </div>
      </Modal>

      {/* Raw source delete confirmation */}
      <Modal
        title={t('wiki.rawDelete')}
        visible={deleteTarget !== null}
        onCancel={() => setDeleteTarget(null)}
        onOk={() => void handleConfirmDelete()}
        confirmLoading={deleting}
        okText={t('wiki.rawDelete')}
        okButtonProps={{ status: 'danger' }}
      >
        <p className='text-14px text-t-secondary'>
          {t('wiki.rawDeleteConfirm', { name: deleteTarget?.relative ?? '' })}
        </p>
      </Modal>
    </div>
  );
};

export default WikiListPage;
