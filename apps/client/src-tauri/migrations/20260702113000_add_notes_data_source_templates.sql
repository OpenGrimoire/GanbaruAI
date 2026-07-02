CREATE TABLE notes_data_source_templates (
    id TEXT PRIMARY KEY,
    data_source_id TEXT NOT NULL REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    source_page_id TEXT REFERENCES notes_pages(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    properties TEXT NOT NULL DEFAULT '{}',
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_notes_data_source_templates_source
    ON notes_data_source_templates(data_source_id, last_edited_time DESC, name);

CREATE UNIQUE INDEX idx_notes_data_source_templates_default
    ON notes_data_source_templates(data_source_id)
    WHERE is_default = 1;

CREATE TABLE notes_data_source_template_blocks (
    template_id TEXT NOT NULL REFERENCES notes_data_source_templates(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    parent_type TEXT NOT NULL CHECK (parent_type IN ('template', 'block_id')),
    parent_block_id TEXT,
    has_children INTEGER NOT NULL DEFAULT 0 CHECK (has_children IN (0, 1)),
    type TEXT NOT NULL,
    payload TEXT NOT NULL,
    plain_text TEXT NOT NULL DEFAULT '',
    sort_order REAL NOT NULL,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (template_id, id)
);

CREATE INDEX idx_notes_data_source_template_blocks_parent
    ON notes_data_source_template_blocks(template_id, parent_type, parent_block_id, sort_order, id);
