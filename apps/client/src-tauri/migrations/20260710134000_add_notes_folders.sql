CREATE TABLE notes_folders (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    parent_folder_id TEXT REFERENCES notes_folders(id) ON DELETE SET NULL,
    name TEXT NOT NULL CHECK (
        trim(name) <> ''
        AND length(name) <= 200
    ),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''),
    CHECK (parent_folder_id IS NULL OR parent_folder_id <> id)
);

CREATE INDEX idx_notes_folders_project_parent
    ON notes_folders(project_id, parent_folder_id, name COLLATE NOCASE, id);

ALTER TABLE notes_pages
ADD COLUMN folder_id TEXT REFERENCES notes_folders(id) ON DELETE SET NULL;

ALTER TABLE notes_page_history_snapshots
ADD COLUMN folder_id TEXT;

CREATE INDEX idx_notes_pages_folder
    ON notes_pages(folder_id, in_trash, archived, last_edited_time DESC, title);

CREATE TRIGGER notes_folders_validate_parent_insert
BEFORE INSERT ON notes_folders
WHEN NEW.parent_folder_id IS NOT NULL
BEGIN
    SELECT CASE
        WHEN NEW.parent_folder_id = NEW.id
        THEN RAISE(ABORT, 'notes folder cannot parent itself')
    END;
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM notes_folders AS parent
            WHERE parent.id = NEW.parent_folder_id
              AND parent.project_id = NEW.project_id
        )
        THEN RAISE(ABORT, 'notes folder parent must belong to the same project')
    END;
END;

CREATE TRIGGER notes_folders_validate_parent_update
BEFORE UPDATE OF parent_folder_id, project_id ON notes_folders
WHEN NEW.parent_folder_id IS NOT NULL
BEGIN
    SELECT CASE
        WHEN NEW.parent_folder_id = NEW.id
        THEN RAISE(ABORT, 'notes folder cannot parent itself')
    END;
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM notes_folders AS parent
            WHERE parent.id = NEW.parent_folder_id
              AND parent.project_id = NEW.project_id
        )
        THEN RAISE(ABORT, 'notes folder parent must belong to the same project')
    END;
    SELECT CASE
        WHEN EXISTS (
            WITH RECURSIVE descendants(id) AS (
                SELECT id
                FROM notes_folders
                WHERE parent_folder_id = OLD.id
                UNION
                SELECT child.id
                FROM notes_folders AS child
                JOIN descendants AS parent ON child.parent_folder_id = parent.id
            )
            SELECT 1
            FROM descendants
            WHERE id = NEW.parent_folder_id
        )
        THEN RAISE(ABORT, 'notes folder cannot be moved under its descendant')
    END;
END;

CREATE TRIGGER notes_folders_validate_project_update
BEFORE UPDATE OF project_id ON notes_folders
BEGIN
    SELECT CASE
        WHEN EXISTS (
            SELECT 1
            FROM notes_folders AS child
            WHERE child.parent_folder_id = OLD.id
              AND child.project_id <> NEW.project_id
        )
        THEN RAISE(ABORT, 'notes folder children must belong to the same project')
    END;
    SELECT CASE
        WHEN EXISTS (
            SELECT 1
            FROM notes_pages AS page
            WHERE page.folder_id = OLD.id
              AND (
                  json_type(page.properties, '$.__ganbaru_project_id') <> 'text'
                  OR trim(json_extract(page.properties, '$.__ganbaru_project_id')) <> NEW.project_id
              )
        )
        THEN RAISE(ABORT, 'notes folder pages must belong to the same project')
    END;
END;

CREATE TRIGGER notes_pages_validate_folder_insert
BEFORE INSERT ON notes_pages
WHEN NEW.folder_id IS NOT NULL
BEGIN
    SELECT CASE
        WHEN NEW.parent_type <> 'workspace'
        THEN RAISE(ABORT, 'notes folder pages must have a workspace parent')
    END;
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM notes_folders AS folder
            WHERE folder.id = NEW.folder_id
              AND json_type(NEW.properties, '$.__ganbaru_project_id') = 'text'
              AND folder.project_id = trim(json_extract(
                  NEW.properties,
                  '$.__ganbaru_project_id'
              ))
        )
        THEN RAISE(ABORT, 'notes folder page must belong to the same project')
    END;
END;

CREATE TRIGGER notes_pages_validate_folder_update
BEFORE UPDATE OF folder_id, parent_type, parent_page_id, parent_block_id, parent_data_source_id, properties
ON notes_pages
WHEN NEW.folder_id IS NOT NULL
BEGIN
    SELECT CASE
        WHEN NEW.parent_type <> 'workspace'
        THEN RAISE(ABORT, 'notes folder pages must have a workspace parent')
    END;
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM notes_folders AS folder
            WHERE folder.id = NEW.folder_id
              AND json_type(NEW.properties, '$.__ganbaru_project_id') = 'text'
              AND folder.project_id = trim(json_extract(
                  NEW.properties,
                  '$.__ganbaru_project_id'
              ))
        )
        THEN RAISE(ABORT, 'notes folder page must belong to the same project')
    END;
END;
