use serde::{Deserialize, Serialize};

/// A single wiki page (Markdown document) with optional frontmatter metadata.
///
/// Frontmatter fields (`category` / `doc_type` / `source_ref` / `status` / `extra`)
/// are a *general substrate*: they carry no industry-specific semantics on their
/// own. Vertical industries supply their vocabulary through these field values
/// (e.g. `doc_type="statute"`, `category="contract-law"`) and via free-form tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub content_md: String,
    pub tags: Vec<String>,
    /// Coarse classification bucket (user-defined vocabulary). Empty if unset.
    #[serde(default)]
    pub category: String,
    /// Document kind within its domain (user-defined). Empty if unset.
    #[serde(default)]
    pub doc_type: String,
    /// Canonical external reference (article no., case no., URL, file path).
    #[serde(default)]
    pub source_ref: String,
    /// Lifecycle status (user-defined: draft / active / archived / ...).
    #[serde(default)]
    pub status: String,
    /// Free-form structured metadata as JSON (e.g. {"jurisdiction":"CN"}).
    #[serde(default)]
    pub extra: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Create a new wiki page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWikiPageRequest {
    pub title: String,
    pub content_md: String,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub doc_type: Option<String>,
    #[serde(default)]
    pub source_ref: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub extra: Option<serde_json::Value>,
}

/// Update an existing wiki page (partial).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWikiPageRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub content_md: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub doc_type: Option<String>,
    #[serde(default)]
    pub source_ref: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub extra: Option<serde_json::Value>,
}

/// Full-text + structured search query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSearchRequest {
    pub query: String,
    #[serde(default)]
    pub limit: Option<u32>,
    /// Restrict matches to a frontmatter `category` value (exact, case-sensitive).
    #[serde(default)]
    pub category: Option<String>,
    /// Restrict matches to a frontmatter `doc_type` value (exact, case-sensitive).
    #[serde(default)]
    pub doc_type: Option<String>,
}

/// Full-text search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSearchResponse {
    pub hits: Vec<WikiPage>,
}

/// A wiki tag with its usage count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiTag {
    pub name: String,
    pub count: i64,
}

/// Controlled vocabulary of typed edges between pages.
///
/// Kept closed and domain-neutral so association reasoning stays interpretable.
/// `Relates` is the default umbrella edge (equivalent to an untyped link).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WikiEdgeType {
    Cites,
    Supersedes,
    Conflicts,
    Exemplifies,
    Relates,
}

impl WikiEdgeType {
    /// Parse a string into an edge type; unknown values fall back to `Relates`.
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "cites" => WikiEdgeType::Cites,
            "supersedes" => WikiEdgeType::Supersedes,
            "conflicts" => WikiEdgeType::Conflicts,
            "exemplifies" => WikiEdgeType::Exemplifies,
            _ => WikiEdgeType::Relates,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            WikiEdgeType::Cites => "cites",
            WikiEdgeType::Supersedes => "supersedes",
            WikiEdgeType::Conflicts => "conflicts",
            WikiEdgeType::Exemplifies => "exemplifies",
            WikiEdgeType::Relates => "relates",
        }
    }
}

/// A directed typed edge from one page to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiEdge {
    pub from_page_id: String,
    pub to_page_id: String,
    pub edge_type: WikiEdgeType,
    pub created_at: i64,
}

/// One node in a page's link graph (used by `wiki_link_graph`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiGraphNode {
    /// Page id of the linked page.
    pub page_id: String,
    /// Page title of the linked page.
    pub title: String,
    /// Edge type of the relationship from the root page to this node.
    pub edge_type: WikiEdgeType,
    /// Direction relative to the root: `outgoing` (root -> node) or `incoming`
    /// (node -> root, e.g. a page that `cites` the root).
    pub direction: String,
}

/// An ingest record for a raw-source file (migration 038).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestRow {
    pub raw_path: String,
    pub checksum: String,
    /// pending | parsing | ingesting | done | failed | deleted
    pub status: String,
    /// Wiki page ids produced by the last ingest (summary first).
    pub page_ids: Vec<String>,
    pub slices: u32,
    pub error: String,
    pub updated_at: i64,
}
