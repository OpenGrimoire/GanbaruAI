CREATE TABLE notes_page_aliases (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    alias TEXT NOT NULL CHECK (trim(alias) <> '' AND length(alias) <= 200),
    normalized_alias TEXT NOT NULL CHECK (trim(normalized_alias) <> ''),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
);

CREATE UNIQUE INDEX idx_notes_page_aliases_normalized
    ON notes_page_aliases(normalized_alias);
CREATE INDEX idx_notes_page_aliases_page
    ON notes_page_aliases(page_id, alias COLLATE NOCASE, id);

CREATE TABLE notes_unresolved_link_index_state (
    key TEXT PRIMARY KEY CHECK (trim(key) <> ''),
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE notes_unresolved_link_index (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_type TEXT NOT NULL CHECK (source_type IN ('block', 'comment')),
    source_page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    source_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    source_comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    raw_url TEXT NOT NULL CHECK (trim(raw_url) <> ''),
    raw_target TEXT NOT NULL CHECK (trim(raw_target) <> ''),
    normalized_target TEXT NOT NULL CHECK (trim(normalized_target) <> ''),
    link_text TEXT NOT NULL DEFAULT '',
    snippet TEXT NOT NULL DEFAULT '',
    created_time TEXT NOT NULL CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL CHECK (trim(last_edited_time) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    CHECK (
        (
            source_type = 'block'
            AND source_block_id IS NOT NULL
            AND source_comment_id IS NULL
        )
        OR (
            source_type = 'comment'
            AND source_comment_id IS NOT NULL
        )
    )
);

CREATE INDEX idx_notes_unresolved_link_page
    ON notes_unresolved_link_index(source_page_id, last_edited_time DESC, id);
CREATE INDEX idx_notes_unresolved_link_target
    ON notes_unresolved_link_index(normalized_target, source_page_id, id);
CREATE INDEX idx_notes_unresolved_link_block
    ON notes_unresolved_link_index(source_block_id, id)
    WHERE source_block_id IS NOT NULL;
CREATE INDEX idx_notes_unresolved_link_comment
    ON notes_unresolved_link_index(source_comment_id, id)
    WHERE source_comment_id IS NOT NULL;
