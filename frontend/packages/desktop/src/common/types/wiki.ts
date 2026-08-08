/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

/** A Wiki page entity as returned by the backend. */
export interface WikiPage {
  id: string;
  title: string;
  slug: string;
  content_md: string;
  tags: string[];
  /** Coarse classification bucket (user-defined vocabulary). */
  category: string;
  /** Document kind within its domain (user-defined). */
  doc_type: string;
  /** Canonical external reference (article no., case no., URL, file path). */
  source_ref: string;
  /** Lifecycle status (user-defined: draft / active / archived / ...). */
  status: string;
  /** Free-form structured metadata as a JSON object. */
  extra: Record<string, unknown>;
  created_at: number;
  updated_at: number;
}

/** Payload for creating a new Wiki page. */
export interface CreateWikiPageRequest {
  title: string;
  content_md: string;
  tags?: string[];
  category?: string;
  doc_type?: string;
  source_ref?: string;
  status?: string;
  extra?: Record<string, unknown>;
}

/** Payload for updating an existing Wiki page (partial). */
export interface UpdateWikiPageRequest {
  title?: string;
  content_md?: string;
  tags?: string[];
  category?: string;
  doc_type?: string;
  source_ref?: string;
  status?: string;
  extra?: Record<string, unknown>;
}

/** Response from the full-text search endpoint. */
export interface WikiSearchResponse {
  hits: WikiPage[];
}

/** A tag with its usage count. */
export interface WikiTag {
  name: string;
  count: number;
}

/** Controlled vocabulary of typed edges between pages. */
export type WikiEdgeType =
  | 'cites'
  | 'supersedes'
  | 'conflicts'
  | 'exemplifies'
  | 'relates';

/** A directed typed edge from one page to another. */
export interface WikiEdge {
  from_page_id: string;
  to_page_id: string;
  edge_type: WikiEdgeType;
  created_at: number;
}

/** One node in a page's link graph. */
export interface WikiGraphNode {
  page_id: string;
  title: string;
  edge_type: WikiEdgeType;
  direction: 'outgoing' | 'incoming';
}

/** Paginated response wrapper for listings. */
export interface PaginatedWikiPages {
  items: WikiPage[];
  total: number;
  offset: number;
  limit: number;
}

/** A raw-source file entry in the read-only `wiki/raw/` folder. */
export interface RawEntryView {
  /** Relative path under wiki/raw/, e.g. "contracts/2024-xx.pdf". */
  relative: string;
  /** File size in bytes. */
  size: number;
  /** Lowercased file extension, e.g. "pdf". */
  ext: string;
  /** Ingest status: pending | parsing | ingesting | done | failed | deleted. */
  status: string;
}

/** Result of ingesting a raw file into wiki pages. */
export interface IngestResult {
  raw_path: string;
  status: string;
  summary_page_id: string | null;
  slice_page_ids: string[];
  message: string;
}
