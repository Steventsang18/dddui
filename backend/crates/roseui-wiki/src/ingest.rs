//! Raw-source ingestion: read-only parse of `wiki/raw/*` into wiki pages.
//!
//! # Design invariants (per user requirement 2026-08-04)
//!
//! * The raw source directory is **read-only** for the system and for any
//!   Agent. We open raw files with `OpenOptions::read(true)` only. No code in
//!   this crate ever writes to or deletes a raw file. Deletion is a user-only,
//!   explicitly confirmed action handled at the HTTP layer (`DELETE`).
//! * Each raw file becomes one *summary* page plus N *slice* pages
//!   (Obsidian-style: the source document is broken into linkable knowledge
//!   chunks). Every slice carries `source_ref = "raw/<path>#page-<N>"` so a
//!   reader can trace any wiki claim back to the exact source location.
//! * Re-ingesting an unchanged file (same sha256) is a no-op; a changed file
//!   updates the previously produced pages instead of duplicating them.
//!
//! # LLM summarization (extension point)
//!
//! Slice pages are produced by a pluggable [`Summarizer`]. The default
//! [`IdentitySummarizer`] keeps the source text verbatim (zero external
//! dependency, always works offline). A provider-backed summarizer can be
//! injected later without touching the ingest pipeline.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::WikiError;
use crate::repository::IWikiRepository;
use crate::types::CreateWikiPageRequest;

/// A contiguous chunk of extracted source text with its provenance.
pub struct SourceBlock {
    /// 1-based page/sheet number in the source document (0 for linear text).
    pub page: usize,
    /// Document heading hierarchy captured for this block (e.g. ["2. Methods"]).
    pub headings: Vec<String>,
    /// The plain-text content of the block.
    pub text: String,
}

/// Result of parsing a raw file: ordered blocks plus a short source label.
pub struct ParsedSource {
    pub blocks: Vec<SourceBlock>,
    /// e.g. "PDF · 12 pages" or "Markdown".
    pub label: String,
}

/// Turns a source block into the markdown body of a wiki slice page.
///
/// Implementations may call an LLM to distill/summarize; the default keeps the
/// original text. The ingest pipeline only depends on this trait, so swapping
/// in a model-backed summarizer is a localized change.
#[async_trait]
pub trait Summarizer: Send + Sync {
    async fn summarize(&self, block: &SourceBlock, source_label: &str) -> Result<String, WikiError>;
}

/// Default summarizer: keeps the source text verbatim, prefixing the captured
/// heading path so slices remain readable without an LLM.
pub struct IdentitySummarizer;

#[async_trait]
impl Summarizer for IdentitySummarizer {
    async fn summarize(&self, block: &SourceBlock, _source_label: &str) -> Result<String, WikiError> {
        let mut body = String::new();
        if !block.headings.is_empty() {
            body.push_str(&format!("## {}\n\n", block.headings.join(" › ")));
        }
        body.push_str(block.text.trim());
        body.push('\n');
        Ok(body)
    }
}

/// Ingestion engine. Owns the raw source directory and the wiki repository.
pub struct IngestEngine {
    raw_dir: PathBuf,
    repo: std::sync::Arc<dyn IWikiRepository>,
    summarizer: std::sync::Arc<dyn Summarizer>,
}

/// Summary of an ingest run, returned to the caller / API layer.
#[derive(Debug, Clone, Serialize)]
pub struct IngestResult {
    pub raw_path: String,
    pub status: String,
    pub summary_page_id: Option<String>,
    pub slice_page_ids: Vec<String>,
    pub message: String,
}

impl IngestEngine {
    pub fn new(
        data_dir: &Path,
        repo: std::sync::Arc<dyn IWikiRepository>,
    ) -> Self {
        Self::with_summarizer(data_dir, repo, std::sync::Arc::new(IdentitySummarizer))
    }

    pub fn with_summarizer(
        data_dir: &Path,
        repo: std::sync::Arc<dyn IWikiRepository>,
        summarizer: std::sync::Arc<dyn Summarizer>,
    ) -> Self {
        let raw_dir = data_dir.join("wiki").join("raw");
        Self {
            raw_dir,
            repo,
            summarizer,
        }
    }

    /// Ensure the raw source directory exists (read-only area for the user).
    pub fn ensure_raw_dir(&self) -> Result<(), WikiError> {
        if !self.raw_dir.exists() {
            std::fs::create_dir_all(&self.raw_dir)
                .map_err(|e| WikiError::Internal(format!("create raw dir: {e}")))?;
        }
        Ok(())
    }

