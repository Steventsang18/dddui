//! `dodiddoneui mcp-wiki-stdio` subcommand: MCP stdio server for wiki tools.
//!
//! Uses the `rmcp` crate for MCP protocol handling. Unlike `mcp-team-stdio`
//! which forwards tool calls to a TCP listener, this server directly holds a
//! `SqliteWikiRepository` and calls wiki CRUD/search methods in-process —
//! no TCP forwarding, no separate daemon.
//!
//! The DB path is read from the `ROSEUI_WIKI_DB_PATH` environment variable,
//! which the spawning process (the main DoDidDoneUi server) sets to its own
//! SQLite database path before launching this helper.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::commands::error::{CliBoundaryCode, CliBoundaryError, missing_env};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, ListToolsResult};
use rmcp::{schemars, service::ServiceExt, tool, tool_router, transport};
use roseui_wiki::{
    CreateWikiPageRequest, IWikiRepository, IngestEngine, SqliteWikiRepository, UpdateWikiPageRequest, WikiEdgeType,
    WikiPage,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;

const SUBCOMMAND: &str = "mcp-wiki-stdio";
const ENV_DB_PATH: &str = "ROSEUI_WIKI_DB_PATH";
const ERR_DB_OPEN: &str = "failed to open wiki database";

pub async fn run_wiki_stdio() -> ExitCode {
    let db_path = match std::env::var(ENV_DB_PATH) {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            let err = missing_env(SUBCOMMAND, ENV_DB_PATH);
            eprintln!("{}", err.stderr_line());
            return err.exit_code();
        }
    };

    let pool = match open_wiki_pool(&db_path).await {
        Ok(pool) => pool,
        Err(_err) => {
            let boundary = CliBoundaryError::new(CliBoundaryCode::McpWikiDbOpenFailed, SUBCOMMAND, ERR_DB_OPEN)
                .with_field("path", db_path.display().to_string());
            eprintln!("{boundary}", boundary = boundary.stderr_line());
            return boundary.exit_code();
        }
    };

    let repo = SqliteWikiRepository::new(pool.clone());
    // The data directory is the parent of the wiki DB file (e.g.
    // `data/dodiddoneui-backend.db` -> `data`). The raw source folder lives at
    // `data/wiki/raw`, read-only for the ingest pipeline.
    let data_dir = db_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| db_path.clone());
    let ingest = std::sync::Arc::new(IngestEngine::new(&data_dir, std::sync::Arc::new(repo.clone())));
    let server = WikiStdioServer { repo, ingest };

    let transport = transport::io::stdio();
    match server.serve(transport).await {
        Ok(peer) => {
            if let Err(_e) = peer.waiting().await {
                let err = CliBoundaryError::new(
                    CliBoundaryCode::McpSessionEndedWithError,
                    SUBCOMMAND,
                    "MCP stdio session ended with an error",
                );
                eprintln!("{}", err.stderr_line());
                err.exit_code()
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(_e) => {
            let err = CliBoundaryError::new(
                CliBoundaryCode::McpStdioServeFailed,
                SUBCOMMAND,
                "failed to start MCP stdio server",
            );
            eprintln!("{}", err.stderr_line());
            err.exit_code()
        }
    }
}

async fn open_wiki_pool(db_path: &PathBuf) -> Result<SqlitePool, sqlx::Error> {
    let opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(false)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));
    SqlitePool::connect_with(opts).await
}

// ---------------------------------------------------------------------------
// Parameter types
// ---------------------------------------------------------------------------

