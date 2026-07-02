CREATE TABLE notes_assets (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    asset_path TEXT NOT NULL UNIQUE CHECK (
        trim(asset_path) <> ''
        AND instr(asset_path, '..') = 0
        AND instr(asset_path, '\') = 0
        AND (
            (
                asset_path GLOB 'notes/page-icons/*'
                AND instr(substr(asset_path, length('notes/page-icons/') + 1), '/') = 0
                AND (
                    lower(asset_path) GLOB '*.png'
                    OR lower(asset_path) GLOB '*.jpg'
                    OR lower(asset_path) GLOB '*.jpeg'
                    OR lower(asset_path) GLOB '*.webp'
                )
            )
            OR (
                asset_path GLOB 'notes/page-covers/*'
                AND instr(substr(asset_path, length('notes/page-covers/') + 1), '/') = 0
                AND (
                    lower(asset_path) GLOB '*.png'
                    OR lower(asset_path) GLOB '*.jpg'
                    OR lower(asset_path) GLOB '*.jpeg'
                    OR lower(asset_path) GLOB '*.webp'
                )
            )
            OR (
                asset_path GLOB 'notes/files/*'
                AND instr(substr(asset_path, length('notes/files/') + 1), '/') = 0
            )
        )
    ),
    kind TEXT NOT NULL CHECK (kind IN ('image', 'video', 'audio', 'pdf', 'file')),
    source_type TEXT NOT NULL CHECK (
        source_type IN ('local_upload', 'generated', 'imported', 'external_reference')
    ),
    original_name TEXT,
    content_type TEXT NOT NULL CHECK (
        trim(content_type) <> ''
        AND instr(content_type, '/') > 1
        AND instr(content_type, ' ') = 0
    ),
    byte_size INTEGER NOT NULL CHECK (byte_size > 0),
    sha256 TEXT NOT NULL CHECK (
        length(sha256) = 64
        AND sha256 = lower(sha256)
        AND sha256 NOT GLOB '*[^0-9a-f]*'
    ),
    storage_state TEXT NOT NULL DEFAULT 'available' CHECK (storage_state IN ('available', 'missing')),
    missing_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    CHECK (
        (
            storage_state = 'available'
            AND missing_at IS NULL
        )
        OR (
            storage_state = 'missing'
            AND missing_at IS NOT NULL
            AND trim(missing_at) <> ''
        )
    ),
    CHECK (
        (
            kind = 'image'
            AND content_type IN ('image/png', 'image/jpeg', 'image/webp')
        )
        OR (
            kind = 'video'
            AND content_type GLOB 'video/*'
        )
        OR (
            kind = 'audio'
            AND content_type GLOB 'audio/*'
        )
        OR (
            kind = 'pdf'
            AND content_type = 'application/pdf'
        )
        OR kind = 'file'
    )
);

CREATE INDEX idx_notes_assets_path ON notes_assets(asset_path);
CREATE INDEX idx_notes_assets_hash ON notes_assets(sha256, byte_size, content_type);
CREATE INDEX idx_notes_assets_state ON notes_assets(storage_state, updated_at DESC, id);

CREATE TABLE notes_asset_references (
    asset_id TEXT NOT NULL REFERENCES notes_assets(id) ON DELETE CASCADE,
    owner_type TEXT NOT NULL CHECK (
        owner_type IN ('page', 'block', 'data_source_property', 'comment', 'import')
    ),
    owner_id TEXT NOT NULL CHECK (trim(owner_id) <> ''),
    page_id TEXT REFERENCES notes_pages(id) ON DELETE CASCADE,
    block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    data_source_id TEXT REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    property_id TEXT,
    role TEXT NOT NULL CHECK (
        role IN (
            'page_icon',
            'page_cover',
            'block_file',
            'property_file',
            'comment_attachment',
            'import_source'
        )
    ),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    PRIMARY KEY (asset_id, owner_type, owner_id, role),
    CHECK (
        (
            owner_type = 'page'
            AND role IN ('page_icon', 'page_cover')
            AND page_id = owner_id
            AND block_id IS NULL
            AND data_source_id IS NULL
            AND comment_id IS NULL
            AND property_id IS NULL
        )
        OR (
            owner_type = 'block'
            AND role = 'block_file'
            AND page_id IS NOT NULL
            AND block_id = owner_id
            AND data_source_id IS NULL
            AND comment_id IS NULL
            AND property_id IS NULL
        )
        OR (
            owner_type = 'data_source_property'
            AND role = 'property_file'
            AND page_id IS NULL
            AND block_id IS NULL
            AND data_source_id = owner_id
            AND comment_id IS NULL
            AND property_id IS NOT NULL
            AND trim(property_id) <> ''
        )
        OR (
            owner_type = 'comment'
            AND role = 'comment_attachment'
            AND page_id IS NOT NULL
            AND block_id IS NULL
            AND data_source_id IS NULL
            AND comment_id = owner_id
            AND property_id IS NULL
        )
        OR (
            owner_type = 'import'
            AND role = 'import_source'
            AND page_id IS NULL
            AND block_id IS NULL
            AND data_source_id IS NULL
            AND comment_id IS NULL
            AND property_id IS NULL
        )
    )
);

CREATE INDEX idx_notes_asset_references_owner
    ON notes_asset_references(owner_type, owner_id, role, asset_id);
CREATE INDEX idx_notes_asset_references_page
    ON notes_asset_references(page_id, role, asset_id)
    WHERE page_id IS NOT NULL;
CREATE INDEX idx_notes_asset_references_block
    ON notes_asset_references(block_id, role, asset_id)
    WHERE block_id IS NOT NULL;
CREATE INDEX idx_notes_asset_references_data_source
    ON notes_asset_references(data_source_id, property_id, asset_id)
    WHERE data_source_id IS NOT NULL;
CREATE INDEX idx_notes_asset_references_comment
    ON notes_asset_references(comment_id, asset_id)
    WHERE comment_id IS NOT NULL;

INSERT INTO notes_assets (
    id,
    asset_path,
    kind,
    source_type,
    original_name,
    content_type,
    byte_size,
    sha256
)
SELECT
    asset_path,
    asset_path,
    'image',
    'local_upload',
    original_name,
    content_type,
    byte_size,
    lower(sha256)
FROM notes_page_icon_assets
WHERE content_type IN ('image/png', 'image/jpeg', 'image/webp')
    AND byte_size > 0
    AND length(sha256) = 64
    AND lower(sha256) NOT GLOB '*[^0-9a-f]*'
ON CONFLICT(asset_path) DO UPDATE SET
    original_name = COALESCE(excluded.original_name, notes_assets.original_name),
    content_type = excluded.content_type,
    byte_size = excluded.byte_size,
    sha256 = excluded.sha256,
    storage_state = 'available',
    missing_at = NULL,
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now');

INSERT INTO notes_assets (
    id,
    asset_path,
    kind,
    source_type,
    original_name,
    content_type,
    byte_size,
    sha256
)
SELECT
    asset_path,
    asset_path,
    'image',
    'local_upload',
    original_name,
    content_type,
    byte_size,
    lower(sha256)
FROM notes_page_cover_assets
WHERE content_type IN ('image/png', 'image/jpeg', 'image/webp')
    AND byte_size > 0
    AND length(sha256) = 64
    AND lower(sha256) NOT GLOB '*[^0-9a-f]*'
ON CONFLICT(asset_path) DO UPDATE SET
    original_name = COALESCE(excluded.original_name, notes_assets.original_name),
    content_type = excluded.content_type,
    byte_size = excluded.byte_size,
    sha256 = excluded.sha256,
    storage_state = 'available',
    missing_at = NULL,
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now');

INSERT OR IGNORE INTO notes_assets (
    id,
    asset_path,
    kind,
    source_type,
    original_name,
    content_type,
    byte_size,
    sha256
)
SELECT
    json_extract(icon, '$.file.ganbaru_asset_path'),
    json_extract(icon, '$.file.ganbaru_asset_path'),
    'image',
    'local_upload',
    json_extract(icon, '$.file.name'),
    json_extract(icon, '$.file.content_type'),
    json_extract(icon, '$.file.byte_size'),
    lower(json_extract(icon, '$.file.sha256'))
FROM notes_pages
WHERE json_extract(icon, '$.type') = 'file'
    AND json_extract(icon, '$.file.ganbaru_asset_path') GLOB 'notes/page-icons/*'
    AND instr(substr(json_extract(icon, '$.file.ganbaru_asset_path'), length('notes/page-icons/') + 1), '/') = 0
    AND instr(json_extract(icon, '$.file.ganbaru_asset_path'), '..') = 0
    AND instr(json_extract(icon, '$.file.ganbaru_asset_path'), '\') = 0
    AND (
        lower(json_extract(icon, '$.file.ganbaru_asset_path')) GLOB '*.png'
        OR lower(json_extract(icon, '$.file.ganbaru_asset_path')) GLOB '*.jpg'
        OR lower(json_extract(icon, '$.file.ganbaru_asset_path')) GLOB '*.jpeg'
        OR lower(json_extract(icon, '$.file.ganbaru_asset_path')) GLOB '*.webp'
    )
    AND json_extract(icon, '$.file.content_type') IN ('image/png', 'image/jpeg', 'image/webp')
    AND json_extract(icon, '$.file.byte_size') > 0
    AND length(json_extract(icon, '$.file.sha256')) = 64
    AND lower(json_extract(icon, '$.file.sha256')) NOT GLOB '*[^0-9a-f]*';

INSERT OR IGNORE INTO notes_assets (
    id,
    asset_path,
    kind,
    source_type,
    original_name,
    content_type,
    byte_size,
    sha256
)
SELECT
    json_extract(cover, '$.file.ganbaru_asset_path'),
    json_extract(cover, '$.file.ganbaru_asset_path'),
    'image',
    'local_upload',
    json_extract(cover, '$.file.name'),
    json_extract(cover, '$.file.content_type'),
    json_extract(cover, '$.file.byte_size'),
    lower(json_extract(cover, '$.file.sha256'))
FROM notes_pages
WHERE json_extract(cover, '$.type') = 'file'
    AND json_extract(cover, '$.file.ganbaru_asset_path') GLOB 'notes/page-covers/*'
    AND instr(substr(json_extract(cover, '$.file.ganbaru_asset_path'), length('notes/page-covers/') + 1), '/') = 0
    AND instr(json_extract(cover, '$.file.ganbaru_asset_path'), '..') = 0
    AND instr(json_extract(cover, '$.file.ganbaru_asset_path'), '\') = 0
    AND (
        lower(json_extract(cover, '$.file.ganbaru_asset_path')) GLOB '*.png'
        OR lower(json_extract(cover, '$.file.ganbaru_asset_path')) GLOB '*.jpg'
        OR lower(json_extract(cover, '$.file.ganbaru_asset_path')) GLOB '*.jpeg'
        OR lower(json_extract(cover, '$.file.ganbaru_asset_path')) GLOB '*.webp'
    )
    AND json_extract(cover, '$.file.content_type') IN ('image/png', 'image/jpeg', 'image/webp')
    AND json_extract(cover, '$.file.byte_size') > 0
    AND length(json_extract(cover, '$.file.sha256')) = 64
    AND lower(json_extract(cover, '$.file.sha256')) NOT GLOB '*[^0-9a-f]*';

INSERT OR IGNORE INTO notes_asset_references (
    asset_id,
    owner_type,
    owner_id,
    page_id,
    role
)
SELECT
    json_extract(icon, '$.file.ganbaru_asset_path'),
    'page',
    id,
    id,
    'page_icon'
FROM notes_pages
WHERE json_extract(icon, '$.type') = 'file'
    AND json_extract(icon, '$.file.ganbaru_asset_path') IN (
        SELECT id FROM notes_assets
    );

INSERT OR IGNORE INTO notes_asset_references (
    asset_id,
    owner_type,
    owner_id,
    page_id,
    role
)
SELECT
    json_extract(cover, '$.file.ganbaru_asset_path'),
    'page',
    id,
    id,
    'page_cover'
FROM notes_pages
WHERE json_extract(cover, '$.type') = 'file'
    AND json_extract(cover, '$.file.ganbaru_asset_path') IN (
        SELECT id FROM notes_assets
    );
