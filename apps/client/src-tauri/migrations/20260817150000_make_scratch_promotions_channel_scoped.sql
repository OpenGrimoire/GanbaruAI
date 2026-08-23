DROP TRIGGER chat_scratch_attachment_promotion_completed;

ALTER TABLE chat_scratch_promotions RENAME TO chat_scratch_promotions_legacy;

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
    request_digest TEXT CHECK (
        request_digest IS NULL OR (
            length(request_digest) = 64
            AND request_digest NOT GLOB '*[^0-9a-f]*'
        )
    ),
    destination_kind TEXT NOT NULL CHECK (
        destination_kind IN ('working_folder', 'managed_attachment')
    ),
    destination_channel_id TEXT NOT NULL REFERENCES chat_channels(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    destination_working_folder_id TEXT REFERENCES project_working_folders(id)
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
            AND destination_working_folder_id IS NOT NULL
            AND destination_relative_path IS NOT NULL
            AND attachment_id IS NULL)
        OR (destination_kind = 'managed_attachment'
            AND destination_working_folder_id IS NULL
            AND destination_relative_path IS NULL
            AND attachment_id IS NOT NULL)
    )
) STRICT;

INSERT INTO chat_scratch_promotions (
    id, scratch_generation_id, source_relative_path, source_content_revision,
    source_sha256, request_digest, destination_kind, destination_channel_id,
    destination_working_folder_id, destination_relative_path, attachment_id,
    state, last_error_code, created_at, updated_at, completed_at
)
SELECT
    promotion.id,
    promotion.scratch_generation_id,
    promotion.source_relative_path,
    promotion.source_content_revision,
    promotion.source_sha256,
    NULL,
    promotion.destination_kind,
    channel.id,
    CASE
        WHEN promotion.destination_kind = 'working_folder'
        THEN promotion.destination_working_folder_id
        ELSE NULL
    END,
    promotion.destination_relative_path,
    promotion.attachment_id,
    promotion.state,
    promotion.last_error_code,
    promotion.created_at,
    promotion.updated_at,
    promotion.completed_at
FROM chat_scratch_promotions_legacy promotion
JOIN chat_scratch_generations generation
  ON generation.id = promotion.scratch_generation_id
JOIN chat_scratch_scopes scope ON scope.id = generation.scratch_scope_id
JOIN chat_reply_threads thread ON thread.id = scope.reply_thread_id
JOIN chat_channels channel ON channel.conversation_id = thread.conversation_id;

DROP TABLE chat_scratch_promotions_legacy;

CREATE INDEX idx_chat_scratch_promotions_generation
ON chat_scratch_promotions(scratch_generation_id, created_at DESC, id);

CREATE INDEX idx_chat_scratch_promotions_destination
ON chat_scratch_promotions(destination_channel_id, created_at DESC, id);

CREATE TRIGGER chat_scratch_promotion_request_digest_required
BEFORE INSERT ON chat_scratch_promotions
WHEN NEW.request_digest IS NULL
BEGIN
    SELECT RAISE(ABORT, 'New scratch promotions require an idempotency digest');
END;

CREATE TRIGGER chat_scratch_attachment_promotion_completed
BEFORE UPDATE OF state ON chat_scratch_promotions
WHEN NEW.state = 'completed'
 AND NEW.destination_kind = 'managed_attachment'
 AND NOT EXISTS (
    SELECT 1
    FROM chat_attachments attachment
    JOIN project_working_folders storage_folder
      ON storage_folder.id = attachment.working_folder_id
     AND storage_folder.kind = 'managed'
     AND storage_folder.archived_at IS NULL
    JOIN chat_channels channel
      ON channel.id = NEW.destination_channel_id
     AND channel.project_id = storage_folder.project_id
    WHERE attachment.id = NEW.attachment_id
 )
BEGIN
    SELECT RAISE(ABORT, 'Completed scratch attachment promotion requires its channel attachment');
END;

CREATE TRIGGER chat_scratch_promotion_completion_provenance
BEFORE UPDATE OF state ON chat_scratch_promotions
WHEN NEW.state = 'completed' AND NEW.source_sha256 IS NULL
BEGIN
    SELECT RAISE(ABORT, 'Completed scratch promotion requires source provenance');
END;

CREATE TRIGGER chat_scratch_promotion_authority_immutable
BEFORE UPDATE OF scratch_generation_id, source_relative_path, source_content_revision,
    request_digest, destination_kind, destination_channel_id,
    destination_working_folder_id, destination_relative_path, attachment_id, created_at
ON chat_scratch_promotions
BEGIN
    SELECT RAISE(ABORT, 'Scratch promotion authority is immutable');
END;

CREATE TRIGGER chat_scratch_promotion_terminal_immutable
BEFORE UPDATE ON chat_scratch_promotions
WHEN OLD.state != 'pending'
BEGIN
    SELECT RAISE(ABORT, 'Terminal scratch promotion is immutable');
END;

CREATE TRIGGER chat_scratch_promotion_audit_delete_denied
BEFORE DELETE ON chat_scratch_promotions
BEGIN
    SELECT RAISE(ABORT, 'Scratch promotion audit rows cannot be deleted');
END;
