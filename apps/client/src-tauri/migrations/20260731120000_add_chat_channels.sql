CREATE TABLE chat_participants (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    participant_kind TEXT NOT NULL CHECK (
        participant_kind IN ('local_user', 'ai_teammate', 'human')
    ),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) BETWEEN 1 AND 160),
    normalized_handle TEXT CHECK (
        normalized_handle IS NULL OR (
            normalized_handle = lower(normalized_handle)
            AND length(normalized_handle) BETWEEN 1 AND 80
            AND normalized_handle NOT GLOB '*[^a-z0-9._-]*'
        )
    ),
    avatar_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (avatar_schema_version >= 1),
    avatar_data TEXT NOT NULL DEFAULT '{"kind":"initials"}' CHECK (
        json_valid(avatar_data) AND json_type(avatar_data) = 'object'
    ),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    archived_at TEXT CHECK (archived_at IS NULL OR length(archived_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    CHECK (
        (participant_kind = 'local_user' AND normalized_handle IS NULL)
        OR (participant_kind != 'local_user' AND normalized_handle IS NOT NULL)
    )
) STRICT;

CREATE UNIQUE INDEX idx_chat_participants_handle
ON chat_participants(normalized_handle)
WHERE normalized_handle IS NOT NULL;

CREATE UNIQUE INDEX idx_chat_participants_local_user
ON chat_participants(participant_kind)
WHERE participant_kind = 'local_user';

INSERT INTO chat_participants (
    id, participant_kind, display_name, normalized_handle, created_at, updated_at
) VALUES (
    'participant:local-owner',
    'local_user',
    'You',
    NULL,
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
);

CREATE TABLE chat_ai_teammates (
    participant_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    purpose TEXT NOT NULL DEFAULT '' CHECK (length(purpose) <= 1000),
    instructions TEXT NOT NULL DEFAULT '' CHECK (length(instructions) <= 65536),
    latest_policy_revision INTEGER NOT NULL DEFAULT 0 CHECK (latest_policy_revision >= 0),
    configuration_state TEXT NOT NULL DEFAULT 'needs_setup' CHECK (
        configuration_state IN ('healthy', 'needs_setup', 'provider_unavailable', 'folder_access_missing')
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE TABLE chat_teammate_policy_revisions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    teammate_id TEXT NOT NULL REFERENCES chat_ai_teammates(participant_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    revision INTEGER NOT NULL CHECK (revision >= 1),
    provider_instance_id TEXT NOT NULL CHECK (length(provider_instance_id) BETWEEN 1 AND 1024),
    model_selection_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (
        model_selection_schema_version >= 1
    ),
    model_selection_data TEXT NOT NULL CHECK (
        json_valid(model_selection_data) AND json_type(model_selection_data) = 'object'
    ),
    effort TEXT CHECK (
        effort IS NULL OR effort IN ('none', 'minimal', 'low', 'medium', 'high', 'xhigh')
    ),
    speed TEXT CHECK (speed IS NULL OR speed IN ('standard', 'fast')),
    provider_options_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (
        provider_options_schema_version >= 1
    ),
    provider_options_data TEXT NOT NULL DEFAULT '{}' CHECK (
        json_valid(provider_options_data) AND json_type(provider_options_data) = 'object'
    ),
    interaction_mode TEXT NOT NULL DEFAULT 'build' CHECK (interaction_mode = 'build'),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    UNIQUE (teammate_id, revision)
) STRICT;

CREATE TRIGGER chat_teammate_policy_identity_insert
BEFORE INSERT ON chat_teammate_policy_revisions
WHEN NOT EXISTS (
    SELECT 1
    FROM chat_participants participant
    WHERE participant.id = NEW.teammate_id
      AND participant.participant_kind = 'ai_teammate'
)
BEGIN
    SELECT RAISE(ABORT, 'Teammate policies require an AI teammate participant');
END;

CREATE TRIGGER chat_teammate_policy_immutable_update
BEFORE UPDATE ON chat_teammate_policy_revisions
BEGIN
    SELECT RAISE(ABORT, 'Teammate policy revisions are immutable');
END;

CREATE TRIGGER chat_teammate_policy_immutable_delete
BEFORE DELETE ON chat_teammate_policy_revisions
BEGIN
    SELECT RAISE(ABORT, 'Teammate policy revisions are immutable');
END;

CREATE TRIGGER chat_teammate_policy_publish
AFTER INSERT ON chat_teammate_policy_revisions
BEGIN
    UPDATE chat_ai_teammates
    SET latest_policy_revision = NEW.revision,
        configuration_state = CASE
            WHEN configuration_state = 'needs_setup' THEN 'healthy'
            ELSE configuration_state
        END,
        updated_at = NEW.created_at
    WHERE participant_id = NEW.teammate_id
      AND latest_policy_revision < NEW.revision;
END;

CREATE TABLE chat_conversations (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    project_id TEXT NOT NULL REFERENCES projects(id) ON UPDATE CASCADE ON DELETE CASCADE,
    conversation_kind TEXT NOT NULL CHECK (
        conversation_kind IN ('channel', 'direct_message', 'task_discussion')
    ),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    last_activity_at TEXT NOT NULL CHECK (length(last_activity_at) >= 20),
    archived_at TEXT CHECK (archived_at IS NULL OR length(archived_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    UNIQUE (id, project_id)
) STRICT;

CREATE TABLE chat_channels (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    project_id TEXT NOT NULL REFERENCES projects(id) ON UPDATE CASCADE ON DELETE CASCADE,
    conversation_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 80),
    topic TEXT NOT NULL DEFAULT '' CHECK (length(topic) <= 250),
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    archived_at TEXT CHECK (archived_at IS NULL OR length(archived_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    FOREIGN KEY (conversation_id, project_id)
        REFERENCES chat_conversations(id, project_id)
        ON UPDATE CASCADE ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX idx_chat_channels_project_name
ON chat_channels(project_id, name COLLATE NOCASE);

CREATE UNIQUE INDEX idx_chat_channels_project_default
ON chat_channels(project_id)
WHERE is_default = 1;

CREATE INDEX idx_chat_channels_active_project
ON chat_channels(project_id, archived_at, updated_at DESC, id);

CREATE TRIGGER chat_channels_conversation_kind_insert
BEFORE INSERT ON chat_channels
WHEN NOT EXISTS (
    SELECT 1 FROM chat_conversations conversation
    WHERE conversation.id = NEW.conversation_id
      AND conversation.project_id = NEW.project_id
      AND conversation.conversation_kind = 'channel'
)
BEGIN
    SELECT RAISE(ABORT, 'Chat channel conversation is invalid');
END;

CREATE TRIGGER chat_channels_default_name_insert
BEFORE INSERT ON chat_channels
WHEN NEW.is_default = 1 AND NEW.name != 'general'
BEGIN
    SELECT RAISE(ABORT, 'The default Chat channel must be named general');
END;

CREATE TRIGGER chat_channels_default_protected_update
BEFORE UPDATE OF name, is_default, archived_at ON chat_channels
WHEN OLD.is_default = 1 AND (
    NEW.name != OLD.name OR NEW.is_default != 1 OR NEW.archived_at IS NOT NULL
)
BEGIN
    SELECT RAISE(ABORT, 'The default Chat channel is protected');
END;

CREATE TABLE chat_conversation_memberships (
    conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    participant_id TEXT NOT NULL REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    membership_role TEXT NOT NULL DEFAULT 'member' CHECK (
        membership_role IN ('owner', 'member')
    ),
    addressable INTEGER NOT NULL DEFAULT 1 CHECK (addressable IN (0, 1)),
    approval_policy TEXT NOT NULL DEFAULT 'ask_for_approval' CHECK (
        approval_policy IN ('ask_for_approval', 'approve_for_me', 'full_access', 'custom')
    ),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    removed_at TEXT CHECK (removed_at IS NULL OR length(removed_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    PRIMARY KEY (conversation_id, participant_id)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_chat_conversation_memberships_participant
ON chat_conversation_memberships(participant_id, removed_at, conversation_id);

CREATE TABLE chat_teammate_working_folder_grants (
    conversation_id TEXT NOT NULL,
    teammate_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    working_folder_id TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
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
    WHERE teammate.participant_id = NEW.teammate_id
)
BEGIN
    SELECT RAISE(ABORT, 'Working folder grants require an active teammate membership');
END;

CREATE TABLE chat_reply_threads (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    root_item_id TEXT NOT NULL UNIQUE CHECK (length(root_item_id) BETWEEN 1 AND 1024),
    reply_count INTEGER NOT NULL DEFAULT 0 CHECK (reply_count >= 0),
    last_activity_at TEXT NOT NULL CHECK (length(last_activity_at) >= 20),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_reply_threads_conversation
ON chat_reply_threads(conversation_id, last_activity_at DESC, id);

CREATE TABLE chat_conversation_items (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    reply_thread_id TEXT REFERENCES chat_reply_threads(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    item_kind TEXT NOT NULL CHECK (
        item_kind IN ('message', 'work_update', 'approval_request', 'question', 'review_packet')
    ),
    ordinal INTEGER NOT NULL CHECK (ordinal >= 1),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    UNIQUE (conversation_id, reply_thread_id, ordinal),
    CHECK (reply_thread_id IS NULL OR item_kind != 'message' OR ordinal >= 1)
) STRICT;

CREATE INDEX idx_chat_conversation_items_page
ON chat_conversation_items(conversation_id, reply_thread_id, ordinal DESC, id);

CREATE UNIQUE INDEX idx_chat_conversation_items_root_ordinal
ON chat_conversation_items(conversation_id, ordinal)
WHERE reply_thread_id IS NULL;

CREATE UNIQUE INDEX idx_chat_conversation_items_reply_ordinal
ON chat_conversation_items(reply_thread_id, ordinal)
WHERE reply_thread_id IS NOT NULL;

CREATE TRIGGER chat_reply_threads_root_item_insert
BEFORE INSERT ON chat_reply_threads
WHEN NOT EXISTS (
    SELECT 1 FROM chat_conversation_items item
    WHERE item.id = NEW.root_item_id
      AND item.conversation_id = NEW.conversation_id
      AND item.reply_thread_id IS NULL
      AND item.item_kind = 'message'
)
BEGIN
    SELECT RAISE(ABORT, 'Reply thread root must be a root communication message');
END;

CREATE TRIGGER chat_conversation_items_reply_thread_insert
BEFORE INSERT ON chat_conversation_items
WHEN NEW.reply_thread_id IS NOT NULL AND NOT EXISTS (
    SELECT 1 FROM chat_reply_threads thread
    WHERE thread.id = NEW.reply_thread_id
      AND thread.conversation_id = NEW.conversation_id
)
BEGIN
    SELECT RAISE(ABORT, 'Reply item belongs to another conversation');
END;

CREATE TABLE chat_communication_messages (
    item_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_conversation_items(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    author_participant_id TEXT NOT NULL REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    current_revision_id TEXT CHECK (
        current_revision_id IS NULL OR length(current_revision_id) BETWEEN 1 AND 1024
    ),
    edited_at TEXT CHECK (edited_at IS NULL OR length(edited_at) >= 20),
    deleted_at TEXT CHECK (deleted_at IS NULL OR length(deleted_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20)
) STRICT;

CREATE TABLE chat_communication_message_revisions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    message_item_id TEXT NOT NULL REFERENCES chat_communication_messages(item_id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK (revision >= 1),
    normalized_markdown TEXT NOT NULL CHECK (length(normalized_markdown) <= 131072),
    rich_content_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (rich_content_schema_version >= 1),
    rich_content_data TEXT NOT NULL DEFAULT '{"type":"doc","content":[]}' CHECK (
        json_valid(rich_content_data) AND json_type(rich_content_data) = 'object'
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    UNIQUE (message_item_id, revision),
    UNIQUE (id, message_item_id)
) STRICT;

CREATE TRIGGER chat_communication_message_current_revision_update
BEFORE UPDATE OF current_revision_id ON chat_communication_messages
WHEN NEW.current_revision_id IS NOT NULL AND NOT EXISTS (
    SELECT 1 FROM chat_communication_message_revisions revision
    WHERE revision.id = NEW.current_revision_id
      AND revision.message_item_id = NEW.item_id
)
BEGIN
    SELECT RAISE(ABORT, 'Current message revision belongs to another message');
END;

CREATE TRIGGER chat_communication_message_revision_immutable_update
BEFORE UPDATE ON chat_communication_message_revisions
BEGIN
    SELECT RAISE(ABORT, 'Communication message revisions are immutable');
END;

CREATE TABLE chat_participant_mentions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    message_revision_id TEXT NOT NULL REFERENCES chat_communication_message_revisions(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    participant_id TEXT NOT NULL REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    participant_kind TEXT NOT NULL CHECK (
        participant_kind IN ('local_user', 'ai_teammate', 'human')
    ),
    handle_snapshot TEXT CHECK (handle_snapshot IS NULL OR length(handle_snapshot) <= 80),
    label_snapshot TEXT NOT NULL CHECK (length(label_snapshot) BETWEEN 1 AND 160),
    start_offset INTEGER NOT NULL CHECK (start_offset >= 0),
    end_offset INTEGER NOT NULL CHECK (end_offset > start_offset),
    UNIQUE (message_revision_id, start_offset, end_offset)
) STRICT;

CREATE INDEX idx_chat_participant_mentions_participant
ON chat_participant_mentions(participant_id, message_revision_id);

CREATE TABLE chat_communication_attachment_references (
    message_revision_id TEXT NOT NULL REFERENCES chat_communication_message_revisions(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    attachment_id TEXT NOT NULL REFERENCES chat_attachments(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    PRIMARY KEY (message_revision_id, attachment_id),
    UNIQUE (message_revision_id, ordinal)
) STRICT, WITHOUT ROWID;

CREATE TABLE chat_communication_resource_references (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    message_revision_id TEXT NOT NULL REFERENCES chat_communication_message_revisions(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    working_folder_id TEXT NOT NULL REFERENCES project_working_folders(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    reference_kind TEXT NOT NULL CHECK (reference_kind IN ('file', 'folder')),
    relative_path TEXT NOT NULL CHECK (
        length(relative_path) BETWEEN 1 AND 4096
        AND relative_path NOT LIKE '/%'
        AND relative_path NOT LIKE '../%'
        AND relative_path NOT LIKE '%/../%'
        AND relative_path NOT LIKE '%/..'
        AND relative_path NOT LIKE '%\%'
    ),
    display_label TEXT NOT NULL CHECK (length(display_label) BETWEEN 1 AND 4096),
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    UNIQUE (message_revision_id, ordinal)
) STRICT;

CREATE TABLE chat_conversation_read_cursors (
    conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    participant_id TEXT NOT NULL REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    last_read_root_ordinal INTEGER NOT NULL DEFAULT 0 CHECK (last_read_root_ordinal >= 0),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    PRIMARY KEY (conversation_id, participant_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE chat_reply_thread_read_cursors (
    reply_thread_id TEXT NOT NULL REFERENCES chat_reply_threads(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    participant_id TEXT NOT NULL REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    last_read_reply_ordinal INTEGER NOT NULL DEFAULT 0 CHECK (last_read_reply_ordinal >= 0),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    PRIMARY KEY (reply_thread_id, participant_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE chat_organizational_drafts (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    participant_id TEXT NOT NULL REFERENCES chat_participants(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    reply_thread_id TEXT REFERENCES chat_reply_threads(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    normalized_markdown TEXT NOT NULL DEFAULT '' CHECK (length(normalized_markdown) <= 131072),
    rich_content_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (rich_content_schema_version >= 1),
    rich_content_data TEXT NOT NULL DEFAULT '{"type":"doc","content":[]}' CHECK (
        json_valid(rich_content_data) AND json_type(rich_content_data) = 'object'
    ),
    selection_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (selection_schema_version >= 1),
    selection_data TEXT NOT NULL DEFAULT '{}' CHECK (
        json_valid(selection_data) AND json_type(selection_data) = 'object'
    ),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE UNIQUE INDEX idx_chat_organizational_drafts_destination
ON chat_organizational_drafts(
    participant_id,
    conversation_id,
    ifnull(reply_thread_id, '')
);

CREATE TABLE chat_organizational_command_receipts (
    client_command_id TEXT PRIMARY KEY NOT NULL CHECK (length(client_command_id) BETWEEN 1 AND 1024),
    command_kind TEXT NOT NULL CHECK (
        command_kind IN ('post_message', 'publish_teammate', 'update_membership', 'assignment_action')
    ),
    state TEXT NOT NULL CHECK (state IN ('accepted', 'completed', 'failed')),
    result_schema_version INTEGER CHECK (result_schema_version IS NULL OR result_schema_version >= 1),
    result_data TEXT CHECK (result_data IS NULL OR json_valid(result_data)),
    error_data TEXT CHECK (error_data IS NULL OR json_valid(error_data)),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE TABLE chat_work_assignments (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    reply_thread_id TEXT NOT NULL REFERENCES chat_reply_threads(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    teammate_id TEXT NOT NULL REFERENCES chat_ai_teammates(participant_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    triggering_message_item_id TEXT NOT NULL REFERENCES chat_communication_messages(item_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    previous_assignment_id TEXT REFERENCES chat_work_assignments(id)
        ON UPDATE CASCADE ON DELETE SET NULL,
    state TEXT NOT NULL CHECK (
        state IN (
            'queued', 'working', 'waiting_for_answer', 'waiting_for_approval',
            'ready_for_review', 'completed', 'failed', 'cancelled'
        )
    ),
    state_reason TEXT CHECK (state_reason IS NULL OR length(state_reason) <= 4000),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    settled_at TEXT CHECK (settled_at IS NULL OR length(settled_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE UNIQUE INDEX idx_chat_work_assignments_one_active
ON chat_work_assignments(reply_thread_id)
WHERE state IN (
    'queued', 'working', 'waiting_for_answer', 'waiting_for_approval', 'ready_for_review'
);

CREATE INDEX idx_chat_work_assignments_teammate
ON chat_work_assignments(teammate_id, state, updated_at DESC, id);

CREATE TABLE chat_work_assignment_inputs (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    assignment_id TEXT NOT NULL REFERENCES chat_work_assignments(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    message_item_id TEXT NOT NULL UNIQUE REFERENCES chat_communication_messages(item_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 1),
    routing_kind TEXT NOT NULL CHECK (
        routing_kind IN ('trigger', 'steer', 'queued_continuation', 'follow_up')
    ),
    delivery_state TEXT NOT NULL DEFAULT 'pending' CHECK (
        delivery_state IN ('pending', 'delivered', 'failed', 'ignored')
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    delivered_at TEXT CHECK (delivered_at IS NULL OR length(delivered_at) >= 20),
    UNIQUE (assignment_id, ordinal)
) STRICT;

CREATE TABLE chat_work_semantic_updates (
    item_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_conversation_items(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    assignment_id TEXT NOT NULL REFERENCES chat_work_assignments(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    update_kind TEXT NOT NULL CHECK (
        update_kind IN ('plan', 'replan', 'question', 'approval', 'failure', 'result', 'review')
    ),
    payload_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (payload_schema_version >= 1),
    payload_data TEXT NOT NULL CHECK (json_valid(payload_data) AND json_type(payload_data) = 'object'),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20)
) STRICT;

CREATE TABLE chat_assignment_context_packages (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    assignment_id TEXT NOT NULL REFERENCES chat_work_assignments(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK (revision >= 1),
    triggering_message_item_id TEXT NOT NULL REFERENCES chat_communication_messages(item_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    serialized_text TEXT NOT NULL CHECK (length(serialized_text) <= 131072),
    serialized_bytes INTEGER NOT NULL CHECK (serialized_bytes BETWEEN 0 AND 131072),
    excluded_thread_reply_count INTEGER NOT NULL DEFAULT 0 CHECK (excluded_thread_reply_count >= 0),
    excluded_channel_message_count INTEGER NOT NULL DEFAULT 0 CHECK (excluded_channel_message_count >= 0),
    sha256 TEXT NOT NULL CHECK (length(sha256) = 64),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    UNIQUE (assignment_id, revision)
) STRICT;

CREATE TABLE chat_assignment_context_sources (
    context_package_id TEXT NOT NULL REFERENCES chat_assignment_context_packages(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    source_kind TEXT NOT NULL CHECK (
        source_kind IN ('trigger', 'thread_root', 'thread_reply', 'channel_message', 'attachment', 'resource')
    ),
    source_id TEXT NOT NULL CHECK (length(source_id) BETWEEN 1 AND 1024),
    source_revision INTEGER NOT NULL DEFAULT 1 CHECK (source_revision >= 1),
    content_sha256 TEXT NOT NULL CHECK (length(content_sha256) = 64),
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    PRIMARY KEY (context_package_id, source_kind, source_id),
    UNIQUE (context_package_id, ordinal)
) STRICT, WITHOUT ROWID;

CREATE TABLE chat_assignment_authorization_decisions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    assignment_id TEXT NOT NULL UNIQUE REFERENCES chat_work_assignments(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    context_package_id TEXT NOT NULL REFERENCES chat_assignment_context_packages(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    teammate_policy_revision_id TEXT NOT NULL REFERENCES chat_teammate_policy_revisions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    conversation_id TEXT NOT NULL REFERENCES chat_conversations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    working_folder_id TEXT NOT NULL REFERENCES project_working_folders(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    approval_policy TEXT NOT NULL CHECK (
        approval_policy IN ('ask_for_approval', 'approve_for_me', 'full_access', 'custom')
    ),
    decision_state TEXT NOT NULL CHECK (decision_state IN ('allowed', 'blocked')),
    reason TEXT NOT NULL DEFAULT '' CHECK (length(reason) <= 4000),
    policy_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (policy_schema_version >= 1),
    policy_data TEXT NOT NULL DEFAULT '{}' CHECK (
        json_valid(policy_data) AND json_type(policy_data) = 'object'
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20)
) STRICT;

CREATE TABLE chat_agent_runs (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    assignment_id TEXT NOT NULL REFERENCES chat_work_assignments(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES projects(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    working_folder_id TEXT NOT NULL,
    teammate_policy_revision_id TEXT NOT NULL REFERENCES chat_teammate_policy_revisions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    authorization_decision_id TEXT NOT NULL REFERENCES chat_assignment_authorization_decisions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
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
    UNIQUE (assignment_id, run_ordinal)
) STRICT;

CREATE INDEX idx_chat_agent_runs_provider_thread
ON chat_agent_runs(provider_thread_id, assignment_id);

CREATE TABLE chat_assignment_dispatch_jobs (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    assignment_id TEXT NOT NULL UNIQUE REFERENCES chat_work_assignments(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    state TEXT NOT NULL DEFAULT 'queued' CHECK (
        state IN ('queued', 'claimed', 'completed', 'failed', 'cancelled')
    ),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    available_at TEXT NOT NULL CHECK (length(available_at) >= 20),
    claimed_at TEXT CHECK (claimed_at IS NULL OR length(claimed_at) >= 20),
    claim_token TEXT CHECK (claim_token IS NULL OR length(claim_token) BETWEEN 1 AND 1024),
    last_error TEXT CHECK (last_error IS NULL OR length(last_error) <= 4000),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_assignment_dispatch_jobs_ready
ON chat_assignment_dispatch_jobs(state, available_at, created_at, id);

CREATE TABLE chat_project_primary_working_folders (
    project_id TEXT PRIMARY KEY NOT NULL REFERENCES projects(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    working_folder_id TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    FOREIGN KEY (working_folder_id, project_id)
        REFERENCES project_working_folders(id, project_id)
        ON UPDATE CASCADE ON DELETE RESTRICT
) STRICT;

CREATE TRIGGER chat_project_primary_active_insert
BEFORE INSERT ON chat_project_primary_working_folders
WHEN NOT EXISTS (
    SELECT 1 FROM project_working_folders folder
    WHERE folder.id = NEW.working_folder_id
      AND folder.project_id = NEW.project_id
      AND folder.archived_at IS NULL
)
BEGIN
    SELECT RAISE(ABORT, 'Primary working folder must be active');
END;

CREATE TRIGGER chat_project_primary_active_update
BEFORE UPDATE OF working_folder_id, project_id ON chat_project_primary_working_folders
WHEN NOT EXISTS (
    SELECT 1 FROM project_working_folders folder
    WHERE folder.id = NEW.working_folder_id
      AND folder.project_id = NEW.project_id
      AND folder.archived_at IS NULL
)
BEGIN
    SELECT RAISE(ABORT, 'Primary working folder must be active');
END;

CREATE TRIGGER chat_project_primary_prevent_archive
BEFORE UPDATE OF archived_at ON project_working_folders
WHEN NEW.archived_at IS NOT NULL AND EXISTS (
    SELECT 1 FROM chat_project_primary_working_folders primary_folder
    WHERE primary_folder.working_folder_id = OLD.id
)
BEGIN
    SELECT RAISE(ABORT, 'Promote another working folder before archiving the primary folder');
END;

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
        conversation_id, participant_id, membership_role, addressable,
        approval_policy, created_at, updated_at
    ) VALUES (
        (SELECT conversation_id FROM chat_channels
         WHERE project_id = NEW.project_id AND is_default = 1),
        'participant:local-owner',
        'owner',
        0,
        'ask_for_approval',
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

INSERT INTO chat_conversations (
    id, project_id, conversation_kind, last_activity_at, created_at, updated_at
)
SELECT
    'conversation:' || lower(hex(randomblob(16))),
    project.id,
    'channel',
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM projects project;

INSERT INTO chat_channels (
    id, project_id, conversation_id, name, topic, is_default, created_at, updated_at
)
SELECT
    'channel:' || lower(hex(randomblob(16))),
    conversation.project_id,
    conversation.id,
    'general',
    '',
    1,
    conversation.created_at,
    conversation.updated_at
FROM chat_conversations conversation
WHERE conversation.conversation_kind = 'channel';

INSERT INTO chat_conversation_memberships (
    conversation_id, participant_id, membership_role, addressable,
    approval_policy, created_at, updated_at
)
SELECT
    conversation.id,
    'participant:local-owner',
    'owner',
    0,
    'ask_for_approval',
    conversation.created_at,
    conversation.updated_at
FROM chat_conversations conversation
WHERE conversation.conversation_kind = 'channel';

INSERT INTO chat_project_primary_working_folders (
    project_id, working_folder_id, created_at, updated_at
)
SELECT
    folder.project_id,
    folder.id,
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM project_working_folders folder
WHERE folder.kind = 'managed';

CREATE VIRTUAL TABLE chat_communication_search_fts USING fts5(
    message_item_id UNINDEXED,
    conversation_id UNINDEXED,
    reply_thread_id UNINDEXED,
    author_display_name,
    normalized_markdown,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TRIGGER chat_communication_search_insert
AFTER UPDATE OF current_revision_id ON chat_communication_messages
WHEN NEW.current_revision_id IS NOT NULL
BEGIN
    DELETE FROM chat_communication_search_fts WHERE message_item_id = NEW.item_id;
    INSERT INTO chat_communication_search_fts (
        message_item_id, conversation_id, reply_thread_id,
        author_display_name, normalized_markdown
    )
    SELECT
        NEW.item_id,
        item.conversation_id,
        item.reply_thread_id,
        participant.display_name,
        revision.normalized_markdown
    FROM chat_conversation_items item
    JOIN chat_participants participant ON participant.id = NEW.author_participant_id
    JOIN chat_communication_message_revisions revision
      ON revision.id = NEW.current_revision_id
    WHERE item.id = NEW.item_id;
END;

CREATE TRIGGER chat_communication_search_delete
AFTER DELETE ON chat_communication_messages
BEGIN
    DELETE FROM chat_communication_search_fts WHERE message_item_id = OLD.item_id;
END;

DROP INDEX idx_chat_drafts_new_working_folder;
