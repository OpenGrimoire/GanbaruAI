ALTER TABLE projects
ADD COLUMN default_event_name TEXT CHECK (
    default_event_name IS NULL OR trim(default_event_name) <> ''
);

UPDATE projects
SET default_event_name = name
WHERE id IN (
    'project-routine-eat',
    'project-routine-learning',
    'project-routine-reading',
    'project-routine-exercise',
    'project-routine-hygiene',
    'project-routine-social',
    'project-routine-chores',
    'project-routine-leisure',
    'project-routine-meditate',
    'project-routine-sleep'
);
