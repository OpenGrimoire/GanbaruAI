CREATE TABLE chat_scheduled_messages (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    client_command_id TEXT NOT NULL UNIQUE CHECK (length(client_command_id) BETWEEN 1 AND 1024),
    channel_id TEXT NOT NULL REFERENCES chat_channels(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    reply_thread_id TEXT REFERENCES chat_reply_threads(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    request_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (request_schema_version = 1),
    request_data TEXT NOT NULL CHECK (
        json_valid(request_data) AND json_type(request_data) = 'object'
    ),
    state TEXT NOT NULL DEFAULT 'scheduled' CHECK (
        state IN ('scheduled', 'dispatching', 'failed')
    ),
    scheduled_for TEXT NOT NULL CHECK (length(scheduled_for) >= 20),
    available_at TEXT NOT NULL CHECK (length(available_at) >= 20),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    claimed_at TEXT CHECK (claimed_at IS NULL OR length(claimed_at) >= 20),
    claim_token TEXT CHECK (claim_token IS NULL OR length(claim_token) BETWEEN 1 AND 1024),
    last_error TEXT CHECK (last_error IS NULL OR length(last_error) <= 4000),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_scheduled_messages_due
ON chat_scheduled_messages(state, available_at, scheduled_for, id);

CREATE INDEX idx_chat_scheduled_messages_destination
ON chat_scheduled_messages(channel_id, reply_thread_id, scheduled_for, id);

CREATE TRIGGER chat_scheduled_messages_reply_thread_insert
BEFORE INSERT ON chat_scheduled_messages
WHEN NEW.reply_thread_id IS NOT NULL AND NOT EXISTS (
    SELECT 1
    FROM chat_reply_threads thread
    JOIN chat_channels channel ON channel.conversation_id = thread.conversation_id
    WHERE thread.id = NEW.reply_thread_id
      AND channel.id = NEW.channel_id
)
BEGIN
    SELECT RAISE(ABORT, 'Scheduled reply belongs to another channel');
END;

CREATE TABLE chat_scheduled_message_attachment_references (
    scheduled_message_id TEXT NOT NULL REFERENCES chat_scheduled_messages(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    attachment_id TEXT NOT NULL REFERENCES chat_attachments(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    PRIMARY KEY (scheduled_message_id, attachment_id),
    UNIQUE (scheduled_message_id, ordinal)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_chat_scheduled_message_attachments_attachment
ON chat_scheduled_message_attachment_references(attachment_id, scheduled_message_id);

CREATE TABLE chat_scheduled_message_resource_references (
    scheduled_message_id TEXT NOT NULL REFERENCES chat_scheduled_messages(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    working_folder_id TEXT NOT NULL REFERENCES project_working_folders(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    PRIMARY KEY (scheduled_message_id, ordinal)
) STRICT, WITHOUT ROWID;
