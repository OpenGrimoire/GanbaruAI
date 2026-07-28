ALTER TABLE chat_drafts
ADD COLUMN rich_content_schema_version INTEGER
CHECK (rich_content_schema_version IS NULL OR rich_content_schema_version >= 1);

ALTER TABLE chat_drafts
ADD COLUMN rich_content_data TEXT
CHECK (rich_content_data IS NULL OR json_valid(rich_content_data));
