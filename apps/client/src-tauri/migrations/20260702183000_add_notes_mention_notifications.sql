CREATE TABLE notes_mention_notifications (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_type TEXT NOT NULL CHECK (source_type IN ('block', 'comment')),
    source_id TEXT NOT NULL CHECK (trim(source_id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('reminder', 'user_mention', 'task_mention')),
    target_type TEXT NOT NULL CHECK (target_type IN ('date', 'user', 'project_task')),
    target_id TEXT,
    trigger_at TEXT,
    plain_text TEXT NOT NULL DEFAULT '' CHECK (length(plain_text) <= 500),
    source_plain_text TEXT NOT NULL DEFAULT '' CHECK (length(source_plain_text) <= 2000),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'delivered', 'dismissed')),
    delivered_at TEXT,
    fingerprint TEXT NOT NULL CHECK (trim(fingerprint) <> ''),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (
        (source_type = 'block' AND block_id = source_id AND comment_id IS NULL)
        OR (source_type = 'comment' AND comment_id = source_id)
    ),
    CHECK (
        (kind = 'reminder' AND target_type = 'date' AND trigger_at IS NOT NULL)
        OR (kind = 'user_mention' AND target_type = 'user' AND target_id IS NOT NULL)
        OR (kind = 'task_mention' AND target_type = 'project_task' AND target_id IS NOT NULL)
    ),
    UNIQUE (source_type, source_id, fingerprint)
);

CREATE INDEX idx_notes_mention_notifications_status
    ON notes_mention_notifications(status, kind, trigger_at, created_time, id);

CREATE INDEX idx_notes_mention_notifications_source
    ON notes_mention_notifications(source_type, source_id);

CREATE INDEX idx_notes_mention_notifications_page
    ON notes_mention_notifications(page_id, status, created_time, id);
