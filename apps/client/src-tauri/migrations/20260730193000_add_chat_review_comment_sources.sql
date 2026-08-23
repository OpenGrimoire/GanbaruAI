ALTER TABLE chat_review_comments ADD COLUMN source_kind TEXT NOT NULL DEFAULT 'file'
    CHECK (source_kind IN (
        'file', 'working_tree', 'checkpoint', 'commit', 'branch',
        'change_request', 'provider_turn'
    ));

ALTER TABLE chat_review_comments ADD COLUMN source_data TEXT
    CHECK (source_data IS NULL OR json_valid(source_data));

ALTER TABLE chat_review_comments ADD COLUMN review_revision TEXT
    CHECK (review_revision IS NULL OR length(review_revision) BETWEEN 16 AND 128);

ALTER TABLE chat_review_comments ADD COLUMN snapshot_id TEXT
    CHECK (snapshot_id IS NULL OR length(snapshot_id) BETWEEN 1 AND 1024);

ALTER TABLE chat_review_comments ADD COLUMN file_id TEXT
    CHECK (file_id IS NULL OR length(file_id) BETWEEN 1 AND 1024);

ALTER TABLE chat_review_comments ADD COLUMN selection_side TEXT NOT NULL DEFAULT 'file'
    CHECK (selection_side IN ('file', 'old', 'new'));

ALTER TABLE chat_review_comments ADD COLUMN previous_relative_path TEXT
    CHECK (
        previous_relative_path IS NULL
        OR (
            length(previous_relative_path) BETWEEN 1 AND 4096
            AND previous_relative_path NOT LIKE '/%'
            AND previous_relative_path NOT LIKE '../%'
            AND previous_relative_path NOT LIKE '%/../%'
            AND previous_relative_path NOT LIKE '%\%'
        )
    );

ALTER TABLE chat_review_comments ADD COLUMN applicability TEXT NOT NULL DEFAULT 'current'
    CHECK (applicability IN ('current', 'outdated', 'source_unavailable'));

ALTER TABLE chat_review_comments ADD COLUMN queued_for_send INTEGER NOT NULL DEFAULT 0
    CHECK (queued_for_send IN (0, 1));

CREATE INDEX idx_chat_review_comments_thread_queue
ON chat_review_comments(thread_id, queued_for_send, state, created_at, id);
