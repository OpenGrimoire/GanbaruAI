CREATE TABLE notes_comment_threads (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_type TEXT NOT NULL CHECK (parent_type IN ('page_id', 'block_id')),
    parent_page_id TEXT REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'resolved')),
    resolved_at TEXT,
    resolved_by TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''),
    CHECK (
        (
            parent_type = 'page_id'
            AND parent_page_id IS NOT NULL
            AND parent_block_id IS NULL
            AND parent_page_id = page_id
        )
        OR (
            parent_type = 'block_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NOT NULL
        )
    ),
    CHECK (
        (
            status = 'open'
            AND resolved_at IS NULL
            AND resolved_by IS NULL
        )
        OR (
            status = 'resolved'
            AND resolved_at IS NOT NULL
            AND resolved_by IS NOT NULL
        )
    )
);

CREATE INDEX idx_notes_comment_threads_page ON notes_comment_threads(page_id, status, created_time, id);
CREATE INDEX idx_notes_comment_threads_parent_block ON notes_comment_threads(parent_block_id, status, created_time, id);

CREATE TABLE notes_comments (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    thread_id TEXT NOT NULL REFERENCES notes_comment_threads(id) ON DELETE CASCADE,
    rich_text TEXT NOT NULL CHECK (json_valid(rich_text)),
    plain_text TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL DEFAULT 'local-user' CHECK (trim(created_by) <> ''),
    display_name TEXT NOT NULL DEFAULT '{"type":"user","resolved_name":"You"}' CHECK (json_valid(display_name)),
    attachments TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(attachments)),
    deleted_at TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
);

CREATE INDEX idx_notes_comments_thread ON notes_comments(thread_id, deleted_at, created_time, id);
