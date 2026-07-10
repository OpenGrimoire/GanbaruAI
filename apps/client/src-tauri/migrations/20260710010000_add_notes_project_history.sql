ALTER TABLE projects
ADD COLUMN notes_history_retention_days INTEGER CHECK (
    notes_history_retention_days IS NULL
    OR notes_history_retention_days IN (7, 30, 90, 180, 365)
);

UPDATE notes_page_history_settings
SET retention_days = CASE
    WHEN retention_days IS NULL THEN 365
    WHEN retention_days <= 7 THEN 7
    WHEN retention_days <= 30 THEN 30
    WHEN retention_days <= 90 THEN 90
    WHEN retention_days <= 180 THEN 180
    ELSE 365
END,
updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE retention_days IS NULL
   OR retention_days NOT IN (7, 30, 90, 180, 365);

CREATE TABLE notes_page_history_settings_next (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    retention_days INTEGER NOT NULL CHECK (retention_days IN (7, 30, 90, 180, 365)),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        CHECK (trim(updated_at) <> '')
);

INSERT INTO notes_page_history_settings_next (id, retention_days, updated_at)
SELECT id, retention_days, updated_at
FROM notes_page_history_settings;

DROP TABLE notes_page_history_settings;
ALTER TABLE notes_page_history_settings_next RENAME TO notes_page_history_settings;

