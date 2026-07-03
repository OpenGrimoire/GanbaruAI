ALTER TABLE notes_comment_threads
    ADD COLUMN source_provider TEXT;

ALTER TABLE notes_comment_threads
    ADD COLUMN source_object_id TEXT;

ALTER TABLE notes_comment_threads
    ADD COLUMN source_workspace_id TEXT;

ALTER TABLE notes_comment_threads
    ADD COLUMN source_last_edited_time TEXT;

ALTER TABLE notes_comments
    ADD COLUMN source_provider TEXT;

ALTER TABLE notes_comments
    ADD COLUMN source_object_id TEXT;

ALTER TABLE notes_comments
    ADD COLUMN source_workspace_id TEXT;

ALTER TABLE notes_comments
    ADD COLUMN source_last_edited_time TEXT;

CREATE INDEX idx_notes_comment_threads_source
    ON notes_comment_threads(source_provider, source_workspace_id, source_object_id);

CREATE INDEX idx_notes_comments_source
    ON notes_comments(source_provider, source_workspace_id, source_object_id);
