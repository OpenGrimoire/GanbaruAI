CREATE TABLE chat_workspaces (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    project_id TEXT REFERENCES projects(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) BETWEEN 1 AND 240),
    repository_kind TEXT NOT NULL CHECK (repository_kind IN ('git', 'none')),
    repository_identity TEXT CHECK (
        repository_identity IS NULL OR length(repository_identity) BETWEEN 1 AND 1024
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    archived_at TEXT CHECK (archived_at IS NULL OR length(archived_at) >= 20),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    UNIQUE (id, project_id),
    CHECK (
        (repository_kind = 'git' AND repository_identity IS NOT NULL)
        OR (repository_kind = 'none' AND repository_identity IS NULL)
    )
) STRICT;

CREATE UNIQUE INDEX idx_chat_workspaces_project_name
ON chat_workspaces(project_id, display_name COLLATE NOCASE)
WHERE project_id IS NOT NULL;

CREATE UNIQUE INDEX idx_chat_workspaces_standalone_name
ON chat_workspaces(display_name COLLATE NOCASE)
WHERE project_id IS NULL;

CREATE INDEX idx_chat_workspaces_project_active
ON chat_workspaces(project_id, archived_at, display_name COLLATE NOCASE, id);

CREATE TABLE chat_threads (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    workspace_id TEXT NOT NULL,
    project_id TEXT,
    title TEXT NOT NULL DEFAULT '' CHECK (length(title) <= 1000),
    title_search TEXT GENERATED ALWAYS AS (lower(trim(title))) STORED,
    title_source TEXT NOT NULL DEFAULT 'user' CHECK (title_source IN ('user', 'provider')),
    provider_family_id TEXT NOT NULL CHECK (length(provider_family_id) BETWEEN 1 AND 1024),
    provider_instance_id TEXT NOT NULL CHECK (length(provider_instance_id) BETWEEN 1 AND 1024),
    continuation_group_id TEXT NOT NULL CHECK (length(continuation_group_id) BETWEEN 1 AND 1024),
    provider_thread_id TEXT CHECK (provider_thread_id IS NULL OR length(provider_thread_id) BETWEEN 1 AND 1024),
    resume_cursor_schema_version INTEGER CHECK (
        resume_cursor_schema_version IS NULL OR resume_cursor_schema_version >= 1
    ),
    resume_cursor_data TEXT CHECK (resume_cursor_data IS NULL OR json_valid(resume_cursor_data)),
    model_selection_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (model_selection_schema_version >= 1),
    model_selection_data TEXT NOT NULL DEFAULT '{}' CHECK (
        json_valid(model_selection_data) AND json_type(model_selection_data) = 'object'
    ),
    safety_mode TEXT NOT NULL CHECK (safety_mode IN ('supervised', 'auto_accept_edits', 'full_access')),
    interaction_mode TEXT NOT NULL CHECK (interaction_mode IN ('build', 'plan')),
    state TEXT NOT NULL CHECK (state IN ('draft', 'active', 'waiting', 'idle', 'error', 'archived', 'closed')),
    latest_turn_state TEXT CHECK (
        latest_turn_state IS NULL OR latest_turn_state IN (
            'pending', 'dispatching', 'active', 'waiting_for_approval',
            'waiting_for_user_input', 'completed', 'interrupted', 'failed'
        )
    ),
    latest_preview TEXT CHECK (latest_preview IS NULL OR length(latest_preview) <= 2000),
    changed_file_summary_schema_version INTEGER CHECK (
        changed_file_summary_schema_version IS NULL OR changed_file_summary_schema_version >= 1
    ),
    changed_file_summary_data TEXT CHECK (
        changed_file_summary_data IS NULL OR (
            json_valid(changed_file_summary_data) AND json_type(changed_file_summary_data) = 'object'
        )
    ),
    message_count INTEGER NOT NULL DEFAULT 0 CHECK (message_count >= 0),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    last_event_sequence INTEGER NOT NULL DEFAULT 0 CHECK (last_event_sequence >= 0),
    last_projected_sequence INTEGER NOT NULL DEFAULT 0 CHECK (
        last_projected_sequence >= 0 AND last_projected_sequence <= last_event_sequence
    ),
    read_revision INTEGER NOT NULL DEFAULT 0 CHECK (read_revision >= 0),
    unread_at TEXT CHECK (unread_at IS NULL OR length(unread_at) >= 20),
    last_activity_at TEXT NOT NULL CHECK (length(last_activity_at) >= 20),
    archived_at TEXT CHECK (archived_at IS NULL OR length(archived_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    FOREIGN KEY (workspace_id, project_id)
        REFERENCES chat_workspaces(id, project_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    FOREIGN KEY (workspace_id)
        REFERENCES chat_workspaces(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    CHECK (
        (resume_cursor_schema_version IS NULL AND resume_cursor_data IS NULL)
        OR (resume_cursor_schema_version IS NOT NULL AND resume_cursor_data IS NOT NULL)
    )
) STRICT;

CREATE INDEX idx_chat_threads_active_project
ON chat_threads(project_id, last_activity_at DESC, id)
WHERE archived_at IS NULL AND state != 'closed';

CREATE INDEX idx_chat_threads_active_workspace
ON chat_threads(workspace_id, last_activity_at DESC, id)
WHERE archived_at IS NULL AND state != 'closed';

CREATE INDEX idx_chat_threads_archived
ON chat_threads(archived_at DESC, id)
WHERE archived_at IS NOT NULL;

CREATE INDEX idx_chat_threads_title_search
ON chat_threads(title_search, last_activity_at DESC, id);

CREATE TRIGGER chat_threads_project_matches_workspace_insert
BEFORE INSERT ON chat_threads
WHEN NOT EXISTS (
    SELECT 1 FROM chat_workspaces
    WHERE id = NEW.workspace_id AND project_id IS NEW.project_id
)
BEGIN
    SELECT RAISE(ABORT, 'Chat thread project must match its workspace');
END;

CREATE TRIGGER chat_threads_project_matches_workspace_update
BEFORE UPDATE OF workspace_id, project_id ON chat_threads
WHEN NOT EXISTS (
    SELECT 1 FROM chat_workspaces
    WHERE id = NEW.workspace_id AND project_id IS NEW.project_id
)
BEGIN
    SELECT RAISE(ABORT, 'Chat thread project must match its workspace');
END;

CREATE TABLE chat_checkpoints (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    turn_count INTEGER NOT NULL CHECK (turn_count >= 0),
    repository_identity TEXT NOT NULL CHECK (length(repository_identity) BETWEEN 1 AND 1024),
    hidden_ref_name TEXT NOT NULL UNIQUE CHECK (
        hidden_ref_name GLOB 'refs/ganbaru-ai/chat/*'
        AND hidden_ref_name NOT GLOB '*[[:space:]]*'
    ),
    git_object_id TEXT NOT NULL CHECK (length(git_object_id) BETWEEN 40 AND 128),
    status TEXT NOT NULL CHECK (status IN ('capturing', 'available', 'invalid', 'cleanup_pending', 'cleanup_failed')),
    changed_files_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (changed_files_schema_version >= 1),
    changed_files_data TEXT NOT NULL DEFAULT '[]' CHECK (
        json_valid(changed_files_data) AND json_type(changed_files_data) = 'array'
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    cleanup_state TEXT NOT NULL DEFAULT 'retained' CHECK (
        cleanup_state IN ('retained', 'queued', 'cleaned', 'failed')
    ),
    UNIQUE (thread_id, turn_count)
) STRICT;

CREATE INDEX idx_chat_checkpoints_thread_turn
ON chat_checkpoints(thread_id, turn_count DESC, id);

CREATE TABLE chat_turns (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    user_message_id TEXT,
    provider_turn_id TEXT CHECK (provider_turn_id IS NULL OR length(provider_turn_id) BETWEEN 1 AND 1024),
    source_proposed_plan_id TEXT,
    state TEXT NOT NULL CHECK (state IN (
        'pending', 'dispatching', 'active', 'waiting_for_approval',
        'waiting_for_user_input', 'completed', 'interrupted', 'failed'
    )),
    started_at TEXT CHECK (started_at IS NULL OR length(started_at) >= 20),
    completed_at TEXT CHECK (completed_at IS NULL OR length(completed_at) >= 20),
    stop_reason TEXT CHECK (stop_reason IS NULL OR length(stop_reason) <= 1000),
    error_schema_version INTEGER CHECK (error_schema_version IS NULL OR error_schema_version >= 1),
    error_data TEXT CHECK (error_data IS NULL OR json_valid(error_data)),
    model_selection_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (model_selection_schema_version >= 1),
    model_selection_data TEXT NOT NULL DEFAULT '{}' CHECK (
        json_valid(model_selection_data) AND json_type(model_selection_data) = 'object'
    ),
    safety_mode TEXT NOT NULL CHECK (safety_mode IN ('supervised', 'auto_accept_edits', 'full_access')),
    interaction_mode TEXT NOT NULL CHECK (interaction_mode IN ('build', 'plan')),
    usage_schema_version INTEGER CHECK (usage_schema_version IS NULL OR usage_schema_version >= 1),
    usage_data TEXT CHECK (usage_data IS NULL OR json_valid(usage_data)),
    pre_checkpoint_id TEXT REFERENCES chat_checkpoints(id) ON UPDATE CASCADE ON DELETE SET NULL,
    post_checkpoint_id TEXT REFERENCES chat_checkpoints(id) ON UPDATE CASCADE ON DELETE SET NULL,
    changed_file_summary_schema_version INTEGER CHECK (
        changed_file_summary_schema_version IS NULL OR changed_file_summary_schema_version >= 1
    ),
    changed_file_summary_data TEXT CHECK (
        changed_file_summary_data IS NULL OR json_valid(changed_file_summary_data)
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    UNIQUE (thread_id, ordinal),
    CHECK (
        (error_schema_version IS NULL AND error_data IS NULL)
        OR (error_schema_version IS NOT NULL AND error_data IS NOT NULL)
    ),
    CHECK (
        (usage_schema_version IS NULL AND usage_data IS NULL)
        OR (usage_schema_version IS NOT NULL AND usage_data IS NOT NULL)
    )
) STRICT;

CREATE INDEX idx_chat_turns_thread_ordinal
ON chat_turns(thread_id, ordinal DESC, id);

CREATE TABLE chat_messages (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    turn_id TEXT REFERENCES chat_turns(id) ON UPDATE CASCADE ON DELETE CASCADE,
    sequence_anchor INTEGER NOT NULL CHECK (sequence_anchor >= 0),
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    normalized_markdown TEXT NOT NULL DEFAULT '' CHECK (length(normalized_markdown) <= 16777216),
    streaming_state TEXT NOT NULL CHECK (streaming_state IN ('pending', 'streaming', 'complete', 'interrupted', 'failed')),
    provider_item_id TEXT CHECK (provider_item_id IS NULL OR length(provider_item_id) BETWEEN 1 AND 1024),
    content_metadata_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (content_metadata_schema_version >= 1),
    content_metadata_data TEXT NOT NULL DEFAULT '{}' CHECK (
        json_valid(content_metadata_data) AND json_type(content_metadata_data) = 'object'
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_messages_thread_sequence
ON chat_messages(thread_id, sequence_anchor, id);

CREATE INDEX idx_chat_messages_turn
ON chat_messages(turn_id, sequence_anchor, id);

CREATE TABLE chat_activities (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    turn_id TEXT REFERENCES chat_turns(id) ON UPDATE CASCADE ON DELETE CASCADE,
    sequence_anchor INTEGER NOT NULL CHECK (sequence_anchor >= 0),
    item_kind TEXT NOT NULL CHECK (length(item_kind) BETWEEN 1 AND 128),
    status TEXT NOT NULL CHECK (length(status) BETWEEN 1 AND 128),
    title TEXT NOT NULL CHECK (length(title) <= 2000),
    detail TEXT CHECK (detail IS NULL OR length(detail) <= 16777216),
    provider_item_id TEXT CHECK (provider_item_id IS NULL OR length(provider_item_id) BETWEEN 1 AND 1024),
    safe_metadata_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (safe_metadata_schema_version >= 1),
    safe_metadata_data TEXT NOT NULL DEFAULT '{}' CHECK (
        json_valid(safe_metadata_data) AND json_type(safe_metadata_data) = 'object'
    ),
    started_at TEXT CHECK (started_at IS NULL OR length(started_at) >= 20),
    completed_at TEXT CHECK (completed_at IS NULL OR length(completed_at) >= 20),
    source_event_type TEXT NOT NULL CHECK (length(source_event_type) BETWEEN 1 AND 128),
    output_artifact_reference TEXT CHECK (
        output_artifact_reference IS NULL OR length(output_artifact_reference) BETWEEN 1 AND 1024
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_activities_thread_sequence
ON chat_activities(thread_id, sequence_anchor, id);

CREATE INDEX idx_chat_activities_turn
ON chat_activities(turn_id, sequence_anchor, id);

CREATE TABLE chat_pending_requests (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    turn_id TEXT REFERENCES chat_turns(id) ON UPDATE CASCADE ON DELETE CASCADE,
    provider_request_id TEXT NOT NULL CHECK (length(provider_request_id) BETWEEN 1 AND 1024),
    request_kind TEXT NOT NULL CHECK (request_kind IN ('approval', 'user_input')),
    safe_display_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (safe_display_schema_version >= 1),
    safe_display_data TEXT NOT NULL CHECK (json_valid(safe_display_data)),
    allowed_decisions_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (allowed_decisions_schema_version >= 1),
    allowed_decisions_data TEXT NOT NULL CHECK (
        json_valid(allowed_decisions_data) AND json_type(allowed_decisions_data) = 'array'
    ),
    opened_sequence INTEGER NOT NULL CHECK (opened_sequence >= 1),
    resolution_state TEXT NOT NULL DEFAULT 'open' CHECK (
        resolution_state IN ('open', 'resolved', 'stale', 'interrupted')
    ),
    resolution_schema_version INTEGER CHECK (
        resolution_schema_version IS NULL OR resolution_schema_version >= 1
    ),
    resolution_data TEXT CHECK (resolution_data IS NULL OR json_valid(resolution_data)),
    opened_at TEXT NOT NULL CHECK (length(opened_at) >= 20),
    resolved_at TEXT CHECK (resolved_at IS NULL OR length(resolved_at) >= 20),
    CHECK (
        (resolution_state = 'open' AND resolution_data IS NULL AND resolved_at IS NULL)
        OR (resolution_state != 'open' AND resolved_at IS NOT NULL)
    )
) STRICT;

CREATE UNIQUE INDEX idx_chat_pending_requests_unresolved
ON chat_pending_requests(thread_id, provider_request_id)
WHERE resolution_state = 'open';

CREATE INDEX idx_chat_pending_requests_thread
ON chat_pending_requests(thread_id, opened_sequence, id);

CREATE TABLE chat_plans (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    origin_turn_id TEXT,
    sequence_anchor INTEGER NOT NULL CHECK (sequence_anchor >= 0),
    markdown TEXT NOT NULL DEFAULT '' CHECK (length(markdown) <= 16777216),
    steps_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (steps_schema_version >= 1),
    steps_data TEXT NOT NULL DEFAULT '[]' CHECK (
        json_valid(steps_data) AND json_type(steps_data) = 'array'
    ),
    state TEXT NOT NULL CHECK (state IN ('proposed', 'accepted', 'dismissed', 'implemented')),
    implementation_turn_id TEXT,
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_plans_thread_sequence
ON chat_plans(thread_id, sequence_anchor, id);

CREATE TABLE chat_drafts (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    workspace_id TEXT NOT NULL REFERENCES chat_workspaces(id) ON UPDATE CASCADE ON DELETE CASCADE,
    thread_id TEXT REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    text TEXT NOT NULL DEFAULT '' CHECK (length(text) <= 16777216),
    mentions_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (mentions_schema_version >= 1),
    mentions_data TEXT NOT NULL DEFAULT '[]' CHECK (
        json_valid(mentions_data) AND json_type(mentions_data) = 'array'
    ),
    provider_instance_id TEXT CHECK (
        provider_instance_id IS NULL OR length(provider_instance_id) BETWEEN 1 AND 1024
    ),
    model_selection_schema_version INTEGER CHECK (
        model_selection_schema_version IS NULL OR model_selection_schema_version >= 1
    ),
    model_selection_data TEXT CHECK (model_selection_data IS NULL OR json_valid(model_selection_data)),
    safety_mode TEXT CHECK (
        safety_mode IS NULL OR safety_mode IN ('supervised', 'auto_accept_edits', 'full_access')
    ),
    interaction_mode TEXT CHECK (interaction_mode IS NULL OR interaction_mode IN ('build', 'plan')),
    sent_snapshot_schema_version INTEGER CHECK (
        sent_snapshot_schema_version IS NULL OR sent_snapshot_schema_version >= 1
    ),
    sent_snapshot_data TEXT CHECK (sent_snapshot_data IS NULL OR json_valid(sent_snapshot_data)),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    CHECK (
        (model_selection_schema_version IS NULL AND model_selection_data IS NULL)
        OR (model_selection_schema_version IS NOT NULL AND model_selection_data IS NOT NULL)
    )
) STRICT;

CREATE UNIQUE INDEX idx_chat_drafts_thread
ON chat_drafts(thread_id)
WHERE thread_id IS NOT NULL;

CREATE UNIQUE INDEX idx_chat_drafts_new_workspace
ON chat_drafts(workspace_id)
WHERE thread_id IS NULL;

CREATE TABLE chat_attachments (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    workspace_id TEXT NOT NULL REFERENCES chat_workspaces(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    kind TEXT NOT NULL CHECK (kind IN ('image', 'text_snippet')),
    original_display_name TEXT NOT NULL CHECK (length(trim(original_display_name)) BETWEEN 1 AND 1000),
    mime_type TEXT NOT NULL CHECK (length(mime_type) BETWEEN 1 AND 255),
    byte_size INTEGER NOT NULL CHECK (byte_size BETWEEN 0 AND 52428800),
    sha256 TEXT NOT NULL CHECK (length(sha256) = 64 AND sha256 NOT GLOB '*[^0-9a-f]*'),
    managed_relative_path TEXT NOT NULL UNIQUE CHECK (
        length(managed_relative_path) BETWEEN 1 AND 2048
        AND managed_relative_path NOT LIKE '/%'
        AND managed_relative_path NOT LIKE '%/../%'
        AND managed_relative_path NOT LIKE '../%'
        AND managed_relative_path NOT LIKE '%/..'
        AND managed_relative_path NOT LIKE '%\\%'
    ),
    signature_kind TEXT NOT NULL CHECK (length(signature_kind) BETWEEN 1 AND 128),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    deletion_state TEXT NOT NULL DEFAULT 'active' CHECK (
        deletion_state IN ('active', 'pending_delete', 'deleted', 'cleanup_failed')
    ),
    unreferenced_at TEXT CHECK (unreferenced_at IS NULL OR length(unreferenced_at) >= 20),
    deleted_at TEXT CHECK (deleted_at IS NULL OR length(deleted_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_attachments_hash
ON chat_attachments(sha256, byte_size, id);

CREATE INDEX idx_chat_attachments_cleanup
ON chat_attachments(deletion_state, unreferenced_at, id);

CREATE TABLE chat_attachment_references (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    attachment_id TEXT NOT NULL REFERENCES chat_attachments(id) ON UPDATE CASCADE ON DELETE CASCADE,
    message_id TEXT REFERENCES chat_messages(id) ON UPDATE CASCADE ON DELETE CASCADE,
    draft_id TEXT REFERENCES chat_drafts(id) ON UPDATE CASCADE ON DELETE CASCADE,
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    CHECK ((message_id IS NOT NULL) != (draft_id IS NOT NULL))
) STRICT;

CREATE UNIQUE INDEX idx_chat_attachment_references_unique_message
ON chat_attachment_references(attachment_id, message_id)
WHERE message_id IS NOT NULL;

CREATE UNIQUE INDEX idx_chat_attachment_references_unique_draft
ON chat_attachment_references(attachment_id, draft_id)
WHERE draft_id IS NOT NULL;

CREATE INDEX idx_chat_attachment_references_message
ON chat_attachment_references(message_id, attachment_id);

CREATE INDEX idx_chat_attachment_references_draft
ON chat_attachment_references(draft_id, attachment_id);

CREATE TABLE chat_events (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    sequence INTEGER NOT NULL CHECK (sequence >= 1),
    event_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (event_schema_version >= 1),
    turn_id TEXT,
    provider_turn_id TEXT CHECK (provider_turn_id IS NULL OR length(provider_turn_id) BETWEEN 1 AND 1024),
    provider_item_id TEXT CHECK (provider_item_id IS NULL OR length(provider_item_id) BETWEEN 1 AND 1024),
    provider_request_id TEXT CHECK (provider_request_id IS NULL OR length(provider_request_id) BETWEEN 1 AND 1024),
    provider_task_id TEXT CHECK (provider_task_id IS NULL OR length(provider_task_id) BETWEEN 1 AND 1024),
    provider_family_id TEXT NOT NULL CHECK (length(provider_family_id) BETWEEN 1 AND 1024),
    provider_instance_id TEXT NOT NULL CHECK (length(provider_instance_id) BETWEEN 1 AND 1024),
    event_type TEXT NOT NULL CHECK (length(event_type) BETWEEN 1 AND 128),
    payload_schema_version INTEGER NOT NULL CHECK (payload_schema_version >= 1),
    payload_data TEXT NOT NULL CHECK (json_valid(payload_data)),
    provider_reference_schema_version INTEGER CHECK (
        provider_reference_schema_version IS NULL OR provider_reference_schema_version >= 1
    ),
    provider_reference_data TEXT CHECK (
        provider_reference_data IS NULL OR json_valid(provider_reference_data)
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    ingested_at TEXT NOT NULL CHECK (length(ingested_at) >= 20),
    redacted_diagnostic_schema_version INTEGER CHECK (
        redacted_diagnostic_schema_version IS NULL OR redacted_diagnostic_schema_version >= 1
    ),
    redacted_diagnostic_data TEXT CHECK (
        redacted_diagnostic_data IS NULL OR json_valid(redacted_diagnostic_data)
    ),
    diagnostic_expires_at TEXT CHECK (
        diagnostic_expires_at IS NULL OR length(diagnostic_expires_at) >= 20
    ),
    UNIQUE (thread_id, sequence),
    CHECK (
        (provider_reference_schema_version IS NULL AND provider_reference_data IS NULL)
        OR (provider_reference_schema_version IS NOT NULL AND provider_reference_data IS NOT NULL)
    ),
    CHECK (
        (redacted_diagnostic_schema_version IS NULL AND redacted_diagnostic_data IS NULL AND diagnostic_expires_at IS NULL)
        OR (redacted_diagnostic_schema_version IS NOT NULL AND redacted_diagnostic_data IS NOT NULL AND diagnostic_expires_at IS NOT NULL)
    )
) STRICT;

CREATE INDEX idx_chat_events_thread_sequence
ON chat_events(thread_id, sequence, id);

CREATE INDEX idx_chat_events_turn_sequence
ON chat_events(turn_id, sequence, id);

CREATE INDEX idx_chat_events_diagnostic_expiry
ON chat_events(diagnostic_expires_at, id)
WHERE diagnostic_expires_at IS NOT NULL;

CREATE TRIGGER chat_events_require_next_sequence
BEFORE INSERT ON chat_events
WHEN NEW.sequence != (
    SELECT last_event_sequence + 1 FROM chat_threads WHERE id = NEW.thread_id
)
BEGIN
    SELECT RAISE(ABORT, 'Chat event sequence must be contiguous');
END;

CREATE TABLE chat_command_receipts (
    client_command_id TEXT PRIMARY KEY NOT NULL CHECK (length(client_command_id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    command_kind TEXT NOT NULL CHECK (length(command_kind) BETWEEN 1 AND 128),
    submitted_revision INTEGER CHECK (submitted_revision IS NULL OR submitted_revision >= 0),
    state TEXT NOT NULL CHECK (state IN ('accepted', 'completed', 'failed')),
    result_schema_version INTEGER CHECK (result_schema_version IS NULL OR result_schema_version >= 1),
    result_data TEXT CHECK (result_data IS NULL OR json_valid(result_data)),
    error_schema_version INTEGER CHECK (error_schema_version IS NULL OR error_schema_version >= 1),
    error_data TEXT CHECK (error_data IS NULL OR json_valid(error_data)),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    CHECK (result_data IS NULL OR error_data IS NULL)
) STRICT;

CREATE INDEX idx_chat_command_receipts_thread
ON chat_command_receipts(thread_id, created_at DESC, client_command_id);

CREATE TABLE chat_cleanup_queue (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    source_thread_id TEXT CHECK (source_thread_id IS NULL OR length(source_thread_id) BETWEEN 1 AND 1024),
    cleanup_kind TEXT NOT NULL CHECK (
        cleanup_kind IN ('checkpoint_ref', 'attachment_file', 'diagnostic_event')
    ),
    exact_target TEXT NOT NULL CHECK (length(exact_target) BETWEEN 1 AND 4096),
    repository_identity TEXT CHECK (
        repository_identity IS NULL OR length(repository_identity) BETWEEN 1 AND 1024
    ),
    state TEXT NOT NULL DEFAULT 'pending' CHECK (
        state IN ('pending', 'running', 'failed', 'completed')
    ),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    not_before TEXT NOT NULL CHECK (length(not_before) >= 20),
    last_error_code TEXT CHECK (last_error_code IS NULL OR length(last_error_code) BETWEEN 1 AND 128),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    UNIQUE (cleanup_kind, exact_target)
) STRICT;

CREATE INDEX idx_chat_cleanup_queue_due
ON chat_cleanup_queue(state, not_before, id);
