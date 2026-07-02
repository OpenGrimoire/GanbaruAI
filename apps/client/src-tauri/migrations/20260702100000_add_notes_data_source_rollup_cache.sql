CREATE TABLE IF NOT EXISTS notes_data_source_rollup_cache (
    source_page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    source_data_source_id TEXT NOT NULL REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    source_property_id TEXT NOT NULL,
    source_property_name TEXT NOT NULL,
    relation_property_id TEXT NOT NULL,
    rollup_property_id TEXT NOT NULL,
    target_data_source_id TEXT NOT NULL REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    value TEXT NOT NULL,
    computed_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (source_page_id, source_property_id)
);

CREATE INDEX IF NOT EXISTS idx_notes_rollup_cache_source_data_source
    ON notes_data_source_rollup_cache(source_data_source_id);

CREATE INDEX IF NOT EXISTS idx_notes_rollup_cache_target_data_source
    ON notes_data_source_rollup_cache(target_data_source_id);
