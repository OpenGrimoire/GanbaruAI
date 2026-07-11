ALTER TABLE projects
ADD COLUMN default_event_name TEXT CHECK (
    default_event_name IS NULL OR trim(default_event_name) <> ''
);