CREATE TABLE notes_history_bundles (
    hash TEXT PRIMARY KEY CHECK (
        length(hash) = 64
        AND hash = lower(hash)
        AND hash NOT GLOB '*[^0-9a-f]*'
    ),
    kind TEXT NOT NULL CHECK (kind IN ('row', 'manifest', 'chunk')),
    encoding TEXT NOT NULL CHECK (
        encoding IN (
            'raw-json-v1',
            'zlib-json-v1',
            'raw-bytes-v1',
            'zlib-bytes-v1',
            'chunked-json-v1'
        )
    ),
    payload BLOB NOT NULL,
    uncompressed_bytes INTEGER NOT NULL CHECK (uncompressed_bytes >= 0),
    stored_bytes INTEGER NOT NULL CHECK (stored_bytes >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        CHECK (trim(created_at) <> ''),
    CHECK (length(payload) = stored_bytes)
);

CREATE INDEX idx_notes_history_bundles_created
    ON notes_history_bundles(created_at, hash);

CREATE TABLE notes_history_bundle_chunks (
    parent_hash TEXT NOT NULL REFERENCES notes_history_bundles(hash) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL CHECK (chunk_index >= 0),
    chunk_hash TEXT NOT NULL REFERENCES notes_history_bundles(hash) ON DELETE RESTRICT,
    PRIMARY KEY (parent_hash, chunk_index),
    UNIQUE (parent_hash, chunk_hash)
);

CREATE INDEX idx_notes_history_bundle_chunks_chunk
    ON notes_history_bundle_chunks(chunk_hash, parent_hash);

ALTER TABLE notes_page_history_snapshots
ADD COLUMN block_bundle_hash TEXT REFERENCES notes_history_bundles(hash) ON DELETE RESTRICT;

CREATE INDEX idx_notes_page_history_snapshots_bundle
    ON notes_page_history_snapshots(block_bundle_hash)
    WHERE block_bundle_hash IS NOT NULL;

CREATE TABLE notes_project_history_versions (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    manifest_hash TEXT NOT NULL REFERENCES notes_history_bundles(hash) ON DELETE RESTRICT,
    reason TEXT NOT NULL CHECK (trim(reason) <> '' AND length(reason) <= 80),
    created_by TEXT NOT NULL CHECK (trim(created_by) <> ''),
    display_name TEXT NOT NULL CHECK (json_valid(display_name)),
    changed_note_summary TEXT NOT NULL DEFAULT '',
    page_count INTEGER NOT NULL DEFAULT 0 CHECK (page_count >= 0),
    active_page_count INTEGER NOT NULL DEFAULT 0 CHECK (active_page_count >= 0),
    archived_page_count INTEGER NOT NULL DEFAULT 0 CHECK (archived_page_count >= 0),
    deleted_page_count INTEGER NOT NULL DEFAULT 0 CHECK (deleted_page_count >= 0),
    manifest_uncompressed_bytes INTEGER NOT NULL DEFAULT 0
        CHECK (manifest_uncompressed_bytes >= 0),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        CHECK (trim(created_time) <> '')
);

CREATE INDEX idx_notes_project_history_versions_project
    ON notes_project_history_versions(project_id, created_time DESC, id DESC);

CREATE TABLE notes_project_history_bundle_references (
    version_id TEXT NOT NULL REFERENCES notes_project_history_versions(id) ON DELETE CASCADE,
    bundle_hash TEXT NOT NULL REFERENCES notes_history_bundles(hash) ON DELETE RESTRICT,
    PRIMARY KEY (version_id, bundle_hash)
);

CREATE INDEX idx_notes_project_history_bundle_references_hash
    ON notes_project_history_bundle_references(bundle_hash, version_id);

CREATE TABLE notes_project_history_dirty (
    project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    first_dirty_at TEXT NOT NULL CHECK (trim(first_dirty_at) <> ''),
    last_dirty_at TEXT NOT NULL CHECK (trim(last_dirty_at) <> ''),
    actor_id TEXT NOT NULL CHECK (trim(actor_id) <> ''),
    actor_display_name TEXT NOT NULL CHECK (json_valid(actor_display_name)),
    changed_note_summary TEXT NOT NULL DEFAULT '',
    force_checkpoint INTEGER NOT NULL DEFAULT 0 CHECK (force_checkpoint IN (0, 1))
);

CREATE INDEX idx_notes_project_history_dirty_due
    ON notes_project_history_dirty(force_checkpoint DESC, last_dirty_at, first_dirty_at, project_id);

CREATE TABLE notes_project_history_asset_pins (
    version_id TEXT NOT NULL REFERENCES notes_project_history_versions(id) ON DELETE CASCADE,
    asset_id TEXT NOT NULL REFERENCES notes_assets(id) ON DELETE RESTRICT,
    PRIMARY KEY (version_id, asset_id)
);

CREATE INDEX idx_notes_project_history_asset_pins_asset
    ON notes_project_history_asset_pins(asset_id, version_id);

ALTER TABLE notes_mention_notifications
ADD COLUMN suppressed_by_history_restore INTEGER NOT NULL DEFAULT 0 CHECK (
    suppressed_by_history_restore IN (0, 1)
);

CREATE TABLE notes_collaboration_operations_next (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE CHECK (trim(id) <> ''),
    entity_type TEXT NOT NULL CHECK (entity_type IN ('comment_thread', 'comment', 'suggestion')),
    entity_id TEXT NOT NULL CHECK (trim(entity_id) <> ''),
    operation_type TEXT NOT NULL CHECK (
        operation_type IN (
            'comment_thread_create',
            'comment_thread_resolve',
            'comment_thread_reopen',
            'comment_create',
            'comment_update',
            'comment_delete',
            'suggestion_create',
            'suggestion_accept',
            'suggestion_reject'
        )
    ),
    page_id TEXT NOT NULL CHECK (trim(page_id) <> ''),
    block_id TEXT,
    actor_id TEXT NOT NULL REFERENCES notes_local_users(id),
    actor_display_name TEXT NOT NULL CHECK (json_valid(actor_display_name)),
    base_version INTEGER NOT NULL CHECK (base_version >= 0),
    entity_version INTEGER NOT NULL CHECK (entity_version > base_version),
    conflict_policy TEXT NOT NULL CHECK (
        conflict_policy IN ('append_only', 'last_writer_wins', 'state_transition')
    ),
    payload TEXT NOT NULL CHECK (json_valid(payload)),
    sync_state TEXT NOT NULL DEFAULT 'local' CHECK (
        sync_state IN ('local', 'exported', 'acknowledged')
    ),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        CHECK (trim(created_time) <> ''),
    CHECK (
        (
            entity_type = 'comment_thread'
            AND operation_type IN (
                'comment_thread_create',
                'comment_thread_resolve',
                'comment_thread_reopen'
            )
        )
        OR (
            entity_type = 'comment'
            AND operation_type IN ('comment_create', 'comment_update', 'comment_delete')
        )
        OR (
            entity_type = 'suggestion'
            AND operation_type IN (
                'suggestion_create',
                'suggestion_accept',
                'suggestion_reject'
            )
        )
    ),
    CHECK (
        (
            conflict_policy = 'append_only'
            AND operation_type IN (
                'comment_thread_create',
                'comment_create',
                'suggestion_create'
            )
        )
        OR (
            conflict_policy = 'last_writer_wins'
            AND operation_type = 'comment_update'
        )
        OR (
            conflict_policy = 'state_transition'
            AND operation_type IN (
                'comment_thread_resolve',
                'comment_thread_reopen',
                'comment_delete',
                'suggestion_accept',
                'suggestion_reject'
            )
        )
    )
);

INSERT INTO notes_collaboration_operations_next (
    sequence,
    id,
    entity_type,
    entity_id,
    operation_type,
    page_id,
    block_id,
    actor_id,
    actor_display_name,
    base_version,
    entity_version,
    conflict_policy,
    payload,
    sync_state,
    created_time
)
SELECT
    sequence,
    id,
    entity_type,
    entity_id,
    operation_type,
    page_id,
    block_id,
    actor_id,
    actor_display_name,
    base_version,
    entity_version,
    conflict_policy,
    payload,
    sync_state,
    created_time
FROM notes_collaboration_operations
ORDER BY sequence;

DROP TABLE notes_collaboration_operations;
ALTER TABLE notes_collaboration_operations_next RENAME TO notes_collaboration_operations;

CREATE INDEX idx_notes_collaboration_operations_entity
    ON notes_collaboration_operations(entity_type, entity_id, sequence);

CREATE INDEX idx_notes_collaboration_operations_page
    ON notes_collaboration_operations(page_id, sequence);

CREATE INDEX idx_notes_collaboration_operations_sync_state
    ON notes_collaboration_operations(sync_state, sequence);
