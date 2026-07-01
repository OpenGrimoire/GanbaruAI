CREATE TABLE notes_databases (
    id TEXT PRIMARY KEY REFERENCES notes_blocks(id) ON DELETE CASCADE CHECK (trim(id) <> ''),
    parent_type TEXT NOT NULL CHECK (parent_type IN ('page_id', 'block_id')),
    parent_page_id TEXT REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    title TEXT NOT NULL DEFAULT '',
    title_rich_text TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(title_rich_text)),
    description TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(description)),
    icon TEXT CHECK (icon IS NULL OR json_valid(icon)),
    cover TEXT CHECK (cover IS NULL OR json_valid(cover)),
    is_inline INTEGER NOT NULL DEFAULT 1 CHECK (is_inline IN (0, 1)),
    in_trash INTEGER NOT NULL DEFAULT 0 CHECK (in_trash IN (0, 1)),
    source_provider TEXT,
    source_object_id TEXT,
    source_workspace_id TEXT,
    source_last_edited_time TEXT,
    url TEXT,
    public_url TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''),
    CHECK (
        (
            parent_type = 'page_id'
            AND parent_page_id IS NOT NULL
            AND parent_block_id IS NULL
        )
        OR (
            parent_type = 'block_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NOT NULL
        )
    )
);

CREATE INDEX idx_notes_databases_parent ON notes_databases(parent_type, parent_page_id, parent_block_id);
CREATE INDEX idx_notes_databases_source ON notes_databases(source_provider, source_workspace_id, source_object_id);

CREATE TABLE notes_data_sources (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    database_id TEXT NOT NULL REFERENCES notes_databases(id) ON DELETE CASCADE,
    title TEXT NOT NULL DEFAULT '',
    title_rich_text TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(title_rich_text)),
    description TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(description)),
    icon TEXT CHECK (icon IS NULL OR json_valid(icon)),
    properties TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(properties)),
    in_trash INTEGER NOT NULL DEFAULT 0 CHECK (in_trash IN (0, 1)),
    source_provider TEXT,
    source_object_id TEXT,
    source_workspace_id TEXT,
    source_last_edited_time TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
);

CREATE INDEX idx_notes_data_sources_database ON notes_data_sources(database_id, in_trash, title);
CREATE INDEX idx_notes_data_sources_source ON notes_data_sources(source_provider, source_workspace_id, source_object_id);

CREATE TABLE notes_database_views (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    database_id TEXT NOT NULL REFERENCES notes_databases(id) ON DELETE CASCADE,
    data_source_id TEXT NOT NULL REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT '',
    type TEXT NOT NULL CHECK (
        type IN (
            'table',
            'board',
            'list',
            'calendar',
            'timeline',
            'gallery',
            'form',
            'chart',
            'map',
            'dashboard'
        )
    ),
    filter TEXT CHECK (filter IS NULL OR json_valid(filter)),
    sorts TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(sorts)),
    configuration TEXT CHECK (configuration IS NULL OR json_valid(configuration)),
    sort_order REAL NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    source_provider TEXT,
    source_object_id TEXT,
    source_workspace_id TEXT,
    source_last_edited_time TEXT,
    url TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
);

CREATE INDEX idx_notes_database_views_database ON notes_database_views(database_id, sort_order, id);
CREATE INDEX idx_notes_database_views_data_source ON notes_database_views(data_source_id, sort_order, id);
CREATE INDEX idx_notes_database_views_source ON notes_database_views(source_provider, source_workspace_id, source_object_id);
