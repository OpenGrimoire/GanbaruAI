CREATE TABLE chat_communication_message_revision_ordinals (
    ordinal INTEGER PRIMARY KEY AUTOINCREMENT,
    message_revision_id TEXT NOT NULL UNIQUE
        REFERENCES chat_communication_message_revisions(id)
        ON UPDATE CASCADE ON DELETE CASCADE
) STRICT;

INSERT INTO chat_communication_message_revision_ordinals (message_revision_id)
SELECT id
FROM chat_communication_message_revisions
ORDER BY rowid;

CREATE TRIGGER chat_communication_message_revision_assign_ordinal
AFTER INSERT ON chat_communication_message_revisions
BEGIN
    INSERT INTO chat_communication_message_revision_ordinals (message_revision_id)
    VALUES (NEW.id);
END;

ALTER TABLE chat_assignment_authorized_folder_sources
ADD COLUMN resolved_runtime_approval_policy TEXT CHECK (
    resolved_runtime_approval_policy IS NULL OR resolved_runtime_approval_policy IN (
        'ask', 'auto_approve', 'unattended', 'provider_custom'
    )
);

UPDATE chat_assignment_authorized_folder_sources
SET resolved_runtime_approval_policy = (
    SELECT authorization.resolved_runtime_approval_policy
    FROM chat_assignment_authorization_revisions authorization
    WHERE authorization.id = chat_assignment_authorized_folder_sources.authorization_revision_id
)
WHERE resolved_runtime_approval_policy IS NULL;

CREATE TRIGGER chat_authorized_folder_source_requires_approval
BEFORE INSERT ON chat_assignment_authorized_folder_sources
WHEN NEW.resolved_runtime_approval_policy IS NULL
BEGIN
    SELECT RAISE(ABORT, 'Authorized folder sources require a frozen approval policy');
END;

CREATE TRIGGER chat_authorized_channel_source_immutable_update
BEFORE UPDATE ON chat_assignment_authorized_channel_sources
BEGIN
    SELECT RAISE(ABORT, 'Authorized channel sources are immutable');
END;

CREATE TRIGGER chat_authorized_channel_source_immutable_delete
BEFORE DELETE ON chat_assignment_authorized_channel_sources
BEGIN
    SELECT RAISE(ABORT, 'Authorized channel sources are immutable');
END;

CREATE TRIGGER chat_authorized_folder_source_immutable_update
BEFORE UPDATE ON chat_assignment_authorized_folder_sources
BEGIN
    SELECT RAISE(ABORT, 'Authorized folder sources are immutable');
END;

CREATE TRIGGER chat_authorized_folder_source_immutable_delete
BEFORE DELETE ON chat_assignment_authorized_folder_sources
BEGIN
    SELECT RAISE(ABORT, 'Authorized folder sources are immutable');
END;

CREATE TRIGGER chat_assignment_authorization_scope_immutable
BEFORE UPDATE OF
    assignment_id, revision, requester_participant_id,
    teammate_policy_revision_id, teammate_access_revision,
    destination_conversation_id, access_profile_revision_id,
    execution_environment_id, working_folder_id,
    resolved_runtime_approval_policy, scope_digest, created_at
ON chat_assignment_authorization_revisions
BEGIN
    SELECT RAISE(ABORT, 'Assignment authorization scopes are immutable');
END;

CREATE TRIGGER chat_agent_run_authorization_target_insert
BEFORE INSERT ON chat_agent_runs
WHEN NOT EXISTS (
    SELECT 1
    FROM chat_assignment_authorization_revisions authorization
    WHERE authorization.id = NEW.authorization_revision_id
      AND authorization.assignment_id = NEW.assignment_id
      AND authorization.scope_digest = NEW.authorization_scope_digest
      AND authorization.execution_environment_id = NEW.execution_environment_id
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
