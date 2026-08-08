-- Wiki Phase 3: stable page identity + rename safety.
--
-- 1. Enforce a UNIQUE title so `[[link]]` resolution is unambiguous
--    (mirrors Obsidian, which forbids in-vault title collisions).
-- 2. Add `wiki_aliases` so that after a rename, legacy `[[Old Title]]`
--    links still resolve to the renamed page (Obsidian alias mechanism).

-- Add a unique index on title. Existing duplicate titles (if any) would
-- block this; the application guarantees titles are unique at write time,
-- so this is safe for a fresh single-user knowledge base.
CREATE UNIQUE INDEX IF NOT EXISTS idx_wiki_pages_title_unique ON wiki_pages(title);

-- Historical title aliases. When a page is renamed from A to B we keep
-- (page_id, 'A') here so `[[A]]` still resolves to the page.
CREATE TABLE IF NOT EXISTS wiki_aliases (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id    TEXT    NOT NULL,
    alias      TEXT    NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE (page_id, alias),
    FOREIGN KEY (page_id) REFERENCES wiki_pages(page_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_wiki_aliases_alias ON wiki_aliases(alias);