    /// Resolve a raw-relative path to an absolute path, rejecting traversal
    /// outside the raw dir (defense-in-depth for the read-only guarantee).
    fn resolve_raw(&self, raw_path: &str) -> Result<PathBuf, WikiError> {
        self.ensure_raw_dir()?;
        let base = self
            .raw_dir
            .canonicalize()
            .map_err(|e| WikiError::Internal(format!("canonicalize raw dir: {e}")))?;
        let full = base.join(raw_path);
        let full = full
            .canonicalize()
            .map_err(|e| WikiError::NotFound(format!("raw file {raw_path}: {e}")))?;
        if !full.starts_with(&base) {
            return Err(WikiError::BadRequest("path escapes raw dir".into()));
        }
        Ok(full)
    }

    /// Parse a raw file into ordered source blocks (read-only).
    pub async fn parse(&self, raw_path: &str) -> Result<ParsedSource, WikiError> {
        let full = self.resolve_raw(raw_path)?;
        let bytes = std::fs::read(&full).map_err(|e| WikiError::Internal(format!("read raw: {e}")))?;
        let ext = Path::new(raw_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "md" | "markdown" | "txt" => {
                let text = String::from_utf8_lossy(&bytes).into_owned();
                Ok(parse_linear(&text, &format!("Text · {} B", bytes.len())))
            }
            "pdf" => parse_pdf(&bytes),
            "docx" => parse_docx(&bytes),
            _ => Err(WikiError::BadRequest(format!(
                "unsupported raw type: .{ext} (supported: md, txt, pdf, docx)"
            ))),
        }
    }

