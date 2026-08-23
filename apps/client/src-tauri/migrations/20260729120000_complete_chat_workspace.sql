CREATE TABLE chat_execution_environments (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 2048),
    working_folder_id TEXT NOT NULL REFERENCES project_working_folders(id) ON UPDATE CASCADE ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('current_folder', 'worktree')),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) BETWEEN 1 AND 240),
    repository_identity TEXT CHECK (repository_identity IS NULL OR length(repository_identity) BETWEEN 1 AND 1024),
    lifecycle_state TEXT NOT NULL DEFAULT 'available' CHECK (
        lifecycle_state IN ('creating', 'available', 'missing', 'cleanup_pending', 'cleanup_failed', 'removed')
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    archived_at TEXT CHECK (archived_at IS NULL OR length(archived_at) >= 20)
) STRICT;

CREATE UNIQUE INDEX idx_chat_execution_environments_current_folder
ON chat_execution_environments(working_folder_id)
WHERE kind = 'current_folder' AND archived_at IS NULL;

INSERT INTO chat_execution_environments (
    id,
    working_folder_id,
    kind,
    display_name,
    repository_identity,
    lifecycle_state,
    created_at,
    updated_at
)
SELECT
    'current-folder:' || id,
    id,
    'current_folder',
    display_name,
    repository_identity,
    'available',
    created_at,
    updated_at
FROM project_working_folders;

ALTER TABLE chat_threads ADD COLUMN execution_environment_id TEXT
    REFERENCES chat_execution_environments(id) ON UPDATE CASCADE ON DELETE SET NULL;

UPDATE chat_threads
SET execution_environment_id = 'current-folder:' || working_folder_id
WHERE execution_environment_id IS NULL;

CREATE TRIGGER project_working_folders_create_chat_environment
AFTER INSERT ON project_working_folders
BEGIN
    INSERT INTO chat_execution_environments (
        id,
        working_folder_id,
        kind,
        display_name,
        repository_identity,
        lifecycle_state,
        created_at,
        updated_at
    ) VALUES (
        'current-folder:' || NEW.id,
        NEW.id,
        'current_folder',
        NEW.display_name,
        NEW.repository_identity,
        'available',
        NEW.created_at,
        NEW.updated_at
    );
END;

CREATE TRIGGER chat_threads_assign_current_environment
AFTER INSERT ON chat_threads
WHEN NEW.execution_environment_id IS NULL
BEGIN
    UPDATE chat_threads
    SET execution_environment_id = 'current-folder:' || NEW.working_folder_id
    WHERE id = NEW.id;
END;

CREATE INDEX idx_chat_threads_execution_environment
ON chat_threads(execution_environment_id, last_activity_at DESC, id);

CREATE TABLE chat_thread_relations (
    child_thread_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    parent_thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    source_turn_id TEXT REFERENCES chat_turns(id) ON UPDATE CASCADE ON DELETE SET NULL,
    relation_kind TEXT NOT NULL CHECK (relation_kind IN ('fork', 'continuation')),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    CHECK (child_thread_id != parent_thread_id)
) STRICT;

CREATE INDEX idx_chat_thread_relations_parent
ON chat_thread_relations(parent_thread_id, created_at DESC, child_thread_id);

