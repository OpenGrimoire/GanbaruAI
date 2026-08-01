CREATE TABLE chat_channels (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    project_id TEXT NOT NULL REFERENCES projects(id) ON UPDATE CASCADE ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 320),
    topic TEXT NOT NULL DEFAULT '' CHECK (length(topic) <= 1000),
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    working_folder_id TEXT,
    provider_instance_id TEXT CHECK (
        provider_instance_id IS NULL OR length(provider_instance_id) BETWEEN 1 AND 1024
    ),
    model_selection_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (
        model_selection_schema_version >= 1
    ),
    model_selection_data TEXT NOT NULL DEFAULT '{"providerManagedModel":true,"modelId":null,"modelOptions":[]}' CHECK (
        json_valid(model_selection_data) AND json_type(model_selection_data) = 'object'
    ),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    archived_at TEXT CHECK (archived_at IS NULL OR length(archived_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    FOREIGN KEY (working_folder_id, project_id)
        REFERENCES project_working_folders(id, project_id)
        ON UPDATE CASCADE ON DELETE RESTRICT
) STRICT;

CREATE UNIQUE INDEX idx_chat_channels_project_name
ON chat_channels(project_id, name);

CREATE UNIQUE INDEX idx_chat_channels_project_default
ON chat_channels(project_id)
WHERE is_default = 1;

CREATE INDEX idx_chat_channels_active_project
ON chat_channels(project_id, archived_at, updated_at DESC, id);

CREATE TABLE chat_channel_sessions (
    channel_id TEXT NOT NULL REFERENCES chat_channels(id) ON UPDATE CASCADE ON DELETE CASCADE,
    thread_id TEXT NOT NULL UNIQUE REFERENCES chat_threads(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 1),
    is_current INTEGER NOT NULL DEFAULT 1 CHECK (is_current IN (0, 1)),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    PRIMARY KEY (channel_id, thread_id),
    UNIQUE (channel_id, ordinal)
) STRICT;

CREATE UNIQUE INDEX idx_chat_channel_sessions_current
ON chat_channel_sessions(channel_id)
WHERE is_current = 1;

CREATE INDEX idx_chat_channel_sessions_thread
ON chat_channel_sessions(thread_id, channel_id);

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

CREATE TRIGGER chat_channels_create_project_general
AFTER INSERT ON project_working_folders
WHEN NEW.kind = 'managed' AND NOT EXISTS (
    SELECT 1 FROM chat_channels WHERE project_id = NEW.project_id AND is_default = 1
)
BEGIN
    INSERT INTO chat_channels (
        id, project_id, name, topic, is_default, working_folder_id,
        created_at, updated_at
    ) VALUES (
        'channel:' || lower(hex(randomblob(16))),
        NEW.project_id,
        'general',
        '',
        1,
        NEW.id,
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    );
END;

DROP INDEX idx_chat_drafts_new_working_folder;

INSERT INTO chat_channels (
    id, project_id, name, topic, is_default, working_folder_id,
    created_at, updated_at
)
SELECT
    'channel:' || lower(hex(randomblob(16))),
    p.id,
    'general',
    '',
    1,
    (
        SELECT w.id
        FROM project_working_folders w
        WHERE w.project_id = p.id AND w.kind = 'managed'
        ORDER BY w.sort_order, w.id
        LIMIT 1
    ),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM projects p;
