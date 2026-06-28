DROP INDEX IF EXISTS idx_project_labels_project_name;
DROP INDEX IF EXISTS idx_project_labels_project_sort;
DROP INDEX IF EXISTS idx_project_task_label_links_label;

ALTER TABLE project_labels RENAME TO project_tags;
ALTER TABLE project_task_label_links RENAME TO project_task_tag_links;
ALTER TABLE project_task_tag_links RENAME COLUMN label_id TO tag_id;

UPDATE project_tags
SET color = 8
WHERE color IS NULL;

CREATE UNIQUE INDEX idx_project_tags_project_name ON project_tags(project_id, lower(name));
CREATE INDEX idx_project_tags_project_sort ON project_tags(project_id, sort_order, name);
CREATE INDEX idx_project_task_tag_links_tag ON project_task_tag_links(tag_id);
