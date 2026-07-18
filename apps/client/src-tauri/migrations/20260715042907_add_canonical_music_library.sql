ALTER TABLE music_playlists ADD COLUMN description TEXT NOT NULL DEFAULT '';
ALTER TABLE music_playlists ADD COLUMN shuffle_enabled INTEGER NOT NULL DEFAULT 0 CHECK (shuffle_enabled IN (0, 1));
ALTER TABLE music_playlists ADD COLUMN repeat_mode TEXT NOT NULL DEFAULT 'all' CHECK (repeat_mode IN ('off', 'all', 'one'));
ALTER TABLE music_playlists ADD COLUMN version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0);

CREATE TABLE music_library_items (
    id TEXT PRIMARY KEY,
    identity_key TEXT NOT NULL UNIQUE CHECK (trim(identity_key) <> ''),
    source_kind TEXT NOT NULL CHECK (source_kind IN ('local-file', 'youtube-video')),
    media_kind TEXT NOT NULL DEFAULT 'unknown' CHECK (media_kind IN ('audio', 'video', 'unknown')),
    youtube_video_id TEXT UNIQUE,
    original_title TEXT NOT NULL DEFAULT '',
    original_artist TEXT NOT NULL DEFAULT '',
    original_album TEXT NOT NULL DEFAULT '',
    title_override TEXT,
    artist_override TEXT,
    album_override TEXT,
    artwork_override TEXT,
    duration_ms INTEGER CHECK (duration_ms IS NULL OR duration_ms >= 0),
    availability TEXT NOT NULL DEFAULT 'unknown'
        CHECK (availability IN ('available', 'missing', 'unavailable', 'ambiguous', 'unknown')),
    review_state TEXT NOT NULL DEFAULT 'unreviewed'
        CHECK (review_state IN ('unreviewed', 'reviewed', 'deferred', 'ignored')),
    review_changed_at INTEGER,
    discovered_at INTEGER NOT NULL CHECK (discovered_at > 0),
    updated_at INTEGER NOT NULL CHECK (updated_at > 0),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    CHECK (
        (source_kind = 'local-file' AND youtube_video_id IS NULL)
        OR (source_kind = 'youtube-video' AND youtube_video_id IS NOT NULL AND trim(youtube_video_id) <> '')
    )
);

CREATE TABLE music_local_roots (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    created_at INTEGER NOT NULL CHECK (created_at > 0),
    updated_at INTEGER NOT NULL CHECK (updated_at > 0),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0)
);

CREATE TABLE music_local_locations (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES music_library_items(id) ON DELETE CASCADE,
    root_id TEXT NOT NULL REFERENCES music_local_roots(id) ON DELETE CASCADE,
    relative_path TEXT NOT NULL CHECK (trim(relative_path) <> ''),
    file_size_bytes INTEGER CHECK (file_size_bytes IS NULL OR file_size_bytes >= 0),
    modified_at_ms INTEGER,
    lightweight_fingerprint TEXT,
    strong_fingerprint TEXT,
    availability TEXT NOT NULL DEFAULT 'unknown'
        CHECK (availability IN ('available', 'missing', 'ambiguous', 'unsupported', 'unknown')),
    last_seen_generation INTEGER CHECK (last_seen_generation IS NULL OR last_seen_generation >= 0),
    first_seen_at INTEGER NOT NULL CHECK (first_seen_at > 0),
    updated_at INTEGER NOT NULL CHECK (updated_at > 0),
    UNIQUE (root_id, relative_path),
    UNIQUE (root_id, item_id, relative_path)
);

CREATE TABLE music_source_collections (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('local-root', 'youtube-playlist')),
    identity_key TEXT NOT NULL UNIQUE CHECK (trim(identity_key) <> ''),
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    local_root_id TEXT REFERENCES music_local_roots(id) ON DELETE CASCADE,
    youtube_playlist_id TEXT UNIQUE,
    refresh_state TEXT NOT NULL DEFAULT 'idle'
        CHECK (refresh_state IN ('idle', 'queued', 'running', 'partial', 'failed')),
    last_successful_refresh_at INTEGER,
    last_refresh_error_code TEXT,
    snapshot_generation INTEGER NOT NULL DEFAULT 0 CHECK (snapshot_generation >= 0),
    created_at INTEGER NOT NULL CHECK (created_at > 0),
    updated_at INTEGER NOT NULL CHECK (updated_at > 0),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    UNIQUE (local_root_id),
    CHECK (
        (kind = 'local-root' AND local_root_id IS NOT NULL AND youtube_playlist_id IS NULL)
        OR (
            kind = 'youtube-playlist'
            AND local_root_id IS NULL
            AND youtube_playlist_id IS NOT NULL
            AND trim(youtube_playlist_id) <> ''
        )
    )
);

