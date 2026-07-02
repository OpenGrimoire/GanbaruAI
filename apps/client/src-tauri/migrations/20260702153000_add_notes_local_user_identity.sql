CREATE TABLE notes_local_users (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    display_name TEXT NOT NULL CHECK (
        trim(display_name) <> ''
        AND length(display_name) <= 80
    ),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
);

INSERT INTO notes_local_users (id, display_name)
SELECT
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-' ||
    lower(hex(randomblob(2))) || '-' ||
    lower(hex(randomblob(2))) || '-' ||
    lower(hex(randomblob(6))),
    'You'
WHERE NOT EXISTS (SELECT 1 FROM notes_local_users);

CREATE INDEX idx_notes_local_users_display_name
    ON notes_local_users(display_name);

UPDATE notes_comments
SET created_by = (SELECT id FROM notes_local_users ORDER BY created_time ASC, id ASC LIMIT 1),
    display_name = json_object(
        'type',
        'user',
        'resolved_name',
        (SELECT display_name FROM notes_local_users ORDER BY created_time ASC, id ASC LIMIT 1)
    )
WHERE created_by = 'local-user';

UPDATE notes_comment_threads
SET resolved_by = (SELECT id FROM notes_local_users ORDER BY created_time ASC, id ASC LIMIT 1)
WHERE resolved_by = 'local-user';

PRAGMA foreign_keys = OFF;

CREATE TABLE notes_page_history_snapshots_next (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_type TEXT NOT NULL CHECK (parent_type IN ('workspace', 'page_id', 'block_id', 'data_source_id')),
    parent_page_id TEXT,
    parent_block_id TEXT,
    parent_data_source_id TEXT,
    title TEXT NOT NULL DEFAULT '',
    properties TEXT NOT NULL CHECK (json_valid(properties)),
    icon TEXT CHECK (icon IS NULL OR json_valid(icon)),
    cover TEXT CHECK (cover IS NULL OR json_valid(cover)),
    in_trash INTEGER NOT NULL DEFAULT 0 CHECK (in_trash IN (0, 1)),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    blocks TEXT NOT NULL CHECK (json_valid(blocks)),
    block_count INTEGER NOT NULL DEFAULT 0 CHECK (block_count >= 0),
    reason TEXT NOT NULL CHECK (trim(reason) <> ''),
    created_by TEXT NOT NULL REFERENCES notes_local_users(id),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    page_created_time TEXT NOT NULL CHECK (trim(page_created_time) <> ''),
    page_last_edited_time TEXT NOT NULL CHECK (trim(page_last_edited_time) <> ''),
    CHECK (
        (
            parent_type = 'workspace'
            AND parent_page_id IS NULL
            AND parent_block_id IS NULL
            AND parent_data_source_id IS NULL
        )
        OR (
            parent_type = 'page_id'
            AND parent_page_id IS NOT NULL
            AND parent_block_id IS NULL
            AND parent_data_source_id IS NULL
        )
        OR (
            parent_type = 'block_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NOT NULL
            AND parent_data_source_id IS NULL
        )
        OR (
            parent_type = 'data_source_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NULL
            AND parent_data_source_id IS NOT NULL
        )
    )
);

INSERT INTO notes_page_history_snapshots_next (
    id,
    page_id,
    parent_type,
    parent_page_id,
    parent_block_id,
    parent_data_source_id,
    title,
    properties,
    icon,
    cover,
    in_trash,
    archived,
    blocks,
    block_count,
    reason,
    created_by,
    created_time,
    page_created_time,
    page_last_edited_time
)
SELECT
    id,
    page_id,
    parent_type,
    parent_page_id,
    parent_block_id,
    parent_data_source_id,
    title,
    properties,
    icon,
    cover,
    in_trash,
    archived,
    blocks,
    block_count,
    reason,
    (SELECT id FROM notes_local_users ORDER BY created_time ASC, id ASC LIMIT 1),
    created_time,
    page_created_time,
    page_last_edited_time
FROM notes_page_history_snapshots;

DROP TABLE notes_page_history_snapshots;
ALTER TABLE notes_page_history_snapshots_next RENAME TO notes_page_history_snapshots;

CREATE INDEX idx_notes_page_history_snapshots_page
    ON notes_page_history_snapshots(page_id, created_time DESC, id);
CREATE INDEX idx_notes_page_history_snapshots_created
    ON notes_page_history_snapshots(created_time);
CREATE INDEX idx_notes_page_history_snapshots_created_by
    ON notes_page_history_snapshots(created_by, created_time DESC, id);

PRAGMA foreign_keys = ON;
