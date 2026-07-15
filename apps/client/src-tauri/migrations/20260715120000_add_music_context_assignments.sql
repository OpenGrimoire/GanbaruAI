CREATE TABLE music_context_assignments (
    owner_kind TEXT NOT NULL CHECK (owner_kind IN (
        'project-default',
        'event-snapshot',
        'event-override',
        'work-environment'
    )),
    owner_id TEXT NOT NULL CHECK (trim(owner_id) <> ''),
    phase TEXT NOT NULL CHECK (phase IN ('focus', 'short-break', 'long-break')),
    behavior TEXT NOT NULL CHECK (behavior IN (
        'inherit',
        'play-automatically',
        'prepare-silently',
        'pause-music',
        'keep-current-music'
    )),
    playlist_id TEXT,
    soundscape_id TEXT,
    provenance_kind TEXT NOT NULL CHECK (provenance_kind IN (
        'explicit',
        'copied-project',
        'work-environment'
    )),
    provenance_id TEXT,
    updated_at INTEGER NOT NULL CHECK (updated_at > 0),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    PRIMARY KEY (owner_kind, owner_id, phase),
    CHECK (playlist_id IS NULL OR trim(playlist_id) <> ''),
    CHECK (soundscape_id IS NULL OR trim(soundscape_id) <> ''),
    CHECK (provenance_id IS NULL OR trim(provenance_id) <> '')
);

CREATE INDEX idx_music_context_assignments_playlist
    ON music_context_assignments(playlist_id, owner_kind, owner_id);

CREATE INDEX idx_music_context_assignments_owner
    ON music_context_assignments(owner_kind, owner_id, phase);

INSERT OR IGNORE INTO music_context_assignments (
    owner_kind, owner_id, phase, behavior, playlist_id, soundscape_id,
    provenance_kind, provenance_id, updated_at, version
)
SELECT
    'project-default', id, 'focus', 'play-automatically', focus_playlist_id,
    NULL, 'explicit', NULL, updated_at, 1
FROM projects
WHERE focus_playlist_id IS NOT NULL AND trim(focus_playlist_id) <> '';

INSERT OR IGNORE INTO music_context_assignments (
    owner_kind, owner_id, phase, behavior, playlist_id, soundscape_id,
    provenance_kind, provenance_id, updated_at, version
)
SELECT
    'project-default', id, phase, 'play-automatically', break_playlist_id,
    NULL, 'explicit', NULL, updated_at, 1
FROM projects
CROSS JOIN (
    SELECT 'short-break' AS phase
    UNION ALL
    SELECT 'long-break' AS phase
)
WHERE break_playlist_id IS NOT NULL AND trim(break_playlist_id) <> '';

INSERT OR IGNORE INTO music_context_assignments (
    owner_kind, owner_id, phase, behavior, playlist_id, soundscape_id,
    provenance_kind, provenance_id, updated_at, version
)
SELECT
    'event-override', id, 'focus', 'play-automatically', playlist_id,
    NULL, 'explicit', NULL, updated_at, 1
FROM calendar_events
WHERE playlist_id IS NOT NULL AND trim(playlist_id) <> '';
