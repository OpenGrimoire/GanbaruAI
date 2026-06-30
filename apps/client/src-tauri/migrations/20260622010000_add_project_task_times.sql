ALTER TABLE project_tasks
ADD COLUMN start_time TEXT CHECK (
    start_time IS NULL OR (
        start_date IS NOT NULL
        AND length(start_time) = 5
        AND start_time GLOB '[0-9][0-9]:[0-9][0-9]'
        AND substr(start_time, 3, 1) = ':'
        AND substr(start_time, 1, 2) BETWEEN '00' AND '23'
        AND substr(start_time, 4, 2) BETWEEN '00' AND '59'
    )
);

ALTER TABLE project_tasks
ADD COLUMN due_time TEXT CHECK (
    due_time IS NULL OR (
        due_date IS NOT NULL
        AND length(due_time) = 5
        AND due_time GLOB '[0-9][0-9]:[0-9][0-9]'
        AND substr(due_time, 3, 1) = ':'
        AND substr(due_time, 1, 2) BETWEEN '00' AND '23'
        AND substr(due_time, 4, 2) BETWEEN '00' AND '59'
    )
);
