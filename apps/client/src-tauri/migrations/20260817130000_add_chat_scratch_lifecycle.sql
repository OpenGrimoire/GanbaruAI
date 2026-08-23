CREATE TABLE chat_scratch_promotions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    scratch_generation_id TEXT NOT NULL REFERENCES chat_scratch_generations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    source_relative_path TEXT NOT NULL CHECK (
        length(source_relative_path) BETWEEN 1 AND 4096
        AND source_relative_path NOT LIKE '/%'
        AND source_relative_path NOT LIKE '../%'
        AND source_relative_path NOT LIKE '%/../%'
        AND source_relative_path NOT LIKE '%/..'
        AND source_relative_path NOT LIKE '%\%'
    ),
    source_content_revision TEXT NOT NULL CHECK (length(source_content_revision) = 64),
    source_sha256 TEXT CHECK (source_sha256 IS NULL OR length(source_sha256) = 64),
    destination_kind TEXT NOT NULL CHECK (
        destination_kind IN ('working_folder', 'managed_attachment')
    ),
    destination_working_folder_id TEXT NOT NULL REFERENCES project_working_folders(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    destination_relative_path TEXT CHECK (
        destination_relative_path IS NULL OR (
            length(destination_relative_path) BETWEEN 1 AND 4096
            AND destination_relative_path NOT LIKE '/%'
            AND destination_relative_path NOT LIKE '../%'
            AND destination_relative_path NOT LIKE '%/../%'
            AND destination_relative_path NOT LIKE '%/..'
            AND destination_relative_path NOT LIKE '%\%'
        )
    ),
    attachment_id TEXT CHECK (
        attachment_id IS NULL OR length(attachment_id) BETWEEN 1 AND 1024
    ),
    state TEXT NOT NULL DEFAULT 'pending' CHECK (
        state IN ('pending', 'completed', 'failed')
    ),
    last_error_code TEXT CHECK (
        last_error_code IS NULL OR length(last_error_code) BETWEEN 1 AND 128
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    completed_at TEXT CHECK (completed_at IS NULL OR length(completed_at) >= 20),
    CHECK (
        (destination_kind = 'working_folder'
            AND destination_relative_path IS NOT NULL
            AND attachment_id IS NULL)
        OR (destination_kind = 'managed_attachment'
            AND destination_relative_path IS NULL
            AND attachment_id IS NOT NULL)
    )
) STRICT;

CREATE INDEX idx_chat_scratch_promotions_generation
ON chat_scratch_promotions(scratch_generation_id, created_at DESC, id);

CREATE TRIGGER chat_scratch_attachment_promotion_completed
BEFORE UPDATE OF state ON chat_scratch_promotions
WHEN NEW.state = 'completed'
 AND NEW.destination_kind = 'managed_attachment'
 AND NOT EXISTS (
    SELECT 1 FROM chat_attachments attachment WHERE attachment.id = NEW.attachment_id
 )
BEGIN
    SELECT RAISE(ABORT, 'Completed scratch attachment promotion requires its managed attachment');
END;

CREATE TABLE chat_scratch_cleanup_jobs (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    scratch_scope_id TEXT NOT NULL REFERENCES chat_scratch_scopes(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    scratch_generation_id TEXT NOT NULL UNIQUE REFERENCES chat_scratch_generations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    expected_scope_revision INTEGER NOT NULL CHECK (expected_scope_revision >= 1),
    state TEXT NOT NULL DEFAULT 'pending' CHECK (
        state IN ('pending', 'running', 'completed', 'failed', 'unavailable_on_device')
    ),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    removed_bytes INTEGER NOT NULL DEFAULT 0 CHECK (removed_bytes >= 0),
    last_error_code TEXT CHECK (
        last_error_code IS NULL OR length(last_error_code) BETWEEN 1 AND 128
    ),
    confirmed_at TEXT NOT NULL CHECK (length(confirmed_at) >= 20),
    available_at TEXT NOT NULL CHECK (length(available_at) >= 20),
    created_at TEXT NOT NULL CHECK (length(created_at) >= 20),
    updated_at TEXT NOT NULL CHECK (length(updated_at) >= 20),
    completed_at TEXT CHECK (completed_at IS NULL OR length(completed_at) >= 20)
) STRICT;

CREATE INDEX idx_chat_scratch_cleanup_jobs_ready
ON chat_scratch_cleanup_jobs(state, available_at, created_at, id);
