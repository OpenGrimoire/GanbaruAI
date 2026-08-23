DELETE FROM chat_conversation_memberships
WHERE participant_id IN (
    SELECT participant_id FROM chat_ai_teammates
);

ALTER TABLE chat_communication_messages
ADD COLUMN author_label_snapshot TEXT NOT NULL DEFAULT 'Unknown participant' CHECK (
    length(trim(author_label_snapshot)) BETWEEN 1 AND 160
);

UPDATE chat_communication_messages
SET author_label_snapshot = coalesce((
    SELECT participant.display_name
    FROM chat_participants participant
    WHERE participant.id = chat_communication_messages.author_participant_id
), 'Unknown participant');

CREATE TRIGGER chat_communication_author_snapshot_immutable
BEFORE UPDATE OF author_label_snapshot ON chat_communication_messages
WHEN NEW.author_label_snapshot IS NOT OLD.author_label_snapshot
BEGIN
    SELECT RAISE(ABORT, 'Communication message author snapshots are immutable');
END;

DROP TRIGGER chat_teammate_policy_publish;
DROP TABLE chat_ai_teammates;

CREATE TABLE chat_ai_teammates (
    participant_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    role TEXT NOT NULL CHECK (length(trim(role)) BETWEEN 1 AND 1000),
    instructions TEXT NOT NULL DEFAULT '' CHECK (length(instructions) <= 65536),
    latest_policy_revision INTEGER NOT NULL DEFAULT 0 CHECK (latest_policy_revision >= 0),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE TRIGGER chat_teammate_policy_publish
AFTER INSERT ON chat_teammate_policy_revisions
BEGIN
    UPDATE chat_ai_teammates
    SET latest_policy_revision = NEW.revision,
        updated_at = NEW.created_at
    WHERE participant_id = NEW.teammate_id
      AND latest_policy_revision < NEW.revision;
END;

DROP TRIGGER chat_teammate_folder_grant_identity_insert;
DROP TRIGGER chat_channels_create_project_general;
DROP TABLE chat_teammate_working_folder_grants;

ALTER TABLE chat_conversation_memberships
RENAME TO chat_conversation_memberships_legacy;

CREATE TABLE chat_conversation_memberships (
    conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    participant_id TEXT NOT NULL REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    membership_role TEXT NOT NULL DEFAULT 'member' CHECK (
        membership_role IN ('owner', 'member')
    ),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    removed_at TEXT CHECK (removed_at IS NULL OR length(removed_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    PRIMARY KEY (conversation_id, participant_id)
) STRICT, WITHOUT ROWID;

INSERT INTO chat_conversation_memberships (
    conversation_id, participant_id, membership_role, revision,
    removed_at, created_at, updated_at
)
SELECT
    conversation_id, participant_id, membership_role, revision,
    removed_at, created_at, updated_at
FROM chat_conversation_memberships_legacy;

DROP TABLE chat_conversation_memberships_legacy;

CREATE INDEX idx_chat_conversation_memberships_participant
ON chat_conversation_memberships(participant_id, removed_at, conversation_id);

CREATE TABLE chat_teammate_working_folder_grants (
    conversation_id TEXT NOT NULL,
    teammate_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    working_folder_id TEXT NOT NULL,
    capability TEXT NOT NULL CHECK (
        capability IN ('none', 'read', 'edit', 'execute', 'publish')
    ),
    capability_inherits_profile INTEGER NOT NULL DEFAULT 0 CHECK (
        capability_inherits_profile IN (0, 1)
    ),
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    runtime_approval_policy TEXT CHECK (
        runtime_approval_policy IS NULL OR runtime_approval_policy IN (
            'ask', 'auto_approve', 'unattended', 'provider_custom'
        )
    ),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    revoked_at TEXT CHECK (revoked_at IS NULL OR length(revoked_at) >= 20),
    PRIMARY KEY (conversation_id, teammate_id, working_folder_id),
    FOREIGN KEY (conversation_id, teammate_id)
        REFERENCES chat_conversation_memberships(conversation_id, participant_id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (conversation_id, project_id)
        REFERENCES chat_conversations(id, project_id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (working_folder_id, project_id)
        REFERENCES project_working_folders(id, project_id)
        ON UPDATE CASCADE ON DELETE RESTRICT
) STRICT, WITHOUT ROWID;

CREATE UNIQUE INDEX idx_chat_teammate_folder_default
ON chat_teammate_working_folder_grants(conversation_id, teammate_id)
WHERE is_default = 1 AND revoked_at IS NULL;

CREATE TRIGGER chat_teammate_folder_grant_identity_insert
BEFORE INSERT ON chat_teammate_working_folder_grants
WHEN NOT EXISTS (
    SELECT 1
    FROM chat_ai_teammates teammate
    JOIN chat_conversation_memberships membership
      ON membership.conversation_id = NEW.conversation_id
     AND membership.participant_id = NEW.teammate_id
     AND membership.removed_at IS NULL
    JOIN chat_ai_channel_memberships ai_membership
      ON ai_membership.conversation_id = membership.conversation_id
     AND ai_membership.teammate_id = membership.participant_id
    WHERE teammate.participant_id = NEW.teammate_id
)
BEGIN
    SELECT RAISE(ABORT, 'Working folder grants require active AI channel access');
END;

DROP TABLE chat_threads;
DROP TRIGGER project_working_folders_create_chat_environment;
DROP TABLE chat_worktrees;
DROP TABLE chat_execution_environments;

CREATE TABLE chat_execution_environments (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 2048),
    working_folder_id TEXT REFERENCES project_working_folders(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    scratch_generation_id TEXT REFERENCES chat_scratch_generations(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('current_folder', 'worktree', 'scratch')),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) BETWEEN 1 AND 240),
    repository_identity TEXT CHECK (
        repository_identity IS NULL OR length(repository_identity) BETWEEN 1 AND 1024
    ),
    lifecycle_state TEXT NOT NULL DEFAULT 'available' CHECK (
        lifecycle_state IN (
            'creating', 'available', 'missing',
            'cleanup_pending', 'cleanup_failed', 'removed'
        )
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    archived_at TEXT CHECK (archived_at IS NULL OR length(archived_at) >= 20),
    CHECK (
        (
            kind IN ('current_folder', 'worktree')
            AND working_folder_id IS NOT NULL
            AND scratch_generation_id IS NULL
        ) OR (
            kind = 'scratch'
            AND working_folder_id IS NULL
            AND scratch_generation_id IS NOT NULL
            AND repository_identity IS NULL
        )
    )
) STRICT;

CREATE UNIQUE INDEX idx_chat_execution_environments_current_folder
ON chat_execution_environments(working_folder_id)
WHERE kind = 'current_folder' AND archived_at IS NULL;

CREATE UNIQUE INDEX idx_chat_execution_environments_scratch
ON chat_execution_environments(scratch_generation_id)
WHERE kind = 'scratch' AND archived_at IS NULL;

CREATE TRIGGER project_working_folders_create_chat_environment
AFTER INSERT ON project_working_folders
BEGIN
    INSERT INTO chat_execution_environments (
        id, working_folder_id, kind, display_name, repository_identity,
        lifecycle_state, created_at, updated_at
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

CREATE TABLE chat_worktrees (
    execution_environment_id TEXT PRIMARY KEY NOT NULL
        REFERENCES chat_execution_environments(id) ON UPDATE CASCADE ON DELETE CASCADE,
    branch_name TEXT NOT NULL CHECK (length(trim(branch_name)) BETWEEN 1 AND 1024),
    base_reference TEXT NOT NULL CHECK (length(trim(base_reference)) BETWEEN 1 AND 1024),
    remote_name TEXT CHECK (
        remote_name IS NULL OR length(trim(remote_name)) BETWEEN 1 AND 240
    ),
    head_object_id TEXT CHECK (
        head_object_id IS NULL OR length(head_object_id) BETWEEN 40 AND 128
    ),
    cleanup_policy TEXT NOT NULL DEFAULT 'ask' CHECK (
        cleanup_policy IN ('ask', 'retain', 'remove_when_clean')
    ),
    cleanup_state TEXT NOT NULL DEFAULT 'retained' CHECK (
        cleanup_state IN ('retained', 'queued', 'checking', 'cleaned', 'failed', 'dirty')
    ),
    cleanup_error_code TEXT CHECK (
        cleanup_error_code IS NULL OR length(cleanup_error_code) BETWEEN 1 AND 128
    ),
    cleanup_error_detail TEXT CHECK (
        cleanup_error_detail IS NULL OR length(cleanup_error_detail) <= 2000
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    removed_at TEXT CHECK (removed_at IS NULL OR length(removed_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_worktrees_cleanup
ON chat_worktrees(cleanup_state, updated_at, execution_environment_id);

CREATE TABLE chat_threads (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    working_folder_id TEXT,
    project_id TEXT NOT NULL REFERENCES projects(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    execution_environment_id TEXT REFERENCES chat_execution_environments(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    scratch_generation_id TEXT REFERENCES chat_scratch_generations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    title TEXT NOT NULL DEFAULT '' CHECK (length(title) <= 1000),
    title_search TEXT GENERATED ALWAYS AS (lower(trim(title))) STORED,
    title_source TEXT NOT NULL DEFAULT 'user' CHECK (title_source IN ('user', 'provider')),
    provider_family_id TEXT NOT NULL CHECK (length(provider_family_id) BETWEEN 1 AND 1024),
    provider_instance_id TEXT NOT NULL CHECK (length(provider_instance_id) BETWEEN 1 AND 1024),
    continuation_group_id TEXT NOT NULL CHECK (length(continuation_group_id) BETWEEN 1 AND 1024),
    provider_thread_id TEXT CHECK (
        provider_thread_id IS NULL OR length(provider_thread_id) BETWEEN 1 AND 1024
    ),
    resume_cursor_schema_version INTEGER CHECK (
        resume_cursor_schema_version IS NULL OR resume_cursor_schema_version >= 1
    ),
    resume_cursor_data TEXT CHECK (resume_cursor_data IS NULL OR json_valid(resume_cursor_data)),
    model_selection_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (
        model_selection_schema_version >= 1
    ),
    model_selection_data TEXT NOT NULL DEFAULT '{}' CHECK (
        json_valid(model_selection_data) AND json_type(model_selection_data) = 'object'
    ),
    safety_mode TEXT NOT NULL CHECK (
        safety_mode IN ('ask_for_approval', 'approve_for_me', 'full_access', 'custom')
    ),
    interaction_mode TEXT NOT NULL CHECK (interaction_mode IN ('build', 'plan')),
    state TEXT NOT NULL CHECK (
        state IN ('draft', 'active', 'waiting', 'idle', 'error', 'archived', 'closed')
    ),
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
            json_valid(changed_file_summary_data)
            AND json_type(changed_file_summary_data) = 'object'
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
    FOREIGN KEY (working_folder_id, project_id)
        REFERENCES project_working_folders(id, project_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    CHECK (
        (resume_cursor_schema_version IS NULL AND resume_cursor_data IS NULL)
        OR (resume_cursor_schema_version IS NOT NULL AND resume_cursor_data IS NOT NULL)
    ),
    CHECK (
        (
            working_folder_id IS NOT NULL
            AND scratch_generation_id IS NULL
        ) OR (
            working_folder_id IS NULL
            AND execution_environment_id IS NOT NULL
            AND scratch_generation_id IS NOT NULL
        )
    )
) STRICT;

CREATE INDEX idx_chat_threads_active_project
ON chat_threads(project_id, last_activity_at DESC, id)
WHERE archived_at IS NULL AND state != 'closed';

CREATE INDEX idx_chat_threads_active_working_folder
ON chat_threads(working_folder_id, last_activity_at DESC, id)
WHERE working_folder_id IS NOT NULL AND archived_at IS NULL AND state != 'closed';

CREATE INDEX idx_chat_threads_archived
ON chat_threads(archived_at DESC, id)
WHERE archived_at IS NOT NULL;

CREATE INDEX idx_chat_threads_title_search
ON chat_threads(title_search, last_activity_at DESC, id);

CREATE INDEX idx_chat_threads_execution_environment
ON chat_threads(execution_environment_id, last_activity_at DESC, id)
WHERE execution_environment_id IS NOT NULL;

CREATE INDEX idx_chat_threads_scratch_generation
ON chat_threads(scratch_generation_id, last_activity_at DESC, id)
WHERE scratch_generation_id IS NOT NULL;

CREATE TRIGGER chat_threads_project_matches_working_folder_insert
BEFORE INSERT ON chat_threads
WHEN NEW.working_folder_id IS NOT NULL AND NOT EXISTS (
    SELECT 1 FROM project_working_folders
    WHERE id = NEW.working_folder_id AND project_id = NEW.project_id
)
BEGIN
    SELECT RAISE(ABORT, 'Chat thread project must match its working folder');
END;

CREATE TRIGGER chat_threads_project_matches_working_folder_update
BEFORE UPDATE OF working_folder_id, project_id ON chat_threads
WHEN NEW.working_folder_id IS NOT NULL AND NOT EXISTS (
    SELECT 1 FROM project_working_folders
    WHERE id = NEW.working_folder_id AND project_id = NEW.project_id
)
BEGIN
    SELECT RAISE(ABORT, 'Chat thread project must match its working folder');
END;

CREATE TRIGGER chat_threads_assign_current_environment
AFTER INSERT ON chat_threads
WHEN NEW.working_folder_id IS NOT NULL AND NEW.execution_environment_id IS NULL
BEGIN
    UPDATE chat_threads
    SET execution_environment_id = 'current-folder:' || NEW.working_folder_id
    WHERE id = NEW.id;
END;

CREATE TRIGGER chat_threads_execution_target_insert
BEFORE INSERT ON chat_threads
WHEN NEW.execution_environment_id IS NOT NULL AND NOT EXISTS (
    SELECT 1 FROM chat_execution_environments environment
    WHERE environment.id = NEW.execution_environment_id
      AND (
        (
            NEW.working_folder_id IS NOT NULL
            AND environment.working_folder_id = NEW.working_folder_id
            AND environment.scratch_generation_id IS NULL
        ) OR (
            NEW.scratch_generation_id IS NOT NULL
            AND environment.scratch_generation_id = NEW.scratch_generation_id
            AND environment.working_folder_id IS NULL
        )
      )
)
BEGIN
    SELECT RAISE(ABORT, 'Chat thread execution target is inconsistent');
END;

CREATE TRIGGER chat_threads_execution_target_update
BEFORE UPDATE OF working_folder_id, execution_environment_id, scratch_generation_id ON chat_threads
WHEN NEW.execution_environment_id IS NULL OR NOT EXISTS (
    SELECT 1 FROM chat_execution_environments environment
    WHERE environment.id = NEW.execution_environment_id
      AND (
        (
            NEW.working_folder_id IS NOT NULL
            AND environment.working_folder_id = NEW.working_folder_id
            AND environment.scratch_generation_id IS NULL
        ) OR (
            NEW.scratch_generation_id IS NOT NULL
            AND environment.scratch_generation_id = NEW.scratch_generation_id
            AND environment.working_folder_id IS NULL
        )
      )
)
BEGIN
    SELECT RAISE(ABORT, 'Chat thread execution target is inconsistent');
END;

CREATE TABLE chat_access_profiles (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    builtin_key TEXT CHECK (
        builtin_key IS NULL OR builtin_key IN (
            'conversation_only', 'read_only', 'edit_files',
            'build_and_test', 'publish_changes'
        )
    ),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) BETWEEN 1 AND 160),
    latest_revision INTEGER NOT NULL DEFAULT 1 CHECK (latest_revision >= 1),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    archived_at TEXT CHECK (archived_at IS NULL OR length(archived_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE UNIQUE INDEX idx_chat_access_profiles_builtin
ON chat_access_profiles(builtin_key)
WHERE builtin_key IS NOT NULL;

CREATE UNIQUE INDEX idx_chat_access_profiles_custom_name
ON chat_access_profiles(lower(trim(display_name)))
WHERE builtin_key IS NULL AND archived_at IS NULL;

CREATE TABLE chat_access_profile_revisions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    access_profile_id TEXT NOT NULL REFERENCES chat_access_profiles(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    revision INTEGER NOT NULL CHECK (revision >= 1),
    default_read_history INTEGER NOT NULL CHECK (default_read_history IN (0, 1)),
    default_participate INTEGER NOT NULL CHECK (default_participate IN (0, 1)),
    default_history_boundary TEXT NOT NULL CHECK (
        default_history_boundary IN ('entire', 'from_grant')
    ),
    maximum_folder_capability TEXT NOT NULL CHECK (
        maximum_folder_capability IN ('none', 'read', 'edit', 'execute', 'publish')
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    UNIQUE (access_profile_id, revision)
) STRICT;

CREATE TRIGGER chat_access_profile_revision_immutable_update
BEFORE UPDATE ON chat_access_profile_revisions
BEGIN
    SELECT RAISE(ABORT, 'Access profile revisions are immutable');
END;

CREATE TRIGGER chat_access_profile_revision_immutable_delete
BEFORE DELETE ON chat_access_profile_revisions
BEGIN
    SELECT RAISE(ABORT, 'Access profile revisions are immutable');
END;

CREATE TRIGGER chat_access_profile_builtin_revision_protected
BEFORE INSERT ON chat_access_profile_revisions
WHEN NEW.revision > 1 AND EXISTS (
    SELECT 1 FROM chat_access_profiles profile
    WHERE profile.id = NEW.access_profile_id
      AND profile.builtin_key IS NOT NULL
)
BEGIN
    SELECT RAISE(ABORT, 'Built-in access profiles are immutable');
END;

CREATE TRIGGER chat_access_profile_publish
AFTER INSERT ON chat_access_profile_revisions
BEGIN
    UPDATE chat_access_profiles
    SET latest_revision = NEW.revision,
        revision = revision + 1,
        updated_at = NEW.created_at
    WHERE id = NEW.access_profile_id
      AND latest_revision < NEW.revision;
END;

CREATE TRIGGER chat_access_profile_builtin_protected
BEFORE UPDATE ON chat_access_profiles
WHEN OLD.builtin_key IS NOT NULL AND (
    NEW.id IS NOT OLD.id
    OR NEW.builtin_key IS NOT OLD.builtin_key
    OR NEW.display_name IS NOT OLD.display_name
    OR NEW.latest_revision IS NOT OLD.latest_revision
    OR NEW.revision IS NOT OLD.revision
    OR NEW.archived_at IS NOT OLD.archived_at
    OR NEW.created_at IS NOT OLD.created_at
    OR NEW.updated_at IS NOT OLD.updated_at
)
BEGIN
    SELECT RAISE(ABORT, 'Built-in access profiles are protected');
END;

INSERT INTO chat_access_profiles (
    id, builtin_key, display_name, created_at, updated_at
) VALUES
    (
        'access-profile:conversation-only', 'conversation_only', 'Conversation only',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    (
        'access-profile:read-only', 'read_only', 'Read only',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    (
        'access-profile:edit-files', 'edit_files', 'Edit files',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    (
        'access-profile:build-and-test', 'build_and_test', 'Build and test',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    (
        'access-profile:publish-changes', 'publish_changes', 'Publish changes',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

INSERT INTO chat_access_profile_revisions (
    id, access_profile_id, revision, default_read_history, default_participate,
    default_history_boundary, maximum_folder_capability, created_at
) VALUES
    (
        'access-profile-revision:conversation-only:1',
        'access-profile:conversation-only', 1, 1, 1, 'entire', 'none',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    (
        'access-profile-revision:read-only:1',
        'access-profile:read-only', 1, 1, 1, 'entire', 'read',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    (
        'access-profile-revision:edit-files:1',
        'access-profile:edit-files', 1, 1, 1, 'entire', 'edit',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    (
        'access-profile-revision:build-and-test:1',
        'access-profile:build-and-test', 1, 1, 1, 'entire', 'execute',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    (
        'access-profile-revision:publish-changes:1',
        'access-profile:publish-changes', 1, 1, 1, 'entire', 'publish',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

CREATE TABLE chat_ai_teammate_access_state (
    teammate_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_ai_teammates(participant_id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    access_revision INTEGER NOT NULL DEFAULT 0 CHECK (access_revision >= 0),
    runtime_approval_policy TEXT NOT NULL DEFAULT 'ask' CHECK (
        runtime_approval_policy IN ('ask', 'auto_approve', 'unattended', 'provider_custom')
    ),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

INSERT INTO chat_ai_teammate_access_state (teammate_id, updated_at)
SELECT participant_id, updated_at FROM chat_ai_teammates;

CREATE TRIGGER chat_ai_teammate_create_access_state
AFTER INSERT ON chat_ai_teammates
BEGIN
    INSERT INTO chat_ai_teammate_access_state (teammate_id, updated_at)
    VALUES (NEW.participant_id, NEW.updated_at);
END;

CREATE TABLE chat_ai_channel_memberships (
    conversation_id TEXT NOT NULL,
    teammate_id TEXT NOT NULL,
    access_profile_id TEXT NOT NULL,
    read_history INTEGER NOT NULL DEFAULT 0 CHECK (read_history IN (0, 1)),
    read_history_inherits_profile INTEGER NOT NULL DEFAULT 0 CHECK (
        read_history_inherits_profile IN (0, 1)
    ),
    participate INTEGER NOT NULL DEFAULT 0 CHECK (participate IN (0, 1)),
    participate_inherits_profile INTEGER NOT NULL DEFAULT 0 CHECK (
        participate_inherits_profile IN (0, 1)
    ),
    history_boundary TEXT NOT NULL DEFAULT 'entire' CHECK (
        history_boundary IN ('entire', 'from_grant')
    ),
    history_boundary_inherits_profile INTEGER NOT NULL DEFAULT 0 CHECK (
        history_boundary_inherits_profile IN (0, 1)
    ),
    history_from_ordinal INTEGER CHECK (
        history_from_ordinal IS NULL OR history_from_ordinal >= 1
    ),
    runtime_approval_policy TEXT CHECK (
        runtime_approval_policy IS NULL OR runtime_approval_policy IN (
            'ask', 'auto_approve', 'unattended', 'provider_custom'
        )
    ),
    scratch_runtime_approval_policy TEXT CHECK (
        scratch_runtime_approval_policy IS NULL OR scratch_runtime_approval_policy IN (
            'ask', 'auto_approve', 'unattended', 'provider_custom'
        )
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    PRIMARY KEY (conversation_id, teammate_id),
    FOREIGN KEY (conversation_id, teammate_id)
        REFERENCES chat_conversation_memberships(conversation_id, participant_id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (access_profile_id)
        REFERENCES chat_access_profiles(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    CHECK (
        (history_boundary = 'entire' AND history_from_ordinal IS NULL)
        OR (history_boundary = 'from_grant' AND history_from_ordinal IS NOT NULL)
    )
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_chat_ai_channel_memberships_teammate
ON chat_ai_channel_memberships(teammate_id, conversation_id);

CREATE TRIGGER chat_ai_channel_membership_identity_insert
BEFORE INSERT ON chat_ai_channel_memberships
WHEN NOT EXISTS (
    SELECT 1 FROM chat_ai_teammates teammate
    WHERE teammate.participant_id = NEW.teammate_id
)
BEGIN
    SELECT RAISE(ABORT, 'AI channel access requires an AI teammate');
END;

CREATE TABLE chat_conversation_audience_state (
    conversation_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

INSERT INTO chat_conversation_audience_state (conversation_id, updated_at)
SELECT id, updated_at FROM chat_conversations;

CREATE TRIGGER chat_conversation_create_audience_state
AFTER INSERT ON chat_conversations
BEGIN
    INSERT INTO chat_conversation_audience_state (conversation_id, updated_at)
    VALUES (NEW.id, NEW.updated_at);
END;

CREATE TRIGGER chat_generic_membership_audience_insert
AFTER INSERT ON chat_conversation_memberships
BEGIN
    UPDATE chat_conversation_audience_state
    SET revision = revision + 1,
        updated_at = NEW.updated_at
    WHERE conversation_id = NEW.conversation_id;
END;

CREATE TRIGGER chat_generic_membership_audience_update
AFTER UPDATE ON chat_conversation_memberships
BEGIN
    UPDATE chat_conversation_audience_state
    SET revision = revision + 1,
        updated_at = NEW.updated_at
    WHERE conversation_id IN (OLD.conversation_id, NEW.conversation_id);
END;

CREATE TRIGGER chat_generic_membership_audience_delete
AFTER DELETE ON chat_conversation_memberships
BEGIN
    UPDATE chat_conversation_audience_state
    SET revision = revision + 1,
        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE conversation_id = OLD.conversation_id;
END;

CREATE TRIGGER chat_ai_membership_audience_insert
AFTER INSERT ON chat_ai_channel_memberships
BEGIN
    UPDATE chat_conversation_audience_state
    SET revision = revision + 1,
        updated_at = NEW.updated_at
    WHERE conversation_id = NEW.conversation_id;
END;

CREATE TRIGGER chat_ai_membership_audience_update
AFTER UPDATE ON chat_ai_channel_memberships
BEGIN
    UPDATE chat_conversation_audience_state
    SET revision = revision + 1,
        updated_at = NEW.updated_at
    WHERE conversation_id IN (OLD.conversation_id, NEW.conversation_id);
END;

CREATE TRIGGER chat_ai_membership_audience_delete
AFTER DELETE ON chat_ai_channel_memberships
BEGIN
    UPDATE chat_conversation_audience_state
    SET revision = revision + 1,
        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE conversation_id = OLD.conversation_id;
END;

CREATE TRIGGER chat_access_profile_audience_publish
AFTER INSERT ON chat_access_profile_revisions
BEGIN
    UPDATE chat_conversation_audience_state
    SET revision = revision + 1,
        updated_at = NEW.created_at
    WHERE conversation_id IN (
        SELECT membership.conversation_id
        FROM chat_ai_channel_memberships membership
        WHERE membership.access_profile_id = NEW.access_profile_id
    );
END;

DROP TRIGGER chat_teammate_policy_immutable_delete;
DROP TABLE chat_participant_mentions;
DROP TABLE chat_communication_resource_references;
DROP TABLE chat_scheduled_message_resource_references;

CREATE TABLE chat_message_references (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    message_revision_id TEXT NOT NULL REFERENCES chat_communication_message_revisions(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    reference_kind TEXT NOT NULL CHECK (
        reference_kind IN (
            'participant', 'channel', 'working_folder',
            'workspace_path', 'execution_environment'
        )
    ),
    label_snapshot TEXT NOT NULL CHECK (length(label_snapshot) BETWEEN 1 AND 4096),
    plain_text_projection TEXT NOT NULL CHECK (length(plain_text_projection) BETWEEN 1 AND 4096),
    start_offset INTEGER NOT NULL CHECK (start_offset >= 0),
    end_offset INTEGER NOT NULL CHECK (end_offset > start_offset),
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    UNIQUE (message_revision_id, start_offset, end_offset),
    UNIQUE (message_revision_id, ordinal)
) STRICT;

CREATE TABLE chat_participant_reference_targets (
    reference_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_message_references(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    participant_id TEXT NOT NULL REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT
) STRICT;

CREATE TABLE chat_channel_reference_targets (
    reference_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_message_references(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    channel_id TEXT NOT NULL REFERENCES chat_channels(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    source_conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    destination_conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    source_lower_ordinal INTEGER NOT NULL CHECK (source_lower_ordinal >= 0),
    source_high_ordinal INTEGER NOT NULL CHECK (source_high_ordinal >= source_lower_ordinal),
    source_revision_cutoff_id TEXT REFERENCES chat_communication_message_revisions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    destination_audience_revision INTEGER NOT NULL CHECK (destination_audience_revision >= 1),
    CHECK (
        (source_high_ordinal = 0 AND source_revision_cutoff_id IS NULL)
        OR (source_high_ordinal > 0 AND source_revision_cutoff_id IS NOT NULL)
    )
) STRICT;

CREATE TABLE chat_working_folder_reference_targets (
    reference_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_message_references(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    working_folder_id TEXT NOT NULL REFERENCES project_working_folders(id)
        ON UPDATE CASCADE ON DELETE RESTRICT
) STRICT;

CREATE TABLE chat_workspace_path_reference_targets (
    reference_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_message_references(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    working_folder_id TEXT NOT NULL REFERENCES project_working_folders(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    path_kind TEXT NOT NULL CHECK (path_kind IN ('file', 'folder')),
    relative_path TEXT NOT NULL CHECK (
        length(relative_path) BETWEEN 1 AND 4096
        AND relative_path NOT LIKE '/%'
        AND relative_path NOT LIKE '../%'
        AND relative_path NOT LIKE '%/../%'
        AND relative_path NOT LIKE '%/..'
        AND relative_path NOT LIKE '%\%'
    )
) STRICT;

CREATE TABLE chat_execution_environment_reference_targets (
    reference_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_message_references(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    execution_environment_id TEXT NOT NULL REFERENCES chat_execution_environments(id)
        ON UPDATE CASCADE ON DELETE RESTRICT
) STRICT;

CREATE TABLE chat_scheduled_message_references (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    scheduled_message_id TEXT NOT NULL REFERENCES chat_scheduled_messages(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    reference_kind TEXT NOT NULL CHECK (
        reference_kind IN (
            'participant', 'channel', 'working_folder',
            'workspace_path', 'execution_environment'
        )
    ),
    label_snapshot TEXT NOT NULL CHECK (length(label_snapshot) BETWEEN 1 AND 4096),
    plain_text_projection TEXT NOT NULL CHECK (length(plain_text_projection) BETWEEN 1 AND 4096),
    start_offset INTEGER NOT NULL CHECK (start_offset >= 0),
    end_offset INTEGER NOT NULL CHECK (end_offset > start_offset),
    participant_id TEXT REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    participant_kind TEXT CHECK (
        participant_kind IS NULL OR participant_kind IN ('local_user', 'ai_teammate', 'human')
    ),
    channel_id TEXT REFERENCES chat_channels(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    working_folder_id TEXT REFERENCES project_working_folders(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    path_kind TEXT CHECK (path_kind IS NULL OR path_kind IN ('file', 'folder')),
    relative_path TEXT CHECK (
        relative_path IS NULL OR (
            length(relative_path) BETWEEN 1 AND 4096
            AND relative_path NOT LIKE '/%'
            AND relative_path NOT LIKE '../%'
            AND relative_path NOT LIKE '%/../%'
            AND relative_path NOT LIKE '%/..'
            AND relative_path NOT LIKE '%\%'
        )
    ),
    execution_environment_id TEXT REFERENCES chat_execution_environments(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    UNIQUE (scheduled_message_id, start_offset, end_offset),
    UNIQUE (scheduled_message_id, ordinal),
    CHECK (
        (reference_kind = 'participant' AND participant_id IS NOT NULL
            AND participant_kind IS NOT NULL AND channel_id IS NULL
            AND working_folder_id IS NULL AND path_kind IS NULL
            AND relative_path IS NULL AND execution_environment_id IS NULL)
        OR (reference_kind = 'channel' AND participant_id IS NULL
            AND participant_kind IS NULL AND channel_id IS NOT NULL
            AND working_folder_id IS NULL AND path_kind IS NULL
            AND relative_path IS NULL AND execution_environment_id IS NULL)
        OR (reference_kind = 'working_folder' AND participant_id IS NULL
            AND participant_kind IS NULL AND channel_id IS NULL
            AND working_folder_id IS NOT NULL AND path_kind IS NULL
            AND relative_path IS NULL AND execution_environment_id IS NULL)
        OR (reference_kind = 'workspace_path' AND participant_id IS NULL
            AND participant_kind IS NULL AND channel_id IS NULL
            AND working_folder_id IS NOT NULL AND path_kind IS NOT NULL
            AND relative_path IS NOT NULL AND execution_environment_id IS NULL)
        OR (reference_kind = 'execution_environment' AND participant_id IS NULL
            AND participant_kind IS NULL AND channel_id IS NULL
            AND working_folder_id IS NULL AND path_kind IS NULL
            AND relative_path IS NULL AND execution_environment_id IS NOT NULL)
    )
) STRICT;

CREATE INDEX idx_chat_scheduled_message_references_target
ON chat_scheduled_message_references(scheduled_message_id, reference_kind, ordinal);

CREATE TRIGGER chat_teammate_policy_immutable_delete
BEFORE DELETE ON chat_teammate_policy_revisions
WHEN NOT EXISTS (
    SELECT 1
    FROM chat_participants participant
    WHERE participant.id = OLD.teammate_id
      AND participant.participant_kind = 'ai_teammate'
      AND participant.archived_at IS NOT NULL
)
OR EXISTS (
    SELECT 1
    FROM chat_communication_messages message
    WHERE message.author_participant_id = OLD.teammate_id
)
OR EXISTS (
    SELECT 1
    FROM chat_participant_reference_targets target
    WHERE target.participant_id = OLD.teammate_id
)
OR EXISTS (
    SELECT 1
    FROM chat_work_assignments assignment
    WHERE assignment.teammate_id = OLD.teammate_id
)
BEGIN
    SELECT RAISE(ABORT, 'Teammate policy revisions are immutable');
END;

DROP TABLE chat_agent_runs;
DROP TABLE chat_assignment_authorization_decisions;

CREATE TABLE chat_assignment_authorization_revisions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    assignment_id TEXT NOT NULL REFERENCES chat_work_assignments(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK (revision >= 1),
    requester_participant_id TEXT NOT NULL REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    teammate_policy_revision_id TEXT NOT NULL REFERENCES chat_teammate_policy_revisions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    teammate_access_revision INTEGER NOT NULL CHECK (teammate_access_revision >= 1),
    destination_conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    access_profile_revision_id TEXT NOT NULL REFERENCES chat_access_profile_revisions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    execution_environment_id TEXT REFERENCES chat_execution_environments(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    working_folder_id TEXT REFERENCES project_working_folders(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    resolved_runtime_approval_policy TEXT NOT NULL CHECK (
        resolved_runtime_approval_policy IN (
            'ask', 'auto_approve', 'unattended', 'provider_custom'
        )
    ),
    scope_digest TEXT NOT NULL CHECK (length(scope_digest) = 64),
    decision_state TEXT NOT NULL CHECK (decision_state IN ('allowed', 'blocked', 'revoked')),
    reason TEXT NOT NULL DEFAULT '' CHECK (length(reason) <= 4000),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    revoked_at TEXT CHECK (revoked_at IS NULL OR length(revoked_at) >= 20),
    UNIQUE (assignment_id, revision),
    CHECK (
        (execution_environment_id IS NULL AND working_folder_id IS NULL)
        OR execution_environment_id IS NOT NULL
    )
) STRICT;

CREATE INDEX idx_chat_assignment_authorization_active
ON chat_assignment_authorization_revisions(assignment_id, decision_state, revision DESC);

CREATE TABLE chat_assignment_authorized_channel_sources (
    authorization_revision_id TEXT NOT NULL REFERENCES chat_assignment_authorization_revisions(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    source_handle TEXT NOT NULL UNIQUE CHECK (length(source_handle) BETWEEN 32 AND 1024),
    message_reference_id TEXT NOT NULL REFERENCES chat_message_references(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    label_snapshot TEXT NOT NULL CHECK (length(label_snapshot) BETWEEN 1 AND 4096),
    lower_ordinal INTEGER NOT NULL CHECK (lower_ordinal >= 1),
    high_ordinal INTEGER NOT NULL CHECK (high_ordinal >= lower_ordinal),
    source_revision_cutoff_id TEXT NOT NULL REFERENCES chat_communication_message_revisions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    destination_audience_revision INTEGER NOT NULL CHECK (destination_audience_revision >= 1),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    PRIMARY KEY (authorization_revision_id, conversation_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE chat_assignment_authorized_folder_sources (
    authorization_revision_id TEXT NOT NULL REFERENCES chat_assignment_authorization_revisions(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    root_handle TEXT NOT NULL UNIQUE CHECK (length(root_handle) BETWEEN 32 AND 1024),
    working_folder_id TEXT NOT NULL REFERENCES project_working_folders(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    capability TEXT NOT NULL CHECK (
        capability IN ('read', 'edit', 'execute', 'publish')
    ),
    is_execution_target INTEGER NOT NULL DEFAULT 0 CHECK (is_execution_target IN (0, 1)),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    PRIMARY KEY (authorization_revision_id, working_folder_id)
) STRICT, WITHOUT ROWID;

CREATE UNIQUE INDEX idx_chat_authorized_folder_execution_target
ON chat_assignment_authorized_folder_sources(authorization_revision_id)
WHERE is_execution_target = 1;

CREATE TABLE chat_agent_runs (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    assignment_id TEXT NOT NULL REFERENCES chat_work_assignments(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES projects(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    working_folder_id TEXT,
    execution_environment_id TEXT NOT NULL REFERENCES chat_execution_environments(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    scratch_generation_id TEXT REFERENCES chat_scratch_generations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    teammate_policy_revision_id TEXT NOT NULL REFERENCES chat_teammate_policy_revisions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    authorization_revision_id TEXT NOT NULL REFERENCES chat_assignment_authorization_revisions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    authorization_scope_digest TEXT NOT NULL CHECK (length(authorization_scope_digest) = 64),
    provider_turn_id TEXT NOT NULL UNIQUE REFERENCES chat_turns(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    provider_thread_id TEXT REFERENCES chat_threads(id)
        ON UPDATE CASCADE ON DELETE SET NULL,
    state TEXT NOT NULL CHECK (
        state IN ('queued', 'starting', 'working', 'waiting', 'completed', 'failed', 'cancelled')
    ),
    run_ordinal INTEGER NOT NULL CHECK (run_ordinal >= 1),
    started_at TEXT CHECK (started_at IS NULL OR length(started_at) >= 20),
    settled_at TEXT CHECK (settled_at IS NULL OR length(settled_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    FOREIGN KEY (working_folder_id, project_id)
        REFERENCES project_working_folders(id, project_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    UNIQUE (assignment_id, run_ordinal),
    CHECK (
        (working_folder_id IS NOT NULL AND scratch_generation_id IS NULL)
        OR (working_folder_id IS NULL AND scratch_generation_id IS NOT NULL)
    )
) STRICT;

CREATE INDEX idx_chat_agent_runs_provider_thread
ON chat_agent_runs(provider_thread_id, assignment_id);

CREATE INDEX idx_chat_agent_runs_authorization
ON chat_agent_runs(authorization_revision_id, state, id);

CREATE TRIGGER chat_agent_run_execution_target_insert
BEFORE INSERT ON chat_agent_runs
WHEN NOT EXISTS (
    SELECT 1 FROM chat_execution_environments environment
    WHERE environment.id = NEW.execution_environment_id
      AND (
        (
            NEW.working_folder_id IS NOT NULL
            AND environment.working_folder_id = NEW.working_folder_id
            AND environment.scratch_generation_id IS NULL
        ) OR (
            NEW.scratch_generation_id IS NOT NULL
            AND environment.scratch_generation_id = NEW.scratch_generation_id
            AND environment.working_folder_id IS NULL
        )
      )
)
BEGIN
    SELECT RAISE(ABORT, 'Agent run execution target is inconsistent');
END;

CREATE TRIGGER chat_agent_run_authorization_insert
BEFORE INSERT ON chat_agent_runs
WHEN NOT EXISTS (
    SELECT 1 FROM chat_assignment_authorization_revisions authorization
    WHERE authorization.id = NEW.authorization_revision_id
      AND authorization.assignment_id = NEW.assignment_id
      AND authorization.scope_digest = NEW.authorization_scope_digest
      AND authorization.decision_state = 'allowed'
      AND authorization.revoked_at IS NULL
)
BEGIN
    SELECT RAISE(ABORT, 'Agent run authorization is invalid');
END;

CREATE TABLE chat_host_tool_invocations (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    authorization_revision_id TEXT NOT NULL REFERENCES chat_assignment_authorization_revisions(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    tool_name TEXT NOT NULL CHECK (length(tool_name) BETWEEN 1 AND 160),
    request_hash TEXT NOT NULL CHECK (length(request_hash) = 64),
    decision_state TEXT NOT NULL CHECK (decision_state IN ('allowed', 'denied')),
    denial_category TEXT CHECK (
        denial_category IS NULL OR length(denial_category) BETWEEN 1 AND 160
    ),
    response_bytes INTEGER NOT NULL DEFAULT 0 CHECK (response_bytes >= 0),
    truncated INTEGER NOT NULL DEFAULT 0 CHECK (truncated IN (0, 1)),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_host_tool_invocations_authorization
ON chat_host_tool_invocations(authorization_revision_id, created_at, id);

CREATE TABLE chat_host_tool_returned_message_revisions (
    invocation_id TEXT NOT NULL REFERENCES chat_host_tool_invocations(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    message_revision_id TEXT NOT NULL REFERENCES chat_communication_message_revisions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    content_sha256 TEXT NOT NULL CHECK (length(content_sha256) = 64),
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    PRIMARY KEY (invocation_id, message_revision_id),
    UNIQUE (invocation_id, ordinal)
) STRICT, WITHOUT ROWID;

CREATE TABLE chat_scratch_scopes (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    reply_thread_id TEXT NOT NULL REFERENCES chat_reply_threads(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    teammate_id TEXT NOT NULL REFERENCES chat_ai_teammates(participant_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    lifecycle_state TEXT NOT NULL DEFAULT 'active' CHECK (
        lifecycle_state IN ('active', 'archived', 'cleanup_pending', 'cleanup_failed', 'removed')
    ),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    removed_at TEXT CHECK (removed_at IS NULL OR length(removed_at) >= 20),
    UNIQUE (reply_thread_id, teammate_id)
) STRICT;

CREATE TABLE chat_scratch_generations (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    scratch_scope_id TEXT NOT NULL REFERENCES chat_scratch_scopes(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    generation INTEGER NOT NULL CHECK (generation >= 1),
    lifecycle_state TEXT NOT NULL DEFAULT 'active' CHECK (
        lifecycle_state IN ('active', 'quarantined', 'cleanup_pending', 'cleanup_failed', 'removed')
    ),
    byte_size INTEGER NOT NULL DEFAULT 0 CHECK (byte_size >= 0),
    provenance_digest TEXT CHECK (provenance_digest IS NULL OR length(provenance_digest) = 64),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    removed_at TEXT CHECK (removed_at IS NULL OR length(removed_at) >= 20),
    UNIQUE (scratch_scope_id, generation)
) STRICT;

CREATE UNIQUE INDEX idx_chat_scratch_generation_active
ON chat_scratch_generations(scratch_scope_id)
WHERE lifecycle_state = 'active';

INSERT INTO chat_execution_environments (
    id, working_folder_id, kind, display_name, repository_identity,
    lifecycle_state, created_at, updated_at
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

CREATE TABLE chat_scratch_generation_sources (
    scratch_generation_id TEXT NOT NULL REFERENCES chat_scratch_generations(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    lower_ordinal INTEGER NOT NULL CHECK (lower_ordinal >= 1),
    high_ordinal INTEGER NOT NULL CHECK (high_ordinal >= lower_ordinal),
    audience_revision INTEGER NOT NULL CHECK (audience_revision >= 1),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    PRIMARY KEY (scratch_generation_id, conversation_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE chat_access_revocation_jobs (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    teammate_id TEXT NOT NULL REFERENCES chat_ai_teammates(participant_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    conversation_id TEXT REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    authorization_revision_id TEXT REFERENCES chat_assignment_authorization_revisions(id)
        ON UPDATE CASCADE ON DELETE SET NULL,
    state TEXT NOT NULL DEFAULT 'queued' CHECK (
        state IN ('queued', 'claimed', 'completed', 'failed', 'cancelled')
    ),
    reason TEXT NOT NULL CHECK (length(reason) BETWEEN 1 AND 4000),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    available_at TEXT NOT NULL CHECK (length(available_at) >= 20),
    claimed_at TEXT CHECK (claimed_at IS NULL OR length(claimed_at) >= 20),
    claim_token TEXT CHECK (claim_token IS NULL OR length(claim_token) BETWEEN 1 AND 1024),
    last_error TEXT CHECK (last_error IS NULL OR length(last_error) <= 4000),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_access_revocation_jobs_ready
ON chat_access_revocation_jobs(state, available_at, created_at, id);

CREATE TRIGGER chat_channels_create_project_general
AFTER INSERT ON project_working_folders
WHEN NEW.kind = 'managed' AND NOT EXISTS (
    SELECT 1 FROM chat_channels WHERE project_id = NEW.project_id AND is_default = 1
)
BEGIN
    INSERT INTO chat_conversations (
        id, project_id, conversation_kind, last_activity_at, created_at, updated_at
    ) VALUES (
        'conversation:' || lower(hex(randomblob(16))),
        NEW.project_id,
        'channel',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

    INSERT INTO chat_channels (
        id, project_id, conversation_id, name, topic, is_default, created_at, updated_at
    ) VALUES (
        'channel:' || lower(hex(randomblob(16))),
        NEW.project_id,
        (SELECT id FROM chat_conversations
         WHERE project_id = NEW.project_id AND conversation_kind = 'channel'
         ORDER BY rowid DESC LIMIT 1),
        'general',
        '',
        1,
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

    INSERT INTO chat_conversation_memberships (
        conversation_id, participant_id, membership_role, created_at, updated_at
    ) VALUES (
        (SELECT conversation_id FROM chat_channels
         WHERE project_id = NEW.project_id AND is_default = 1),
        'participant:local-owner',
        'owner',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

    INSERT INTO chat_project_primary_working_folders (
        project_id, working_folder_id, created_at, updated_at
    ) VALUES (
        NEW.project_id,
        NEW.id,
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    );
END;
