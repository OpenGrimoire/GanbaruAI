ALTER TABLE chat_checkpoints ADD COLUMN checkpoint_kind TEXT
    CHECK (checkpoint_kind IS NULL OR checkpoint_kind IN ('initial', 'pre_turn', 'post_turn', 'recovery'));
ALTER TABLE chat_checkpoints ADD COLUMN turn_id TEXT
    REFERENCES chat_turns(id) ON UPDATE CASCADE ON DELETE SET NULL;
ALTER TABLE chat_checkpoints ADD COLUMN index_commit_oid TEXT
    CHECK (index_commit_oid IS NULL OR length(index_commit_oid) BETWEEN 40 AND 128);
ALTER TABLE chat_checkpoints ADD COLUMN index_tree_oid TEXT
    CHECK (index_tree_oid IS NULL OR length(index_tree_oid) BETWEEN 40 AND 128);
ALTER TABLE chat_checkpoints ADD COLUMN worktree_tree_oid TEXT
    CHECK (worktree_tree_oid IS NULL OR length(worktree_tree_oid) BETWEEN 40 AND 128);
ALTER TABLE chat_checkpoints ADD COLUMN head_oid TEXT
    CHECK (head_oid IS NULL OR length(head_oid) BETWEEN 40 AND 128);
ALTER TABLE chat_checkpoints ADD COLUMN head_ref TEXT
    CHECK (head_ref IS NULL OR length(head_ref) BETWEEN 1 AND 4096);
ALTER TABLE chat_checkpoints ADD COLUMN index_fingerprint TEXT
    CHECK (index_fingerprint IS NULL OR length(index_fingerprint) BETWEEN 16 AND 128);
ALTER TABLE chat_checkpoints ADD COLUMN invalidated_at TEXT
    CHECK (invalidated_at IS NULL OR length(invalidated_at) >= 20);
ALTER TABLE chat_checkpoints ADD COLUMN invalidated_by_checkpoint_id TEXT
    REFERENCES chat_checkpoints(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE chat_turns ADD COLUMN invalidated_at TEXT
    CHECK (invalidated_at IS NULL OR length(invalidated_at) >= 20);
ALTER TABLE chat_turns ADD COLUMN invalidation_reason TEXT
    CHECK (invalidation_reason IS NULL OR length(invalidation_reason) BETWEEN 1 AND 1000);

ALTER TABLE chat_events ADD COLUMN invalidated_at TEXT
    CHECK (invalidated_at IS NULL OR length(invalidated_at) >= 20);
ALTER TABLE chat_events ADD COLUMN invalidation_reason TEXT
    CHECK (invalidation_reason IS NULL OR length(invalidation_reason) BETWEEN 1 AND 1000);

CREATE INDEX idx_chat_events_valid_thread_sequence
ON chat_events(thread_id, sequence, id)
WHERE invalidated_at IS NULL;

ALTER TABLE chat_cleanup_queue ADD COLUMN workspace_id TEXT
    REFERENCES chat_workspaces(id) ON UPDATE CASCADE ON DELETE SET NULL;
ALTER TABLE chat_cleanup_queue ADD COLUMN expected_object_id TEXT
    CHECK (expected_object_id IS NULL OR length(expected_object_id) BETWEEN 40 AND 128);

CREATE TABLE chat_checkpoint_failures (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    turn_id TEXT REFERENCES chat_turns(id) ON UPDATE CASCADE ON DELETE CASCADE,
    checkpoint_kind TEXT NOT NULL CHECK (checkpoint_kind IN ('initial', 'pre_turn', 'post_turn')),
    error_code TEXT NOT NULL CHECK (length(error_code) BETWEEN 1 AND 128),
    detail TEXT NOT NULL CHECK (length(detail) BETWEEN 1 AND 2000),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_checkpoint_failures_thread
ON chat_checkpoint_failures(thread_id, created_at DESC, id);

CREATE TABLE chat_restore_previews (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    checkpoint_id TEXT NOT NULL REFERENCES chat_checkpoints(id) ON UPDATE CASCADE ON DELETE CASCADE,
    expected_thread_revision INTEGER NOT NULL CHECK (expected_thread_revision >= 0),
    repository_identity TEXT NOT NULL CHECK (length(repository_identity) BETWEEN 1 AND 1024),
    head_oid TEXT,
    head_ref TEXT,
    current_worktree_tree_oid TEXT NOT NULL CHECK (length(current_worktree_tree_oid) BETWEEN 40 AND 128),
    current_index_tree_oid TEXT NOT NULL CHECK (length(current_index_tree_oid) BETWEEN 40 AND 128),
    current_index_fingerprint TEXT NOT NULL CHECK (length(current_index_fingerprint) BETWEEN 16 AND 128),
    affected_files_data TEXT NOT NULL CHECK (
        json_valid(affected_files_data) AND json_type(affected_files_data) = 'array'
    ),
    state TEXT NOT NULL CHECK (state IN ('ready', 'stale', 'executing', 'completed', 'failed')),
    expires_at TEXT NOT NULL CHECK (length(expires_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_restore_previews_thread
ON chat_restore_previews(thread_id, created_at DESC, id);

CREATE TABLE chat_restore_operations (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    checkpoint_id TEXT NOT NULL REFERENCES chat_checkpoints(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    preview_id TEXT REFERENCES chat_restore_previews(id) ON UPDATE CASCADE ON DELETE SET NULL,
    provider_history_action TEXT CHECK (
        provider_history_action IS NULL OR provider_history_action IN ('rolled_back', 'fork_required')
    ),
    recovery_state TEXT NOT NULL CHECK (
        recovery_state IN ('pending', 'complete', 'recovered', 'recovery_required')
    ),
    recovery_ref_name TEXT,
    recovery_object_id TEXT,
    error_code TEXT,
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_restore_operations_thread
ON chat_restore_operations(thread_id, created_at DESC, id);

CREATE TABLE chat_terminal_attachment_contexts (
    attachment_id TEXT PRIMARY KEY NOT NULL
        REFERENCES chat_attachments(id) ON UPDATE CASCADE ON DELETE CASCADE,
    terminal_runtime_id TEXT NOT NULL CHECK (length(terminal_runtime_id) BETWEEN 1 AND 1024),
    terminal_name_snapshot TEXT NOT NULL CHECK (length(terminal_name_snapshot) BETWEEN 1 AND 240),
    source_kind TEXT NOT NULL CHECK (source_kind IN ('selection', 'last_command_output')),
    start_output_sequence INTEGER CHECK (start_output_sequence IS NULL OR start_output_sequence >= 0),
    end_output_sequence INTEGER CHECK (end_output_sequence IS NULL OR end_output_sequence >= 0),
    line_count INTEGER NOT NULL CHECK (line_count >= 0),
    truncated INTEGER NOT NULL DEFAULT 0 CHECK (truncated IN (0, 1)),
    captured_at TEXT NOT NULL CHECK (length(captured_at) >= 20)
) STRICT;
