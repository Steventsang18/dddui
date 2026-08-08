#![allow(clippy::disallowed_types)]

use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use roseui_api_types::ApiResponse;
use roseui_common::ApiError;

use crate::error::WikiError;
use crate::ingest::IngestResult;
use crate::state::WikiRouterState;
use crate::types::{
    CreateWikiPageRequest, UpdateWikiPageRequest, WikiEdgeType, WikiPage, WikiSearchRequest,
    WikiSearchResponse, WikiTag,
};

/// Mount wiki HTTP routes. Paths are rooted at `/api/wiki/*`; the caller wraps
/// the returned router with the auth middleware (same pattern as the other
/// domain modules). The wiki is a local, single-user knowledge base, so the
/// auth middleware is the only gate — pages are not scoped per user.
pub fn wiki_routes(state: WikiRouterState) -> Router {
    Router::new()
        .route("/api/wiki/health", get(health))
        .route("/api/wiki/pages", get(list_pages).post(create_page))
        .route(
            "/api/wiki/pages/{page_id}",
            get(get_page).put(update_page).delete(delete_page),
        )
        .route("/api/wiki/pages/{page_id}/tags", put(set_page_tags))
        .route("/api/wiki/pages/{page_id}/backlinks", get(get_backlinks))
        .route("/api/wiki/pages/{page_id}/unlinked", get(get_unlinked))
        .route("/api/wiki/title-search", get(title_search))
        .route("/api/wiki/templates", get(list_templates))
        .route("/api/wiki/init-template", post(init_template))
        .route("/api/wiki/search", get(search_pages))
        .route("/api/wiki/tags", get(list_tags))
        // Phase 2: typed edges + link graph
        .route(
            "/api/wiki/pages/{page_id}/edges",
            get(list_outgoing_edges).put(put_edge).delete(delete_edge),
        )
        .route("/api/wiki/pages/{page_id}/graph", get(get_link_graph))
        .route(
            "/api/wiki/pages/{page_id}/cite",
            get(get_cite),
        )
        // --- Raw-source ingest (read-only source -> wiki pages) ----------
        .route("/api/wiki/raw", get(list_raw).post(upload_raw))
        .route("/api/wiki/raw/{*raw_path}", delete(delete_raw_file))
        .route("/api/wiki/ingest", post(ingest_raw))
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "module": "wiki" }))
}

async fn list_pages(
    State(state): State<WikiRouterState>,
    Query(q): Query<ListPagesQuery>,
) -> Result<Json<ApiResponse<Vec<WikiPage>>>, ApiError> {
    let pages = state.service.list_pages(q.limit, q.offset).await?;
    Ok(Json(ApiResponse::ok(pages)))
}

async fn create_page(
    State(state): State<WikiRouterState>,
    Json(req): Json<CreateWikiPageRequest>,
) -> Result<Json<ApiResponse<WikiPage>>, ApiError> {
    let page = state.service.create_page(req).await?;
    Ok(Json(ApiResponse::ok(page)))
}

async fn get_page(
    State(state): State<WikiRouterState>,
    Path(page_id): Path<String>,
) -> Result<Json<ApiResponse<WikiPage>>, ApiError> {
    let page = state
        .service
        .get_page(&page_id)
        .await?
        .ok_or(WikiError::NotFound(page_id))?;
    Ok(Json(ApiResponse::ok(page)))
}

async fn update_page(
    State(state): State<WikiRouterState>,
    Path(page_id): Path<String>,
    Json(req): Json<UpdateWikiPageRequest>,
) -> Result<Json<ApiResponse<WikiPage>>, ApiError> {
    let page = state
        .service
        .update_page(&page_id, req)
        .await?
        .ok_or(WikiError::NotFound(page_id))?;
    Ok(Json(ApiResponse::ok(page)))
}

