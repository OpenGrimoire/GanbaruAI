CREATE TABLE notes_page_templates (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    source_page_id TEXT REFERENCES notes_pages(id) ON DELETE SET NULL,
    properties TEXT NOT NULL DEFAULT '{"title":{"id":"title","type":"title","title":[]}}' CHECK (json_valid(properties)),
    icon TEXT CHECK (icon IS NULL OR json_valid(icon)),
    cover TEXT CHECK (cover IS NULL OR json_valid(cover)),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
);

CREATE INDEX idx_notes_page_templates_recent ON notes_page_templates(last_edited_time DESC, name, id);
CREATE INDEX idx_notes_page_templates_source ON notes_page_templates(source_page_id);

CREATE TABLE notes_page_template_blocks (
    template_id TEXT NOT NULL REFERENCES notes_page_templates(id) ON DELETE CASCADE,
    id TEXT NOT NULL CHECK (trim(id) <> ''),
    parent_type TEXT NOT NULL CHECK (parent_type IN ('template', 'block_id')),
    parent_block_id TEXT,
    has_children INTEGER NOT NULL DEFAULT 0 CHECK (has_children IN (0, 1)),
    type TEXT NOT NULL CHECK (
        type IN (
            'paragraph',
            'heading_1',
            'heading_2',
            'heading_3',
            'heading_4',
            'bulleted_list_item',
            'numbered_list_item',
            'to_do',
            'toggle',
            'callout',
            'quote',
            'child_page',
            'child_database',
            'breadcrumb',
            'table_of_contents',
            'column_list',
            'column',
            'table',
            'table_row',
            'tab',
            'image',
            'video',
            'audio',
            'file',
            'pdf',
            'bookmark',
            'link_preview',
            'synced_block',
            'template',
            'button',
            'embed',
            'equation',
            'divider',
            'code',
            'unsupported'
        )
    ),
    payload TEXT NOT NULL CHECK (json_valid(payload)),
    plain_text TEXT NOT NULL DEFAULT '',
    sort_order REAL NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''),
    PRIMARY KEY (template_id, id),
    FOREIGN KEY (template_id, parent_block_id)
        REFERENCES notes_page_template_blocks(template_id, id)
        ON DELETE CASCADE,
    CHECK (
        (
            parent_type = 'template'
            AND parent_block_id IS NULL
        )
        OR (
            parent_type = 'block_id'
            AND parent_block_id IS NOT NULL
        )
    )
);

CREATE INDEX idx_notes_page_template_blocks_root
    ON notes_page_template_blocks(template_id, parent_type, sort_order, id);
CREATE INDEX idx_notes_page_template_blocks_parent
    ON notes_page_template_blocks(template_id, parent_block_id, sort_order, id);
