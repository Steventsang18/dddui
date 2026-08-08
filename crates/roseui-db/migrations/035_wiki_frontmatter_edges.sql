-- 035_wiki_frontmatter_edges.sql: Phase 2 wiki enhancements.
--
-- Per user directive (2026-07-31): make the wiki a *general substrate* that can
-- absorb vertical-industry professional requirements, without being customized
-- for any single industry. Three concrete enhancements:
--
--   1. frontmatter structured fields on wiki_pages (category / doc_type /
--      source_ref / status / extra JSON) — unlocks filtered retrieval so an
--      industry's domain vocabulary is supplied by the user as tag/field values.
--   2. typed edges (wiki_edges) replacing bare title-links where an explicit
--      relationship is asserted — unlocks association reasoning chains. The
--      legacy Obsidian-style `[[title]]` links (wiki_links) are retained for
--      backward compatibility.
--   3. precise citation + link-graph tooling (added at the MCP layer).
--
-- `extra` is a JSON blob for domain-specific structured metadata that does not
-- warrant its own column (e.g. {"jurisdiction":"CN","article_no":"523"}).

ALTER TABLE wiki_pages ADD COLUMN category   TEXT NOT NULL DEFAULT '';
ALTER TABLE wiki_pages ADD COLUMN doc_type   TEXT NOT NULL DEFAULT '';
ALTER TABLE wiki_pages ADD COLUMN source_ref TEXT NOT NULL DEFAULT '';
ALTER TABLE wiki_pages ADD COLUMN status     TEXT NOT NULL DEFAULT '';
ALTER TABLE wiki_pages ADD COLUMN extra      TEXT NOT NULL DEFAULT '{}';

CREATE INDEX IF NOT EXISTS idx_wiki_pages_category ON wiki_pages(category);
CREATE INDEX IF NOT EXISTS idx_wiki_pages_doc_type ON wiki_pages(doc_type);
CREATE INDEX IF NOT EXISTS idx_wiki_pages_status   ON wiki_pages(status);

-- Typed edges between pages. edge_type is a closed vocabulary so association
-- reasoning stays interpretable (cites / supersedes / conflicts / exemplifies /
-- relates). Both endpoints reference wiki_pages by row id, so edges stay valid
-- under title renames (unlike wiki_links which resolves by title).
CREATE TABLE IF NOT EXISTS wiki_edges (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    from_page_id INTEGER NOT NULL,
    to_page_id   INTEGER NOT NULL,
    edge_type    TEXT    NOT NULL,
    created_at   INTEGER NOT NULL,
    UNIQUE (from_page_id, to_page_id, edge_type),
    FOREIGN KEY (from_page_id) REFERENCES wiki_pages(id) ON DELETE CASCADE,
    FOREIGN KEY (to_page_id)   REFERENCES wiki_pages(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_wiki_edges_to   ON wiki_edges(to_page_id);
CREATE INDEX IF NOT EXISTS idx_wiki_edges_type ON wiki_edges(edge_type);
