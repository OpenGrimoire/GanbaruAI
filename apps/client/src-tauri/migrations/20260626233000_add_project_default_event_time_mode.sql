ALTER TABLE projects
ADD COLUMN default_event_time_mode TEXT NOT NULL DEFAULT 'timed'
CHECK (default_event_time_mode IN ('timed', 'all_day'));