CREATE TABLE music_source_collection_items (
    collection_id TEXT NOT NULL REFERENCES music_source_collections(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES music_library_items(id) ON DELETE CASCADE,
    source_position INTEGER CHECK (source_position IS NULL OR source_position >= 0),
    first_discovered_at INTEGER NOT NULL CHECK (first_discovered_at > 0),
    last_seen_generation INTEGER NOT NULL DEFAULT 0 CHECK (last_seen_generation >= 0),
    missing_from_latest_snapshot INTEGER NOT NULL DEFAULT 0
        CHECK (missing_from_latest_snapshot IN (0, 1)),
    PRIMARY KEY (collection_id, item_id)
);

CREATE TABLE music_item_signals (
    item_id TEXT NOT NULL REFERENCES music_library_items(id) ON DELETE CASCADE,
    signal TEXT NOT NULL
        CHECK (signal IN ('lyrics', 'sudden-changes', 'high-intensity', 'calm', 'repetitive', 'energizing')),
    created_at INTEGER NOT NULL CHECK (created_at > 0),
    PRIMARY KEY (item_id, signal)
);

CREATE TABLE music_playlist_intended_uses (
    playlist_id TEXT NOT NULL REFERENCES music_playlists(id) ON DELETE CASCADE,
    intended_use TEXT NOT NULL
        CHECK (intended_use IN ('general', 'focus', 'reading', 'relaxation', 'energizing')),
    PRIMARY KEY (playlist_id, intended_use)
);

CREATE TABLE music_playlist_memberships (
    id TEXT PRIMARY KEY,
    playlist_id TEXT NOT NULL REFERENCES music_playlists(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES music_library_items(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    weight TEXT NOT NULL DEFAULT 'normal'
        CHECK (weight IN ('rarely', 'less-often', 'normal', 'more-often', 'much-more-often')),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    start_ms INTEGER CHECK (start_ms IS NULL OR start_ms >= 0),
    end_ms INTEGER CHECK (end_ms IS NULL OR end_ms >= 0),
    volume REAL CHECK (volume IS NULL OR (volume >= 0 AND volume <= 1)),
    rate REAL CHECK (rate IS NULL OR (rate >= 0.25 AND rate <= 2)),
    created_at INTEGER NOT NULL CHECK (created_at > 0),
    updated_at INTEGER NOT NULL CHECK (updated_at > 0),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    UNIQUE (playlist_id, item_id),
    CHECK (start_ms IS NULL OR end_ms IS NULL OR end_ms >= start_ms)
);

CREATE TABLE music_membership_skip_ranges (
    id TEXT PRIMARY KEY,
    membership_id TEXT NOT NULL REFERENCES music_playlist_memberships(id) ON DELETE CASCADE,
    start_ms INTEGER NOT NULL CHECK (start_ms >= 0),
    end_ms INTEGER NOT NULL CHECK (end_ms >= start_ms),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    UNIQUE (membership_id, sort_order)
);

CREATE TABLE music_snoozes (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES music_library_items(id) ON DELETE CASCADE,
    scope TEXT NOT NULL CHECK (scope IN ('playlist', 'all-playlists')),
    playlist_id TEXT REFERENCES music_playlists(id) ON DELETE CASCADE,
    starts_at INTEGER NOT NULL CHECK (starts_at > 0),
    ends_at INTEGER CHECK (ends_at IS NULL OR ends_at > starts_at),
    reason TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL CHECK (created_at > 0),
    CHECK (
        (scope = 'playlist' AND playlist_id IS NOT NULL)
        OR (scope = 'all-playlists' AND playlist_id IS NULL)
    )
);

CREATE TABLE music_listening_statistics (
    item_id TEXT PRIMARY KEY REFERENCES music_library_items(id) ON DELETE CASCADE,
    last_played_at INTEGER,
    play_count INTEGER NOT NULL DEFAULT 0 CHECK (play_count >= 0),
    completion_count INTEGER NOT NULL DEFAULT 0 CHECK (completion_count >= 0),
    skip_count INTEGER NOT NULL DEFAULT 0 CHECK (skip_count >= 0),
    updated_at INTEGER NOT NULL CHECK (updated_at > 0)
);

CREATE TABLE music_recent_selections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    playlist_id TEXT REFERENCES music_playlists(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES music_library_items(id) ON DELETE CASCADE,
    selection_kind TEXT NOT NULL CHECK (selection_kind IN ('automatic', 'manual')),
    selected_at INTEGER NOT NULL CHECK (selected_at > 0)
);

CREATE TABLE music_search_index_state (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    schema_version INTEGER NOT NULL CHECK (schema_version > 0),
    fingerprint TEXT NOT NULL,
    rebuilt_at INTEGER NOT NULL CHECK (rebuilt_at > 0)
);

CREATE VIRTUAL TABLE music_search_fts USING fts5(
    item_id UNINDEXED,
    title,
    artist,
    album,
    source_collections,
    relative_paths,
    signals,
    playlist_metadata,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE INDEX idx_music_library_items_review
    ON music_library_items(review_state, discovered_at, id);
CREATE INDEX idx_music_library_items_availability
    ON music_library_items(availability, source_kind, id);
CREATE INDEX idx_music_local_locations_root_availability
    ON music_local_locations(root_id, availability, relative_path);
CREATE INDEX idx_music_local_locations_item
    ON music_local_locations(item_id, availability);
CREATE INDEX idx_music_source_collections_kind_state
    ON music_source_collections(kind, refresh_state, updated_at);
CREATE INDEX idx_music_source_collection_items_item
    ON music_source_collection_items(item_id, collection_id);
CREATE INDEX idx_music_source_collection_items_order
    ON music_source_collection_items(collection_id, source_position, item_id);
CREATE INDEX idx_music_playlist_memberships_order
    ON music_playlist_memberships(playlist_id, position, id);
CREATE INDEX idx_music_playlist_memberships_item
    ON music_playlist_memberships(item_id, playlist_id);
CREATE INDEX idx_music_playlist_memberships_eligibility
    ON music_playlist_memberships(playlist_id, enabled, weight, position);
CREATE INDEX idx_music_snoozes_active_item
    ON music_snoozes(item_id, ends_at, starts_at);
CREATE INDEX idx_music_snoozes_playlist
    ON music_snoozes(playlist_id, item_id, ends_at);
CREATE INDEX idx_music_statistics_last_played
    ON music_listening_statistics(last_played_at DESC, item_id);
CREATE INDEX idx_music_recent_selections_playlist
    ON music_recent_selections(playlist_id, selected_at DESC, id DESC);
CREATE INDEX idx_music_recent_selections_item
    ON music_recent_selections(item_id, selected_at DESC, id DESC);
