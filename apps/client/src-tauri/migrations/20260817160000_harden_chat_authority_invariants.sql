CREATE TRIGGER chat_execution_environment_authority_immutable
BEFORE UPDATE OF
    id, working_folder_id, scratch_generation_id, kind, repository_identity
ON chat_execution_environments
BEGIN
    SELECT RAISE(ABORT, 'Execution environment authority is immutable');
END;

CREATE TRIGGER chat_assignment_authorization_target_required_insert
BEFORE INSERT ON chat_assignment_authorization_revisions
WHEN (
    NEW.decision_state = 'allowed'
    AND NEW.execution_environment_id IS NULL
) OR (
    NEW.execution_environment_id IS NOT NULL
    AND NOT EXISTS (
        SELECT 1
        FROM chat_execution_environments environment
        WHERE environment.id = NEW.execution_environment_id
          AND (
            (
                NEW.working_folder_id IS NOT NULL
                AND environment.kind IN ('current_folder', 'worktree')
                AND environment.working_folder_id = NEW.working_folder_id
                AND environment.scratch_generation_id IS NULL
            ) OR (
                NEW.working_folder_id IS NULL
                AND environment.kind = 'scratch'
                AND environment.working_folder_id IS NULL
                AND environment.scratch_generation_id IS NOT NULL
            )
          )
    )
)
BEGIN
    SELECT RAISE(ABORT, 'Assignment authorization requires one exact execution target');
END;

CREATE TRIGGER chat_assignment_authorization_revocation_insert
BEFORE INSERT ON chat_assignment_authorization_revisions
WHEN (NEW.decision_state = 'revoked') != (NEW.revoked_at IS NOT NULL)
BEGIN
    SELECT RAISE(ABORT, 'Authorization revocation state is inconsistent');
END;

CREATE TRIGGER chat_assignment_authorization_revocation_update
BEFORE UPDATE OF decision_state, revoked_at ON chat_assignment_authorization_revisions
WHEN (NEW.decision_state = 'revoked') != (NEW.revoked_at IS NOT NULL)
BEGIN
    SELECT RAISE(ABORT, 'Authorization revocation state is inconsistent');
END;

CREATE TRIGGER chat_assignment_authorization_state_transition
BEFORE UPDATE OF decision_state ON chat_assignment_authorization_revisions
WHEN NEW.decision_state IS NOT OLD.decision_state
 AND NOT (OLD.decision_state = 'allowed' AND NEW.decision_state = 'revoked')
BEGIN
    SELECT RAISE(ABORT, 'Authorization decisions require a new revision');
END;

CREATE TRIGGER chat_assignment_authorization_reason_immutable
BEFORE UPDATE OF reason ON chat_assignment_authorization_revisions
WHEN NEW.reason IS NOT OLD.reason
 AND NOT (OLD.decision_state = 'allowed' AND NEW.decision_state = 'revoked')
BEGIN
    SELECT RAISE(ABORT, 'Authorization reasons are immutable before revocation');
END;

CREATE TRIGGER chat_assignment_authorization_terminal_immutable
BEFORE UPDATE ON chat_assignment_authorization_revisions
WHEN OLD.decision_state = 'revoked' OR OLD.revoked_at IS NOT NULL
BEGIN
    SELECT RAISE(ABORT, 'Revoked authorization revisions are immutable');
END;

CREATE TRIGGER chat_assignment_authorization_id_immutable
BEFORE UPDATE OF id ON chat_assignment_authorization_revisions
BEGIN
    SELECT RAISE(ABORT, 'Assignment authorization identity is immutable');
END;

CREATE TRIGGER chat_assignment_authorization_delete_denied
BEFORE DELETE ON chat_assignment_authorization_revisions
BEGIN
    SELECT RAISE(ABORT, 'Assignment authorization revisions are immutable');
END;

CREATE TRIGGER chat_agent_run_id_immutable
BEFORE UPDATE OF id ON chat_agent_runs
BEGIN
    SELECT RAISE(ABORT, 'Agent run identity is immutable');
END;

CREATE TRIGGER chat_teammate_access_revision_monotonic
BEFORE UPDATE OF access_revision ON chat_ai_teammate_access_state
WHEN NEW.access_revision <= OLD.access_revision
BEGIN
    SELECT RAISE(ABORT, 'Teammate access revisions must increase');
END;

CREATE TRIGGER chat_conversation_audience_revision_monotonic
BEFORE UPDATE OF revision ON chat_conversation_audience_state
WHEN NEW.revision <= OLD.revision
BEGIN
    SELECT RAISE(ABORT, 'Conversation audience revisions must increase');
END;

CREATE TRIGGER chat_access_profile_revision_counter_monotonic
BEFORE UPDATE OF revision ON chat_access_profiles
WHEN NEW.revision <= OLD.revision
BEGIN
    SELECT RAISE(ABORT, 'Access profile revisions must increase');
END;

CREATE TRIGGER chat_access_profile_latest_revision_monotonic
BEFORE UPDATE OF latest_revision ON chat_access_profiles
WHEN NEW.latest_revision <= OLD.latest_revision
BEGIN
    SELECT RAISE(ABORT, 'Latest access profile revisions must increase');
END;

CREATE TRIGGER chat_scratch_scope_identity_immutable
BEFORE UPDATE OF id, reply_thread_id, teammate_id, created_at ON chat_scratch_scopes
BEGIN
    SELECT RAISE(ABORT, 'Scratch scope identity is immutable');
END;

CREATE TRIGGER chat_scratch_generation_identity_immutable
BEFORE UPDATE OF id, scratch_scope_id, generation, created_at ON chat_scratch_generations
BEGIN
    SELECT RAISE(ABORT, 'Scratch generation identity is immutable');
END;

CREATE TRIGGER chat_scratch_generation_source_monotonic
BEFORE UPDATE ON chat_scratch_generation_sources
WHEN NEW.scratch_generation_id IS NOT OLD.scratch_generation_id
  OR NEW.conversation_id IS NOT OLD.conversation_id
  OR NEW.lower_ordinal > OLD.lower_ordinal
  OR NEW.high_ordinal < OLD.high_ordinal
  OR NEW.audience_revision < OLD.audience_revision
  OR NEW.created_at IS NOT OLD.created_at
BEGIN
    SELECT RAISE(ABORT, 'Scratch source constraints can only expand');
END;

CREATE TRIGGER chat_scratch_generation_source_delete_denied
BEFORE DELETE ON chat_scratch_generation_sources
BEGIN
    SELECT RAISE(ABORT, 'Scratch source constraints are immutable');
END;
