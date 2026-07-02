PRAGMA foreign_keys = OFF;

CREATE TABLE notes_pages_next (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    parent_type TEXT NOT NULL CHECK (parent_type IN ('workspace', 'page_id', 'block_id', 'data_source_id')),
    parent_page_id TEXT REFERENCES notes_pages_next(id) ON DELETE CASCADE,
    parent_block_id TEXT,
    parent_data_source_id TEXT REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    title TEXT NOT NULL DEFAULT '',
    properties TEXT NOT NULL DEFAULT '{"title":{"id":"title","type":"title","title":[]}}' CHECK (json_valid(properties)),
    icon TEXT CHECK (icon IS NULL OR json_valid(icon)),
    cover TEXT CHECK (cover IS NULL OR json_valid(cover)),
    in_trash INTEGER NOT NULL DEFAULT 0 CHECK (in_trash IN (0, 1)),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    source_provider TEXT,
    source_object_id TEXT,
    source_workspace_id TEXT,
    source_last_edited_time TEXT,
    url TEXT,
    public_url TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''),
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

INSERT INTO notes_pages_next (
    id,
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
    source_provider,
    source_object_id,
    source_workspace_id,
    source_last_edited_time,
    url,
    public_url,
    created_time,
    last_edited_time
)
SELECT
    id,
    parent_type,
    parent_page_id,
    parent_block_id,
    NULL,
    title,
    properties,
    icon,
    cover,
    in_trash,
    archived,
    source_provider,
    source_object_id,
    source_workspace_id,
    source_last_edited_time,
    url,
    public_url,
    created_time,
    last_edited_time
FROM notes_pages;

DROP TABLE notes_pages;
ALTER TABLE notes_pages_next RENAME TO notes_pages;

CREATE INDEX idx_notes_pages_parent
    ON notes_pages(parent_type, parent_page_id, parent_block_id, parent_data_source_id);
CREATE INDEX idx_notes_pages_visible
    ON notes_pages(in_trash, last_edited_time DESC, title);
CREATE INDEX idx_notes_pages_source
    ON notes_pages(source_provider, source_workspace_id, source_object_id);
CREATE INDEX idx_notes_pages_active
    ON notes_pages(in_trash, archived, last_edited_time DESC, title);
CREATE INDEX idx_notes_pages_data_source
    ON notes_pages(parent_data_source_id, in_trash, archived, last_edited_time DESC, title);

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
    NULL,
    title,
    properties,
    icon,
    cover,
    in_trash,
    archived,
    blocks,
    block_count,
    reason,
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

PRAGMA foreign_keys = ON;
