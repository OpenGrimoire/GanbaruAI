ALTER TABLE notes_comment_threads
    ADD COLUMN sync_version INTEGER NOT NULL DEFAULT 1 CHECK (sync_version >= 1);

ALTER TABLE notes_comments
    ADD COLUMN sync_version INTEGER NOT NULL DEFAULT 1 CHECK (sync_version >= 1);

ALTER TABLE notes_suggestions
    ADD COLUMN sync_version INTEGER NOT NULL DEFAULT 1 CHECK (sync_version >= 1);

CREATE TABLE notes_collaboration_operations (
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
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    block_id TEXT REFERENCES notes_blocks(id) ON DELETE SET NULL,
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
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
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

CREATE INDEX idx_notes_collaboration_operations_entity
    ON notes_collaboration_operations(entity_type, entity_id, sequence);

CREATE INDEX idx_notes_collaboration_operations_page
    ON notes_collaboration_operations(page_id, sequence);

CREATE INDEX idx_notes_collaboration_operations_sync_state
    ON notes_collaboration_operations(sync_state, sequence);
