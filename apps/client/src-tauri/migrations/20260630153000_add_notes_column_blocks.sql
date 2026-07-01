-- no-transaction
PRAGMA foreign_keys = OFF;

CREATE TABLE notes_blocks_next (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_type TEXT NOT NULL CHECK (parent_type IN ('page_id', 'block_id')),
    parent_page_id TEXT REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_block_id TEXT REFERENCES notes_blocks_next(id) ON DELETE CASCADE,
    has_children INTEGER NOT NULL DEFAULT 0 CHECK (has_children IN (0, 1)),
    in_trash INTEGER NOT NULL DEFAULT 0 CHECK (in_trash IN (0, 1)),
    type TEXT NOT NULL CHECK (
        type IN (
            'paragraph',
            'heading_1',
            'heading_2',
            'heading_3',
            'bulleted_list_item',
            'numbered_list_item',
            'to_do',
            'toggle',
            'callout',
            'quote',
            'child_page',
            'breadcrumb',
            'table_of_contents',
            'column_list',
            'column',
            'table',
            'table_row',
            'bookmark',
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
    source_provider TEXT,
    source_object_id TEXT,
    source_last_edited_time TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''),
    CHECK (
        (
            parent_type = 'page_id'
            AND parent_page_id IS NOT NULL
            AND parent_block_id IS NULL
            AND parent_page_id = page_id
        )
        OR (
            parent_type = 'block_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NOT NULL
        )
    )
);

INSERT INTO notes_blocks_next (
    id,
    page_id,
    parent_type,
    parent_page_id,
    parent_block_id,
    has_children,
    in_trash,
    type,
    payload,
    plain_text,
    sort_order,
    source_provider,
    source_object_id,
    source_last_edited_time,
    created_time,
    last_edited_time
)
SELECT
    id,
    page_id,
    parent_type,
    parent_page_id,
    parent_block_id,
    has_children,
    in_trash,
    type,
    payload,
    plain_text,
    sort_order,
    source_provider,
    source_object_id,
    source_last_edited_time,
    created_time,
    last_edited_time
FROM notes_blocks;

DROP TABLE notes_blocks;
ALTER TABLE notes_blocks_next RENAME TO notes_blocks;

CREATE INDEX idx_notes_blocks_page ON notes_blocks(page_id, in_trash, sort_order, id);
CREATE INDEX idx_notes_blocks_parent_page ON notes_blocks(parent_page_id, in_trash, sort_order, id);
CREATE INDEX idx_notes_blocks_parent_block ON notes_blocks(parent_block_id, in_trash, sort_order, id);
CREATE INDEX idx_notes_blocks_source ON notes_blocks(source_provider, source_object_id);

PRAGMA foreign_keys = ON;
