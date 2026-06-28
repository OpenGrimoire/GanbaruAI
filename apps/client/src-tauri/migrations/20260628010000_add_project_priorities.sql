CREATE TABLE IF NOT EXISTS project_priorities (
    id TEXT NOT NULL CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    color INTEGER NOT NULL CHECK (color >= 0 AND color < 32),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    PRIMARY KEY (project_id, id)
);

CREATE INDEX IF NOT EXISTS idx_project_priorities_project_sort
ON project_priorities(project_id, sort_order, name);

INSERT OR IGNORE INTO project_priorities (id, project_id, name, color, sort_order)
SELECT 'low', id, 'Low', 30, 0
FROM projects
UNION ALL
SELECT 'normal', id, 'Normal', 19, 10
FROM projects
UNION ALL
SELECT 'high', id, 'High', 7, 20
FROM projects
UNION ALL
SELECT 'urgent', id, 'Urgent', 2, 30
FROM projects;