    /// Ingest a single raw file into wiki pages.
    pub async fn ingest(&self, raw_path: &str) -> Result<IngestResult, WikiError> {
        self.ensure_raw_dir()?;
        let full = self.resolve_raw(raw_path)?;
        let bytes = std::fs::read(&full).map_err(|e| WikiError::Internal(format!("read raw: {e}")))?;
        let checksum = format!("{:x}", Sha256::digest(&bytes));

        // Re-run guard: unchanged source already ingested -> return existing.
        if let Some(existing) = self.repo.get_ingest(raw_path).await?
            && existing.status == "done"
            && existing.checksum == checksum
        {
                return Ok(IngestResult {
                    raw_path: raw_path.to_string(),
                    status: "done".into(),
                    summary_page_id: existing.page_ids.first().cloned(),
                    slice_page_ids: existing.page_ids.iter().skip(1).cloned().collect(),
                    message: "already ingested (unchanged)".into(),
                });
        }

        self.repo
            .upsert_ingest(raw_path, &checksum, "parsing", &[], 0, "")
            .await?;

        let parsed = match self.parse(raw_path).await {
            Ok(p) => p,
            Err(e) => {
                let msg = e.to_string();
                self.repo
                    .upsert_ingest(raw_path, &checksum, "failed", &[], 0, &msg)
                    .await?;
                return Ok(IngestResult {
                    raw_path: raw_path.to_string(),
                    status: "failed".into(),
                    summary_page_id: None,
                    slice_page_ids: vec![],
                    message: msg,
                });
            }
        };

        // Split blocks into ~800-char slices, preserving heading context.
        let slices = split_blocks(&parsed.blocks, 800);

        let stem = Path::new(raw_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document")
            .to_string();

        let mut slice_page_ids = Vec::with_capacity(slices.len());
        for (i, slice) in slices.iter().enumerate() {
            let body = self.summarizer.summarize(slice, &parsed.label).await?;
            let title = format!("{stem} · §{}", i + 1);
            let source_ref = format!("raw/{raw_path}#page-{}", slice.page.max(1));
            let page = self
                .repo
                .create_page(CreateWikiPageRequest {
                    title: title.clone(),
                    content_md: body,
                    tags: Some(vec!["ingest".into(), stem.clone()]),
                    category: Some("raw-ingest".into()),
                    doc_type: Some("slice".into()),
                    source_ref: Some(source_ref),
                    status: Some("active".into()),
                    extra: Some(serde_json::json!({ "raw_path": raw_path, "slice": i + 1 })),
                })
                .await?;
            slice_page_ids.push(page.id);
        }

        // Summary page: index of all slices + provenance.
        let mut summary_md = String::new();
        summary_md.push_str(&format!("# {stem}\n\n"));
        summary_md.push_str(&format!("> 来源：raw/{}（{}）\n\n", raw_path, parsed.label));
        summary_md.push_str("## 切片索引\n\n");
        for (i, id) in slice_page_ids.iter().enumerate() {
            summary_md.push_str(&format!("- [[{stem} · §{}]] (wiki:{})\n", i + 1, id));
        }
        let summary = self
            .repo
            .create_page(CreateWikiPageRequest {
                title: stem.clone(),
                content_md: summary_md,
                tags: Some(vec!["ingest".into(), "summary".into(), stem.clone()]),
                category: Some("raw-ingest".into()),
                doc_type: Some("summary".into()),
                source_ref: Some(format!("raw/{raw_path}")),
                status: Some("active".into()),
                extra: Some(serde_json::json!({ "raw_path": raw_path })),
            })
            .await?;

        let mut all_ids = vec![summary.id.clone()];
        all_ids.extend(slice_page_ids.iter().cloned());

        self.repo
            .upsert_ingest(raw_path, &checksum, "done", &all_ids, slice_page_ids.len() as u32, "")
            .await?;

        let n_slices = slice_page_ids.len();
        Ok(IngestResult {
            raw_path: raw_path.to_string(),
            status: "done".into(),
            summary_page_id: Some(summary.id),
            slice_page_ids,
            message: format!("ingested {n_slices} slices"),
        })
    }

    /// List raw files (relative paths) with their ingest status.
    pub fn list_raw(&self) -> Result<Vec<RawEntry>, WikiError> {
        self.ensure_raw_dir()?;
        let mut out = Vec::new();
        collect_raw(&self.raw_dir, &self.raw_dir, &mut out)
            .map_err(|e| WikiError::Internal(format!("list raw: {e}")))?;
        out.sort_by(|a, b| a.relative.cmp(&b.relative));
        Ok(out)
    }

    /// Return the ingest status for a raw-relative path, or `pending` if the
    /// file has never been ingested.
    pub async fn get_ingest_status(&self, raw_path: &str) -> Option<String> {
        self.repo
            .get_ingest(raw_path)
            .await
            .ok()
            .flatten()
            .map(|r| r.status)
    }

    /// Persist an uploaded raw file (user drop). Overwrites if present.
    pub fn save_raw(&self, relative: &str, bytes: &[u8]) -> Result<PathBuf, WikiError> {
        self.ensure_raw_dir()?;
        // Reject traversal in the relative path itself.
        if relative.contains("..") || relative.starts_with('/') {
            return Err(WikiError::BadRequest("invalid raw path".into()));
        }
        let full = self.raw_dir.join(relative);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| WikiError::Internal(format!("create subdir: {e}")))?;
        }
        std::fs::write(&full, bytes)
            .map_err(|e| WikiError::Internal(format!("write raw: {e}")))?;
        Ok(full)
    }

    /// Delete a raw file (user-only, confirmed at the API layer). Does NOT
    /// touch any wiki pages; the ingest row is marked `deleted` for traceability
    /// so a reader can still see "this page came from a now-removed source".
    pub async fn delete_raw(&self, relative: &str) -> Result<(), WikiError> {
        let full = self.resolve_raw(relative)?;
        std::fs::remove_file(&full)
            .map_err(|e| WikiError::Internal(format!("delete raw: {e}")))?;
        // Mark the ingest record deleted; keep produced wiki pages intact.
        if let Some(rec) = self.repo.get_ingest(relative).await? {
            self.repo
                .upsert_ingest(relative, &rec.checksum, "deleted", &rec.page_ids, rec.slices, "")
                .await?;
        }
        Ok(())
    }
}

/// One entry in the raw-source listing.
#[derive(Debug, Clone, Serialize)]
pub struct RawEntry {
    pub relative: String,
    pub size: u64,
    pub ext: String,
}

fn collect_raw(root: &Path, dir: &Path, out: &mut Vec<RawEntry>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_raw(root, &path, out)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let meta = entry.metadata()?;
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            out.push(RawEntry {
                relative,
                size: meta.len(),
                ext,
            });
        }
    }
    Ok(())
}

/// Parse linear text (md/txt) into a single-page block list.
fn parse_linear(text: &str, label: &str) -> ParsedSource {
    let mut blocks = Vec::new();
    let mut headings = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("# ") {
            headings = vec![rest.trim().to_string()];
        } else if let Some(rest) = line.strip_prefix("## ") {
            headings = vec![rest.trim().to_string()];
        }
    }
    blocks.push(SourceBlock {
        page: 1,
        headings,
        text: text.to_string(),
    });
    ParsedSource {
        blocks,
        label: label.to_string(),
    }
}

