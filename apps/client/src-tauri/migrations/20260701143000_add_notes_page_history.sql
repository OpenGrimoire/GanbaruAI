CREATE TABLE notes_page_history_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    retention_days INTEGER CHECK (
        retention_days IS NULL
        OR (
            retention_days >= 1
            AND retention_days <= 3650
        )
    ),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

INSERT INTO notes_page_history_settings (id, retention_days)
VALUES (1, 30);

CREATE TABLE notes_page_history_snapshots (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_type TEXT NOT NULL CHECK (parent_type IN ('workspace', 'page_id', 'block_id')),
    parent_page_id TEXT,
    parent_block_id TEXT,
    title TEXT NOT NULL DEFAULT '',
    properties TEXT NOT NULL CHECK (json_valid(properties)),
    icon TEXT CHECK (icon IS NULL OR json_valid(icon)),
    cover TEXT CHECK (cover IS NULL OR json_valid(cover)),
    in_trash INTEGER NOT NULL DEFAULT 0 CHECK (in_trash IN (0, 1)),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    blocks TEXT NOT NULL CHECK (json_valid(blocks)),
    block_count INTEGER NOT NULL DEFAULT 0 CHECK (block_count >= 0),
    reason TEXT NOT NULL CHECK (trim(reason) <> ''),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    page_created_time TEXT NOT NULL CHECK (trim(page_created_time) <> ''),
    page_last_edited_time TEXT NOT NULL CHECK (trim(page_last_edited_time) <> ''),
    CHECK (
        (
            parent_type = 'workspace'
            AND parent_page_id IS NULL
            AND parent_block_id IS NULL
        )
        OR (
            parent_type = 'page_id'
            AND parent_page_id IS NOT NULL
            AND parent_block_id IS NULL
        )
        OR (
            parent_type = 'block_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NOT NULL
        )
    )
);

CREATE INDEX idx_notes_page_history_snapshots_page
    ON notes_page_history_snapshots(page_id, created_time DESC, id);
CREATE INDEX idx_notes_page_history_snapshots_created
    ON notes_page_history_snapshots(created_time);
