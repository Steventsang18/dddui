use std::sync::Arc;

use crate::error::WikiError;
use crate::repository::IWikiRepository;
use crate::types::{
    CreateWikiPageRequest, UpdateWikiPageRequest, WikiEdge, WikiEdgeType, WikiGraphNode, WikiPage,
    WikiSearchRequest, WikiSearchResponse, WikiTag,
};

/// Wiki domain service.
///
/// Thin delegation over [`IWikiRepository`]; keeps the router handlers free of
/// SQL. The pool is threaded in from `roseui-app` (migrations `034_wiki.sql` and
/// `035_wiki_frontmatter_edges.sql`).
#[derive(Clone)]
pub struct WikiService {
    repo: Arc<dyn IWikiRepository>,
}

impl WikiService {
    pub fn new(repo: Arc<dyn IWikiRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_page(&self, req: CreateWikiPageRequest) -> Result<WikiPage, WikiError> {
        self.repo.create_page(req).await
    }

    pub async fn get_page(&self, page_id: &str) -> Result<Option<WikiPage>, WikiError> {
        self.repo.get_page(page_id).await
    }

    pub async fn get_page_by_title(&self, title: &str) -> Result<Option<WikiPage>, WikiError> {
        self.repo.get_page_by_title(title).await
    }

    pub async fn update_page(
        &self,
        page_id: &str,
        req: UpdateWikiPageRequest,
    ) -> Result<Option<WikiPage>, WikiError> {
        self.repo.update_page(page_id, req).await
    }

    pub async fn delete_page(&self, page_id: &str) -> Result<bool, WikiError> {
        self.repo.delete_page(page_id).await
    }

    pub async fn list_pages(&self, limit: u32, offset: u32) -> Result<Vec<WikiPage>, WikiError> {
        self.repo.list_pages(limit, offset).await
    }

    pub async fn search(&self, req: WikiSearchRequest) -> Result<WikiSearchResponse, WikiError> {
        let limit = req.limit.unwrap_or(20).clamp(1, 100);
        let hits = self
            .repo
            .search(&req.query, limit, req.category.as_deref(), req.doc_type.as_deref())
            .await?;
        Ok(WikiSearchResponse { hits })
    }

    pub async fn set_tags(&self, page_id: &str, tags: &[String]) -> Result<(), WikiError> {
        self.repo.set_tags(page_id, tags).await
    }

    pub async fn list_tags(&self) -> Result<Vec<WikiTag>, WikiError> {
        self.repo.list_tags().await
    }

    pub async fn backlinks(&self, title: &str) -> Result<Vec<WikiPage>, WikiError> {
        self.repo.backlinks(title).await
    }

    pub async fn title_search(&self, q: &str, limit: u32) -> Result<Vec<String>, WikiError> {
        self.repo.title_search(q, limit).await
    }

    pub async fn unlinked_mentions(&self, page_id: &str) -> Result<Vec<String>, WikiError> {
        self.repo.unlinked_mentions(page_id).await
    }

    pub async fn apply_template(&self, template: &str) -> Result<u32, WikiError> {
        self.repo.apply_template(template).await
    }

    // --- Raw ingest (read-only source -> wiki pages) --------------------
    //
    // All ingest operations live on `IngestEngine` (see `WikiRouterState.
    // ingest`), which owns the read-only raw directory and the wiki repository.
    // Routes call `state.ingest` directly rather than going through the
    // service, because the engine needs the data directory that the service
    // does not carry.

    // --- Phase 2: typed edges + graph --------------------------------------

    pub async fn put_edge(
        &self,
        from_page_id: &str,
        to_page_id: &str,
        edge_type: WikiEdgeType,
    ) -> Result<(), WikiError> {
        self.repo.put_edge(from_page_id, to_page_id, edge_type).await
    }

    pub async fn delete_edge(
        &self,
        from_page_id: &str,
        to_page_id: &str,
        edge_type: WikiEdgeType,
    ) -> Result<(), WikiError> {
        self.repo
            .delete_edge(from_page_id, to_page_id, edge_type)
            .await
    }

    pub async fn outgoing_edges(&self, page_id: &str) -> Result<Vec<WikiEdge>, WikiError> {
        self.repo.outgoing_edges(page_id).await
    }

    pub async fn incoming_edges(&self, page_id: &str) -> Result<Vec<WikiEdge>, WikiError> {
        self.repo.incoming_edges(page_id).await
    }

    pub async fn link_graph(&self, page_id: &str) -> Result<Vec<WikiGraphNode>, WikiError> {
        self.repo.link_graph(page_id).await
    }

    /// Liveness probe for `GET /api/wiki/health`.
    pub fn health(&self) -> Result<(), WikiError> {
        Ok(())
    }

    /// Build a precise citation string for a page (see [`crate::cite`]).
    pub async fn cite(
        &self,
        page_id: &str,
        anchor: Option<&str>,
        style: &str,
    ) -> Result<String, WikiError> {
        let page = self
            .repo
            .get_page(page_id)
            .await?
            .ok_or_else(|| WikiError::NotFound(page_id.to_string()))?;
        Ok(crate::cite::build_citation(&page, anchor, style))
    }
}
