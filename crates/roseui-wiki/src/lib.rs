#![warn(clippy::disallowed_types)]

//! Local knowledge base (wiki) for the RoseUi local Agent platform.
//!
//! This is the only product-vision gap among the existing 24 crates: a local,
//! private, searchable knowledge layer that turns one-off conversations into
//! durable, retrievable assets.
//!
//! # Phase 1 scope (per `docs/编译遵守原则.md` 模块 5)
//!
//! - Markdown page CRUD, stored as local files (sandboxed via `roseui-file`)
//! - SQLite FTS5 full-text search (zero new dependencies)
//! - tags + `[[wiki link]]` bidirectional links (Obsidian-style syntax)
//! - ingestion from four sources:
//!   1. manual create
//!   2. Agent write (via `roseui-mcp` builtin tools `wiki_read`/`wiki_write`/`wiki_search`)
//!   3. dropped / received documents (docx→md via the B1 pandoc sidecar, etc.)
//!   4. distillation of valuable content from the user<->Agent conversation flow
//! - Agent access via `roseui-mcp` builtin tools
//!
//! # Phase 2 (NOT implemented, no placeholder code)
//!
//! vector search, auto-summary, knowledge graph.

pub mod cite;
pub mod error;
pub mod ingest;
pub mod repository;
pub mod routes;
pub mod service;
pub mod state;
pub mod template;
pub mod types;

pub use error::WikiError;
pub use ingest::IngestEngine;
pub use repository::{IWikiRepository, SqliteWikiRepository};
pub use routes::wiki_routes;
pub use service::WikiService;
pub use state::{WikiRouterState, build_wiki_state};
pub use types::{
    CreateWikiPageRequest, UpdateWikiPageRequest, WikiEdge, WikiEdgeType, WikiGraphNode, WikiPage,
    WikiSearchRequest, WikiSearchResponse, WikiTag,
};
