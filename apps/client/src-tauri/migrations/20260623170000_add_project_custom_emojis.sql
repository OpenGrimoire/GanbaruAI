CREATE TABLE project_custom_emojis (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    asset_path TEXT NOT NULL CHECK (
        trim(asset_path) <> ''
        AND asset_path GLOB 'project-icons/*'
        AND instr(substr(asset_path, length('project-icons/') + 1), '/') = 0
        AND instr(asset_path, '..') = 0
        AND instr(asset_path, '\') = 0
    ),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);
CREATE INDEX idx_project_custom_emojis_sort ON project_custom_emojis(sort_order, name);
