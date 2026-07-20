CREATE TABLE music_membership_break_items (
    membership_id TEXT PRIMARY KEY REFERENCES music_playlist_memberships(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES music_library_items(id) ON DELETE RESTRICT,
    start_ms INTEGER CHECK (start_ms IS NULL OR start_ms >= 0),
    end_ms INTEGER CHECK (end_ms IS NULL OR end_ms >= 0),
    volume REAL CHECK (volume IS NULL OR (volume >= 0 AND volume <= 1)),
    rate REAL CHECK (rate IS NULL OR (rate >= 0.25 AND rate <= 2)),
    CHECK (start_ms IS NULL OR end_ms IS NULL OR end_ms >= start_ms)
);

CREATE TABLE music_library_repair_issues (
    id TEXT PRIMARY KEY,
    issue_kind TEXT NOT NULL CHECK (
        issue_kind IN (
            'legacy-local-root-required',
            'legacy-youtube-identity-required',
            'duplicate-playlist-membership'
        )
    ),
    item_id TEXT REFERENCES music_library_items(id) ON DELETE CASCADE,
    playlist_id TEXT REFERENCES music_playlists(id) ON DELETE CASCADE,
    legacy_track_id TEXT,
    message TEXT NOT NULL CHECK (trim(message) <> ''),
    created_at INTEGER NOT NULL CHECK (created_at > 0),
    resolved_at INTEGER
);

WITH legacy_sources AS (
    SELECT
        'track:' || track.id AS source_row_key,
        track.source_kind,
        track.source_identity,
        track.title,
        track.created_at,
        track.updated_at
    FROM music_playlist_tracks AS track
    UNION ALL
    SELECT
        'break:' || break_source.track_id,
        break_source.source_kind,
        break_source.source_identity,
        break_source.title,
        track.created_at,
        track.updated_at
    FROM music_track_break_sources AS break_source
    JOIN music_playlist_tracks AS track ON track.id = break_source.track_id
),
grouped_sources AS (
    SELECT
        MIN(source_row_key) AS first_source_row_key,
        source_identity,
        CASE
            WHEN SUM(CASE WHEN source_kind = 'local-file' THEN 1 ELSE 0 END) > 0
                THEN 'local-file'
            ELSE 'youtube-video'
        END AS canonical_source_kind,
        MAX(CASE WHEN trim(title) <> '' THEN title ELSE '' END) AS title,
        MIN(created_at) AS created_at,
        MAX(updated_at) AS updated_at,
        SUM(CASE WHEN source_kind = 'youtube-playlist' THEN 1 ELSE 0 END) AS playlist_source_count
    FROM legacy_sources
    GROUP BY source_identity
)
INSERT INTO music_library_items (
    id,
    identity_key,
    source_kind,
    media_kind,
    youtube_video_id,
    original_title,
    availability,
    review_state,
    review_changed_at,
    discovered_at,
    updated_at
)
SELECT
    'legacy-item:' || first_source_row_key,
    source_identity,
    canonical_source_kind,
    'unknown',
    CASE
        WHEN canonical_source_kind = 'local-file' THEN NULL
        WHEN source_identity LIKE 'youtube:video:%' THEN substr(source_identity, length('youtube:video:') + 1)
        ELSE 'legacy:' || first_source_row_key
    END,
    title,
    CASE WHEN playlist_source_count > 0 THEN 'ambiguous' ELSE 'unknown' END,
    'reviewed',
    updated_at,
    created_at,
    updated_at
FROM grouped_sources;

INSERT INTO music_playlist_memberships (
    id,
    playlist_id,
    item_id,
    position,
    start_ms,
    end_ms,
    volume,
    rate,
    created_at,
    updated_at
)
SELECT
    'legacy-membership:' || track.id,
    track.playlist_id,
    item.id,
    track.position,
    track.start_ms,
    track.end_ms,
    track.volume,
    track.rate,
    track.created_at,
    track.updated_at
FROM music_playlist_tracks AS track
JOIN music_library_items AS item ON item.identity_key = track.source_identity
WHERE NOT EXISTS (
    SELECT 1
    FROM music_playlist_tracks AS earlier
    WHERE earlier.playlist_id = track.playlist_id
      AND earlier.source_identity = track.source_identity
      AND (
          earlier.position < track.position
          OR (earlier.position = track.position AND earlier.id < track.id)
      )
);

INSERT INTO music_membership_skip_ranges (
    id,
    membership_id,
    start_ms,
    end_ms,
    sort_order
)
SELECT
    'legacy-skip:' || skip.id,
    membership.id,
    skip.start_ms,
    skip.end_ms,
    skip.sort_order
FROM music_track_skip_ranges AS skip
JOIN music_playlist_tracks AS track ON track.id = skip.track_id
JOIN music_playlist_memberships AS membership ON membership.id = 'legacy-membership:' || track.id;

INSERT INTO music_membership_break_items (
    membership_id,
    item_id,
    start_ms,
    end_ms,
    volume,
    rate
)
SELECT
    membership.id,
    item.id,
    break_source.start_ms,
    break_source.end_ms,
    break_source.volume,
    break_source.rate
FROM music_track_break_sources AS break_source
JOIN music_playlist_tracks AS track ON track.id = break_source.track_id
JOIN music_playlist_memberships AS membership ON membership.id = 'legacy-membership:' || track.id
JOIN music_library_items AS item ON item.identity_key = break_source.source_identity;

INSERT INTO music_library_repair_issues (
    id,
    issue_kind,
    item_id,
    playlist_id,
    legacy_track_id,
    message,
    created_at
)
SELECT
    'legacy-local-root:' || track.id,
    'legacy-local-root-required',
    item.id,
    track.playlist_id,
    track.id,
    'Choose a local root to reconnect this legacy media path.',
    track.updated_at
FROM music_playlist_tracks AS track
JOIN music_library_items AS item ON item.identity_key = track.source_identity
WHERE track.source_kind = 'local-file';

INSERT INTO music_library_repair_issues (
    id,
    issue_kind,
    item_id,
    playlist_id,
    legacy_track_id,
    message,
    created_at
)
SELECT
    'legacy-youtube-identity:' || track.id,
    'legacy-youtube-identity-required',
    item.id,
    track.playlist_id,
    track.id,
    'Confirm the video identity retained from this legacy YouTube playlist entry.',
    track.updated_at
FROM music_playlist_tracks AS track
JOIN music_library_items AS item ON item.identity_key = track.source_identity
WHERE track.source_kind = 'youtube-playlist';

INSERT INTO music_library_repair_issues (
    id,
    issue_kind,
    item_id,
    playlist_id,
    legacy_track_id,
    message,
    created_at
)
SELECT
    'legacy-duplicate-membership:' || track.id,
    'duplicate-playlist-membership',
    item.id,
    track.playlist_id,
    track.id,
    'This legacy playlist contained the same canonical item more than once. Its original row remains available for repair.',
    track.updated_at
FROM music_playlist_tracks AS track
JOIN music_library_items AS item ON item.identity_key = track.source_identity
WHERE EXISTS (
    SELECT 1
    FROM music_playlist_tracks AS earlier
    WHERE earlier.playlist_id = track.playlist_id
      AND earlier.source_identity = track.source_identity
      AND (
          earlier.position < track.position
          OR (earlier.position = track.position AND earlier.id < track.id)
      )
);

CREATE INDEX idx_music_membership_break_items_item
    ON music_membership_break_items(item_id, membership_id);
CREATE INDEX idx_music_library_repair_issues_open
    ON music_library_repair_issues(resolved_at, issue_kind, id);
CREATE INDEX idx_music_library_repair_issues_item
    ON music_library_repair_issues(item_id, resolved_at, id);
