ALTER TABLE notes_pages
ADD COLUMN trashed_time TEXT CHECK (trashed_time IS NULL OR trim(trashed_time) <> '');

UPDATE notes_pages
SET trashed_time = last_edited_time
WHERE in_trash = 1
  AND trashed_time IS NULL;

CREATE INDEX idx_notes_pages_trash_retention
    ON notes_pages(in_trash, trashed_time);
