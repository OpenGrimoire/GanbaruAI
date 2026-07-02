CREATE TABLE notes_comment_thread_reads (
    thread_id TEXT NOT NULL REFERENCES notes_comment_threads(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES notes_local_users(id) ON DELETE CASCADE,
    read_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(read_at) <> ''),
    PRIMARY KEY (thread_id, user_id)
);

CREATE INDEX idx_notes_comment_thread_reads_user
    ON notes_comment_thread_reads(user_id, read_at DESC, thread_id);