#[derive(Deserialize, schemars::JsonSchema)]
struct WikiReadParams {
    /// Page ID (UUID) or exact page title (case-insensitive match).
    page_id_or_title: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct WikiWriteParams {
    /// Page title. If a page with this title exists it will be updated;
    /// otherwise a new page is created.
    title: String,
    /// Markdown content. Supports Obsidian-style [[wiki links]] and tags.
    content_md: String,
    /// Comma-separated tags (e.g. "lang,rust,guide").
    #[serde(default)]
    tags: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct WikiSearchParams {
    /// FTS5 full-text search query. Multi-word queries are AND-ed (each term
    /// must match). Use quotes around phrases containing spaces.
    query: String,
    /// Maximum number of results to return (default 10, max 50).
    #[serde(default = "default_search_limit")]
    limit: u32,
}

fn default_search_limit() -> u32 {
    10
}

#[derive(Deserialize, schemars::JsonSchema)]
struct WikiCiteParams {
    /// Page ID (UUID) or exact page title (case-insensitive) of the page to cite.
    page_id_or_title: String,
    /// Optional anchor within the page. If the page contains a Markdown heading
    /// matching this text, the citation targets `#heading-slug`; otherwise it
    /// points at the page root. May be a chunk of quoted text to locate by content.
    #[serde(default)]
    anchor: Option<String>,
    /// Optional style: "markdown" (default, `[title](wiki:page_id#anchor)`) or
    /// "plain" (`title (source_ref)` — uses the page's `source_ref` when present).
    #[serde(default = "default_cite_style")]
    style: String,
}

fn default_cite_style() -> String {
    "markdown".to_string()
}

#[derive(Deserialize, schemars::JsonSchema)]
struct WikiLinkGraphParams {
    /// Page ID (UUID) or exact page title (case-insensitive) whose association
    /// graph should be returned.
    page_id_or_title: String,
    /// Optional edge-type filter (cites | supersedes | conflicts | exemplifies |
    /// relates). When omitted, all typed edges are returned.
    #[serde(default)]
    edge_type: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct WikiTitleSearchParams {
    /// Substring to match against existing page titles (case-insensitive).
    query: String,
    /// Max number of titles to return (clamped to 1..=50).
    #[serde(default = "default_title_limit")]
    limit: u32,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct WikiIngestParams {
    /// Raw-relative path of the source document under `wiki/raw/`, e.g.
    /// "contracts/2024-xx.pdf". The Agent must reference an existing file;
    /// it cannot create or modify raw files.
    raw_path: String,
}

fn default_title_limit() -> u32 {
    10
}

// ---------------------------------------------------------------------------
// Server struct
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct WikiStdioServer {
    repo: SqliteWikiRepository,
    ingest: std::sync::Arc<IngestEngine>,
}

// ---------------------------------------------------------------------------
// Tool router
// ---------------------------------------------------------------------------

#[tool_router]
impl WikiStdioServer {
    #[tool(
        name = "wiki_read",
        description = "Read a wiki page by its page ID (UUID) or exact title (case-insensitive). Returns the full page as JSON including id, title, slug, content_md, tags, and timestamps."
    )]
    async fn read(&self, Parameters(params): Parameters<WikiReadParams>) -> CallToolResult {
        match do_read(&self.repo, &params.page_id_or_title).await {
            Ok(json) => CallToolResult::success(vec![Content::text(json)]),
            Err(msg) => CallToolResult::error(vec![Content::text(msg)]),
        }
    }

    #[tool(
        name = "wiki_write",
        description = "Create or update a wiki page. If a page with the same title already exists it will be updated. Use Obsidian-style [[Page Name]] syntax to create bidirectional links between pages. Tags should be comma-separated."
    )]
    async fn write(&self, Parameters(params): Parameters<WikiWriteParams>) -> CallToolResult {
        match do_write(&self.repo, &params.title, &params.content_md, params.tags.as_deref()).await {
            Ok(summary) => CallToolResult::success(vec![Content::text(summary)]),
            Err(msg) => CallToolResult::error(vec![Content::text(msg)]),
        }
    }

