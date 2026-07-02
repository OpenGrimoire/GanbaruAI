CREATE TABLE notes_data_source_relation_links (
    source_page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    source_data_source_id TEXT NOT NULL REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    source_property_id TEXT NOT NULL CHECK (trim(source_property_id) <> ''),
    source_property_name TEXT NOT NULL DEFAULT '',
    target_page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    target_data_source_id TEXT NOT NULL REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    PRIMARY KEY (source_page_id, source_property_id, target_page_id)
);

CREATE INDEX idx_notes_relation_links_source
    ON notes_data_source_relation_links(source_data_source_id, source_property_id, source_page_id);
CREATE INDEX idx_notes_relation_links_target
    ON notes_data_source_relation_links(target_page_id, source_page_id);
CREATE INDEX idx_notes_relation_links_target_data_source
    ON notes_data_source_relation_links(target_data_source_id, target_page_id);
