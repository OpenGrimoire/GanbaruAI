CREATE TABLE notes_link_facts_state (
    key TEXT PRIMARY KEY CHECK (trim(key) <> ''),
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE notes_link_facts (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_object_type TEXT NOT NULL CHECK (
        source_object_type IN ('page', 'database_row', 'block', 'property', 'comment', 'import')
    ),
    source_object_id TEXT NOT NULL CHECK (trim(source_object_id) <> ''),
    source_page_id TEXT REFERENCES notes_pages(id) ON DELETE CASCADE,
    source_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    source_comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    source_data_source_id TEXT REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    source_property_id TEXT,
    source_property_name TEXT NOT NULL DEFAULT '',
    target_object_type TEXT NOT NULL CHECK (
        target_object_type IN (
            'page',
            'database_row',
            'block',
            'database',
            'data_source',
            'property',
            'comment',
            'file',
            'project',
            'project_task',
            'calendar_event',
            'pomodoro_run',
            'music_item',
            'external_url'
        )
    ),
    target_object_id TEXT NOT NULL CHECK (trim(target_object_id) <> ''),
    target_page_id TEXT REFERENCES notes_pages(id) ON DELETE CASCADE,
    target_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    target_comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    target_asset_id TEXT REFERENCES notes_assets(id) ON DELETE CASCADE,
    target_url TEXT,
    link_type TEXT NOT NULL CHECK (
        link_type IN (
            'child_page',
            'page_mention',
            'page_link',
            'block_link',
            'database_mention',
            'database_relation',
            'local_object_mention',
            'external_url',
            'page_icon',
            'page_cover',
            'block_file',
            'property_file',
            'comment_attachment',
            'import_source'
        )
    ),
    snippet TEXT NOT NULL DEFAULT '',
    created_time TEXT NOT NULL CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL CHECK (trim(last_edited_time) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    CHECK (
        (
            source_object_type IN ('page', 'database_row')
            AND source_page_id IS NOT NULL
            AND source_page_id = source_object_id
            AND source_block_id IS NULL
            AND source_comment_id IS NULL
        )
        OR (
            source_object_type = 'block'
            AND source_block_id IS NOT NULL
            AND source_block_id = source_object_id
            AND source_page_id IS NOT NULL
            AND source_comment_id IS NULL
        )
        OR (
            source_object_type = 'comment'
            AND source_comment_id IS NOT NULL
            AND source_comment_id = source_object_id
            AND source_page_id IS NOT NULL
        )
        OR (
            source_object_type = 'property'
            AND source_data_source_id IS NOT NULL
            AND source_property_id IS NOT NULL
            AND trim(source_property_id) <> ''
        )
        OR (
            source_object_type = 'import'
            AND source_page_id IS NULL
            AND source_block_id IS NULL
            AND source_comment_id IS NULL
        )
    ),
    CHECK (
        (
            target_object_type IN ('page', 'database_row')
            AND target_page_id IS NOT NULL
            AND target_page_id = target_object_id
        )
        OR (
            target_object_type = 'block'
            AND target_block_id IS NOT NULL
            AND target_block_id = target_object_id
        )
        OR (
            target_object_type = 'comment'
            AND target_comment_id IS NOT NULL
            AND target_comment_id = target_object_id
        )
        OR (
            target_object_type = 'file'
            AND target_asset_id IS NOT NULL
            AND target_asset_id = target_object_id
        )
        OR (
            target_object_type = 'external_url'
            AND target_url IS NOT NULL
            AND target_url = target_object_id
        )
        OR (
            target_object_type NOT IN ('page', 'database_row', 'block', 'comment', 'file', 'external_url')
        )
    )
);

CREATE INDEX idx_notes_link_facts_source
    ON notes_link_facts(source_object_type, source_object_id, link_type, id);
CREATE INDEX idx_notes_link_facts_source_page
    ON notes_link_facts(source_page_id, link_type, target_object_type, id)
    WHERE source_page_id IS NOT NULL;
CREATE INDEX idx_notes_link_facts_source_block
    ON notes_link_facts(source_block_id, link_type, target_object_type, id)
    WHERE source_block_id IS NOT NULL;
CREATE INDEX idx_notes_link_facts_source_comment
    ON notes_link_facts(source_comment_id, link_type, target_object_type, id)
    WHERE source_comment_id IS NOT NULL;
CREATE INDEX idx_notes_link_facts_source_property
    ON notes_link_facts(source_data_source_id, source_property_id, target_object_type, id)
    WHERE source_data_source_id IS NOT NULL;
CREATE INDEX idx_notes_link_facts_target
    ON notes_link_facts(target_object_type, target_object_id, link_type, id);
CREATE INDEX idx_notes_link_facts_target_page
    ON notes_link_facts(target_page_id, source_object_type, link_type, id)
    WHERE target_page_id IS NOT NULL;
CREATE INDEX idx_notes_link_facts_target_block
    ON notes_link_facts(target_block_id, source_object_type, link_type, id)
    WHERE target_block_id IS NOT NULL;
CREATE INDEX idx_notes_link_facts_target_asset
    ON notes_link_facts(target_asset_id, source_object_type, link_type, id)
    WHERE target_asset_id IS NOT NULL;
CREATE INDEX idx_notes_link_facts_target_url
    ON notes_link_facts(target_url, source_object_type, id)
    WHERE target_url IS NOT NULL;
