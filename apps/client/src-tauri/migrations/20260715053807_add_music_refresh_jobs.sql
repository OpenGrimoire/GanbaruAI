ALTER TABLE music_library_items ADD COLUMN original_track_number INTEGER
    CHECK (original_track_number IS NULL OR original_track_number > 0);
ALTER TABLE music_library_items ADD COLUMN original_artwork_identity TEXT;
ALTER TABLE music_library_items ADD COLUMN youtube_resolution_state TEXT
    CHECK (
        youtube_resolution_state IS NULL
        OR youtube_resolution_state IN (
            'resolving', 'ready', 'unavailable', 'embedding-blocked', 'timed-out'
        )
    );
ALTER TABLE music_source_collections ADD COLUMN discovery_enabled INTEGER NOT NULL DEFAULT 1
    CHECK (discovery_enabled IN (0, 1));
ALTER TABLE music_source_collections ADD COLUMN removed_at INTEGER;
ALTER TABLE music_source_collections ADD COLUMN previous_successful_refresh_at INTEGER;

CREATE TABLE music_refresh_jobs (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_collection_id TEXT NOT NULL REFERENCES music_source_collections(id) ON DELETE CASCADE,
    local_root_id TEXT REFERENCES music_local_roots(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('local-root', 'youtube-playlist')),
    state TEXT NOT NULL CHECK (
        state IN ('queued', 'running', 'completed', 'partial', 'failed', 'cancelled')
    ),
    generation INTEGER NOT NULL CHECK (generation > 0),
    discovered_count INTEGER NOT NULL DEFAULT 0 CHECK (discovered_count >= 0),
    processed_count INTEGER NOT NULL DEFAULT 0 CHECK (processed_count >= 0),
    skipped_count INTEGER NOT NULL DEFAULT 0 CHECK (skipped_count >= 0),
    issue_count INTEGER NOT NULL DEFAULT 0 CHECK (issue_count >= 0),
    truncated_count INTEGER NOT NULL DEFAULT 0 CHECK (truncated_count >= 0),
    absence_determined INTEGER NOT NULL DEFAULT 0 CHECK (absence_determined IN (0, 1)),
    status_message TEXT NOT NULL DEFAULT '',
    requested_at INTEGER NOT NULL CHECK (requested_at > 0),
    started_at INTEGER,
    finished_at INTEGER,
    updated_at INTEGER NOT NULL CHECK (updated_at > 0),
    CHECK (
        (kind = 'local-root' AND local_root_id IS NOT NULL)
        OR (kind = 'youtube-playlist' AND local_root_id IS NULL)
    )
);

CREATE TABLE music_refresh_job_entries (
    job_id TEXT NOT NULL REFERENCES music_refresh_jobs(id) ON DELETE CASCADE,
    relative_path TEXT NOT NULL,
    entry_kind TEXT NOT NULL CHECK (entry_kind IN ('directory', 'media')),
    state TEXT NOT NULL DEFAULT 'pending' CHECK (state IN ('pending', 'processed', 'skipped')),
    PRIMARY KEY (job_id, relative_path)
);

CREATE TABLE music_refresh_job_issues (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    job_id TEXT NOT NULL REFERENCES music_refresh_jobs(id) ON DELETE CASCADE,
    issue_code TEXT NOT NULL CHECK (trim(issue_code) <> ''),
    relative_path TEXT,
    item_id TEXT REFERENCES music_library_items(id) ON DELETE SET NULL,
    message TEXT NOT NULL CHECK (trim(message) <> ''),
    created_at INTEGER NOT NULL CHECK (created_at > 0)
);

CREATE INDEX idx_music_refresh_jobs_source_state
    ON music_refresh_jobs(source_collection_id, state, generation DESC);
CREATE INDEX idx_music_refresh_job_entries_pending
    ON music_refresh_job_entries(job_id, entry_kind, state, relative_path);
CREATE INDEX idx_music_refresh_job_issues_job
    ON music_refresh_job_issues(job_id, created_at, id);

CREATE TABLE music_relink_plans (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    root_id TEXT NOT NULL REFERENCES music_local_roots(id) ON DELETE CASCADE,
    state TEXT NOT NULL CHECK (state IN ('planning', 'ready', 'applied', 'cancelled')),
    exact_count INTEGER NOT NULL DEFAULT 0 CHECK (exact_count >= 0),
    likely_count INTEGER NOT NULL DEFAULT 0 CHECK (likely_count >= 0),
    ambiguous_count INTEGER NOT NULL DEFAULT 0 CHECK (ambiguous_count >= 0),
    missing_count INTEGER NOT NULL DEFAULT 0 CHECK (missing_count >= 0),
    new_count INTEGER NOT NULL DEFAULT 0 CHECK (new_count >= 0),
    created_at INTEGER NOT NULL CHECK (created_at > 0),
    updated_at INTEGER NOT NULL CHECK (updated_at > 0)
);

CREATE TABLE music_relink_plan_entries (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    plan_id TEXT NOT NULL REFERENCES music_relink_plans(id) ON DELETE CASCADE,
    match_kind TEXT NOT NULL CHECK (match_kind IN ('exact', 'likely', 'ambiguous', 'missing', 'new')),
    old_location_id TEXT,
    suggested_item_id TEXT REFERENCES music_library_items(id) ON DELETE SET NULL,
    candidate_relative_path TEXT,
    candidate_item_ids TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(candidate_item_ids)),
    file_size_bytes INTEGER CHECK (file_size_bytes IS NULL OR file_size_bytes >= 0),
    lightweight_fingerprint TEXT,
    resolved_item_id TEXT REFERENCES music_library_items(id) ON DELETE SET NULL,
    resolved_at INTEGER,
    created_at INTEGER NOT NULL CHECK (created_at > 0)
);

CREATE INDEX idx_music_relink_plan_entries_window
    ON music_relink_plan_entries(plan_id, match_kind, candidate_relative_path, id);