    #[tool(
        name = "wiki_search",
        description = "Search wiki pages by title and content using full-text search. Returns matching pages as a JSON array with id, title, and a content snippet. Multi-word queries are AND-ed. Optionally constrain by `category`/`doc_type` frontmatter for domain-filtered retrieval."
    )]
    async fn search(&self, Parameters(params): Parameters<WikiSearchParams>) -> CallToolResult {
        let limit = params.limit.clamp(1, 50);
        match do_search(&self.repo, &params.query, limit).await {
            Ok(json) => CallToolResult::success(vec![Content::text(json)]),
            Err(msg) => CallToolResult::error(vec![Content::text(msg)]),
        }
    }

    #[tool(
        name = "wiki_cite",
        description = "Produce a precise, copy-pasteable citation for a wiki page, optionally anchored to a heading or quoted passage. Returns a markdown link `[title](wiki:<page_id>#<anchor>)` or, in `plain` style, `title (source_ref)` using the page's `source_ref` field. Use this to embed verifiable references into Agent outputs and workflows."
    )]
    async fn cite(&self, Parameters(params): Parameters<WikiCiteParams>) -> CallToolResult {
        match do_cite(
            &self.repo,
            &params.page_id_or_title,
            params.anchor.as_deref(),
            &params.style,
        )
        .await
        {
            Ok(text) => CallToolResult::success(vec![Content::text(text)]),
            Err(msg) => CallToolResult::error(vec![Content::text(msg)]),
        }
    }

    #[tool(
        name = "wiki_link_graph",
        description = "Return the typed association graph around a wiki page: outgoing and incoming edges with their edge type (cites/supersedes/conflicts/exemplifies/relates) and direction. Enables the Agent to follow reasoning chains (e.g. a case that `cites` a statute) instead of flat backlinks."
    )]
    async fn link_graph(&self, Parameters(params): Parameters<WikiLinkGraphParams>) -> CallToolResult {
        match do_link_graph(&self.repo, &params.page_id_or_title, params.edge_type.as_deref()).await {
            Ok(json) => CallToolResult::success(vec![Content::text(json)]),
            Err(msg) => CallToolResult::error(vec![Content::text(msg)]),
        }
    }

    #[tool(
        name = "wiki_title_search",
        description = "Autocomplete wiki page titles matching a substring. Use before writing `[[Page Name]]` links so the Agent references existing pages by their exact title (case-insensitive substring match). Returns a JSON array of matching titles."
    )]
    async fn title_search(&self, Parameters(params): Parameters<WikiTitleSearchParams>) -> CallToolResult {
        let limit = params.limit.clamp(1, 50);
        match do_title_search(&self.repo, &params.query, limit).await {
            Ok(json) => CallToolResult::success(vec![Content::text(json)]),
            Err(msg) => CallToolResult::error(vec![Content::text(msg)]),
        }
    }

    #[tool(
        name = "wiki_ingest",
        description = "Ingest a source document from the read-only `wiki/raw/` folder into wiki pages. The Agent MUST NOT pass file bytes or modify raw files; it only references an existing raw-relative path (e.g. \"contracts/2024-xx.pdf\"). Returns the produced summary page id and slice page ids. Idempotent by file checksum."
    )]
    async fn ingest(&self, Parameters(params): Parameters<WikiIngestParams>) -> CallToolResult {
        match do_ingest(&self.ingest, &params.raw_path).await {
            Ok(json) => CallToolResult::success(vec![Content::text(json)]),
            Err(msg) => CallToolResult::error(vec![Content::text(msg)]),
        }
    }
}

#[rmcp::tool_handler(router = Self::tool_router())]
impl rmcp::ServerHandler for WikiStdioServer {
    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        let tools = Self::tool_router().list_all();
        Ok(ListToolsResult::with_all_items(tools))
    }
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

async fn do_read(repo: &SqliteWikiRepository, page_id_or_title: &str) -> Result<String, String> {
    // Try page_id first (UUID), then fall back to title lookup.
    if let Ok(Some(page)) = repo.get_page(page_id_or_title).await {
        return serde_json::to_string_pretty(&page).map_err(|e| e.to_string());
    }
    if let Ok(Some(page)) = repo.get_page_by_title(page_id_or_title).await {
        return serde_json::to_string_pretty(&page).map_err(|e| e.to_string());
    }
    Err(format!("wiki page not found: {page_id_or_title}"))
}