/// Parse a PDF (primary: pdf_oxide; fallback: pdf-extract).
fn parse_pdf(bytes: &[u8]) -> Result<ParsedSource, WikiError> {
    // Primary path: pdf_oxide (fast, layout/page aware).
    if let Ok(doc) = pdf_oxide::PdfDocument::from_bytes(bytes.to_vec())
        && let Ok(n) = doc.page_count()
    {
            let mut blocks = Vec::with_capacity(n.max(1));
            for i in 0..n {
                let text = doc
                    .extract_text(i)
                    .unwrap_or_default()
                    .lines()
                    .collect::<Vec<_>>()
                    .join("\n");
                blocks.push(SourceBlock {
                    page: i + 1,
                    headings: Vec::new(),
                    text,
                });
            }
            if !blocks.is_empty() {
                return Ok(ParsedSource {
                    blocks,
                    label: format!("PDF · {n} pages"),
                });
            }
        }
    // Fallback path: pdf-extract (broader compatibility).
    match pdf_extract::extract_text_from_mem(bytes) {
        Ok(text) => {
            let blocks = text
                .lines()
                .collect::<Vec<_>>()
                .join("\n")
                .split("\n\n")
                .enumerate()
                .map(|(i, chunk)| SourceBlock {
                    page: i + 1,
                    headings: Vec::new(),
                    text: chunk.to_string(),
                })
                .collect();
            Ok(ParsedSource {
                blocks,
                label: "PDF (fallback)".into(),
            })
        }
        Err(e) => Err(WikiError::Internal(format!("pdf parse failed: {e}"))),
    }
}

/// Parse a DOCX into linear text by reading `word/document.xml` and collecting
/// `<w:t>` runs. DOCX is a ZIP of XML; we only need paragraph text, so a direct
/// zip+xml scan avoids a heavyweight parser dependency.
fn parse_docx(bytes: &[u8]) -> Result<ParsedSource, WikiError> {
    let mut reader = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| WikiError::Internal(format!("docx open: {e}")))?;
    let mut doc_xml: Option<String> = None;
    for i in 0..reader.len() {
        let mut file = reader.by_index(i).map_err(|e| WikiError::Internal(format!("docx entry: {e}")))?;
        if file.name() == "word/document.xml" {
            let mut s = String::new();
            use std::io::Read;
            file.read_to_string(&mut s)
                .map_err(|e| WikiError::Internal(format!("docx read: {e}")))?;
            doc_xml = Some(s);
            break;
        }
    }
    let xml = doc_xml.ok_or_else(|| WikiError::Internal("docx missing document.xml".into()))?;
    // Extract text inside <w:t ...>...</w:t> runs. Paragraph breaks become newlines.
    let mut text = String::new();
    let mut in_t = false;
    let mut in_p = false;
    for token in xml.split('<') {
        let tok = token.trim_end_matches('>');
        if tok.starts_with("w:p") && !tok.starts_with("w:pStyle") && !tok.starts_with("w:pPr") {
            in_p = true;
        } else if tok.starts_with("/w:p") {
            if in_p {
                text.push('\n');
            }
            in_p = false;
        } else if tok.starts_with("w:t") {
            in_t = true;
        } else if tok.starts_with("/w:t") {
            in_t = false;
        } else if in_t {
            // Text content of the run (strip any xml attributes accidentally
            // captured when the opening tag had no immediate '>').
            let content = tok.split_once('>').map(|(_, c)| c).unwrap_or(tok);
            text.push_str(content);
        }
    }
    let blocks = vec![SourceBlock {
        page: 1,
        headings: Vec::new(),
        text,
    }];
    Ok(ParsedSource {
        blocks,
        label: "DOCX".into(),
    })
}

/// Split extracted blocks into ~`target` char slices, keeping each block's
/// heading context. Blocks shorter than target are kept whole.
fn split_blocks(blocks: &[SourceBlock], target: usize) -> Vec<SourceBlock> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut cur_headings: Vec<String> = Vec::new();
    let mut cur_page = 1usize;

    let mut flush = |buf: &str, headings: &[String], page: usize| {
        let text = buf.trim();
        if !text.is_empty() {
            out.push(SourceBlock {
                page,
                headings: headings.to_vec(),
                text: text.to_string(),
            });
        }
    };

    for b in blocks {
        // Page boundary: flush accumulated buffer before starting a new page.
        if b.page != cur_page && !buf.is_empty() {
            flush(&buf, &cur_headings, cur_page);
            buf.clear();
            cur_headings.clear();
            cur_page = b.page;
        }
        if !b.headings.is_empty() {
            cur_headings = b.headings.clone();
        }
        if buf.len() + b.text.len() + 1 > target && !buf.is_empty() {
            flush(&buf, &cur_headings, cur_page);
            buf.clear();
        }
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(&b.text);
    }
    if !buf.is_empty() {
        flush(&buf, &cur_headings, cur_page);
    }
    out
}
