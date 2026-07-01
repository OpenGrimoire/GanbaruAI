CREATE TABLE notes_undo_state (
    page_id TEXT PRIMARY KEY REFERENCES notes_pages(id) ON DELETE CASCADE,
    state_payload TEXT NOT NULL CHECK (length(state_payload) <= 524288),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_notes_undo_state_updated_at ON notes_undo_state(updated_at);
