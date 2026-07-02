CREATE TABLE notes_search_index_state (
    key TEXT PRIMARY KEY CHECK (trim(key) <> ''),
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE notes_search_index (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_type TEXT NOT NULL CHECK (
        source_type IN ('page', 'block', 'comment', 'property', 'file', 'alias', 'metadata')
    ),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    property_id TEXT,
    block_type TEXT,
    title TEXT NOT NULL DEFAULT '',
    body TEXT NOT NULL DEFAULT '',
    metadata TEXT NOT NULL DEFAULT '',
    source_last_edited_time TEXT NOT NULL CHECK (trim(source_last_edited_time) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE INDEX idx_notes_search_index_page
    ON notes_search_index(page_id, source_type, source_last_edited_time DESC, id);
CREATE INDEX idx_notes_search_index_block
    ON notes_search_index(block_id, source_last_edited_time DESC, id);
CREATE INDEX idx_notes_search_index_comment
    ON notes_search_index(comment_id, source_last_edited_time DESC, id);

CREATE VIRTUAL TABLE notes_search_fts USING fts5(
    index_id UNINDEXED,
    title,
    body,
    metadata,
    tokenize = 'unicode61 remove_diacritics 2'
);
