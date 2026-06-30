CREATE TABLE project_view_preferences_new (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    view_id TEXT NOT NULL CHECK (view_id IN ('dashboard', 'list', 'kanban', 'calendar', 'gantt')),
    preference_key TEXT NOT NULL CHECK (trim(preference_key) <> ''),
    preference_value TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    PRIMARY KEY (project_id, view_id, preference_key)
);

INSERT OR REPLACE INTO project_view_preferences_new (
    project_id,
    view_id,
    preference_key,
    preference_value,
    updated_at
)
SELECT
    project_id,
    CASE view_id
        WHEN 'board' THEN 'kanban'
        WHEN 'summary' THEN 'dashboard'
        ELSE view_id
    END,
    preference_key,
    preference_value,
    updated_at
FROM project_view_preferences;

DROP TABLE project_view_preferences;
ALTER TABLE project_view_preferences_new RENAME TO project_view_preferences;
