ALTER TABLE quick_notes ADD COLUMN manual_order REAL NOT NULL DEFAULT 0;

UPDATE quick_notes AS note
SET manual_order = 1024.0 * (
    SELECT COUNT(*)
    FROM quick_notes AS preceding
    WHERE preceding.pinned = note.pinned
      AND (
          preceding.updated_at > note.updated_at
          OR (preceding.updated_at = note.updated_at AND preceding.id < note.id)
      )
);

DROP INDEX idx_quick_notes_tag_active;
DROP INDEX idx_quick_notes_active;

CREATE INDEX idx_quick_notes_tag_active
    ON quick_notes(tag_id, pinned DESC, manual_order ASC, id ASC)
    WHERE archived = 0 AND trashed_at IS NULL;

CREATE INDEX idx_quick_notes_active
    ON quick_notes(pinned DESC, manual_order ASC, id ASC)
    WHERE archived = 0 AND trashed_at IS NULL;
