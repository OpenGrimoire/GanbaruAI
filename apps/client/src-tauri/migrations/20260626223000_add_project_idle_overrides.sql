ALTER TABLE projects
ADD COLUMN default_idle_settings_source TEXT NOT NULL DEFAULT 'global'
CHECK (default_idle_settings_source IN ('global', 'custom'));

ALTER TABLE projects
ADD COLUMN default_idle_pause_enabled INTEGER NOT NULL DEFAULT 1
CHECK (default_idle_pause_enabled IN (0, 1));

ALTER TABLE projects
ADD COLUMN default_idle_threshold_minutes INTEGER NOT NULL DEFAULT 3
CHECK (default_idle_threshold_minutes IN (1, 2, 3, 4, 5, 10, 15));
