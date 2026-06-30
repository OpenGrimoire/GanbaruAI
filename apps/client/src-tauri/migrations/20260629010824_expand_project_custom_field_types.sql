-- no-transaction
PRAGMA foreign_keys = OFF;

CREATE TABLE project_custom_fields_next (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    field_type TEXT NOT NULL CHECK (
        field_type IN (
            'text',
            'number',
            'select',
            'multi_select',
            'status',
            'date',
            'person',
            'files',
            'checkbox',
            'url',
            'phone',
            'email'
        )
    ),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

INSERT INTO project_custom_fields_next (
    id,
    project_id,
    name,
    field_type,
    sort_order,
    created_at,
    updated_at
)
SELECT
    id,
    project_id,
    name,
    field_type,
    sort_order,
    created_at,
    updated_at
FROM project_custom_fields;

DROP TABLE project_custom_fields;
ALTER TABLE project_custom_fields_next RENAME TO project_custom_fields;

CREATE UNIQUE INDEX idx_project_custom_fields_project_name ON project_custom_fields(project_id, lower(name));
CREATE INDEX idx_project_custom_fields_project_sort ON project_custom_fields(project_id, sort_order, name);

PRAGMA foreign_keys = ON;
