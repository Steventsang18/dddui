-- 034_wiki.sql: local knowledge base (wiki) tables + FTS5 full-text index.
--
-- Per docs/编译遵守原则.md 模块 5 (Phase 1). Markdown pages are stored as rows;
-- the on-disk Markdown file path is recorded for the future file-sync use case.
-- Full-text search uses SQLite FTS5 (zero new dependencies, already bundled).

CREATE TABLE IF NOT EXISTS wiki_pages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id     TEXT    NOT NULL UNIQUE,
    title       TEXT    NOT NULL,
    slug        TEXT    NOT NULL,
    file_path   TEXT,
    content_md  TEXT    NOT NULL DEFAULT '',
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_wiki_pages_slug ON wiki_pages(slug);

CREATE TABLE IF NOT EXISTS wiki_tags (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    tag_id  TEXT    NOT NULL UNIQUE,
    name    TEXT    NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS wiki_page_tags (
    page_id INTEGER NOT NULL,
    tag_id  INTEGER NOT NULL,
    PRIMARY KEY (page_id, tag_id),
    FOREIGN KEY (page_id) REFERENCES wiki_pages(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id)  REFERENCES wiki_tags(id)  ON DELETE CASCADE
);

-- Obsidian-style `[[Page Name]]` links resolve by target title (case-insensitive),
-- so forward links to not-yet-created pages stay valid.
CREATE TABLE IF NOT EXISTS wiki_links (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    from_page_id  INTEGER NOT NULL,
    to_title      TEXT    NOT NULL,
    created_at    INTEGER NOT NULL,
    UNIQUE (from_page_id, to_title),
    FOREIGN KEY (from_page_id) REFERENCES wiki_pages(id) ON DELETE CASCADE
);

-- FTS5 external-content table over wiki_pages. `id` is the rowid alias.
CREATE VIRTUAL TABLE IF NOT EXISTS wiki_fts USING fts5(
    title,
    content_md,
    content='wiki_pages',
    content_rowid='id',
    tokenize='unicode61'
);

-- Keep wiki_fts in sync with wiki_pages (canonical external-content recipe).
CREATE TRIGGER IF NOT EXISTS wiki_ai AFTER INSERT ON wiki_pages BEGIN
    INSERT INTO wiki_fts(rowid, title, content_md) VALUES (new.id, new.title, new.content_md);
END;

CREATE TRIGGER IF NOT EXISTS wiki_ad AFTER DELETE ON wiki_pages BEGIN
    INSERT INTO wiki_fts(wiki_fts, rowid, title, content_md) VALUES ('delete', old.id, old.title, old.content_md);
END;

CREATE TRIGGER IF NOT EXISTS wiki_au AFTER UPDATE ON wiki_pages BEGIN
    INSERT INTO wiki_fts(wiki_fts, rowid, title, content_md) VALUES ('delete', old.id, old.title, old.content_md);
    INSERT INTO wiki_fts(rowid, title, content_md) VALUES (new.id, new.title, new.content_md);
END;
