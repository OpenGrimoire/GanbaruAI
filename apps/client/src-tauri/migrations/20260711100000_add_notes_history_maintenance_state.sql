CREATE TABLE notes_history_maintenance_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_run_at TEXT CHECK (last_run_at IS NULL OR trim(last_run_at) <> '')
);

INSERT INTO notes_history_maintenance_state (id, last_run_at)
VALUES (1, NULL);

CREATE INDEX idx_notes_project_history_dirty_deadlines
    ON notes_project_history_dirty(force_checkpoint, first_dirty_at, last_dirty_at, project_id);
