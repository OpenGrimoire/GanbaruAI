CREATE TABLE notes_comment_thread_anchors (
  thread_id TEXT PRIMARY KEY
    REFERENCES notes_comment_threads(id) ON DELETE CASCADE,
  page_id TEXT NOT NULL
    REFERENCES notes_pages(id) ON DELETE CASCADE,
  block_id TEXT NOT NULL
    REFERENCES notes_blocks(id) ON DELETE CASCADE,
  start_offset INTEGER NOT NULL CHECK (start_offset >= 0),
  end_offset INTEGER NOT NULL CHECK (end_offset > start_offset),
  anchor_text TEXT NOT NULL CHECK (trim(anchor_text) <> ''),
  prefix_text TEXT NOT NULL DEFAULT '',
  suffix_text TEXT NOT NULL DEFAULT '',
  created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  CHECK (length(anchor_text) <= 2000),
  CHECK (length(prefix_text) <= 120),
  CHECK (length(suffix_text) <= 120)
);

CREATE INDEX idx_notes_comment_thread_anchors_block
ON notes_comment_thread_anchors(block_id, start_offset, end_offset);

CREATE INDEX idx_notes_comment_thread_anchors_page
ON notes_comment_thread_anchors(page_id, block_id);
