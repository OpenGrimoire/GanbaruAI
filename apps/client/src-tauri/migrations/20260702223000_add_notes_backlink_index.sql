CREATE TABLE notes_backlink_index_state (
    key TEXT PRIMARY KEY CHECK (trim(key) <> ''),
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE notes_backlink_index (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    target_type TEXT NOT NULL CHECK (
        target_type IN ('page', 'database', 'local_object', 'alias', 'external_url')
    ),
    target_id TEXT NOT NULL CHECK (trim(target_id) <> ''),
    target_object_type TEXT,
    source_type TEXT NOT NULL CHECK (
        source_type IN ('block', 'comment', 'database_relation', 'alias')
    ),
    source_page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    source_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    source_comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    source_property_id TEXT,
    source_property_name TEXT NOT NULL DEFAULT '',
    reference_type TEXT NOT NULL CHECK (
        reference_type IN (
            'child_page',
            'page_mention',
            'link',
            'database_relation',
            'comment_mention',
            'comment_link',
            'database_mention',
            'local_object_mention',
            'alias'
        )
    ),
    snippet TEXT NOT NULL DEFAULT '',
    created_time TEXT NOT NULL CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL CHECK (trim(last_edited_time) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    CHECK (
        (
            target_type = 'local_object'
            AND target_object_type IS NOT NULL
            AND trim(target_object_type) <> ''
        )
        OR (
            target_type != 'local_object'
            AND target_object_type IS NULL
        )
    ),
    CHECK (
        (
            source_type = 'block'
            AND source_block_id IS NOT NULL
            AND source_comment_id IS NULL
        )
        OR (
            source_type = 'comment'
            AND source_comment_id IS NOT NULL
        )
        OR (
            source_type = 'database_relation'
            AND source_block_id IS NULL
            AND source_comment_id IS NULL
            AND source_property_id IS NOT NULL
            AND trim(source_property_id) <> ''
        )
        OR (
            source_type = 'alias'
            AND source_block_id IS NULL
            AND source_comment_id IS NULL
        )
    )
);

CREATE INDEX idx_notes_backlink_index_target
    ON notes_backlink_index(target_type, target_id, last_edited_time DESC, id);
CREATE INDEX idx_notes_backlink_index_source_page
    ON notes_backlink_index(source_page_id, source_type, last_edited_time DESC, id);
CREATE INDEX idx_notes_backlink_index_source_block
    ON notes_backlink_index(source_block_id, reference_type, id)
    WHERE source_block_id IS NOT NULL;
CREATE INDEX idx_notes_backlink_index_source_comment
    ON notes_backlink_index(source_comment_id, reference_type, id)
    WHERE source_comment_id IS NOT NULL;