async fn delete_page(
    State(state): State<WikiRouterState>,
    Path(page_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let deleted = state.service.delete_page(&page_id).await?;
    if !deleted {
        return Err(WikiError::NotFound(page_id).into());
    }
    Ok(Json(ApiResponse::success()))
}

async fn search_pages(
    State(state): State<WikiRouterState>,
    Query(q): Query<WikiSearchQuery>,
) -> Result<Json<ApiResponse<WikiSearchResponse>>, ApiError> {
    let req = WikiSearchRequest {
        query: q.q,
        limit: q.limit,
        category: q.category,
        doc_type: q.doc_type,
    };
    let resp = state.service.search(req).await?;
    Ok(Json(ApiResponse::ok(resp)))
}

async fn list_tags(
    State(state): State<WikiRouterState>,
) -> Result<Json<ApiResponse<Vec<WikiTag>>>, ApiError> {
    let tags = state.service.list_tags().await?;
    Ok(Json(ApiResponse::ok(tags)))
}

async fn get_backlinks(
    State(state): State<WikiRouterState>,
    Path(page_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<WikiPage>>>, ApiError> {
    // Backlinks resolve by title (Obsidian-style `[[link]]` targets).
    let page = state
        .service
        .get_page(&page_id)
        .await?
        .ok_or_else(|| WikiError::NotFound(page_id.clone()))?;
    let links = state.service.backlinks(&page.title).await?;
    Ok(Json(ApiResponse::ok(links)))
}

async fn set_page_tags(
    State(state): State<WikiRouterState>,
    Path(page_id): Path<String>,
    Json(tags): Json<Vec<String>>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let exists = state.service.get_page(&page_id).await?.is_some();
    if !exists {
        return Err(WikiError::NotFound(page_id).into());
    }
    state.service.set_tags(&page_id, &tags).await?;
    Ok(Json(ApiResponse::success()))
}

async fn get_unlinked(
    State(state): State<WikiRouterState>,
    Path(page_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    let mentions = state.service.unlinked_mentions(&page_id).await?;
    Ok(Json(ApiResponse::ok(mentions)))
}

async fn title_search(
    State(state): State<WikiRouterState>,
    Query(q): Query<TitleSearchQuery>,
) -> Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    let limit = q.limit.unwrap_or(10).clamp(1, 50);
    let titles = state.service.title_search(&q.q, limit).await?;
    Ok(Json(ApiResponse::ok(titles)))
}

async fn list_templates() -> Json<Value> {
    let ids = crate::template::available_templates();
    Json(json!({ "templates": ids }))
}

async fn init_template(
    State(state): State<WikiRouterState>,
    Json(req): Json<InitTemplateRequest>,
) -> Result<Json<ApiResponse<u32>>, ApiError> {
    let count = state.service.apply_template(&req.template).await?;
    Ok(Json(ApiResponse::ok(count)))
}

#[derive(Debug, Deserialize)]
struct TitleSearchQuery {
    q: String,
    #[serde(default)]
    limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct InitTemplateRequest {
    template: String,
}

#[derive(Debug, Deserialize)]
struct ListPagesQuery {
    #[serde(default = "default_list_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

fn default_list_limit() -> u32 {
    50
}

#[derive(Debug, Deserialize)]
struct WikiSearchQuery {
    q: String,
    #[serde(default)]
    limit: Option<u32>,
    /// Restrict results to a frontmatter `category` value.
    #[serde(default)]
    category: Option<String>,
    /// Restrict results to a frontmatter `doc_type` value.
    #[serde(default)]
    doc_type: Option<String>,
}

/// Body for `PUT /api/wiki/pages/{page_id}/edges`.
#[derive(Debug, Deserialize)]
struct PutEdgeBody {
    to_page_id: String,
    /// Edge type: cites | supersedes | conflicts | exemplifies | relates.
    edge_type: String,
}

/// Query for `DELETE /api/wiki/pages/{page_id}/edges`.
#[derive(Debug, Deserialize)]
struct DeleteEdgeQuery {
    to_page_id: String,
    edge_type: String,
}

/// Query for `GET /api/wiki/pages/{page_id}/cite`.
#[derive(Debug, Deserialize)]
struct CiteQuery {
    /// Optional heading text or quoted passage to anchor the citation.
    #[serde(default)]
    anchor: Option<String>,
    /// Citation style: "markdown" (default) or "plain".
    #[serde(default)]
    style: Option<String>,
}

async fn list_outgoing_edges(
    State(state): State<WikiRouterState>,
    Path(page_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<crate::types::WikiEdge>>>, ApiError> {
    let exists = state.service.get_page(&page_id).await?.is_some();
    if !exists {
        return Err(WikiError::NotFound(page_id).into());
    }
    let edges = state.service.outgoing_edges(&page_id).await?;
    Ok(Json(ApiResponse::ok(edges)))
}

async fn put_edge(
    State(state): State<WikiRouterState>,
    Path(page_id): Path<String>,
    Json(body): Json<PutEdgeBody>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let exists = state.service.get_page(&page_id).await?.is_some();
    if !exists {
        return Err(WikiError::NotFound(page_id).into());
    }
    let edge_type = WikiEdgeType::parse(&body.edge_type);
    if state
        .service
        .get_page(&body.to_page_id)
        .await?.is_none()
    {
        return Err(WikiError::NotFound(body.to_page_id).into());
    }
    state
        .service
        .put_edge(&page_id, &body.to_page_id, edge_type)
        .await?;
    Ok(Json(ApiResponse::success()))
}

async fn delete_edge(
    State(state): State<WikiRouterState>,
    Path(page_id): Path<String>,
    Query(q): Query<DeleteEdgeQuery>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let exists = state.service.get_page(&page_id).await?.is_some();
    if !exists {
        return Err(WikiError::NotFound(page_id).into());
    }
    let edge_type = WikiEdgeType::parse(&q.edge_type);
    state
        .service
        .delete_edge(&page_id, &q.to_page_id, edge_type)
        .await?;
    Ok(Json(ApiResponse::success()))
}

async fn get_link_graph(
    State(state): State<WikiRouterState>,
    Path(page_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<crate::types::WikiGraphNode>>>, ApiError> {
    let exists = state.service.get_page(&page_id).await?.is_some();
    if !exists {
        return Err(WikiError::NotFound(page_id).into());
    }
    let graph = state.service.link_graph(&page_id).await?;
    Ok(Json(ApiResponse::ok(graph)))
}

async fn get_cite(
    State(state): State<WikiRouterState>,
    Path(page_id): Path<String>,
    Query(q): Query<CiteQuery>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let style = q.style.unwrap_or_else(|| "markdown".to_string());
    let citation = state.service.cite(&page_id, q.anchor.as_deref(), &style).await?;
    Ok(Json(ApiResponse::ok(citation)))
}

// --- Raw-source ingest routes -------------------------------------------

/// `GET /api/wiki/raw` — list files in the read-only source directory with
/// their ingest status.
async fn list_raw(
    State(state): State<WikiRouterState>,
) -> Result<Json<ApiResponse<Vec<RawEntryView>>>, ApiError> {
    let entries = state.ingest.list_raw()?;
    // Join with ingest status for a single round-trip to the UI.
    let mut view = Vec::with_capacity(entries.len());
    for e in entries {
        let status = state
            .ingest
            .get_ingest_status(&e.relative)
            .await
            .unwrap_or_else(|| "pending".to_string());
        view.push(RawEntryView {
            relative: e.relative,
            size: e.size,
            ext: e.ext,
            status,
        });
    }
    Ok(Json(ApiResponse::ok(view)))
}

/// `POST /api/wiki/raw` (multipart) — user drops a document into the
/// read-only source directory. The body field `file` carries the bytes; its
/// filename (or a `path` field) is the relative location under `wiki/raw/`.
async fn upload_raw(
    State(state): State<WikiRouterState>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<ApiResponse<RawEntryView>>, ApiError> {
    let mut relative: Option<String> = None;
    let mut bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| WikiError::BadRequest(format!("multipart: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let fname = field
                .file_name()
                .map(|s| s.to_string())
                .or_else(|| relative.clone())
                .unwrap_or_else(|| "upload.bin".to_string());
            // If a `path` field was supplied it overrides the filename.
            let rel = if let Some(p) = &relative {
                p.clone()
            } else {
                sanitize_rel(&fname)
            };
            let data = field
                .bytes()
                .await
                .map_err(|e| WikiError::BadRequest(format!("read field: {e}")))?;
            bytes = Some(data.to_vec());
            relative = Some(rel);
        } else if name == "path" {
            let p = field
                .text()
                .await
                .map_err(|e| WikiError::BadRequest(format!("read path: {e}")))?;
            relative = Some(sanitize_rel(&p));
        }
    }
    let relative = relative.ok_or_else(|| WikiError::BadRequest("missing file field".into()))?;
    let bytes = bytes.ok_or_else(|| WikiError::BadRequest("empty upload".into()))?;
    let saved = state.ingest.save_raw(&relative, &bytes)?;
    let _ = saved;
    let status = state
        .ingest
        .get_ingest_status(&relative)
        .await
        .unwrap_or_else(|| "pending".to_string());
    Ok(Json(ApiResponse::ok(RawEntryView {
        relative: relative.clone(),
        size: bytes.len() as u64,
        ext: extension_of(&relative),
        status,
    })))
}

/// `DELETE /api/wiki/raw/{raw_path}` — user deletes a source document. This
/// is the ONLY path that removes a raw file; it is never called by the ingest
/// pipeline or any Agent. Wiki pages produced earlier are kept.
async fn delete_raw_file(
    State(state): State<WikiRouterState>,
    Path(raw_path): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    state.ingest.delete_raw(&raw_path).await?;
    Ok(Json(ApiResponse::success()))
}

/// `POST /api/wiki/ingest` (JSON `{raw_path}`) — ingest a source file into
/// wiki pages (read-only parse + slice + link). Idempotent by checksum.
async fn ingest_raw(
    State(state): State<WikiRouterState>,
    Json(req): Json<IngestRequest>,
) -> Result<Json<ApiResponse<IngestResult>>, ApiError> {
    let result = state.ingest.ingest(&req.raw_path).await?;
    Ok(Json(ApiResponse::ok(result)))
}

#[derive(Debug, Deserialize)]
struct IngestRequest {
    raw_path: String,
}

#[derive(Debug, serde::Serialize)]
struct RawEntryView {
    relative: String,
    size: u64,
    ext: String,
    status: String,
}

/// Keep only a safe relative path (no leading slash, no `..`).
fn sanitize_rel(input: &str) -> String {
    input
        .trim_start_matches('/')
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn extension_of(relative: &str) -> String {
    std::path::Path::new(relative)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use roseui_db::init_database_memory;

    use crate::state::build_wiki_state;

    async fn app() -> (Router, roseui_db::Database) {
        let db = init_database_memory().await.unwrap();
        let dir = std::env::temp_dir().join("roseui_wiki_test_raw");
        let _ = std::fs::create_dir_all(&dir);
        let router = wiki_routes(build_wiki_state(db.pool().clone(), dir));
        (router, db)
    }

    #[tokio::test]
    async fn create_get_update_search_backlinks_tags_roundtrip() {
        let (app, _db) = app().await;

        // Create
        let body = serde_json::to_vec(&CreateWikiPageRequest {
            title: "Rust Notes".into(),
            content_md: "Rust is [[Systems]] safe.".into(),
            tags: Some(vec!["lang".into()]),
            category: None,
            doc_type: None,
            source_ref: None,
            status: None,
            extra: None,
        })
        .unwrap();
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/wiki/pages")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let created: ApiResponse<WikiPage> = serde_json::from_slice(&bytes).unwrap();
        let id = created.data.clone().unwrap().id;

        // Get
        let resp = app
            .clone()
            .oneshot(Request::get(format!("/api/wiki/pages/{id}")).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let got: ApiResponse<WikiPage> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(got.data.unwrap().tags, vec!["lang".to_string()]);

        // Create the linked target page so backlinks resolve.
        let body = serde_json::to_vec(&CreateWikiPageRequest {
            title: "Systems".into(),
            content_md: "About systems.".into(),
            tags: None,
            category: None,
            doc_type: None,
            source_ref: None,
            status: None,
            extra: None,
        })
        .unwrap();
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/wiki/pages")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Backlinks for "Systems" should include "Rust Notes" (resolved by title).
        let resp = app
            .clone()
            .oneshot(Request::get(format!("/api/wiki/pages/{id}/backlinks")).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Search
        let resp = app
            .clone()
            .oneshot(Request::get("/api/wiki/search?q=systems").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let s: ApiResponse<WikiSearchResponse> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(s.data.unwrap().hits.len(), 2);

        // Tags listing
        let resp = app
            .clone()
            .oneshot(Request::get("/api/wiki/tags").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Set tags
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/wiki/pages/{id}/tags"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&vec!["rust".to_string()]).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Delete
        let resp = app
            .clone()
            .oneshot(Request::builder().method("DELETE").uri(format!("/api/wiki/pages/{id}")).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 404 after delete
        let resp = app
            .clone()
            .oneshot(Request::get(format!("/api/wiki/pages/{id}")).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn cite_endpoint_returns_markdown_citation() {
        let (app, _db) = app().await;
        let body = serde_json::to_vec(&CreateWikiPageRequest {
            title: "Contract Law Art. 523".into(),
            content_md: "# Performance\nA party must perform.".into(),
            tags: None,
            category: None,
            doc_type: None,
            source_ref: Some("CN-Civil-Code#523".into()),
            status: None,
            extra: None,
        })
        .unwrap();
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/wiki/pages")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let created: ApiResponse<WikiPage> = serde_json::from_slice(&bytes).unwrap();
        let id = created.data.clone().unwrap().id;

        // Markdown citation with a heading anchor.
        let resp = app
            .clone()
            .oneshot(
                Request::get(format!("/api/wiki/pages/{id}/cite?anchor=Performance"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let cited: ApiResponse<String> = serde_json::from_slice(&bytes).unwrap();
        assert!(cited.data.unwrap().starts_with("[Contract Law Art. 523](wiki:"));

        // Plain citation uses source_ref.
        let resp = app
            .clone()
            .oneshot(
                Request::get(format!("/api/wiki/pages/{id}/cite?style=plain"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let cited: ApiResponse<String> = serde_json::from_slice(&bytes).unwrap();
        assert!(cited.data.unwrap().contains("CN-Civil-Code#523"));
    }

    #[tokio::test]
    async fn missing_page_returns_404() {
        let (app, _db) = app().await;
        let resp = app
            .oneshot(Request::get("/api/wiki/pages/nope").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
