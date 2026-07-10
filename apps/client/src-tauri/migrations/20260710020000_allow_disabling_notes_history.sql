ALTER TABLE projects
RENAME COLUMN notes_history_retention_days TO notes_history_retention_days_previous;

ALTER TABLE projects
ADD COLUMN notes_history_retention_days INTEGER CHECK (
    notes_history_retention_days IS NULL
    OR notes_history_retention_days IN (0, 7, 30, 90, 180, 365)
);

UPDATE projects
SET notes_history_retention_days = notes_history_retention_days_previous;

ALTER TABLE projects
DROP COLUMN notes_history_retention_days_previous;

CREATE TABLE notes_page_history_settings_disabled_next (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    retention_days INTEGER NOT NULL CHECK (
        retention_days IN (0, 7, 30, 90, 180, 365)
    ),
    updated_at TEXT NOT NULL DEFAULT (
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    )
);

INSERT INTO notes_page_history_settings_disabled_next (id, retention_days, updated_at)
SELECT id, retention_days, updated_at
FROM notes_page_history_settings;

DROP TABLE notes_page_history_settings;

ALTER TABLE notes_page_history_settings_disabled_next
RENAME TO notes_page_history_settings;
