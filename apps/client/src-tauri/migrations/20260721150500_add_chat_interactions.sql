CREATE TABLE chat_queued_followups (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE CASCADE,
    text TEXT NOT NULL CHECK (length(text) BETWEEN 1 AND 16777216),
    provider_instance_id TEXT NOT NULL CHECK (length(provider_instance_id) BETWEEN 1 AND 1024),
    model_selection_schema_version INTEGER NOT NULL CHECK (model_selection_schema_version >= 1),
    model_selection_data TEXT NOT NULL CHECK (json_valid(model_selection_data)),
    safety_mode TEXT NOT NULL CHECK (safety_mode IN ('supervised', 'auto_accept_edits', 'full_access')),
    interaction_mode TEXT NOT NULL CHECK (interaction_mode IN ('build', 'plan')),
    attachment_ids_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (attachment_ids_schema_version >= 1),
    attachment_ids_data TEXT NOT NULL DEFAULT '[]' CHECK (
        json_valid(attachment_ids_data) AND json_type(attachment_ids_data) = 'array'
    ),
    mentions_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (mentions_schema_version >= 1),
    mentions_data TEXT NOT NULL DEFAULT '[]' CHECK (
        json_valid(mentions_data) AND json_type(mentions_data) = 'array'
    ),
    state TEXT NOT NULL CHECK (state IN ('queued', 'dispatched', 'cancelled')),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE UNIQUE INDEX idx_chat_queued_followups_active
ON chat_queued_followups(thread_id)
WHERE state = 'queued';

CREATE INDEX idx_chat_queued_followups_thread
ON chat_queued_followups(thread_id, updated_at DESC, id);

CREATE TABLE chat_queued_attachment_references (
    queued_followup_id TEXT NOT NULL REFERENCES chat_queued_followups(id) ON UPDATE CASCADE ON DELETE CASCADE,
    attachment_id TEXT NOT NULL REFERENCES chat_attachments(id) ON UPDATE CASCADE ON DELETE CASCADE,
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    PRIMARY KEY (queued_followup_id, attachment_id)
) WITHOUT ROWID, STRICT;

CREATE INDEX idx_chat_queued_attachment_references_attachment
ON chat_queued_attachment_references(attachment_id, queued_followup_id);

CREATE TABLE chat_user_input_drafts (
    request_id TEXT PRIMARY KEY NOT NULL REFERENCES chat_pending_requests(id) ON UPDATE CASCADE ON DELETE CASCADE,
    answers_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (answers_schema_version >= 1),
    answers_data TEXT NOT NULL DEFAULT '[]' CHECK (
        json_valid(answers_data) AND json_type(answers_data) = 'array'
    ),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;
