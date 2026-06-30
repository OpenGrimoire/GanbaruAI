ALTER TABLE project_statuses
ADD COLUMN color INTEGER NOT NULL DEFAULT 30 CHECK (color >= 0 AND color < 32);

UPDATE project_statuses
SET sort_order = CASE lower(name)
        WHEN 'backlog' THEN 0
        WHEN 'to do' THEN 10
        WHEN 'in progress' THEN 20
        WHEN 'in review' THEN 30
        WHEN 'blocked' THEN 40
        WHEN 'done' THEN 50
        ELSE sort_order
    END,
    color = CASE lower(name)
        WHEN 'backlog' THEN 30
        WHEN 'to do' THEN 31
        WHEN 'in progress' THEN 19
        WHEN 'in review' THEN 23
        WHEN 'blocked' THEN 2
        WHEN 'done' THEN 13
        ELSE color
    END;

INSERT INTO project_statuses (id, project_id, name, category, color, sort_order, terminal)
SELECT 'status-' || id || '-in-review', id, 'In review', 'active', 23, 30, 0
FROM projects
WHERE NOT EXISTS (
    SELECT 1
    FROM project_statuses
    WHERE project_statuses.project_id = projects.id
        AND lower(project_statuses.name) = 'in review'
);
