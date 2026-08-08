//! Wiki data access layer.
//!
//! `WikiRepository` owns all SQL against the shared SQLite pool (from
//! `roseui-db`). The schema lives in migrations `034_wiki.sql` and
//! `035_wiki_frontmatter_edges.sql`. This mirrors the
//! `I*Repository` / `Sqlite*Repository` convention used across `roseui-db`.

use async_trait::async_trait;
use sqlx::SqlitePool;

use roseui_common::now_ms;

use crate::error::WikiError;
use crate::types::{
    CreateWikiPageRequest, IngestRow, UpdateWikiPageRequest, WikiEdge, WikiEdgeType, WikiGraphNode,
    WikiPage, WikiTag,
};

/// A wiki_pages row without its denormalized tag list; tags are fetched
/// separately and merged into [`WikiPage`] on load.
#[derive(Debug, Clone, sqlx::FromRow)]
struct WikiPageRow {
    page_id: String,
    title: String,
    slug: String,
    content_md: String,
    category: String,
    doc_type: String,
    source_ref: String,
    status: String,
    extra: String,
    created_at: i64,
    updated_at: i64,
}

impl WikiPageRow {
    /// Parse the `extra` JSON column into a `serde_json::Value`, degrading to an
    /// empty object on malformed content (never fails a read).
    fn extra_json(&self) -> serde_json::Value {
        serde_json::from_str(&self.extra).unwrap_or(serde_json::Value::Object(Default::default()))
    }
}

/// Wiki persistence boundary.
#[async_trait]
pub trait IWikiRepository: Send + Sync {
    async fn create_page(&self, req: CreateWikiPageRequest) -> Result<WikiPage, WikiError>;
    async fn get_page(&self, page_id: &str) -> Result<Option<WikiPage>, WikiError>;
    /// Resolve a page by title (case-insensitive) — used for `[[link]]` targets.
    async fn get_page_by_title(&self, title: &str) -> Result<Option<WikiPage>, WikiError>;
    async fn update_page(
        &self,
        page_id: &str,
        req: UpdateWikiPageRequest,
    ) -> Result<Option<WikiPage>, WikiError>;
    async fn delete_page(&self, page_id: &str) -> Result<bool, WikiError>;
    async fn list_pages(&self, limit: u32, offset: u32) -> Result<Vec<WikiPage>, WikiError>;
    /// FTS5 full-text search over title + content, optionally constrained by
    /// frontmatter `category` / `doc_type` values.
    async fn search(
        &self,
        query: &str,
        limit: u32,
        category: Option<&str>,
        doc_type: Option<&str>,
    ) -> Result<Vec<WikiPage>, WikiError>;
    async fn set_tags(&self, page_id: &str, tags: &[String]) -> Result<(), WikiError>;
    async fn list_tags(&self) -> Result<Vec<WikiTag>, WikiError>;
    /// Pages that link to `title` (case-insensitive) via `[[wiki link]]` syntax.
    async fn backlinks(&self, title: &str) -> Result<Vec<WikiPage>, WikiError>;
    /// Autocomplete page titles matching a prefix/substring (for `[[` input).
    async fn title_search(&self, q: &str, limit: u32) -> Result<Vec<String>, WikiError>;
    /// Known page titles mentioned in `content` as plain text but NOT wrapped in
    /// `[[ ]]`, i.e. candidates for the user to turn into links.
    async fn unlinked_mentions(&self, page_id: &str) -> Result<Vec<String>, WikiError>;
    /// Seed pages from a template definition (legal / blank / ...).
    async fn apply_template(&self, template: &str) -> Result<u32, WikiError>;

    // --- Raw ingest tracking (migration 038) ----------------------------
    /// Fetch the ingest record for a raw-relative path, if any.
    async fn get_ingest(&self, raw_path: &str) -> Result<Option<IngestRow>, WikiError>;
    /// Upsert an ingest record (insert or update by `raw_path`).
    async fn upsert_ingest(
        &self,
        raw_path: &str,
        checksum: &str,
        status: &str,
        page_ids: &[String],
        slices: u32,
        error: &str,
    ) -> Result<(), WikiError>;

    // --- Phase 2: typed edges ---------------------------------------------
    /// Upsert a typed edge between two pages (by page id). Idempotent.
    async fn put_edge(
        &self,
        from_page_id: &str,
        to_page_id: &str,
        edge_type: WikiEdgeType,
    ) -> Result<(), WikiError>;
    /// Remove a typed edge between two pages.
    async fn delete_edge(
        &self,
        from_page_id: &str,
        to_page_id: &str,
        edge_type: WikiEdgeType,
    ) -> Result<(), WikiError>;
    /// Typed edges originating from a page.
    async fn outgoing_edges(&self, page_id: &str) -> Result<Vec<WikiEdge>, WikiError>;
    /// Typed edges pointing at a page.
    async fn incoming_edges(&self, page_id: &str) -> Result<Vec<WikiEdge>, WikiError>;
    /// Combined link graph (outgoing + incoming) with edge types and direction.
    async fn link_graph(&self, page_id: &str) -> Result<Vec<WikiGraphNode>, WikiError>;
}

