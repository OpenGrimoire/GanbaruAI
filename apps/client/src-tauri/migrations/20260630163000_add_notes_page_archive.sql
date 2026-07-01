ALTER TABLE notes_pages
ADD COLUMN archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1));

CREATE INDEX idx_notes_pages_active ON notes_pages(in_trash, archived, last_edited_time DESC, title);
