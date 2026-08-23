-- This is a pre-user Chat coordination reset. Development vaults must be recreated.
DELETE FROM chat_work_assignments;
DELETE FROM chat_threads;

DROP TRIGGER IF EXISTS chat_assignment_authorization_target_required_insert;
DROP TRIGGER IF EXISTS chat_agent_run_authorization_target_insert;
DROP TRIGGER IF EXISTS chat_agent_run_authority_immutable;
DROP TRIGGER IF EXISTS chat_agent_run_id_immutable;
DROP TRIGGER IF EXISTS chat_agent_run_execution_target_insert;
DROP TRIGGER IF EXISTS chat_agent_run_authorization_insert;
DROP TABLE chat_agent_runs;

DROP TRIGGER IF EXISTS chat_threads_project_matches_working_folder_insert;
DROP TRIGGER IF EXISTS chat_threads_project_matches_working_folder_update;
DROP TRIGGER IF EXISTS chat_threads_assign_current_environment;
DROP TRIGGER IF EXISTS chat_threads_execution_target_insert;
DROP TRIGGER IF EXISTS chat_threads_execution_target_update;
DROP TABLE chat_threads;

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
        (working_folder_id IS NULL AND execution_environment_id IS NULL
            AND scratch_generation_id IS NULL)
        OR (working_folder_id IS NOT NULL AND scratch_generation_id IS NULL)
        OR (working_folder_id IS NULL AND execution_environment_id IS NOT NULL
            AND scratch_generation_id IS NOT NULL)
    ),
    CHECK (
        (resume_cursor_schema_version IS NULL AND resume_cursor_data IS NULL)
        OR (resume_cursor_schema_version IS NOT NULL AND resume_cursor_data IS NOT NULL)
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
WHEN NOT (
    (NEW.working_folder_id IS NULL AND NEW.execution_environment_id IS NULL
        AND NEW.scratch_generation_id IS NULL)
    OR (NEW.working_folder_id IS NOT NULL
        AND NEW.execution_environment_id IS NULL
        AND NEW.scratch_generation_id IS NULL)
    OR EXISTS (
        SELECT 1 FROM chat_execution_environments environment
        WHERE environment.id = NEW.execution_environment_id
          AND (
            (NEW.working_folder_id IS NOT NULL
                AND environment.working_folder_id = NEW.working_folder_id
                AND NEW.scratch_generation_id IS NULL
                AND environment.scratch_generation_id IS NULL)
            OR (NEW.working_folder_id IS NULL
                AND NEW.scratch_generation_id IS NOT NULL
                AND environment.scratch_generation_id = NEW.scratch_generation_id
                AND environment.working_folder_id IS NULL)
          )
    )
)
BEGIN
    SELECT RAISE(ABORT, 'Chat thread execution target is inconsistent');
END;

CREATE TRIGGER chat_threads_execution_target_update
BEFORE UPDATE OF working_folder_id, execution_environment_id, scratch_generation_id ON chat_threads
WHEN NOT (
    (NEW.working_folder_id IS NULL AND NEW.execution_environment_id IS NULL
        AND NEW.scratch_generation_id IS NULL)
    OR EXISTS (
        SELECT 1 FROM chat_execution_environments environment
        WHERE environment.id = NEW.execution_environment_id
          AND (
            (NEW.working_folder_id IS NOT NULL
                AND environment.working_folder_id = NEW.working_folder_id
                AND NEW.scratch_generation_id IS NULL
                AND environment.scratch_generation_id IS NULL)
            OR (NEW.working_folder_id IS NULL
                AND NEW.scratch_generation_id IS NOT NULL
                AND environment.scratch_generation_id = NEW.scratch_generation_id
                AND environment.working_folder_id IS NULL)
          )
    )
)
BEGIN
    SELECT RAISE(ABORT, 'Chat thread execution target is inconsistent');
END;

CREATE TABLE chat_agent_runs (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    assignment_id TEXT NOT NULL REFERENCES chat_work_assignments(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES projects(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    working_folder_id TEXT,
    execution_environment_id TEXT REFERENCES chat_execution_environments(id)
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
        (working_folder_id IS NULL AND execution_environment_id IS NULL
            AND scratch_generation_id IS NULL)
        OR (working_folder_id IS NOT NULL AND execution_environment_id IS NOT NULL
            AND scratch_generation_id IS NULL)
        OR (working_folder_id IS NULL AND execution_environment_id IS NOT NULL
            AND scratch_generation_id IS NOT NULL)
    )
) STRICT;

CREATE INDEX idx_chat_agent_runs_provider_thread
ON chat_agent_runs(provider_thread_id, assignment_id);

CREATE INDEX idx_chat_agent_runs_authorization
ON chat_agent_runs(authorization_revision_id, state, id);

CREATE TRIGGER chat_agent_run_execution_target_insert
BEFORE INSERT ON chat_agent_runs
WHEN NOT (
    (NEW.working_folder_id IS NULL AND NEW.execution_environment_id IS NULL
        AND NEW.scratch_generation_id IS NULL)
    OR EXISTS (
        SELECT 1 FROM chat_execution_environments environment
        WHERE environment.id = NEW.execution_environment_id
          AND (
            (NEW.working_folder_id IS NOT NULL
                AND environment.working_folder_id = NEW.working_folder_id
                AND NEW.scratch_generation_id IS NULL
                AND environment.scratch_generation_id IS NULL)
            OR (NEW.working_folder_id IS NULL
                AND NEW.scratch_generation_id IS NOT NULL
                AND environment.scratch_generation_id = NEW.scratch_generation_id
                AND environment.working_folder_id IS NULL)
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

CREATE TRIGGER chat_agent_run_authorization_target_insert
BEFORE INSERT ON chat_agent_runs
WHEN NOT EXISTS (
    SELECT 1 FROM chat_assignment_authorization_revisions authorization
    WHERE authorization.id = NEW.authorization_revision_id
      AND authorization.assignment_id = NEW.assignment_id
      AND authorization.scope_digest = NEW.authorization_scope_digest
      AND authorization.execution_environment_id IS NEW.execution_environment_id
      AND authorization.working_folder_id IS NEW.working_folder_id
      AND authorization.decision_state = 'allowed'
      AND authorization.revoked_at IS NULL
)
BEGIN
    SELECT RAISE(ABORT, 'Agent run target does not match its authorization');
END;

CREATE TRIGGER chat_agent_run_authority_immutable
BEFORE UPDATE OF
    assignment_id, project_id, working_folder_id,
    execution_environment_id, scratch_generation_id,
    teammate_policy_revision_id, authorization_revision_id,
    authorization_scope_digest, provider_turn_id,
    provider_thread_id, run_ordinal, created_at
ON chat_agent_runs
BEGIN
    SELECT RAISE(ABORT, 'Agent run authority is immutable');
END;

CREATE TRIGGER chat_agent_run_id_immutable
BEFORE UPDATE OF id ON chat_agent_runs
BEGIN
    SELECT RAISE(ABORT, 'Agent run identity is immutable');
END;

CREATE TRIGGER chat_assignment_authorization_target_required_insert
BEFORE INSERT ON chat_assignment_authorization_revisions
WHEN NOT (
    (NEW.working_folder_id IS NULL AND NEW.execution_environment_id IS NULL)
    OR EXISTS (
        SELECT 1 FROM chat_execution_environments environment
        WHERE environment.id = NEW.execution_environment_id
          AND (
            (NEW.working_folder_id IS NOT NULL
                AND environment.kind IN ('current_folder', 'worktree')
                AND environment.working_folder_id = NEW.working_folder_id
                AND environment.scratch_generation_id IS NULL)
            OR (NEW.working_folder_id IS NULL
                AND environment.kind = 'scratch'
                AND environment.working_folder_id IS NULL
                AND environment.scratch_generation_id IS NOT NULL)
          )
    )
)
BEGIN
    SELECT RAISE(ABORT, 'Assignment authorization target is inconsistent');
END;
