CREATE TABLE notes_page_icon_assets (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    asset_path TEXT NOT NULL UNIQUE CHECK (
        trim(asset_path) <> ''
        AND asset_path GLOB 'notes/page-icons/*'
        AND instr(substr(asset_path, length('notes/page-icons/') + 1), '/') = 0
        AND instr(asset_path, '..') = 0
        AND instr(asset_path, '\') = 0
        AND (
            lower(asset_path) GLOB '*.png'
            OR lower(asset_path) GLOB '*.jpg'
            OR lower(asset_path) GLOB '*.jpeg'
            OR lower(asset_path) GLOB '*.webp'
        )
    ),
    original_name TEXT,
    content_type TEXT NOT NULL CHECK (content_type IN ('image/png', 'image/jpeg', 'image/webp')),
    byte_size INTEGER NOT NULL CHECK (byte_size > 0),
    sha256 TEXT NOT NULL CHECK (length(sha256) = 64),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE INDEX idx_notes_page_icon_assets_recent ON notes_page_icon_assets(updated_at DESC, id);