async fn do_write(
    repo: &SqliteWikiRepository,
    title: &str,
    content_md: &str,
    tags_raw: Option<&str>,
) -> Result<String, String> {
    let tags: Option<Vec<String>> = tags_raw
        .map(|t| {
            t.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty());

    // Try update existing page by title first.
    if let Ok(Some(existing)) = repo.get_page_by_title(title).await {
        let updated = repo
            .update_page(
                &existing.id,
                UpdateWikiPageRequest {
                    title: None, // Title unchanged unless explicitly provided
                    content_md: Some(content_md.to_string()),
                    tags: tags.clone(),
                    category: None,
                    doc_type: None,
                    source_ref: None,
                    status: None,
                    extra: None,
                },
            )
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "wiki page disappeared during update".to_string())?;
        return Ok(format!("updated wiki page: {} ({})", updated.title, updated.id));
    }

    // Create new page.
    let created = repo
        .create_page(CreateWikiPageRequest {
            title: title.to_string(),
            content_md: content_md.to_string(),
            tags,
            category: None,
            doc_type: None,
            source_ref: None,
            status: None,
            extra: None,
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("created wiki page: {} ({})", created.title, created.id))
}

async fn do_search(repo: &SqliteWikiRepository, query: &str, limit: u32) -> Result<String, String> {
    let pages = repo.search(query, limit, None, None).await.map_err(|e| e.to_string())?;
    if pages.is_empty() {
        return Ok("[]".to_string());
    }
    // Return compact summaries: id, title, and first 120 chars of content.
    let summaries: Vec<serde_json::Value> = pages
        .iter()
        .map(|p| {
            let snippet: String = p.content_md.chars().take(120).collect();
            serde_json::json!({
                "id": p.id,
                "title": p.title,
                "snippet": snippet,
                "tags": p.tags,
            })
        })
        .collect();
    serde_json::to_string_pretty(&summaries).map_err(|e| e.to_string())
}

/// Resolve a page by id or title, falling back to title (case-insensitive).
async fn resolve_page(repo: &SqliteWikiRepository, page_id_or_title: &str) -> Result<WikiPage, String> {
    if let Ok(Some(page)) = repo.get_page(page_id_or_title).await {
        return Ok(page);
    }
    if let Ok(Some(page)) = repo.get_page_by_title(page_id_or_title).await {
        return Ok(page);
    }
    Err(format!("wiki page not found: {page_id_or_title}"))
}

async fn do_cite(
    repo: &SqliteWikiRepository,
    page_id_or_title: &str,
    anchor: Option<&str>,
    style: &str,
) -> Result<String, String> {
    let page = resolve_page(repo, page_id_or_title).await?;
    // Shared citation logic with the REST `/cite` endpoint.
    Ok(roseui_wiki::cite::build_citation(&page, anchor, style))
}

async fn do_link_graph(
    repo: &SqliteWikiRepository,
    page_id_or_title: &str,
    edge_type: Option<&str>,
) -> Result<String, String> {
    let page = resolve_page(repo, page_id_or_title).await?;
    let mut graph = repo.link_graph(&page.id).await.map_err(|e| e.to_string())?;

    if let Some(filter) = edge_type {
        let want = WikiEdgeType::parse(filter);
        graph.retain(|n| n.edge_type == want);
    }

    let nodes: Vec<serde_json::Value> = graph
        .iter()
        .map(|n| {
            serde_json::json!({
                "page_id": n.page_id,
                "title": n.title,
                "edge_type": n.edge_type.as_str(),
                "direction": n.direction,
            })
        })
        .collect();
    serde_json::to_string_pretty(&nodes).map_err(|e| e.to_string())
}

async fn do_title_search(repo: &SqliteWikiRepository, query: &str, limit: u32) -> Result<String, String> {
    let titles = repo.title_search(query, limit).await.map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&titles).map_err(|e| e.to_string())
}

