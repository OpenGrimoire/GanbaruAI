ALTER TABLE music_library_items
ADD COLUMN review_deferred_until INTEGER
CHECK (review_deferred_until IS NULL OR review_deferred_until > 0);

CREATE INDEX music_library_items_review_deferred_until_idx
ON music_library_items(review_state, review_deferred_until, discovered_at, id);
