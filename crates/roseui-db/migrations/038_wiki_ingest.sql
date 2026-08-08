-- Wiki ingest tracking (raw source -> wiki pages).
--
-- Records the ingestion state machine for each file dropped into the
-- read-only `wiki/raw/` source directory. The raw file itself is NEVER
-- modified or deleted by the ingest pipeline or by any Agent; only the
-- user may delete it (via an explicit, confirmed action).
--
-- `checksum` (sha256 of the raw file) drives re-run detection: if the
-- source content is unchanged, ingest is a no-op. If it changed, the
-- existing wiki pages are updated rather than duplicated.

CREATE TABLE IF NOT EXISTS wiki_ingest (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    -- Relative path under the raw source dir, e.g. "contracts/2024-xx.pdf".
    raw_path    TEXT    NOT NULL UNIQUE,
    -- sha256 hex of the raw file at ingest time.
    checksum    TEXT    NOT NULL,
    -- pending | parsing | ingesting | done | failed | deleted
    status      TEXT    NOT NULL DEFAULT 'pending',
    -- JSON array of produced wiki page ids (summary + slice pages).
    page_ids    TEXT    NOT NULL DEFAULT '[]',
    -- Human-readable error when status = 'failed'.
    error       TEXT    NOT NULL DEFAULT '',
    -- Number of slice pages produced (excludes the summary page).
    slices      INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_wiki_ingest_status ON wiki_ingest(status);
CREATE INDEX IF NOT EXISTS idx_wiki_ingest_raw_path ON wiki_ingest(raw_path);