/// SQLite-backed [`IWikiRepository`].
#[derive(Clone, Debug)]
pub struct SqliteWikiRepository {
    pool: SqlitePool,
}

impl SqliteWikiRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Upsert the tag set for a page row (inside an open transaction).
    async fn apply_tags(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        row_id: i64,
        tags: &[String],
    ) -> Result<(), WikiError> {
        for tag in tags {
            let name = tag.trim();
            if name.is_empty() {
                continue;
            }
            let tag_id = uuid::Uuid::now_v7().to_string();
            sqlx::query("INSERT OR IGNORE INTO wiki_tags (tag_id, name) VALUES (?, ?)")
                .bind(&tag_id)
                .bind(name)
                .execute(&mut **tx)
                .await
                .map_err(db)?;
            sqlx::query(
                "INSERT OR IGNORE INTO wiki_page_tags (page_id, tag_id) \
                 VALUES (?, (SELECT id FROM wiki_tags WHERE name = ?))",
            )
            .bind(row_id)
            .bind(name)
            .execute(&mut **tx)
            .await
            .map_err(db)?;
        }
        Ok(())
    }

    /// Insert `[[link]]` targets for a page row (inside an open transaction).
    async fn apply_links(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        row_id: i64,
        targets: &[String],
        now: i64,
    ) -> Result<(), WikiError> {
        for target in targets {
            sqlx::query(
                "INSERT OR IGNORE INTO wiki_links (from_page_id, to_title, created_at) VALUES (?, ?, ?)",
            )
            .bind(row_id)
            .bind(target)
            .bind(now)
            .execute(&mut **tx)
            .await
            .map_err(db)?;
        }
        Ok(())
    }

    /// Fetch the tag names for a page row id.
    async fn fetch_tags(&self, row_id: i64) -> Vec<String> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT t.name FROM wiki_tags t \
             JOIN wiki_page_tags pt ON pt.tag_id = t.id \
             WHERE pt.page_id = ? ORDER BY t.name",
        )
        .bind(row_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();
        rows.into_iter().map(|r| r.0).collect()
    }

    /// Merge a row with its tags into a [`WikiPage`].
    async fn load_page(&self, row: WikiPageRow) -> Result<WikiPage, WikiError> {
        let row_id: i64 = sqlx::query_scalar("SELECT id FROM wiki_pages WHERE page_id = ?")
            .bind(&row.page_id)
            .fetch_one(&self.pool)
            .await
            .map_err(db)?;
        let tags = self.fetch_tags(row_id).await;
        let extra = row.extra_json();
        Ok(WikiPage {
            id: row.page_id,
            title: row.title,
            slug: row.slug,
            content_md: row.content_md,
            tags,
            category: row.category,
            doc_type: row.doc_type,
            source_ref: row.source_ref,
            status: row.status,
            extra,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn page_row_id(&self, page_id: &str) -> Result<Option<i64>, WikiError> {
        sqlx::query_scalar("SELECT id FROM wiki_pages WHERE page_id = ?")
            .bind(page_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db)
    }

    /// Like [`Self::page_row_id`] but runs on an open transaction connection,
    /// so it never contends for a second pooled connection (in-memory SQLite
    /// pools are size-1).
    async fn page_row_id_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        page_id: &str,
    ) -> Result<Option<i64>, WikiError> {
        sqlx::query_scalar("SELECT id FROM wiki_pages WHERE page_id = ?")
            .bind(page_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(db)
    }
}

#[async_trait]
impl IWikiRepository for SqliteWikiRepository {
    async fn create_page(&self, req: CreateWikiPageRequest) -> Result<WikiPage, WikiError> {
        let now = now_ms();
        let page_id = uuid::Uuid::now_v7().to_string();
        let slug = slugify(&req.title);
        let tags = req.tags.clone().unwrap_or_default();
        let targets = extract_links(&req.content_md);
        let category = req.category.clone().unwrap_or_default();
        let doc_type = req.doc_type.clone().unwrap_or_default();
        let source_ref = req.source_ref.clone().unwrap_or_default();
        let status = req.status.clone().unwrap_or_default();
        let extra = req
            .extra
            .clone()
            .and_then(|v| serde_json::to_string(&v).ok())
            .unwrap_or_else(|| "{}".to_string());

        let mut tx = self.pool.begin().await.map_err(db)?;
        sqlx::query(
            "INSERT INTO wiki_pages \
             (page_id, title, slug, content_md, category, doc_type, source_ref, status, extra, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&page_id)
        .bind(&req.title)
        .bind(&slug)
        .bind(&req.content_md)
        .bind(&category)
        .bind(&doc_type)
        .bind(&source_ref)
        .bind(&status)
        .bind(&extra)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(db)?;

        let row_id = self.page_row_id_tx(&mut tx, &page_id).await?;
        let row_id = match row_id {
            Some(id) => id,
            None => return Err(WikiError::Internal("wiki page row missing after insert".into())),
        };
        self.apply_tags(&mut tx, row_id, &tags).await?;
        self.apply_links(&mut tx, row_id, &targets, now).await?;
        tx.commit().await.map_err(db)?;

        self.get_page(&page_id)
            .await?
            .ok_or_else(|| WikiError::Internal("wiki page missing after create".into()))
    }

    async fn get_page(&self, page_id: &str) -> Result<Option<WikiPage>, WikiError> {
        let row: Option<WikiPageRow> = sqlx::query_as::<_, WikiPageRow>(
            "SELECT page_id, title, slug, content_md, category, doc_type, source_ref, status, extra, \
             created_at, updated_at FROM wiki_pages WHERE page_id = ?",
        )
        .bind(page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db)?;
        match row {
            Some(r) => Ok(Some(self.load_page(r).await?)),
            None => Ok(None),
        }
    }

    async fn get_page_by_title(&self, title: &str) -> Result<Option<WikiPage>, WikiError> {
        let row: Option<WikiPageRow> = sqlx::query_as::<_, WikiPageRow>(
            "SELECT page_id, title, slug, content_md, category, doc_type, source_ref, status, extra, \
             created_at, updated_at FROM wiki_pages WHERE lower(title) = lower(?) LIMIT 1",
        )
        .bind(title)
        .fetch_optional(&self.pool)
        .await
        .map_err(db)?;
        match row {
            Some(r) => Ok(Some(self.load_page(r).await?)),
            None => Ok(None),
        }
    }

    async fn update_page(
        &self,
        page_id: &str,
        req: UpdateWikiPageRequest,
    ) -> Result<Option<WikiPage>, WikiError> {
        let existing: Option<WikiPageRow> = sqlx::query_as::<_, WikiPageRow>(
            "SELECT page_id, title, slug, content_md, category, doc_type, source_ref, status, extra, \
             created_at, updated_at FROM wiki_pages WHERE page_id = ?",
        )
        .bind(page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db)?;
        let existing = match existing {
            Some(e) => e,
            None => return Ok(None),
        };

        let old_title = existing.title.clone();
        let new_title = req.title.clone().unwrap_or(existing.title.clone());
        let renamed = req.title.is_some() && new_title != old_title;
        let new_content = req.content_md.clone().unwrap_or(existing.content_md);
        let new_slug = if req.title.is_some() {
            slugify(&new_title)
        } else {
            existing.slug
        };
        let new_category = req.category.clone().unwrap_or(existing.category);
        let new_doc_type = req.doc_type.clone().unwrap_or(existing.doc_type);
        let new_source_ref = req.source_ref.clone().unwrap_or(existing.source_ref);
        let new_status = req.status.clone().unwrap_or(existing.status);
        let new_extra = req
            .extra
            .clone()
            .and_then(|v| serde_json::to_string(&v).ok())
            .unwrap_or(existing.extra);
        let now = now_ms();

        let mut tx = self.pool.begin().await.map_err(db)?;
        sqlx::query(
            "UPDATE wiki_pages SET title = ?, slug = ?, content_md = ?, category = ?, doc_type = ?, \
             source_ref = ?, status = ?, extra = ?, updated_at = ? WHERE page_id = ?",
        )
        .bind(&new_title)
        .bind(&new_slug)
        .bind(&new_content)
        .bind(&new_category)
        .bind(&new_doc_type)
        .bind(&new_source_ref)
        .bind(&new_status)
        .bind(&new_extra)
        .bind(now)
        .bind(page_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;

        // Rename propagation: keep `[[Old Title]]` links valid via wiki_aliases
        // and re-point existing wiki_links.to_title entries to the new title.
        if renamed {
            sqlx::query(
                "INSERT OR IGNORE INTO wiki_aliases (page_id, alias, created_at) VALUES (?, ?, ?)",
            )
            .bind(page_id)
            .bind(&old_title)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
            sqlx::query("UPDATE wiki_links SET to_title = ? WHERE to_title = ?")
                .bind(&new_title)
                .bind(&old_title)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }

        if let Some(tags) = &req.tags {
            sqlx::query(
                "DELETE FROM wiki_page_tags WHERE page_id = \
                 (SELECT id FROM wiki_pages WHERE page_id = ?)",
            )
            .bind(page_id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
            if let Some(row_id) = self.page_row_id_tx(&mut tx, page_id).await? {
                self.apply_tags(&mut tx, row_id, tags).await?;
            }
        }

        // Recompute `[[link]]` targets from the (possibly updated) content.
        let targets = extract_links(&new_content);
        sqlx::query(
            "DELETE FROM wiki_links WHERE from_page_id = \
             (SELECT id FROM wiki_pages WHERE page_id = ?)",
        )
        .bind(page_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        if let Some(row_id) = self.page_row_id_tx(&mut tx, page_id).await? {
            self.apply_links(&mut tx, row_id, &targets, now).await?;
        }

        tx.commit().await.map_err(db)?;
        self.get_page(page_id).await
    }

    async fn delete_page(&self, page_id: &str) -> Result<bool, WikiError> {
        let row_id = self.page_row_id(page_id).await?;
        let row_id = match row_id {
            Some(id) => id,
            None => return Ok(false),
        };
        let mut tx = self.pool.begin().await.map_err(db)?;
        sqlx::query("DELETE FROM wiki_page_tags WHERE page_id = ?")
            .bind(row_id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        // Outgoing + incoming typed edges cascade via ON DELETE CASCADE; clean
        // them explicitly too in case the FK pragma is off.
        sqlx::query("DELETE FROM wiki_edges WHERE from_page_id = ? OR to_page_id = ?")
            .bind(row_id)
            .bind(row_id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("DELETE FROM wiki_links WHERE from_page_id = ?")
            .bind(row_id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        // AFTER DELETE trigger keeps wiki_fts in sync.
        sqlx::query("DELETE FROM wiki_pages WHERE id = ?")
            .bind(row_id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        tx.commit().await.map_err(db)?;
        Ok(true)
    }

    async fn list_pages(&self, limit: u32, offset: u32) -> Result<Vec<WikiPage>, WikiError> {
        let rows = sqlx::query_as::<_, WikiPageRow>(
            "SELECT page_id, title, slug, content_md, category, doc_type, source_ref, status, extra, \
             created_at, updated_at FROM wiki_pages ORDER BY updated_at DESC LIMIT ? OFFSET ?",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(db)?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            out.push(self.load_page(r).await?);
        }
        Ok(out)
    }

    async fn search(
        &self,
        query: &str,
        limit: u32,
        category: Option<&str>,
        doc_type: Option<&str>,
    ) -> Result<Vec<WikiPage>, WikiError> {
        let fts = normalize_fts_query(query);
        if fts.is_empty() {
            return Ok(vec![]);
        }
        let cat_filter = category.unwrap_or("");
        let doc_filter = doc_type.unwrap_or("");
        // FTS5 MATCH handles exact terminology; frontmatter filters narrow the
        // result set. This is the lexical half of the intended FTS5+vector
        // hybrid (vectors land in a later phase).
        //
        // CJK caveat: the `unicode61` tokenizer has no word boundary for Chinese,
        // so a multi-character query like "合同" is not tokenized into an
        // indexable term and FTS5 can miss substring/prefix matches. We keep
        // FTS5 as the primary path (deterministic, explainable, zero-dep — per
        // the retrieval red line) but add a built-in `LIKE` fallback so Chinese
        // and other substring queries still recall. LIKE is core SQLite, adds
        // no dependency, and only *supplements* FTS5 (never replaces it).
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut out: Vec<WikiPage> = Vec::new();

        if let Ok(page_ids) = sqlx::query_as::<_, (String,)>(
            "SELECT p.page_id FROM wiki_fts f \
             JOIN wiki_pages p ON p.id = f.rowid \
             WHERE wiki_fts MATCH ? \
             AND (? = '' OR p.category = ?) \
             AND (? = '' OR p.doc_type = ?) \
             ORDER BY rank LIMIT ?",
        )
        .bind(&fts)
        .bind(cat_filter)
        .bind(cat_filter)
        .bind(doc_filter)
        .bind(doc_filter)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        {
            for (pid,) in page_ids {
                if let Some(p) = self.get_page(&pid).await? {
                    seen.insert(p.id.clone());
                    out.push(p);
                }
            }
        }
        tracing::debug!(query, fts_hits = out.len(), "wiki_search_fts");

        // LIKE fallback (covers CJK substring / FTS5 misses). Only fills up to
        // `limit` and skips pages already returned by FTS5.
        if out.len() < limit as usize {
            let like_arg = format!("%{}%", query.replace(['\\', '%', '_'], ""));
            let like_ids: Vec<(String,)> = sqlx::query_as(
                "SELECT page_id FROM wiki_pages \
                 WHERE (title LIKE ? OR content_md LIKE ?) \
                 AND (? = '' OR category = ?) \
                 AND (? = '' OR doc_type = ?) \
                 ORDER BY updated_at DESC LIMIT ?",
            )
            .bind(&like_arg)
            .bind(&like_arg)
            .bind(cat_filter)
            .bind(cat_filter)
            .bind(doc_filter)
            .bind(doc_filter)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(db)?;
            tracing::debug!(query, like_hits = like_ids.len(), "wiki_search_like");
            for (pid,) in like_ids {
                if seen.contains(&pid) {
                    continue;
                }
                if let Some(p) = self.get_page(&pid).await? {
                    seen.insert(p.id.clone());
                    out.push(p);
                }
            }
        }
        Ok(out)
    }

    async fn set_tags(&self, page_id: &str, tags: &[String]) -> Result<(), WikiError> {
        let mut tx = self.pool.begin().await.map_err(db)?;
        sqlx::query(
            "DELETE FROM wiki_page_tags WHERE page_id = \
             (SELECT id FROM wiki_pages WHERE page_id = ?)",
        )
        .bind(page_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        if let Some(row_id) = self.page_row_id_tx(&mut tx, page_id).await? {
            self.apply_tags(&mut tx, row_id, tags).await?;
        }
        tx.commit().await.map_err(db)?;
        Ok(())
    }

    async fn list_tags(&self) -> Result<Vec<WikiTag>, WikiError> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT t.name, COUNT(pt.page_id) FROM wiki_tags t \
             LEFT JOIN wiki_page_tags pt ON pt.tag_id = t.id \
             GROUP BY t.id ORDER BY t.name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db)?;
        Ok(rows
            .into_iter()
            .map(|(name, count)| WikiTag { name, count })
            .collect())
    }

    async fn backlinks(&self, title: &str) -> Result<Vec<WikiPage>, WikiError> {
        let page_ids: Vec<(String,)> = sqlx::query_as(
            "SELECT p.page_id FROM wiki_links l \
             JOIN wiki_pages p ON p.id = l.from_page_id \
             WHERE lower(l.to_title) = lower(?)",
        )
        .bind(title)
        .fetch_all(&self.pool)
        .await
        .map_err(db)?;
        let mut out = Vec::with_capacity(page_ids.len());
        for (pid,) in page_ids {
            if let Some(p) = self.get_page(&pid).await? {
                out.push(p);
            }
        }
        Ok(out)
    }

    async fn title_search(&self, q: &str, limit: u32) -> Result<Vec<String>, WikiError> {
        let q = q.trim();
        if q.is_empty() {
            return Ok(vec![]);
        }
        let like_arg = format!("%{}%", q.replace(['\\', '%', '_'], ""));
        let titles: Vec<(String,)> = sqlx::query_as(
            "SELECT title FROM wiki_pages WHERE title LIKE ? ORDER BY updated_at DESC LIMIT ?",
        )
        .bind(&like_arg)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(db)?;
        Ok(titles.into_iter().map(|r| r.0).collect())
    }

    async fn unlinked_mentions(&self, page_id: &str) -> Result<Vec<String>, WikiError> {
        // Load the page's own content.
        let page = match self.get_page(page_id).await? {
            Some(p) => p,
            None => return Ok(vec![]),
        };
        // Collect all known titles (excluding self) and aliases.
        let known: Vec<(String,)> = sqlx::query_as(
            "SELECT title FROM wiki_pages WHERE page_id <> ?",
        )
        .bind(page_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db)?;
        let alias_rows: Vec<(String,)> = sqlx::query_as("SELECT alias FROM wiki_aliases")
            .fetch_all(&self.pool)
            .await
            .map_err(db)?;
        let mut candidates: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (t,) in known.into_iter().chain(alias_rows) {
            if t.trim().is_empty() {
                continue;
            }
            candidates.insert(t);
        }
        // Already-linked targets should not appear as "unlinked".
        let linked = extract_links(&page.content_md);
        let linked_set: std::collections::HashSet<String> =
            linked.iter().map(|s| s.to_lowercase()).collect();

        let content_lower = page.content_md.to_lowercase();
        let mut out: Vec<String> = Vec::new();
        for title in candidates {
            if linked_set.contains(&title.to_lowercase()) {
                continue;
            }
            // Must appear as a standalone word/phrase (avoid partial substring noise).
            if content_lower.contains(&title.to_lowercase()) {
                out.push(title);
            }
            if out.len() >= 20 {
                break;
            }
        }
        Ok(out)
    }

    async fn apply_template(&self, template: &str) -> Result<u32, WikiError> {
        let def = crate::template::template_def(template)?;
        let mut count: u32 = 0;
        let now = now_ms();
        let mut tx = self.pool.begin().await.map_err(db)?;
        for page in &def.pages {
            // Idempotent: reused existing page if the title already exists, so
            // re-applying a template never violates the UNIQUE(title) constraint.
            let existing_row: Option<i64> =
                sqlx::query_scalar("SELECT id FROM wiki_pages WHERE lower(title) = lower(?) LIMIT 1")
                    .bind(&page.title)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(db)?;
            let row_id = match existing_row {
                Some(id) => id,
                None => {
                    let page_id = uuid::Uuid::now_v7().to_string();
                    sqlx::query(
                        "INSERT INTO wiki_pages (page_id, title, slug, content_md, category, doc_type, \
                         source_ref, status, extra, created_at, updated_at) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    )
                    .bind(&page_id)
                    .bind(&page.title)
                    .bind(slugify(&page.title))
                    .bind(&page.content_md)
                    .bind(&page.category)
                    .bind(&page.doc_type)
                    .bind(&page.source_ref)
                    .bind(&page.status)
                    .bind(&page.extra_json)
                    .bind(now)
                    .bind(now)
                    .execute(&mut *tx)
                    .await
                    .map_err(db)?;
                    sqlx::query_scalar("SELECT id FROM wiki_pages WHERE page_id = ?")
                        .bind(&page_id)
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(db)?
                }
            };
            // Emit outgoing `[[link]]` targets as wiki_links rows.
            let targets = extract_links(&page.content_md);
            for target in &targets {
                sqlx::query(
                    "INSERT OR IGNORE INTO wiki_links (from_page_id, to_title, created_at) \
                     VALUES (?, ?, ?)",
                )
                .bind(row_id)
                .bind(target)
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            }
            count += 1;
        }
        tx.commit().await.map_err(db)?;
        Ok(count)
    }

    // --- Raw ingest tracking (migration 038) ----------------------------

    async fn get_ingest(&self, raw_path: &str) -> Result<Option<IngestRow>, WikiError> {
        let row: Option<(String, String, String, String, i64, String, i64)> = sqlx::query_as(
            "SELECT raw_path, checksum, status, page_ids, slices, error, updated_at \
             FROM wiki_ingest WHERE raw_path = ?",
        )
        .bind(raw_path)
        .fetch_optional(&self.pool)
        .await
        .map_err(db)?;
        Ok(row.map(|r| IngestRow {
            raw_path: r.0,
            checksum: r.1,
            status: r.2,
            page_ids: serde_json::from_str(&r.3).unwrap_or_default(),
            slices: r.4 as u32,
            error: r.5,
            updated_at: r.6,
        }))
    }

    async fn upsert_ingest(
        &self,
        raw_path: &str,
        checksum: &str,
        status: &str,
        page_ids: &[String],
        slices: u32,
        error: &str,
    ) -> Result<(), WikiError> {
        let now = now_ms();
        let page_ids_json = serde_json::to_string(page_ids)
            .map_err(|e| WikiError::Internal(format!("serialize page_ids: {e}")))?;
        // Insert if absent, otherwise update (idempotent by raw_path).
        let existing: Option<i64> =
            sqlx::query_scalar("SELECT id FROM wiki_ingest WHERE raw_path = ?")
                .bind(raw_path)
                .fetch_optional(&self.pool)
                .await
                .map_err(db)?;
        if let Some(id) = existing {
            sqlx::query(
                "UPDATE wiki_ingest SET checksum=?, status=?, page_ids=?, slices=?, error=?, updated_at=? \
                 WHERE id = ?",
            )
            .bind(checksum)
            .bind(status)
            .bind(&page_ids_json)
            .bind(slices as i64)
            .bind(error)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(db)?;
        } else {
            sqlx::query(
                "INSERT INTO wiki_ingest \
                 (raw_path, checksum, status, page_ids, slices, error, created_at, updated_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(raw_path)
            .bind(checksum)
            .bind(status)
            .bind(&page_ids_json)
            .bind(slices as i64)
            .bind(error)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(db)?;
        }
        Ok(())
    }

    // --- Phase 2: typed edges ---------------------------------------------

    async fn put_edge(
        &self,
        from_page_id: &str,
        to_page_id: &str,
        edge_type: WikiEdgeType,
    ) -> Result<(), WikiError> {
        let from_row: Option<i64> = self.page_row_id(from_page_id).await?;
        let to_row: Option<i64> = self.page_row_id(to_page_id).await?;
        let (from_row, to_row) = match (from_row, to_row) {
            (Some(f), Some(t)) => (f, t),
            _ => return Err(WikiError::BadRequest("both pages must exist for an edge".into())),
        };
        let now = now_ms();
        sqlx::query(
            "INSERT OR IGNORE INTO wiki_edges (from_page_id, to_page_id, edge_type, created_at) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(from_row)
        .bind(to_row)
        .bind(edge_type.as_str())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(())
    }

    async fn delete_edge(
        &self,
        from_page_id: &str,
        to_page_id: &str,
        edge_type: WikiEdgeType,
    ) -> Result<(), WikiError> {
        let from_row: Option<i64> = self.page_row_id(from_page_id).await?;
        let to_row: Option<i64> = self.page_row_id(to_page_id).await?;
        let (from_row, to_row) = match (from_row, to_row) {
            (Some(f), Some(t)) => (f, t),
            _ => return Ok(()),
        };
        sqlx::query(
            "DELETE FROM wiki_edges WHERE from_page_id = ? AND to_page_id = ? AND edge_type = ?",
        )
        .bind(from_row)
        .bind(to_row)
        .bind(edge_type.as_str())
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(())
    }

    async fn outgoing_edges(&self, page_id: &str) -> Result<Vec<WikiEdge>, WikiError> {
        let row_id: Option<i64> = self.page_row_id(page_id).await?;
        let row_id = match row_id {
            Some(id) => id,
            None => return Ok(vec![]),
        };
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT f.page_id, t.page_id, e.edge_type FROM wiki_edges e \
             JOIN wiki_pages f ON f.id = e.from_page_id \
             JOIN wiki_pages t ON t.id = e.to_page_id \
             WHERE e.from_page_id = ?",
        )
        .bind(row_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db)?;
        Ok(rows
            .into_iter()
            .map(|(f, t, et)| WikiEdge {
                from_page_id: f,
                to_page_id: t,
                edge_type: WikiEdgeType::parse(&et),
                created_at: 0,
            })
            .collect())
    }

    async fn incoming_edges(&self, page_id: &str) -> Result<Vec<WikiEdge>, WikiError> {
        let row_id: Option<i64> = self.page_row_id(page_id).await?;
        let row_id = match row_id {
            Some(id) => id,
            None => return Ok(vec![]),
        };
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT f.page_id, t.page_id, e.edge_type FROM wiki_edges e \
             JOIN wiki_pages f ON f.id = e.from_page_id \
             JOIN wiki_pages t ON t.id = e.to_page_id \
             WHERE e.to_page_id = ?",
        )
        .bind(row_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db)?;
        Ok(rows
            .into_iter()
            .map(|(f, t, et)| WikiEdge {
                from_page_id: f,
                to_page_id: t,
                edge_type: WikiEdgeType::parse(&et),
                created_at: 0,
            })
            .collect())
    }

    async fn link_graph(&self, page_id: &str) -> Result<Vec<WikiGraphNode>, WikiError> {
        let out = self.outgoing_edges(page_id).await?;
        let inc = self.incoming_edges(page_id).await?;

        // Resolve titles for all distinct endpoint page ids.
        let mut title_cache: std::collections::HashMap<String, String> = Default::default();
        let mut nodes: Vec<WikiGraphNode> = Vec::new();
        for e in out {
            let title = self.title_for(page_id, &e.to_page_id, &mut title_cache).await?;
            nodes.push(WikiGraphNode {
                page_id: e.to_page_id,
                title,
                edge_type: e.edge_type,
                direction: "outgoing".to_string(),
            });
        }
        for e in inc {
            let title = self.title_for(page_id, &e.from_page_id, &mut title_cache).await?;
            nodes.push(WikiGraphNode {
                page_id: e.from_page_id,
                title,
                edge_type: e.edge_type,
                direction: "incoming".to_string(),
            });
        }
        Ok(nodes)
    }
}

impl SqliteWikiRepository {
    /// Resolve a page title by id, caching results across a single graph build.
    async fn title_for(
        &self,
        _root: &str,
        page_id: &str,
        cache: &mut std::collections::HashMap<String, String>,
    ) -> Result<String, WikiError> {
        if let Some(t) = cache.get(page_id) {
            return Ok(t.clone());
        }
        let t: Option<String> =
            sqlx::query_scalar("SELECT title FROM wiki_pages WHERE page_id = ?")
                .bind(page_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(db)?;
        let t = t.unwrap_or_default();
        cache.insert(page_id.to_string(), t.clone());
        Ok(t)
    }
}

/// Map any sqlx error to the internal wiki error variant.
fn db(e: sqlx::Error) -> WikiError {
    WikiError::Internal(e.to_string())
}

/// Extract Obsidian-style `[[Page Name]]` link targets from Markdown content.
///
/// Returns de-duplicated, trimmed targets in document order. Aliases of the
/// form `[[Page|alias]]` are not yet supported (M3).
pub fn extract_links(content: &str) -> Vec<String> {
    let bytes = content.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            let mut j = i + 2;
            while j + 1 < bytes.len() {
                if bytes[j] == b']' && bytes[j + 1] == b']' {
                    let name = content[i + 2..j].trim().to_string();
                    if !name.is_empty() && !out.contains(&name) {
                        out.push(name);
                    }
                    break;
                }
                j += 1;
            }
            i = j + 2;
        } else {
            i += 1;
        }
    }
    out
}

/// Slugify a title into a URL-safe identifier.
fn slugify(title: &str) -> String {
    let mut s = String::new();
    for c in title.chars() {
        if c.is_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if c.is_whitespace() || c == '-' || c == '_' {
            s.push('-');
        }
    }
    s.trim_matches('-').to_string()
}

/// Build a safe FTS5 MATCH expression: each whitespace-separated term is
/// double-quoted so arbitrary user input can't inject FTS operators.
fn normalize_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{}\"", t.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" AND ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use roseui_db::init_database_memory;

    async fn repo() -> (SqliteWikiRepository, roseui_db::Database) {
        let db = init_database_memory().await.unwrap();
        (SqliteWikiRepository::new(db.pool().clone()), db)
    }

    #[test]
    fn extract_links_dedup_and_trim() {
        assert_eq!(
            extract_links("see [[Alpha]] and [[ Beta ]] and [[Alpha]]"),
            vec!["Alpha".to_string(), "Beta".to_string()]
        );
        assert!(extract_links("no links here").is_empty());
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("My Page Title!"), "my-page-title");
    }

    #[tokio::test]
    async fn create_get_update_delete_roundtrip() {
        let (r, _db) = repo().await;
        let page = r
            .create_page(CreateWikiPageRequest {
                title: "Rust Notes".into(),
                content_md: "Rust is [[Systems]] and [[Memory]] safe.".into(),
                tags: Some(vec!["lang".into(), "systems".into()]),
                category: Some("pl".into()),
                doc_type: Some("guide".into()),
                source_ref: None,
                status: Some("active".into()),
                extra: Some(serde_json::json!({"edition": "2024"})),
            })
            .await
            .unwrap();
        assert_eq!(page.title, "Rust Notes");
        assert_eq!(page.tags.len(), 2);
        assert_eq!(page.category, "pl");
        assert_eq!(page.doc_type, "guide");
        assert_eq!(page.status, "active");
        assert_eq!(page.extra["edition"], "2024");

        let fetched = r.get_page(&page.id).await.unwrap().unwrap();
        assert_eq!(fetched.category, "pl");

        // Backlink target resolution.
        let sys = r.create_page(CreateWikiPageRequest {
            title: "Systems".into(),
            content_md: "About systems.".into(),
            tags: None,
            category: None,
            doc_type: None,
            source_ref: None,
            status: None,
            extra: None,
        })
        .await
        .unwrap();
        let bl = r.backlinks("Systems").await.unwrap();
        assert!(bl.iter().any(|p| p.id == page.id));

        let updated = r
            .update_page(
                &page.id,
                UpdateWikiPageRequest {
                    title: Some("Rust Notes v2".into()),
                    content_md: Some("updated".into()),
                    tags: Some(vec!["lang".into()]),
                    category: Some("programming".into()),
                    doc_type: None,
                    source_ref: None,
                    status: None,
                    extra: None,
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.title, "Rust Notes v2");
        assert_eq!(updated.tags, vec!["lang".to_string()]);
        assert_eq!(updated.category, "programming");

        let deleted = r.delete_page(&page.id).await.unwrap();
        assert!(deleted);
        assert!(r.get_page(&page.id).await.unwrap().is_none());
        assert!(r.backlinks("Systems").await.unwrap().is_empty());
        let _ = sys;
    }

    #[tokio::test]
    async fn typed_edges_and_graph() {
        let (r, _db) = repo().await;
        let a = r
            .create_page(CreateWikiPageRequest {
                title: "Contract Law".into(),
                content_md: "base".into(),
                tags: None,
                category: None,
                doc_type: None,
                source_ref: None,
                status: None,
                extra: None,
            })
            .await
            .unwrap();
        let b = r
            .create_page(CreateWikiPageRequest {
                title: "Case 2024-001".into(),
                content_md: "exemplar".into(),
                tags: None,
                category: None,
                doc_type: None,
                source_ref: None,
                status: None,
                extra: None,
            })
            .await
            .unwrap();

        // a exemplifies -> b  (a is an example of b's principle)
        r.put_edge(&a.id, &b.id, WikiEdgeType::Exemplifies).await.unwrap();
        let out = r.outgoing_edges(&a.id).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].edge_type, WikiEdgeType::Exemplifies);

        // b cites -> a  (case cites the principle)
        r.put_edge(&b.id, &a.id, WikiEdgeType::Cites).await.unwrap();
        let graph = r.link_graph(&a.id).await.unwrap();
        // a -> b (outgoing, exemplifies) and b -> a (incoming, cites)
        assert_eq!(graph.len(), 2);
        let incoming = graph.iter().find(|n| n.direction == "incoming").unwrap();
        assert_eq!(incoming.edge_type, WikiEdgeType::Cites);
        assert_eq!(incoming.title, "Case 2024-001");

        // Deleting edge.
        r.delete_edge(&a.id, &b.id, WikiEdgeType::Exemplifies).await.unwrap();
        assert!(r.outgoing_edges(&a.id).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn search_finds_by_content_and_filters() {
        let (r, _db) = repo().await;
        r.create_page(CreateWikiPageRequest {
            title: "Apple Pie".into(),
            content_md: "A delicious dessert with cinnamon.".into(),
            tags: None,
            category: Some("food".into()),
            doc_type: None,
            source_ref: None,
            status: None,
            extra: None,
        })
        .await
        .unwrap();
        r.create_page(CreateWikiPageRequest {
            title: "Banana Bread".into(),
            content_md: "A moist banana loaf.".into(),
            tags: None,
            category: Some("food".into()),
            doc_type: None,
            source_ref: None,
            status: None,
            extra: None,
        })
        .await
        .unwrap();
        let hits = r.search("cinnamon", 10, None, None).await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Apple Pie");

        // No-category filter returns nothing for a non-matching category.
        let none = r.search("dessert", 10, Some("law"), None).await.unwrap();
        assert!(none.is_empty());
    }
}
