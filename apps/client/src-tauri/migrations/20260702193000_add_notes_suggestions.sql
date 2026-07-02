CREATE TABLE notes_suggestions (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    block_id TEXT NOT NULL REFERENCES notes_blocks(id) ON DELETE CASCADE,
    created_by TEXT NOT NULL REFERENCES notes_local_users(id),
    display_name TEXT NOT NULL CHECK (json_valid(display_name)),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'accepted', 'rejected')),
    range_start INTEGER NOT NULL CHECK (range_start >= 0),
    range_end INTEGER NOT NULL CHECK (range_end > range_start),
    original_text TEXT NOT NULL CHECK (trim(original_text) <> '' AND length(original_text) <= 2000),
    proposed_text TEXT NOT NULL DEFAULT '' CHECK (length(proposed_text) <= 2000),
    prefix_text TEXT NOT NULL DEFAULT '' CHECK (length(prefix_text) <= 120),
    suffix_text TEXT NOT NULL DEFAULT '' CHECK (length(suffix_text) <= 120),
    accepted_at TEXT,
    accepted_by TEXT REFERENCES notes_local_users(id),
    rejected_at TEXT,
    rejected_by TEXT REFERENCES notes_local_users(id),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''),
    CHECK (
        (
            status = 'open'
            AND accepted_at IS NULL
            AND accepted_by IS NULL
            AND rejected_at IS NULL
            AND rejected_by IS NULL
        )
        OR (
            status = 'accepted'
            AND accepted_at IS NOT NULL
            AND accepted_by IS NOT NULL
            AND rejected_at IS NULL
            AND rejected_by IS NULL
        )
        OR (
            status = 'rejected'
            AND rejected_at IS NOT NULL
            AND rejected_by IS NOT NULL
            AND accepted_at IS NULL
            AND accepted_by IS NULL
        )
    )
);

CREATE INDEX idx_notes_suggestions_page
    ON notes_suggestions(page_id, status, created_time, id);

CREATE INDEX idx_notes_suggestions_block_range
    ON notes_suggestions(block_id, range_start, range_end, status);