CREATE TABLE chat_worktrees (
    execution_environment_id TEXT PRIMARY KEY NOT NULL
        REFERENCES chat_execution_environments(id) ON UPDATE CASCADE ON DELETE CASCADE,
    branch_name TEXT NOT NULL CHECK (length(trim(branch_name)) BETWEEN 1 AND 1024),
    base_reference TEXT NOT NULL CHECK (length(trim(base_reference)) BETWEEN 1 AND 1024),
    remote_name TEXT CHECK (remote_name IS NULL OR length(trim(remote_name)) BETWEEN 1 AND 240),
    head_object_id TEXT CHECK (head_object_id IS NULL OR length(head_object_id) BETWEEN 40 AND 128),
    cleanup_policy TEXT NOT NULL DEFAULT 'ask' CHECK (cleanup_policy IN ('ask', 'retain', 'remove_when_clean')),
    cleanup_state TEXT NOT NULL DEFAULT 'retained' CHECK (
        cleanup_state IN ('retained', 'queued', 'checking', 'cleaned', 'failed', 'dirty')
    ),
    cleanup_error_code TEXT CHECK (cleanup_error_code IS NULL OR length(cleanup_error_code) BETWEEN 1 AND 128),
    cleanup_error_detail TEXT CHECK (cleanup_error_detail IS NULL OR length(cleanup_error_detail) <= 2000),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    removed_at TEXT CHECK (removed_at IS NULL OR length(removed_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_worktrees_cleanup
ON chat_worktrees(cleanup_state, updated_at, execution_environment_id);

CREATE TABLE chat_review_comments (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    turn_id TEXT REFERENCES chat_turns(id) ON UPDATE CASCADE ON DELETE SET NULL,
    relative_path TEXT NOT NULL CHECK (
        length(relative_path) BETWEEN 1 AND 4096
        AND relative_path NOT LIKE '/%'
        AND relative_path NOT LIKE '../%'
        AND relative_path NOT LIKE '%/../%'
        AND relative_path NOT LIKE '%\\%'
    ),
    content_revision TEXT NOT NULL CHECK (length(content_revision) BETWEEN 16 AND 128),
    start_line INTEGER NOT NULL CHECK (start_line >= 1),
    start_column INTEGER NOT NULL DEFAULT 1 CHECK (start_column >= 1),
    end_line INTEGER NOT NULL CHECK (end_line >= start_line),
    end_column INTEGER NOT NULL DEFAULT 1 CHECK (end_column >= 1),
    selected_text TEXT NOT NULL DEFAULT '' CHECK (length(selected_text) <= 1048576),
    comment_text TEXT NOT NULL CHECK (length(trim(comment_text)) BETWEEN 1 AND 65536),
    state TEXT NOT NULL DEFAULT 'open' CHECK (state IN ('open', 'resolved')),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    resolved_at TEXT CHECK (resolved_at IS NULL OR length(resolved_at) >= 20),
    CHECK ((state = 'open' AND resolved_at IS NULL) OR state = 'resolved')
) STRICT;

CREATE INDEX idx_chat_review_comments_thread_path
ON chat_review_comments(thread_id, relative_path, state, created_at, id);

CREATE TABLE chat_resources (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    working_folder_id TEXT NOT NULL REFERENCES project_working_folders(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    attachment_id TEXT UNIQUE REFERENCES chat_attachments(id) ON UPDATE CASCADE ON DELETE CASCADE,
    resource_kind TEXT NOT NULL CHECK (resource_kind IN ('image', 'text_snippet', 'browser_screenshot', 'browser_recording')),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) BETWEEN 1 AND 1000),
    mime_type TEXT NOT NULL CHECK (length(mime_type) BETWEEN 1 AND 255),
    byte_size INTEGER NOT NULL CHECK (byte_size BETWEEN 0 AND 1073741824),
    sha256 TEXT NOT NULL CHECK (length(sha256) = 64 AND sha256 NOT GLOB '*[^0-9a-f]*'),
    managed_relative_path TEXT NOT NULL UNIQUE CHECK (
        length(managed_relative_path) BETWEEN 1 AND 2048
        AND managed_relative_path NOT LIKE '/%'
        AND managed_relative_path NOT LIKE '../%'
        AND managed_relative_path NOT LIKE '%/../%'
        AND managed_relative_path NOT LIKE '%\\%'
    ),
    resource_uri TEXT NOT NULL UNIQUE CHECK (resource_uri GLOB 'ganbaru://chat/resource/*'),
    integrity_state TEXT NOT NULL DEFAULT 'verified' CHECK (
        integrity_state IN ('verified', 'missing', 'hash_mismatch', 'unsafe_path', 'deleted')
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    deleted_at TEXT CHECK (deleted_at IS NULL OR length(deleted_at) >= 20)
) STRICT;

INSERT INTO chat_resources (
    id,
    working_folder_id,
    attachment_id,
    resource_kind,
    display_name,
    mime_type,
    byte_size,
    sha256,
    managed_relative_path,
    resource_uri,
    integrity_state,
    created_at,
    deleted_at
)
SELECT
    id,
    working_folder_id,
    id,
    kind,
    original_display_name,
    mime_type,
    byte_size,
    sha256,
    managed_relative_path,
    'ganbaru://chat/resource/' || id,
    CASE deletion_state
        WHEN 'deleted' THEN 'deleted'
        ELSE 'verified'
    END,
    created_at,
    deleted_at
FROM chat_attachments;

CREATE INDEX idx_chat_resources_workspace_kind
ON chat_resources(working_folder_id, resource_kind, created_at DESC, id);

CREATE TABLE chat_resource_thread_references (
    resource_id TEXT NOT NULL REFERENCES chat_resources(id) ON UPDATE CASCADE ON DELETE CASCADE,
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    first_message_id TEXT REFERENCES chat_messages(id) ON UPDATE CASCADE ON DELETE SET NULL,
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    PRIMARY KEY (resource_id, thread_id)
) STRICT, WITHOUT ROWID;

INSERT OR IGNORE INTO chat_resource_thread_references (
    resource_id,
    thread_id,
    first_message_id,
    created_at
)
SELECT
    reference.attachment_id,
    message.thread_id,
    message.id,
    reference.created_at
FROM chat_attachment_references reference
JOIN chat_messages message ON message.id = reference.message_id
WHERE reference.message_id IS NOT NULL;

CREATE TRIGGER chat_attachments_create_resource
AFTER INSERT ON chat_attachments
BEGIN
    INSERT INTO chat_resources (
        id,
        working_folder_id,
        attachment_id,
        resource_kind,
        display_name,
        mime_type,
        byte_size,
        sha256,
        managed_relative_path,
        resource_uri,
        integrity_state,
        created_at
    ) VALUES (
        NEW.id,
        NEW.working_folder_id,
        NEW.id,
        NEW.kind,
        NEW.original_display_name,
        NEW.mime_type,
        NEW.byte_size,
        NEW.sha256,
        NEW.managed_relative_path,
        'ganbaru://chat/resource/' || NEW.id,
        'verified',
        NEW.created_at
    );
END;

CREATE TRIGGER chat_attachment_messages_expose_resource
AFTER INSERT ON chat_attachment_references
WHEN NEW.message_id IS NOT NULL
BEGIN
    INSERT OR IGNORE INTO chat_resource_thread_references (
        resource_id,
        thread_id,
        first_message_id,
        created_at
    )
    SELECT NEW.attachment_id, thread_id, id, NEW.created_at
    FROM chat_messages
    WHERE id = NEW.message_id;
END;

CREATE TRIGGER chat_attachments_update_resource_integrity
AFTER UPDATE OF deletion_state, deleted_at ON chat_attachments
BEGIN
    UPDATE chat_resources
    SET integrity_state = CASE NEW.deletion_state
            WHEN 'deleted' THEN 'deleted'
            ELSE integrity_state
        END,
        deleted_at = NEW.deleted_at
    WHERE attachment_id = NEW.id;
END;

CREATE TABLE chat_preview_tabs (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    current_url TEXT NOT NULL DEFAULT '' CHECK (length(current_url) <= 8192),
    title TEXT NOT NULL DEFAULT '' CHECK (length(title) <= 2000),
    viewport_kind TEXT NOT NULL DEFAULT 'responsive' CHECK (
        viewport_kind IN ('responsive', 'mobile', 'tablet', 'desktop', 'freeform')
    ),
    viewport_width INTEGER CHECK (viewport_width IS NULL OR viewport_width BETWEEN 1 AND 16384),
    viewport_height INTEGER CHECK (viewport_height IS NULL OR viewport_height BETWEEN 1 AND 16384),
    visible INTEGER NOT NULL DEFAULT 0 CHECK (visible IN (0, 1)),
    loading_state TEXT NOT NULL DEFAULT 'idle' CHECK (loading_state IN ('idle', 'loading', 'loaded', 'failed')),
    history_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (history_schema_version >= 1),
    history_data TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(history_data) AND json_type(history_data) = 'array'),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    UNIQUE (thread_id, position)
) STRICT;

CREATE TABLE chat_browser_artifacts (
    resource_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_resources(id) ON UPDATE CASCADE ON DELETE CASCADE,
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    preview_tab_id TEXT REFERENCES chat_preview_tabs(id) ON UPDATE CASCADE ON DELETE SET NULL,
    artifact_kind TEXT NOT NULL CHECK (artifact_kind IN ('screenshot', 'recording')),
    source_url TEXT NOT NULL CHECK (length(source_url) <= 8192),
    viewport_width INTEGER NOT NULL CHECK (viewport_width BETWEEN 1 AND 16384),
    viewport_height INTEGER NOT NULL CHECK (viewport_height BETWEEN 1 AND 16384),
    duration_milliseconds INTEGER CHECK (duration_milliseconds IS NULL OR duration_milliseconds >= 0),
    frame_count INTEGER CHECK (frame_count IS NULL OR frame_count >= 0),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_browser_artifacts_thread
ON chat_browser_artifacts(thread_id, created_at DESC, resource_id);

CREATE TABLE chat_provider_cleanup_jobs (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE SET NULL,
    provider_family_id TEXT NOT NULL CHECK (length(provider_family_id) BETWEEN 1 AND 1024),
    provider_instance_id TEXT NOT NULL CHECK (length(provider_instance_id) BETWEEN 1 AND 1024),
    provider_thread_id TEXT NOT NULL CHECK (length(provider_thread_id) BETWEEN 1 AND 1024),
    operation TEXT NOT NULL CHECK (operation IN ('rename', 'archive', 'delete', 'unsubscribe', 'cleanup')),
    payload_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (payload_schema_version >= 1),
    payload_data TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(payload_data) AND json_type(payload_data) = 'object'),
    state TEXT NOT NULL DEFAULT 'queued' CHECK (state IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    last_error_code TEXT CHECK (last_error_code IS NULL OR length(last_error_code) BETWEEN 1 AND 128),
    last_error_detail TEXT CHECK (last_error_detail IS NULL OR length(last_error_detail) <= 2000),
    retry_after TEXT CHECK (retry_after IS NULL OR length(retry_after) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    completed_at TEXT CHECK (completed_at IS NULL OR length(completed_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_provider_cleanup_jobs_retry
ON chat_provider_cleanup_jobs(state, retry_after, created_at, id);

CREATE TABLE chat_terminal_layouts (
    thread_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    layout_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (layout_schema_version >= 1),
    layout_data TEXT NOT NULL DEFAULT '{"groups":[]}' CHECK (
        json_valid(layout_data) AND json_type(layout_data) = 'object'
    ),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE TABLE chat_source_control_state (
    thread_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    adapter_kind TEXT CHECK (adapter_kind IS NULL OR adapter_kind IN ('github', 'gitlab', 'azure_devops', 'bitbucket')),
    repository_slug TEXT CHECK (repository_slug IS NULL OR length(repository_slug) BETWEEN 1 AND 2048),
    remote_name TEXT CHECK (remote_name IS NULL OR length(remote_name) BETWEEN 1 AND 240),
    pull_request_number INTEGER CHECK (pull_request_number IS NULL OR pull_request_number >= 1),
    pull_request_url TEXT CHECK (pull_request_url IS NULL OR length(pull_request_url) <= 8192),
    state_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (state_schema_version >= 1),
    state_data TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(state_data) AND json_type(state_data) = 'object'),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;
