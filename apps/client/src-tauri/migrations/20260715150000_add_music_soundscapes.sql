CREATE TABLE music_soundscapes (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_kind TEXT NOT NULL CHECK (source_kind IN (
        'generated-noise',
        'local-loop',
        'bundled-loop'
    )),
    generated_kind TEXT CHECK (generated_kind IN ('white', 'pink', 'brown')),
    bundled_identity TEXT,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    availability TEXT NOT NULL CHECK (availability IN ('available', 'missing', 'unsupported')),
    created_at INTEGER NOT NULL CHECK (created_at > 0),
    updated_at INTEGER NOT NULL CHECK (updated_at > 0),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    CHECK (
        (source_kind = 'generated-noise' AND generated_kind IS NOT NULL AND bundled_identity IS NULL)
        OR (source_kind = 'local-loop' AND generated_kind IS NULL AND bundled_identity IS NULL)
        OR (source_kind = 'bundled-loop' AND generated_kind IS NULL AND trim(bundled_identity) <> '')
    )
);

CREATE UNIQUE INDEX idx_music_soundscapes_generated_kind
    ON music_soundscapes(generated_kind)
    WHERE generated_kind IS NOT NULL;

CREATE TABLE music_soundscape_locations (
    soundscape_id TEXT NOT NULL REFERENCES music_soundscapes(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL CHECK (trim(device_id) <> ''),
    absolute_path TEXT NOT NULL CHECK (trim(absolute_path) <> ''),
    availability TEXT NOT NULL CHECK (availability IN ('available', 'missing', 'unsupported')),
    file_size_bytes INTEGER CHECK (file_size_bytes IS NULL OR file_size_bytes >= 0),
    modified_at_ms INTEGER,
    updated_at INTEGER NOT NULL CHECK (updated_at > 0),
    PRIMARY KEY (soundscape_id, device_id)
);

CREATE INDEX idx_music_soundscape_locations_device
    ON music_soundscape_locations(device_id, availability, soundscape_id);

CREATE TABLE music_soundscape_state (
    singleton_id INTEGER PRIMARY KEY CHECK (singleton_id = 1),
    active_soundscape_id TEXT REFERENCES music_soundscapes(id) ON DELETE SET NULL,
    desired_playing INTEGER NOT NULL DEFAULT 0 CHECK (desired_playing IN (0, 1)),
    volume REAL NOT NULL DEFAULT 0.35 CHECK (volume >= 0.0 AND volume <= 1.0),
    updated_at INTEGER NOT NULL CHECK (updated_at > 0),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0)
);

INSERT INTO music_soundscape_state (
    singleton_id, active_soundscape_id, desired_playing, volume, updated_at, version
) VALUES (1, NULL, 0, 0.35, CAST(strftime('%s', 'now') AS INTEGER) * 1000, 1);

INSERT INTO music_soundscapes (
    id, source_kind, generated_kind, bundled_identity, name,
    availability, created_at, updated_at, version
) VALUES
    ('generated-white-noise', 'generated-noise', 'white', NULL, 'White noise', 'available', CAST(strftime('%s', 'now') AS INTEGER) * 1000, CAST(strftime('%s', 'now') AS INTEGER) * 1000, 1),
    ('generated-pink-noise', 'generated-noise', 'pink', NULL, 'Pink noise', 'available', CAST(strftime('%s', 'now') AS INTEGER) * 1000, CAST(strftime('%s', 'now') AS INTEGER) * 1000, 1),
    ('generated-brown-noise', 'generated-noise', 'brown', NULL, 'Brown noise', 'available', CAST(strftime('%s', 'now') AS INTEGER) * 1000, CAST(strftime('%s', 'now') AS INTEGER) * 1000, 1);

ALTER TABLE music_context_assignments
    ADD COLUMN soundscape_behavior TEXT NOT NULL DEFAULT 'inherit'
    CHECK (soundscape_behavior IN (
        'inherit',
        'play-selected',
        'pause-soundscape',
        'keep-current-soundscape'
    ));