async fn do_ingest(ingest: &IngestEngine, raw_path: &str) -> Result<String, String> {
    let result = ingest.ingest(raw_path).await.map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use roseui_db::init_database_memory;

    async fn test_repo() -> (SqliteWikiRepository, roseui_db::Database) {
        let db = init_database_memory().await.unwrap();
        (SqliteWikiRepository::new(db.pool().clone()), db)
    }

    #[allow(dead_code)]
    fn first_text(result: &CallToolResult) -> &str {
        result.content[0].as_text().expect("text content").text.as_str()
    }

    #[test]
    fn wiki_stdio_router_has_five_tools() {
        let router = WikiStdioServer::tool_router();
        let tools = router.list_all();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(names.contains(&"wiki_read"));
        assert!(names.contains(&"wiki_write"));
        assert!(names.contains(&"wiki_search"));
        assert!(names.contains(&"wiki_title_search"));
        assert!(names.contains(&"wiki_cite"));
        assert!(names.contains(&"wiki_link_graph"));
    }

    #[test]
    fn default_search_limit_is_10() {
        assert_eq!(default_search_limit(), 10);
    }

    #[test]
    fn wiki_read_schema_has_page_id_or_title() {
        let router = WikiStdioServer::tool_router();
        let tools = router.list_all();
        let read = tools
            .iter()
            .find(|t| t.name == "wiki_read")
            .expect("wiki_read tool missing");
        let props = read.input_schema["properties"].as_object().unwrap();
        assert!(props.contains_key("page_id_or_title"));
    }

    #[test]
    fn wiki_write_schema_exposes_tags_as_optional() {
        let router = WikiStdioServer::tool_router();
        let tools = router.list_all();
        let write = tools
            .iter()
            .find(|t| t.name == "wiki_write")
            .expect("wiki_write tool missing");
        let props = write.input_schema["properties"].as_object().unwrap();
        assert!(props.contains_key("tags"));
    }

    #[test]
    fn wiki_search_schema_exposes_query_and_limit() {
        let router = WikiStdioServer::tool_router();
        let tools = router.list_all();
        let search = tools
            .iter()
            .find(|t| t.name == "wiki_search")
            .expect("wiki_search tool missing");
        let props = search.input_schema["properties"].as_object().unwrap();
        assert!(props.contains_key("query"));
        assert_eq!(props["limit"]["default"], serde_json::json!(10));
    }

    #[test]
    fn wiki_tool_names_stay_within_anthropic_64_char_limit() {
        // Wire-level name is mcp__<server_name>__<tool>.
        let server_name = "roseui-wiki";
        for name in &["wiki_read", "wiki_write", "wiki_search", "wiki_title_search"] {
            let wire_name = format!("mcp__{server_name}__{name}");
            assert!(
                wire_name.len() <= 64,
                "Anthropic 64-char tool-name limit exceeded: {} ({} chars)",
                wire_name,
                wire_name.len()
            );
        }
    }

    #[tokio::test]
    async fn wiki_read_finds_by_id_and_title() {
        let (repo, _db) = test_repo().await;
        let page = repo
            .create_page(CreateWikiPageRequest {
                title: "Test Page".into(),
                content_md: "Hello wiki!".into(),
                tags: Some(vec!["test".into()]),
                category: None,
                doc_type: None,
                source_ref: None,
                status: None,
                extra: None,
            })
            .await
            .unwrap();

        // Read by ID.
        let result = do_read(&repo, &page.id).await.unwrap();
        assert!(result.contains("Test Page"));
        assert!(result.contains("Hello wiki!"));

        // Read by title.
        let result = do_read(&repo, "Test Page").await.unwrap();
        assert!(result.contains(&page.id));

        // Missing page.
        assert!(do_read(&repo, "no-such-page").await.is_err());
    }

    #[tokio::test]
    async fn wiki_write_creates_and_updates() {
        let (repo, _db) = test_repo().await;

        // Create.
        let result = do_write(&repo, "New Page", "content here", Some("a,b")).await.unwrap();
        assert!(result.starts_with("created wiki page: New Page"));

        let page = repo.get_page_by_title("New Page").await.unwrap().unwrap();
        assert_eq!(page.tags.len(), 2);

        // Update same title.
        let result = do_write(&repo, "New Page", "updated content", Some("c")).await.unwrap();
        assert!(result.starts_with("updated wiki page: New Page"));

        let page = repo.get_page_by_title("New Page").await.unwrap().unwrap();
        assert_eq!(page.content_md, "updated content");
        assert_eq!(page.tags, vec!["c"]);
    }

    #[tokio::test]
    async fn wiki_search_returns_matching_pages() {
        let (repo, _db) = test_repo().await;
        repo.create_page(CreateWikiPageRequest {
            title: "Rust Guide".into(),
            content_md: "Memory safety and ownership.".into(),
            tags: None,
            category: None,
            doc_type: None,
            source_ref: None,
            status: None,
            extra: None,
        })
        .await
        .unwrap();
        repo.create_page(CreateWikiPageRequest {
            title: "Python Tips".into(),
            content_md: "List comprehensions.".into(),
            tags: None,
            category: None,
            doc_type: None,
            source_ref: None,
            status: None,
            extra: None,
        })
        .await
        .unwrap();

        let result = do_search(&repo, "ownership", 10).await.unwrap();
        assert!(result.contains("Rust Guide"));
        assert!(!result.contains("Python Tips"));

        let result = do_search(&repo, "no-match", 10).await.unwrap();
        assert_eq!(result, "[]");
    }

    #[tokio::test]
    async fn wiki_search_respects_limit() {
        let (repo, _db) = test_repo().await;
        for i in 0..5 {
            repo.create_page(CreateWikiPageRequest {
                title: format!("Page {i}"),
                content_md: "common keyword".into(),
                tags: None,
                category: None,
                doc_type: None,
                source_ref: None,
                status: None,
                extra: None,
            })
            .await
            .unwrap();
        }
        let result = do_search(&repo, "common", 3).await.unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 3);
    }

    #[tokio::test]
    async fn wiki_read_unknown_page_returns_error_text() {
        let (repo, _db) = test_repo().await;
        let err = do_read(&repo, "does-not-exist").await.unwrap_err();
        assert!(err.contains("wiki page not found"));
        assert!(err.contains("does-not-exist"));
    }

    #[tokio::test]
    async fn wiki_search_empty_query_returns_empty_array() {
        let (repo, _db) = test_repo().await;
        // An empty query normalizes to empty FTS string, which yields no results.
        let result = do_search(&repo, "", 10).await.unwrap();
        assert_eq!(result, "[]");
    }

    #[tokio::test]
    async fn wiki_cite_modes_and_link_graph() {
        let (repo, _db) = test_repo().await;
        let statute = repo
            .create_page(CreateWikiPageRequest {
                title: "Contract Law Art. 523".into(),
                content_md: "# Performance\nA party must perform.".into(),
                tags: None,
                category: Some("law".into()),
                doc_type: Some("statute".into()),
                source_ref: Some("CN-Civil-Code#523".into()),
                status: None,
                extra: None,
            })
            .await
            .unwrap();
        let case = repo
            .create_page(CreateWikiPageRequest {
                title: "Case 2024-001".into(),
                content_md: "The court applied the statute.".into(),
                tags: None,
                category: None,
                doc_type: None,
                source_ref: None,
                status: None,
                extra: None,
            })
            .await
            .unwrap();
        repo.put_edge(&case.id, &statute.id, WikiEdgeType::Cites).await.unwrap();

        // Markdown citation anchored to a heading.
        let cite_md = do_cite(&repo, &statute.id, Some("Performance"), "markdown")
            .await
            .unwrap();
        assert!(cite_md.starts_with("[Contract Law Art. 523](wiki:"));
        assert!(cite_md.contains("#performance"));

        // Plain citation uses source_ref.
        let cite_plain = do_cite(&repo, &statute.id, None, "plain").await.unwrap();
        assert!(cite_plain.contains("CN-Civil-Code#523"));

        // Link graph from the case shows an outgoing cites edge (case -> statute).
        let graph = do_link_graph(&repo, &case.id, None).await.unwrap();
        let nodes: Vec<serde_json::Value> = serde_json::from_str(&graph).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["edge_type"], "cites");
        assert_eq!(nodes[0]["direction"], "outgoing");

        // Edge-type filter to a non-matching type yields empty.
        let graph_filtered = do_link_graph(&repo, &case.id, Some("conflicts")).await.unwrap();
        let filtered: Vec<serde_json::Value> = serde_json::from_str(&graph_filtered).unwrap();
        assert!(filtered.is_empty());
    }
}
