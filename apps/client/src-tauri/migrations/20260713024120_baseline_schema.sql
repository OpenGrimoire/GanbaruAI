-- Fresh-start schema created before Ganbaru AI had external users.
-- After the first user-capable release applies this migration, never edit it.

CREATE TABLE calendar_event_alarms (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    event_id TEXT NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    action TEXT NOT NULL DEFAULT 'display' CHECK (action IN ('display', 'audio', 'email')),
    trigger_type TEXT NOT NULL DEFAULT 'relative' CHECK (trigger_type IN ('relative', 'absolute')),
    trigger_value TEXT NOT NULL CHECK (trim(trigger_value) <> ''),
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    icalendar_component_id TEXT REFERENCES icalendar_components(id) ON DELETE SET NULL
);

CREATE TABLE calendar_event_archive_alarms (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    archive_event_id TEXT NOT NULL REFERENCES calendar_events_archive(id) ON DELETE CASCADE,
    source_alarm_id TEXT NOT NULL CHECK (trim(source_alarm_id) <> ''),
    action TEXT NOT NULL DEFAULT 'display' CHECK (action IN ('display', 'audio', 'email')),
    trigger_type TEXT NOT NULL DEFAULT 'relative' CHECK (trigger_type IN ('relative', 'absolute')),
    trigger_value TEXT NOT NULL CHECK (trim(trigger_value) <> ''),
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    icalendar_component_id TEXT
);

CREATE TABLE calendar_event_archive_attendees (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    archive_event_id TEXT NOT NULL REFERENCES calendar_events_archive(id) ON DELETE CASCADE,
    source_attendee_id TEXT NOT NULL CHECK (trim(source_attendee_id) <> ''),
    name TEXT,
    email TEXT NOT NULL CHECK (trim(email) <> ''),
    role TEXT NOT NULL DEFAULT 'req-participant' CHECK (role IN ('chair', 'req-participant', 'opt-participant', 'non-participant')),
    status TEXT NOT NULL DEFAULT 'needs-action' CHECK (status IN ('needs-action', 'accepted', 'declined', 'tentative', 'delegated')),
    rsvp INTEGER NOT NULL DEFAULT 0 CHECK (rsvp IN (0, 1)),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    icalendar_component_id TEXT,
    icalendar_property_index INTEGER CHECK (icalendar_property_index IS NULL OR icalendar_property_index >= 0)
);

CREATE TABLE calendar_event_archive_categories (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    archive_event_id TEXT NOT NULL REFERENCES calendar_events_archive(id) ON DELETE CASCADE,
    category TEXT NOT NULL CHECK (trim(category) <> ''),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_event_archive_exdates (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    archive_event_id TEXT NOT NULL REFERENCES calendar_events_archive(id) ON DELETE CASCADE,
    occurrence_date TEXT NOT NULL CHECK (trim(occurrence_date) <> ''),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_event_archive_extended_properties (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    archive_event_id TEXT NOT NULL REFERENCES calendar_events_archive(id) ON DELETE CASCADE,
    property_key TEXT NOT NULL CHECK (trim(property_key) <> ''),
    property_value TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_event_archive_notifications (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    archive_event_id TEXT NOT NULL REFERENCES calendar_events_archive(id) ON DELETE CASCADE,
    offset_minutes INTEGER NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_event_archive_organizers (
    archive_event_id TEXT PRIMARY KEY REFERENCES calendar_events_archive(id) ON DELETE CASCADE,
    name TEXT,
    email TEXT NOT NULL CHECK (trim(email) <> '')
);

CREATE TABLE calendar_event_archive_override_extended_properties (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    archive_override_id TEXT NOT NULL REFERENCES calendar_event_archive_overrides(id) ON DELETE CASCADE,
    property_key TEXT NOT NULL CHECK (trim(property_key) <> ''),
    property_value TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_event_archive_overrides (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    archive_event_id TEXT NOT NULL REFERENCES calendar_events_archive(id) ON DELETE CASCADE,
    source_override_id TEXT NOT NULL CHECK (trim(source_override_id) <> ''),
    recurrence_id TEXT NOT NULL CHECK (trim(recurrence_id) <> ''),
    title TEXT,
    start_time TEXT,
    end_time TEXT,
    description TEXT,
    location TEXT,
    url TEXT,
    color INTEGER CHECK (color IS NULL OR (color >= 0 AND color < 32)),
    status TEXT CHECK (status IS NULL OR status IN ('confirmed', 'tentative', 'cancelled')),
    transparency TEXT CHECK (transparency IS NULL OR transparency IN ('opaque', 'transparent')),
    visibility TEXT CHECK (visibility IS NULL OR visibility IN ('public', 'private')),
    created_at TEXT NOT NULL CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL CHECK (trim(updated_at) <> ''),
    icalendar_component_id TEXT,
    recurrence_range TEXT CHECK (recurrence_range IS NULL OR recurrence_range = 'this-and-future')
);

CREATE TABLE calendar_event_archive_pomodoro_config_count_rhythms (
    archive_event_id TEXT PRIMARY KEY REFERENCES calendar_event_archive_pomodoro_configs(archive_event_id) ON DELETE CASCADE,
    focus_duration_minutes INTEGER NOT NULL CHECK (focus_duration_minutes > 0),
    short_break_minutes INTEGER NOT NULL CHECK (short_break_minutes > 0),
    long_break_minutes INTEGER NOT NULL CHECK (long_break_minutes > 0),
    long_break_after_focus_count INTEGER NOT NULL CHECK (
        long_break_after_focus_count >= 1 AND long_break_after_focus_count <= 12
    )
);

CREATE TABLE calendar_event_archive_pomodoro_config_sequence_steps (
    archive_event_id TEXT NOT NULL REFERENCES calendar_event_archive_pomodoro_configs(archive_event_id) ON DELETE CASCADE,
    step_index INTEGER NOT NULL CHECK (step_index >= 0 AND step_index < 12),
    focus_duration_minutes INTEGER NOT NULL CHECK (focus_duration_minutes > 0),
    break_phase TEXT NOT NULL CHECK (break_phase IN ('short_break', 'long_break')),
    break_duration_minutes INTEGER NOT NULL CHECK (break_duration_minutes > 0),
    PRIMARY KEY (archive_event_id, step_index)
);

CREATE TABLE calendar_event_archive_pomodoro_configs (
    archive_event_id TEXT PRIMARY KEY REFERENCES calendar_events_archive(id) ON DELETE CASCADE,
    rhythm_kind TEXT NOT NULL CHECK (rhythm_kind IN ('count', 'sequence')),
    rhythm_source TEXT NOT NULL CHECK (rhythm_source IN ('preset', 'custom')),
    preset_key TEXT CHECK (
        preset_key IS NULL OR preset_key IN ('adaptive', 'creative', 'balanced', 'deep', 'extended')
    ),
    idle_timeout_minutes INTEGER CHECK (idle_timeout_minutes IS NULL OR idle_timeout_minutes > 0)
);

CREATE TABLE calendar_event_archive_rdates (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    archive_event_id TEXT NOT NULL REFERENCES calendar_events_archive(id) ON DELETE CASCADE,
    occurrence_start TEXT NOT NULL CHECK (trim(occurrence_start) <> ''),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_event_attendees (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    event_id TEXT NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    name TEXT,
    email TEXT NOT NULL CHECK (trim(email) <> ''),
    role TEXT NOT NULL DEFAULT 'req-participant' CHECK (role IN ('chair', 'req-participant', 'opt-participant', 'non-participant')),
    status TEXT NOT NULL DEFAULT 'needs-action' CHECK (status IN ('needs-action', 'accepted', 'declined', 'tentative', 'delegated')),
    rsvp INTEGER NOT NULL DEFAULT 0 CHECK (rsvp IN (0, 1)),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    icalendar_component_id TEXT REFERENCES icalendar_components(id) ON DELETE SET NULL,
    icalendar_property_index INTEGER CHECK (icalendar_property_index IS NULL OR icalendar_property_index >= 0)
);

CREATE TABLE calendar_event_categories (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    event_id TEXT NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    category TEXT NOT NULL CHECK (trim(category) <> ''),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_event_exdates (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    event_id TEXT NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    occurrence_date TEXT NOT NULL CHECK (trim(occurrence_date) <> ''),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_event_extended_properties (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    event_id TEXT NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    property_key TEXT NOT NULL CHECK (trim(property_key) <> ''),
    property_value TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_event_notifications (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    event_id TEXT NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    offset_minutes INTEGER NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_event_organizers (
    event_id TEXT PRIMARY KEY REFERENCES calendar_events(id) ON DELETE CASCADE,
    name TEXT,
    email TEXT NOT NULL CHECK (trim(email) <> '')
);

CREATE TABLE calendar_event_override_extended_properties (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    override_id TEXT NOT NULL REFERENCES calendar_event_overrides(id) ON DELETE CASCADE,
    property_key TEXT NOT NULL CHECK (trim(property_key) <> ''),
    property_value TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_event_overrides (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    parent_event_id TEXT NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    recurrence_id TEXT NOT NULL CHECK (trim(recurrence_id) <> ''),
    title TEXT,
    start_time TEXT,
    end_time TEXT,
    description TEXT,
    location TEXT,
    url TEXT,
    color INTEGER CHECK (color IS NULL OR (color >= 0 AND color < 32)),
    status TEXT CHECK (status IS NULL OR status IN ('confirmed', 'tentative', 'cancelled')),
    transparency TEXT CHECK (transparency IS NULL OR transparency IN ('opaque', 'transparent')),
    visibility TEXT CHECK (visibility IS NULL OR visibility IN ('public', 'private')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    icalendar_component_id TEXT REFERENCES icalendar_components(id) ON DELETE SET NULL,
    recurrence_range TEXT CHECK (recurrence_range IS NULL OR recurrence_range = 'this-and-future')
);

CREATE TABLE calendar_event_rdates (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    event_id TEXT NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    occurrence_start TEXT NOT NULL CHECK (trim(occurrence_start) <> ''),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0)
);

CREATE TABLE calendar_events (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    title TEXT NOT NULL DEFAULT '',
    start_time TEXT NOT NULL CHECK (trim(start_time) <> ''),
    end_time TEXT NOT NULL CHECK (trim(end_time) <> ''),
    timezone TEXT NOT NULL DEFAULT 'UTC' CHECK (trim(timezone) <> ''),
    calendar_id TEXT NOT NULL DEFAULT 'local' REFERENCES calendars(id) ON DELETE RESTRICT,
    project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
    color INTEGER CHECK (color IS NULL OR (color >= 0 AND color < 32)),
    description TEXT NOT NULL DEFAULT '',
    rrule TEXT,
    repeat_until TEXT,
    environment_id TEXT,
    playlist_id TEXT,
    all_day INTEGER NOT NULL DEFAULT 0 CHECK (all_day IN (0, 1)),
    location TEXT NOT NULL DEFAULT '',
    url TEXT NOT NULL DEFAULT '',
    transparency TEXT NOT NULL DEFAULT 'opaque' CHECK (transparency IN ('opaque', 'transparent')),
    status TEXT NOT NULL DEFAULT 'confirmed' CHECK (status IN ('confirmed', 'tentative', 'cancelled')),
    source_uid TEXT,
    visibility TEXT NOT NULL DEFAULT 'public' CHECK (visibility IN ('public', 'private')),
    priority INTEGER CHECK (priority IS NULL OR (priority >= 0 AND priority <= 9)),
    geo_lat REAL,
    geo_lng REAL,
    sequence INTEGER NOT NULL DEFAULT 0 CHECK (sequence >= 0),
    guest_can_modify INTEGER NOT NULL DEFAULT 0 CHECK (guest_can_modify IN (0, 1)),
    guest_can_invite_others INTEGER NOT NULL DEFAULT 1 CHECK (guest_can_invite_others IN (0, 1)),
    guest_can_see_other_guests INTEGER NOT NULL DEFAULT 1 CHECK (guest_can_see_other_guests IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    icalendar_component_id TEXT REFERENCES icalendar_components(id) ON DELETE SET NULL,
    local_rsvp_status TEXT CHECK (local_rsvp_status IS NULL OR local_rsvp_status IN ('needs-action', 'accepted', 'declined', 'tentative', 'delegated')),
    meeting_enabled INTEGER NOT NULL DEFAULT 0 CHECK (meeting_enabled IN (0, 1)),
    CHECK (
        (geo_lat IS NULL AND geo_lng IS NULL)
        OR (
            geo_lat IS NOT NULL
            AND geo_lng IS NOT NULL
            AND geo_lat >= -90
            AND geo_lat <= 90
            AND geo_lng >= -180
            AND geo_lng <= 180
        )
    )
);

CREATE TABLE calendar_events_archive (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_event_id TEXT NOT NULL CHECK (trim(source_event_id) <> ''),
    archived_at TEXT NOT NULL CHECK (trim(archived_at) <> ''),
    title TEXT NOT NULL DEFAULT '',
    start_time TEXT NOT NULL CHECK (trim(start_time) <> ''),
    end_time TEXT NOT NULL CHECK (trim(end_time) <> ''),
    timezone TEXT NOT NULL DEFAULT 'UTC' CHECK (trim(timezone) <> ''),
    calendar_id TEXT NOT NULL CHECK (trim(calendar_id) <> ''),
    project_id TEXT,
    color INTEGER CHECK (color IS NULL OR (color >= 0 AND color < 32)),
    description TEXT NOT NULL DEFAULT '',
    rrule TEXT,
    repeat_until TEXT,
    environment_id TEXT,
    playlist_id TEXT,
    all_day INTEGER NOT NULL DEFAULT 0 CHECK (all_day IN (0, 1)),
    location TEXT NOT NULL DEFAULT '',
    url TEXT NOT NULL DEFAULT '',
    transparency TEXT NOT NULL DEFAULT 'opaque' CHECK (transparency IN ('opaque', 'transparent')),
    status TEXT NOT NULL DEFAULT 'confirmed' CHECK (status IN ('confirmed', 'tentative', 'cancelled')),
    source_uid TEXT,
    visibility TEXT NOT NULL DEFAULT 'public' CHECK (visibility IN ('public', 'private')),
    priority INTEGER CHECK (priority IS NULL OR (priority >= 0 AND priority <= 9)),
    geo_lat REAL,
    geo_lng REAL,
    sequence INTEGER NOT NULL DEFAULT 0 CHECK (sequence >= 0),
    guest_can_modify INTEGER NOT NULL DEFAULT 0 CHECK (guest_can_modify IN (0, 1)),
    guest_can_invite_others INTEGER NOT NULL DEFAULT 1 CHECK (guest_can_invite_others IN (0, 1)),
    guest_can_see_other_guests INTEGER NOT NULL DEFAULT 1 CHECK (guest_can_see_other_guests IN (0, 1)),
    created_at TEXT NOT NULL CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL CHECK (trim(updated_at) <> ''),
    icalendar_component_id TEXT,
    local_rsvp_status TEXT CHECK (local_rsvp_status IS NULL OR local_rsvp_status IN ('needs-action', 'accepted', 'declined', 'tentative', 'delegated')),
    meeting_enabled INTEGER NOT NULL DEFAULT 0 CHECK (meeting_enabled IN (0, 1)),
    CHECK (
        (geo_lat IS NULL AND geo_lng IS NULL)
        OR (
            geo_lat IS NOT NULL
            AND geo_lng IS NOT NULL
            AND geo_lat >= -90
            AND geo_lat <= 90
            AND geo_lng >= -180
            AND geo_lng <= 180
        )
    )
);

CREATE TABLE calendars (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    color TEXT NOT NULL DEFAULT '',
    source TEXT NOT NULL DEFAULT 'local' CHECK (source IN ('local', 'ics')),
    visible INTEGER NOT NULL DEFAULT 1 CHECK (visible IN (0, 1)),
    read_only INTEGER NOT NULL DEFAULT 0 CHECK (read_only IN (0, 1)),
    source_url TEXT,
    last_synced TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE doomscrolling_block_event_rule_snapshots (
    block_event_id TEXT PRIMARY KEY REFERENCES doomscrolling_block_events(id) ON DELETE CASCADE,
    rule_id TEXT,
    rule_kind TEXT CHECK (
        rule_kind IS NULL OR
        rule_kind IN ('domain', 'url_pattern', 'category', 'custom_category', 'usage_limit', 'desktop_app')
    ),
    rule_label TEXT,
    environment_id TEXT,
    blocker_mode TEXT CHECK (
        blocker_mode IS NULL OR
        blocker_mode IN ('blacklist', 'whitelist', 'limit')
    )
);

CREATE TABLE doomscrolling_block_events (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    run_id TEXT REFERENCES pomodoro_runs(id) ON DELETE SET NULL,
    segment_id TEXT REFERENCES pomodoro_segments(id) ON DELETE SET NULL,
    occurred_at TEXT NOT NULL CHECK (trim(occurred_at) <> ''),
    source_type TEXT NOT NULL CHECK (source_type IN ('browser', 'desktop_app', 'mobile_app')),
    source_key TEXT NOT NULL CHECK (trim(source_key) <> '' AND instr(source_key, '://') = 0),
    display_name TEXT,
    phase TEXT CHECK (
        phase IS NULL OR
        phase IN ('focus', 'short_break', 'long_break', 'manual_pause', 'idle_pause', 'suspend_pause')
    ),
    decision TEXT NOT NULL CHECK (
        decision IN ('blocked', 'temporary_allowed', 'false_positive_reported', 'limit_exhausted')
    ),
    rule_id TEXT,
    category_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE doomscrolling_usage_samples (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_type TEXT NOT NULL CHECK (source_type IN ('website', 'desktop-app', 'mobile-app')),
    source_key TEXT NOT NULL CHECK (trim(source_key) <> ''),
    display_name TEXT,
    started_at INTEGER NOT NULL CHECK (started_at >= 0),
    elapsed_seconds INTEGER NOT NULL CHECK (elapsed_seconds > 0 AND elapsed_seconds <= 86400),
    local_date TEXT NOT NULL CHECK (
        length(local_date) = 10
        AND substr(local_date, 5, 1) = '-'
        AND substr(local_date, 8, 1) = '-'
    ),
    created_at INTEGER NOT NULL CHECK (created_at >= 0)
);

CREATE TABLE icalendar_component_projection_warnings (
    id TEXT PRIMARY KEY,
    component_id TEXT NOT NULL REFERENCES icalendar_components(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE icalendar_component_properties (
    id TEXT PRIMARY KEY,
    component_id TEXT NOT NULL REFERENCES icalendar_components(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    value_type TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE icalendar_components (
    id TEXT PRIMARY KEY,
    object_id TEXT NOT NULL REFERENCES icalendar_objects(id) ON DELETE CASCADE,
    parent_component_id TEXT REFERENCES icalendar_components(id) ON DELETE CASCADE,
    calendar_id TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    component_type TEXT NOT NULL,
    uid TEXT,
    recurrence_id TEXT,
    recurrence_id_value_type TEXT,
    sequence INTEGER,
    dtstart_key TEXT,
    projected_kind TEXT,
    projected_id TEXT,
    preservation_status TEXT NOT NULL CHECK (preservation_status IN ('lossless', 'partial', 'unsupported', 'needs-review', 'regenerated', 'invalid')),
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE icalendar_object_diagnostics (
    id TEXT PRIMARY KEY,
    object_id TEXT NOT NULL REFERENCES icalendar_objects(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE icalendar_objects (
    id TEXT PRIMARY KEY,
    calendar_id TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('import-file', 'import-zip-entry', 'local-export-base', 'subscription')),
    source_name TEXT NOT NULL DEFAULT '',
    source_fingerprint TEXT NOT NULL,
    prodid TEXT,
    version TEXT,
    method TEXT,
    calendar_scale TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE icalendar_property_parameters (
    id TEXT PRIMARY KEY,
    property_id TEXT NOT NULL REFERENCES icalendar_component_properties(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE icalendar_value_nodes (
    id TEXT PRIMARY KEY,
    property_id TEXT REFERENCES icalendar_component_properties(id) ON DELETE CASCADE,
    parameter_id TEXT REFERENCES icalendar_property_parameters(id) ON DELETE CASCADE,
    parent_node_id TEXT REFERENCES icalendar_value_nodes(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    value_kind TEXT NOT NULL CHECK (value_kind IN ('array', 'object', 'text', 'number', 'boolean', 'null')),
    object_key TEXT,
    text_value TEXT,
    number_value REAL,
    boolean_value INTEGER
);

CREATE TABLE music_playback_states (
    source_identity TEXT PRIMARY KEY,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('local-file', 'youtube-video', 'youtube-playlist')),
    position_ms INTEGER NOT NULL CHECK (position_ms >= 0),
    duration_ms INTEGER CHECK (duration_ms IS NULL OR duration_ms >= 0),
    status TEXT NOT NULL CHECK (status IN ('idle', 'loading', 'ready', 'playing', 'paused', 'ended', 'error')),
    updated_at INTEGER NOT NULL
);

CREATE TABLE music_playlist_tracks (
    id TEXT PRIMARY KEY,
    playlist_id TEXT NOT NULL REFERENCES music_playlists(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    source_kind TEXT NOT NULL CHECK (source_kind IN ('local-file', 'youtube-video', 'youtube-playlist')),
    source_uri TEXT NOT NULL,
    source_identity TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    start_ms INTEGER CHECK (start_ms IS NULL OR start_ms >= 0),
    end_ms INTEGER CHECK (end_ms IS NULL OR end_ms >= 0),
    volume REAL CHECK (volume IS NULL OR (volume >= 0 AND volume <= 1)),
    rate REAL CHECK (rate IS NULL OR (rate >= 0.25 AND rate <= 2)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE music_playlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE music_track_break_sources (
    track_id TEXT PRIMARY KEY REFERENCES music_playlist_tracks(id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('local-file', 'youtube-video', 'youtube-playlist')),
    source_uri TEXT NOT NULL,
    source_identity TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    start_ms INTEGER CHECK (start_ms IS NULL OR start_ms >= 0),
    end_ms INTEGER CHECK (end_ms IS NULL OR end_ms >= 0),
    volume REAL CHECK (volume IS NULL OR (volume >= 0 AND volume <= 1)),
    rate REAL CHECK (rate IS NULL OR (rate >= 0.25 AND rate <= 2))
);

CREATE TABLE music_track_skip_ranges (
    id TEXT PRIMARY KEY,
    track_id TEXT NOT NULL REFERENCES music_playlist_tracks(id) ON DELETE CASCADE,
    start_ms INTEGER NOT NULL CHECK (start_ms >= 0),
    end_ms INTEGER NOT NULL CHECK (end_ms >= 0),
    sort_order INTEGER NOT NULL DEFAULT 0,
    CHECK (end_ms >= start_ms)
);

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

CREATE TABLE notes_backlink_index (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    target_type TEXT NOT NULL CHECK (
        target_type IN ('page', 'database', 'local_object', 'alias', 'external_url')
    ),
    target_id TEXT NOT NULL CHECK (trim(target_id) <> ''),
    target_object_type TEXT,
    source_type TEXT NOT NULL CHECK (
        source_type IN ('block', 'comment', 'database_relation', 'alias')
    ),
    source_page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    source_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    source_comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    source_property_id TEXT,
    source_property_name TEXT NOT NULL DEFAULT '',
    reference_type TEXT NOT NULL CHECK (
        reference_type IN (
            'child_page',
            'page_mention',
            'link',
            'database_relation',
            'comment_mention',
            'comment_link',
            'database_mention',
            'local_object_mention',
            'alias'
        )
    ),
    snippet TEXT NOT NULL DEFAULT '',
    created_time TEXT NOT NULL CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL CHECK (trim(last_edited_time) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    CHECK (
        (
            target_type = 'local_object'
            AND target_object_type IS NOT NULL
            AND trim(target_object_type) <> ''
        )
        OR (
            target_type != 'local_object'
            AND target_object_type IS NULL
        )
    ),
    CHECK (
        (
            source_type = 'block'
            AND source_block_id IS NOT NULL
            AND source_comment_id IS NULL
        )
        OR (
            source_type = 'comment'
            AND source_comment_id IS NOT NULL
        )
        OR (
            source_type = 'database_relation'
            AND source_block_id IS NULL
            AND source_comment_id IS NULL
            AND source_property_id IS NOT NULL
            AND trim(source_property_id) <> ''
        )
        OR (
            source_type = 'alias'
            AND source_block_id IS NULL
            AND source_comment_id IS NULL
        )
    )
);

CREATE TABLE notes_backlink_index_state (
    key TEXT PRIMARY KEY CHECK (trim(key) <> ''),
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE "notes_blocks" (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_type TEXT NOT NULL CHECK (parent_type IN ('page_id', 'block_id')),
    parent_page_id TEXT REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_block_id TEXT REFERENCES "notes_blocks"(id) ON DELETE CASCADE,
    has_children INTEGER NOT NULL DEFAULT 0 CHECK (has_children IN (0, 1)),
    in_trash INTEGER NOT NULL DEFAULT 0 CHECK (in_trash IN (0, 1)),
    type TEXT NOT NULL CHECK (
        type IN (
            'paragraph',
            'heading_1',
            'heading_2',
            'heading_3',
            'heading_4',
            'bulleted_list_item',
            'numbered_list_item',
            'to_do',
            'toggle',
            'callout',
            'quote',
            'child_page',
            'child_database',
            'breadcrumb',
            'table_of_contents',
            'column_list',
            'column',
            'table',
            'table_row',
            'tab',
            'image',
            'video',
            'audio',
            'file',
            'pdf',
            'bookmark',
            'link_preview',
            'synced_block',
            'template',
            'button',
            'embed',
            'equation',
            'divider',
            'code',
            'unsupported'
        )
    ),
    payload TEXT NOT NULL CHECK (json_valid(payload)),
    plain_text TEXT NOT NULL DEFAULT '',
    sort_order REAL NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    source_provider TEXT,
    source_object_id TEXT,
    source_last_edited_time TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''),
    CHECK (
        (
            parent_type = 'page_id'
            AND parent_page_id IS NOT NULL
            AND parent_block_id IS NULL
            AND parent_page_id = page_id
        )
        OR (
            parent_type = 'block_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NOT NULL
        )
    )
);

CREATE TABLE "notes_collaboration_operations" (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE CHECK (trim(id) <> ''),
    entity_type TEXT NOT NULL CHECK (entity_type IN ('comment_thread', 'comment', 'suggestion')),
    entity_id TEXT NOT NULL CHECK (trim(entity_id) <> ''),
    operation_type TEXT NOT NULL CHECK (
        operation_type IN (
            'comment_thread_create',
            'comment_thread_resolve',
            'comment_thread_reopen',
            'comment_create',
            'comment_update',
            'comment_delete',
            'suggestion_create',
            'suggestion_accept',
            'suggestion_reject'
        )
    ),
    page_id TEXT NOT NULL CHECK (trim(page_id) <> ''),
    block_id TEXT,
    actor_id TEXT NOT NULL REFERENCES notes_local_users(id),
    actor_display_name TEXT NOT NULL CHECK (json_valid(actor_display_name)),
    base_version INTEGER NOT NULL CHECK (base_version >= 0),
    entity_version INTEGER NOT NULL CHECK (entity_version > base_version),
    conflict_policy TEXT NOT NULL CHECK (
        conflict_policy IN ('append_only', 'last_writer_wins', 'state_transition')
    ),
    payload TEXT NOT NULL CHECK (json_valid(payload)),
    sync_state TEXT NOT NULL DEFAULT 'local' CHECK (
        sync_state IN ('local', 'exported', 'acknowledged')
    ),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        CHECK (trim(created_time) <> ''),
    CHECK (
        (
            entity_type = 'comment_thread'
            AND operation_type IN (
                'comment_thread_create',
                'comment_thread_resolve',
                'comment_thread_reopen'
            )
        )
        OR (
            entity_type = 'comment'
            AND operation_type IN ('comment_create', 'comment_update', 'comment_delete')
        )
        OR (
            entity_type = 'suggestion'
            AND operation_type IN (
                'suggestion_create',
                'suggestion_accept',
                'suggestion_reject'
            )
        )
    ),
    CHECK (
        (
            conflict_policy = 'append_only'
            AND operation_type IN (
                'comment_thread_create',
                'comment_create',
                'suggestion_create'
            )
        )
        OR (
            conflict_policy = 'last_writer_wins'
            AND operation_type = 'comment_update'
        )
        OR (
            conflict_policy = 'state_transition'
            AND operation_type IN (
                'comment_thread_resolve',
                'comment_thread_reopen',
                'comment_delete',
                'suggestion_accept',
                'suggestion_reject'
            )
        )
    )
);

CREATE TABLE notes_comment_thread_anchors (
  thread_id TEXT PRIMARY KEY
    REFERENCES notes_comment_threads(id) ON DELETE CASCADE,
  page_id TEXT NOT NULL
    REFERENCES notes_pages(id) ON DELETE CASCADE,
  block_id TEXT NOT NULL
    REFERENCES notes_blocks(id) ON DELETE CASCADE,
  start_offset INTEGER NOT NULL CHECK (start_offset >= 0),
  end_offset INTEGER NOT NULL CHECK (end_offset > start_offset),
  anchor_text TEXT NOT NULL CHECK (trim(anchor_text) <> ''),
  prefix_text TEXT NOT NULL DEFAULT '',
  suffix_text TEXT NOT NULL DEFAULT '',
  created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  CHECK (length(anchor_text) <= 2000),
  CHECK (length(prefix_text) <= 120),
  CHECK (length(suffix_text) <= 120)
);

CREATE TABLE notes_comment_thread_reads (
    thread_id TEXT NOT NULL REFERENCES notes_comment_threads(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES notes_local_users(id) ON DELETE CASCADE,
    read_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(read_at) <> ''),
    PRIMARY KEY (thread_id, user_id)
);

CREATE TABLE notes_comment_threads (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_type TEXT NOT NULL CHECK (parent_type IN ('page_id', 'block_id')),
    parent_page_id TEXT REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'resolved')),
    resolved_at TEXT,
    resolved_by TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''), sync_version INTEGER NOT NULL DEFAULT 1 CHECK (sync_version >= 1), source_provider TEXT, source_object_id TEXT, source_workspace_id TEXT, source_last_edited_time TEXT,
    CHECK (
        (
            parent_type = 'page_id'
            AND parent_page_id IS NOT NULL
            AND parent_block_id IS NULL
            AND parent_page_id = page_id
        )
        OR (
            parent_type = 'block_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NOT NULL
        )
    ),
    CHECK (
        (
            status = 'open'
            AND resolved_at IS NULL
            AND resolved_by IS NULL
        )
        OR (
            status = 'resolved'
            AND resolved_at IS NOT NULL
            AND resolved_by IS NOT NULL
        )
    )
);

CREATE TABLE notes_comments (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    thread_id TEXT NOT NULL REFERENCES notes_comment_threads(id) ON DELETE CASCADE,
    rich_text TEXT NOT NULL CHECK (json_valid(rich_text)),
    plain_text TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL DEFAULT 'local-user' CHECK (trim(created_by) <> ''),
    display_name TEXT NOT NULL DEFAULT '{"type":"user","resolved_name":"You"}' CHECK (json_valid(display_name)),
    attachments TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(attachments)),
    deleted_at TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
, sync_version INTEGER NOT NULL DEFAULT 1 CHECK (sync_version >= 1), source_provider TEXT, source_object_id TEXT, source_workspace_id TEXT, source_last_edited_time TEXT);

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

CREATE TABLE notes_data_source_rollup_cache (
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

CREATE TABLE notes_data_source_template_blocks (
    template_id TEXT NOT NULL REFERENCES notes_data_source_templates(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    parent_type TEXT NOT NULL CHECK (parent_type IN ('template', 'block_id')),
    parent_block_id TEXT,
    has_children INTEGER NOT NULL DEFAULT 0 CHECK (has_children IN (0, 1)),
    type TEXT NOT NULL,
    payload TEXT NOT NULL,
    plain_text TEXT NOT NULL DEFAULT '',
    sort_order REAL NOT NULL,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (template_id, id)
);

CREATE TABLE notes_data_source_templates (
    id TEXT PRIMARY KEY,
    data_source_id TEXT NOT NULL REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    source_page_id TEXT REFERENCES notes_pages(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    properties TEXT NOT NULL DEFAULT '{}',
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE notes_data_sources (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    database_id TEXT NOT NULL REFERENCES notes_databases(id) ON DELETE CASCADE,
    title TEXT NOT NULL DEFAULT '',
    title_rich_text TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(title_rich_text)),
    description TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(description)),
    icon TEXT CHECK (icon IS NULL OR json_valid(icon)),
    properties TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(properties)),
    in_trash INTEGER NOT NULL DEFAULT 0 CHECK (in_trash IN (0, 1)),
    source_provider TEXT,
    source_object_id TEXT,
    source_workspace_id TEXT,
    source_last_edited_time TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
);

CREATE TABLE notes_database_views (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    database_id TEXT NOT NULL REFERENCES notes_databases(id) ON DELETE CASCADE,
    data_source_id TEXT NOT NULL REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT '',
    type TEXT NOT NULL CHECK (
        type IN (
            'table',
            'board',
            'list',
            'calendar',
            'timeline',
            'gallery',
            'form',
            'chart',
            'map',
            'dashboard'
        )
    ),
    filter TEXT CHECK (filter IS NULL OR json_valid(filter)),
    sorts TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(sorts)),
    configuration TEXT CHECK (configuration IS NULL OR json_valid(configuration)),
    sort_order REAL NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    source_provider TEXT,
    source_object_id TEXT,
    source_workspace_id TEXT,
    source_last_edited_time TEXT,
    url TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
);

CREATE TABLE notes_databases (
    id TEXT PRIMARY KEY REFERENCES notes_blocks(id) ON DELETE CASCADE CHECK (trim(id) <> ''),
    parent_type TEXT NOT NULL CHECK (parent_type IN ('page_id', 'block_id')),
    parent_page_id TEXT REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    title TEXT NOT NULL DEFAULT '',
    title_rich_text TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(title_rich_text)),
    description TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(description)),
    icon TEXT CHECK (icon IS NULL OR json_valid(icon)),
    cover TEXT CHECK (cover IS NULL OR json_valid(cover)),
    is_inline INTEGER NOT NULL DEFAULT 1 CHECK (is_inline IN (0, 1)),
    in_trash INTEGER NOT NULL DEFAULT 0 CHECK (in_trash IN (0, 1)),
    source_provider TEXT,
    source_object_id TEXT,
    source_workspace_id TEXT,
    source_last_edited_time TEXT,
    url TEXT,
    public_url TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''),
    CHECK (
        (
            parent_type = 'page_id'
            AND parent_page_id IS NOT NULL
            AND parent_block_id IS NULL
        )
        OR (
            parent_type = 'block_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NOT NULL
        )
    )
);

CREATE TABLE notes_folders (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    parent_folder_id TEXT REFERENCES notes_folders(id) ON DELETE SET NULL,
    name TEXT NOT NULL CHECK (
        trim(name) <> ''
        AND length(name) <= 200
    ),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''),
    CHECK (parent_folder_id IS NULL OR parent_folder_id <> id)
);

CREATE TABLE notes_history_bundle_chunks (
    parent_hash TEXT NOT NULL REFERENCES notes_history_bundles(hash) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL CHECK (chunk_index >= 0),
    chunk_hash TEXT NOT NULL REFERENCES notes_history_bundles(hash) ON DELETE RESTRICT,
    PRIMARY KEY (parent_hash, chunk_index),
    UNIQUE (parent_hash, chunk_hash)
);

CREATE TABLE notes_history_bundles (
    hash TEXT PRIMARY KEY CHECK (
        length(hash) = 64
        AND hash = lower(hash)
        AND hash NOT GLOB '*[^0-9a-f]*'
    ),
    kind TEXT NOT NULL CHECK (kind IN ('row', 'manifest', 'chunk')),
    encoding TEXT NOT NULL CHECK (
        encoding IN (
            'raw-json-v1',
            'zlib-json-v1',
            'raw-bytes-v1',
            'zlib-bytes-v1',
            'chunked-json-v1'
        )
    ),
    payload BLOB NOT NULL,
    uncompressed_bytes INTEGER NOT NULL CHECK (uncompressed_bytes >= 0),
    stored_bytes INTEGER NOT NULL CHECK (stored_bytes >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        CHECK (trim(created_at) <> ''),
    CHECK (length(payload) = stored_bytes)
);

CREATE TABLE notes_history_maintenance_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_run_at TEXT CHECK (last_run_at IS NULL OR trim(last_run_at) <> '')
);

CREATE TABLE notes_link_facts (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_object_type TEXT NOT NULL CHECK (
        source_object_type IN ('page', 'database_row', 'block', 'property', 'comment', 'import')
    ),
    source_object_id TEXT NOT NULL CHECK (trim(source_object_id) <> ''),
    source_page_id TEXT REFERENCES notes_pages(id) ON DELETE CASCADE,
    source_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    source_comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    source_data_source_id TEXT REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    source_property_id TEXT,
    source_property_name TEXT NOT NULL DEFAULT '',
    target_object_type TEXT NOT NULL CHECK (
        target_object_type IN (
            'page',
            'database_row',
            'block',
            'database',
            'data_source',
            'property',
            'comment',
            'file',
            'project',
            'project_task',
            'calendar_event',
            'pomodoro_run',
            'music_item',
            'external_url'
        )
    ),
    target_object_id TEXT NOT NULL CHECK (trim(target_object_id) <> ''),
    target_page_id TEXT REFERENCES notes_pages(id) ON DELETE CASCADE,
    target_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    target_comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    target_asset_id TEXT REFERENCES notes_assets(id) ON DELETE CASCADE,
    target_url TEXT,
    link_type TEXT NOT NULL CHECK (
        link_type IN (
            'child_page',
            'page_mention',
            'page_link',
            'block_link',
            'database_mention',
            'database_relation',
            'local_object_mention',
            'external_url',
            'page_icon',
            'page_cover',
            'block_file',
            'property_file',
            'comment_attachment',
            'import_source'
        )
    ),
    snippet TEXT NOT NULL DEFAULT '',
    created_time TEXT NOT NULL CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL CHECK (trim(last_edited_time) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    CHECK (
        (
            source_object_type IN ('page', 'database_row')
            AND source_page_id IS NOT NULL
            AND source_page_id = source_object_id
            AND source_block_id IS NULL
            AND source_comment_id IS NULL
        )
        OR (
            source_object_type = 'block'
            AND source_block_id IS NOT NULL
            AND source_block_id = source_object_id
            AND source_page_id IS NOT NULL
            AND source_comment_id IS NULL
        )
        OR (
            source_object_type = 'comment'
            AND source_comment_id IS NOT NULL
            AND source_comment_id = source_object_id
            AND source_page_id IS NOT NULL
        )
        OR (
            source_object_type = 'property'
            AND source_data_source_id IS NOT NULL
            AND source_property_id IS NOT NULL
            AND trim(source_property_id) <> ''
        )
        OR (
            source_object_type = 'import'
            AND source_page_id IS NULL
            AND source_block_id IS NULL
            AND source_comment_id IS NULL
        )
    ),
    CHECK (
        (
            target_object_type IN ('page', 'database_row')
            AND target_page_id IS NOT NULL
            AND target_page_id = target_object_id
        )
        OR (
            target_object_type = 'block'
            AND target_block_id IS NOT NULL
            AND target_block_id = target_object_id
        )
        OR (
            target_object_type = 'comment'
            AND target_comment_id IS NOT NULL
            AND target_comment_id = target_object_id
        )
        OR (
            target_object_type = 'file'
            AND target_asset_id IS NOT NULL
            AND target_asset_id = target_object_id
        )
        OR (
            target_object_type = 'external_url'
            AND target_url IS NOT NULL
            AND target_url = target_object_id
        )
        OR (
            target_object_type NOT IN ('page', 'database_row', 'block', 'comment', 'file', 'external_url')
        )
    )
);

CREATE TABLE notes_link_facts_state (
    key TEXT PRIMARY KEY CHECK (trim(key) <> ''),
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE notes_local_users (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    display_name TEXT NOT NULL CHECK (
        trim(display_name) <> ''
        AND length(display_name) <= 80
    ),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
);

CREATE TABLE notes_mention_notifications (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_type TEXT NOT NULL CHECK (source_type IN ('block', 'comment')),
    source_id TEXT NOT NULL CHECK (trim(source_id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('reminder', 'user_mention', 'task_mention')),
    target_type TEXT NOT NULL CHECK (target_type IN ('date', 'user', 'project_task')),
    target_id TEXT,
    trigger_at TEXT,
    plain_text TEXT NOT NULL DEFAULT '' CHECK (length(plain_text) <= 500),
    source_plain_text TEXT NOT NULL DEFAULT '' CHECK (length(source_plain_text) <= 2000),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'delivered', 'dismissed')),
    delivered_at TEXT,
    fingerprint TEXT NOT NULL CHECK (trim(fingerprint) <> ''),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')), suppressed_by_history_restore INTEGER NOT NULL DEFAULT 0 CHECK (
    suppressed_by_history_restore IN (0, 1)
),
    CHECK (
        (source_type = 'block' AND block_id = source_id AND comment_id IS NULL)
        OR (source_type = 'comment' AND comment_id = source_id)
    ),
    CHECK (
        (kind = 'reminder' AND target_type = 'date' AND trigger_at IS NOT NULL)
        OR (kind = 'user_mention' AND target_type = 'user' AND target_id IS NOT NULL)
        OR (kind = 'task_mention' AND target_type = 'project_task' AND target_id IS NOT NULL)
    ),
    UNIQUE (source_type, source_id, fingerprint)
);

CREATE TABLE notes_page_aliases (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    alias TEXT NOT NULL CHECK (trim(alias) <> '' AND length(alias) <= 200),
    normalized_alias TEXT NOT NULL CHECK (trim(normalized_alias) <> ''),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
);

CREATE TABLE notes_page_cover_assets (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    asset_path TEXT NOT NULL UNIQUE CHECK (
        trim(asset_path) <> ''
        AND asset_path GLOB 'notes/page-covers/*'
        AND instr(substr(asset_path, length('notes/page-covers/') + 1), '/') = 0
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

CREATE TABLE "notes_page_history_settings" (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    retention_days INTEGER NOT NULL CHECK (
        retention_days IN (0, 7, 30, 90, 180, 365)
    ),
    updated_at TEXT NOT NULL DEFAULT (
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    )
);

CREATE TABLE "notes_page_history_snapshots" (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    parent_type TEXT NOT NULL CHECK (parent_type IN ('workspace', 'page_id', 'block_id', 'data_source_id')),
    parent_page_id TEXT,
    parent_block_id TEXT,
    parent_data_source_id TEXT,
    title TEXT NOT NULL DEFAULT '',
    properties TEXT NOT NULL CHECK (json_valid(properties)),
    icon TEXT CHECK (icon IS NULL OR json_valid(icon)),
    cover TEXT CHECK (cover IS NULL OR json_valid(cover)),
    in_trash INTEGER NOT NULL DEFAULT 0 CHECK (in_trash IN (0, 1)),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    blocks TEXT NOT NULL CHECK (json_valid(blocks)),
    block_count INTEGER NOT NULL DEFAULT 0 CHECK (block_count >= 0),
    reason TEXT NOT NULL CHECK (trim(reason) <> ''),
    created_by TEXT NOT NULL REFERENCES notes_local_users(id),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    page_created_time TEXT NOT NULL CHECK (trim(page_created_time) <> ''),
    page_last_edited_time TEXT NOT NULL CHECK (trim(page_last_edited_time) <> ''), block_bundle_hash TEXT REFERENCES notes_history_bundles(hash) ON DELETE RESTRICT, folder_id TEXT,
    CHECK (
        (
            parent_type = 'workspace'
            AND parent_page_id IS NULL
            AND parent_block_id IS NULL
            AND parent_data_source_id IS NULL
        )
        OR (
            parent_type = 'page_id'
            AND parent_page_id IS NOT NULL
            AND parent_block_id IS NULL
            AND parent_data_source_id IS NULL
        )
        OR (
            parent_type = 'block_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NOT NULL
            AND parent_data_source_id IS NULL
        )
        OR (
            parent_type = 'data_source_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NULL
            AND parent_data_source_id IS NOT NULL
        )
    )
);

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

CREATE TABLE notes_page_template_blocks (
    template_id TEXT NOT NULL REFERENCES notes_page_templates(id) ON DELETE CASCADE,
    id TEXT NOT NULL CHECK (trim(id) <> ''),
    parent_type TEXT NOT NULL CHECK (parent_type IN ('template', 'block_id')),
    parent_block_id TEXT,
    has_children INTEGER NOT NULL DEFAULT 0 CHECK (has_children IN (0, 1)),
    type TEXT NOT NULL CHECK (
        type IN (
            'paragraph',
            'heading_1',
            'heading_2',
            'heading_3',
            'heading_4',
            'bulleted_list_item',
            'numbered_list_item',
            'to_do',
            'toggle',
            'callout',
            'quote',
            'child_page',
            'child_database',
            'breadcrumb',
            'table_of_contents',
            'column_list',
            'column',
            'table',
            'table_row',
            'tab',
            'image',
            'video',
            'audio',
            'file',
            'pdf',
            'bookmark',
            'link_preview',
            'synced_block',
            'template',
            'button',
            'embed',
            'equation',
            'divider',
            'code',
            'unsupported'
        )
    ),
    payload TEXT NOT NULL CHECK (json_valid(payload)),
    plain_text TEXT NOT NULL DEFAULT '',
    sort_order REAL NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''),
    PRIMARY KEY (template_id, id),
    FOREIGN KEY (template_id, parent_block_id)
        REFERENCES notes_page_template_blocks(template_id, id)
        ON DELETE CASCADE,
    CHECK (
        (
            parent_type = 'template'
            AND parent_block_id IS NULL
        )
        OR (
            parent_type = 'block_id'
            AND parent_block_id IS NOT NULL
        )
    )
);

CREATE TABLE notes_page_templates (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    source_page_id TEXT REFERENCES notes_pages(id) ON DELETE SET NULL,
    properties TEXT NOT NULL DEFAULT '{"title":{"id":"title","type":"title","title":[]}}' CHECK (json_valid(properties)),
    icon TEXT CHECK (icon IS NULL OR json_valid(icon)),
    cover TEXT CHECK (cover IS NULL OR json_valid(cover)),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> '')
);

CREATE TABLE "notes_pages" (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    parent_type TEXT NOT NULL CHECK (parent_type IN ('workspace', 'page_id', 'block_id', 'data_source_id')),
    parent_page_id TEXT REFERENCES "notes_pages"(id) ON DELETE CASCADE,
    parent_block_id TEXT,
    parent_data_source_id TEXT REFERENCES notes_data_sources(id) ON DELETE CASCADE,
    title TEXT NOT NULL DEFAULT '',
    properties TEXT NOT NULL DEFAULT '{"title":{"id":"title","type":"title","title":[]}}' CHECK (json_valid(properties)),
    icon TEXT CHECK (icon IS NULL OR json_valid(icon)),
    cover TEXT CHECK (cover IS NULL OR json_valid(cover)),
    in_trash INTEGER NOT NULL DEFAULT 0 CHECK (in_trash IN (0, 1)),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    source_provider TEXT,
    source_object_id TEXT,
    source_workspace_id TEXT,
    source_last_edited_time TEXT,
    url TEXT,
    public_url TEXT,
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''), trashed_time TEXT CHECK (trashed_time IS NULL OR trim(trashed_time) <> ''), folder_id TEXT REFERENCES notes_folders(id) ON DELETE SET NULL,
    CHECK (
        (
            parent_type = 'workspace'
            AND parent_page_id IS NULL
            AND parent_block_id IS NULL
            AND parent_data_source_id IS NULL
        )
        OR (
            parent_type = 'page_id'
            AND parent_page_id IS NOT NULL
            AND parent_block_id IS NULL
            AND parent_data_source_id IS NULL
        )
        OR (
            parent_type = 'block_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NOT NULL
            AND parent_data_source_id IS NULL
        )
        OR (
            parent_type = 'data_source_id'
            AND parent_page_id IS NULL
            AND parent_block_id IS NULL
            AND parent_data_source_id IS NOT NULL
        )
    )
);

CREATE TABLE notes_project_history_asset_pins (
    version_id TEXT NOT NULL REFERENCES notes_project_history_versions(id) ON DELETE CASCADE,
    asset_id TEXT NOT NULL REFERENCES notes_assets(id) ON DELETE RESTRICT,
    PRIMARY KEY (version_id, asset_id)
);

CREATE TABLE notes_project_history_bundle_references (
    version_id TEXT NOT NULL REFERENCES notes_project_history_versions(id) ON DELETE CASCADE,
    bundle_hash TEXT NOT NULL REFERENCES notes_history_bundles(hash) ON DELETE RESTRICT,
    PRIMARY KEY (version_id, bundle_hash)
);

CREATE TABLE notes_project_history_dirty (
    project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    first_dirty_at TEXT NOT NULL CHECK (trim(first_dirty_at) <> ''),
    last_dirty_at TEXT NOT NULL CHECK (trim(last_dirty_at) <> ''),
    actor_id TEXT NOT NULL CHECK (trim(actor_id) <> ''),
    actor_display_name TEXT NOT NULL CHECK (json_valid(actor_display_name)),
    changed_note_summary TEXT NOT NULL DEFAULT '',
    force_checkpoint INTEGER NOT NULL DEFAULT 0 CHECK (force_checkpoint IN (0, 1))
);

CREATE TABLE notes_project_history_versions (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    manifest_hash TEXT NOT NULL REFERENCES notes_history_bundles(hash) ON DELETE RESTRICT,
    reason TEXT NOT NULL CHECK (trim(reason) <> '' AND length(reason) <= 80),
    created_by TEXT NOT NULL CHECK (trim(created_by) <> ''),
    display_name TEXT NOT NULL CHECK (json_valid(display_name)),
    changed_note_summary TEXT NOT NULL DEFAULT '',
    page_count INTEGER NOT NULL DEFAULT 0 CHECK (page_count >= 0),
    active_page_count INTEGER NOT NULL DEFAULT 0 CHECK (active_page_count >= 0),
    archived_page_count INTEGER NOT NULL DEFAULT 0 CHECK (archived_page_count >= 0),
    deleted_page_count INTEGER NOT NULL DEFAULT 0 CHECK (deleted_page_count >= 0),
    manifest_uncompressed_bytes INTEGER NOT NULL DEFAULT 0
        CHECK (manifest_uncompressed_bytes >= 0),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        CHECK (trim(created_time) <> '')
);

CREATE TABLE notes_search_index (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_type TEXT NOT NULL CHECK (
        source_type IN ('page', 'block', 'comment', 'property', 'file', 'alias', 'metadata')
    ),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    property_id TEXT,
    block_type TEXT,
    title TEXT NOT NULL DEFAULT '',
    body TEXT NOT NULL DEFAULT '',
    metadata TEXT NOT NULL DEFAULT '',
    source_last_edited_time TEXT NOT NULL CHECK (trim(source_last_edited_time) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE notes_search_index_state (
    key TEXT PRIMARY KEY CHECK (trim(key) <> ''),
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE notes_suggestions (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    block_id TEXT NOT NULL REFERENCES notes_blocks(id) ON DELETE CASCADE,
    created_by TEXT NOT NULL REFERENCES notes_local_users(id),
    display_name TEXT NOT NULL CHECK (json_valid(display_name)),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'accepted', 'rejected')),
    range_start INTEGER NOT NULL CHECK (range_start >= 0),
    range_end INTEGER NOT NULL CHECK (range_end > range_start),
    original_text TEXT NOT NULL CHECK (trim(original_text) <> '' AND length(original_text) <= 2000),
    proposed_text TEXT NOT NULL DEFAULT '' CHECK (length(proposed_text) <= 2000),
    prefix_text TEXT NOT NULL DEFAULT '' CHECK (length(prefix_text) <= 120),
    suffix_text TEXT NOT NULL DEFAULT '' CHECK (length(suffix_text) <= 120),
    accepted_at TEXT,
    accepted_by TEXT REFERENCES notes_local_users(id),
    rejected_at TEXT,
    rejected_by TEXT REFERENCES notes_local_users(id),
    created_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(last_edited_time) <> ''), sync_version INTEGER NOT NULL DEFAULT 1 CHECK (sync_version >= 1),
    CHECK (
        (
            status = 'open'
            AND accepted_at IS NULL
            AND accepted_by IS NULL
            AND rejected_at IS NULL
            AND rejected_by IS NULL
        )
        OR (
            status = 'accepted'
            AND accepted_at IS NOT NULL
            AND accepted_by IS NOT NULL
            AND rejected_at IS NULL
            AND rejected_by IS NULL
        )
        OR (
            status = 'rejected'
            AND rejected_at IS NOT NULL
            AND rejected_by IS NOT NULL
            AND accepted_at IS NULL
            AND accepted_by IS NULL
        )
    )
);

CREATE TABLE notes_undo_state (
    page_id TEXT PRIMARY KEY REFERENCES notes_pages(id) ON DELETE CASCADE,
    state_payload TEXT NOT NULL CHECK (length(state_payload) <= 524288),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE notes_unresolved_link_index (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    source_type TEXT NOT NULL CHECK (source_type IN ('block', 'comment')),
    source_page_id TEXT NOT NULL REFERENCES notes_pages(id) ON DELETE CASCADE,
    source_block_id TEXT REFERENCES notes_blocks(id) ON DELETE CASCADE,
    source_comment_id TEXT REFERENCES notes_comments(id) ON DELETE CASCADE,
    raw_url TEXT NOT NULL CHECK (trim(raw_url) <> ''),
    raw_target TEXT NOT NULL CHECK (trim(raw_target) <> ''),
    normalized_target TEXT NOT NULL CHECK (trim(normalized_target) <> ''),
    link_text TEXT NOT NULL DEFAULT '',
    snippet TEXT NOT NULL DEFAULT '',
    created_time TEXT NOT NULL CHECK (trim(created_time) <> ''),
    last_edited_time TEXT NOT NULL CHECK (trim(last_edited_time) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    CHECK (
        (
            source_type = 'block'
            AND source_block_id IS NOT NULL
            AND source_comment_id IS NULL
        )
        OR (
            source_type = 'comment'
            AND source_comment_id IS NOT NULL
        )
    )
);

CREATE TABLE notes_unresolved_link_index_state (
    key TEXT PRIMARY KEY CHECK (trim(key) <> ''),
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE pomodoro_adaptive_assignments (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    experiment_id TEXT NOT NULL REFERENCES pomodoro_adaptive_experiments(id) ON DELETE CASCADE,
    variant_key TEXT NOT NULL,
    run_id TEXT REFERENCES pomodoro_runs(id) ON DELETE SET NULL,
    segment_id TEXT REFERENCES pomodoro_segments(id) ON DELETE SET NULL,
    context_snapshot_id TEXT REFERENCES pomodoro_adaptive_context_snapshots(id) ON DELETE SET NULL,
    assignment_seed TEXT NOT NULL CHECK (trim(assignment_seed) <> ''),
    assigned_at TEXT NOT NULL CHECK (trim(assigned_at) <> ''),
    FOREIGN KEY (experiment_id, variant_key)
        REFERENCES pomodoro_adaptive_experiment_variants(experiment_id, variant_key)
        ON DELETE CASCADE
);

CREATE TABLE pomodoro_adaptive_context_snapshot_features (
    snapshot_id TEXT NOT NULL REFERENCES pomodoro_adaptive_context_snapshots(id) ON DELETE CASCADE,
    feature_key TEXT NOT NULL CHECK (trim(feature_key) <> ''),
    numeric_value REAL,
    categorical_value TEXT,
    boolean_value INTEGER CHECK (boolean_value IS NULL OR boolean_value IN (0, 1)),
    missing INTEGER NOT NULL DEFAULT 0 CHECK (missing IN (0, 1)),
    source_kind TEXT NOT NULL CHECK (
        source_kind IN ('pomodoro', 'doomscrolling', 'calendar', 'diary', 'project', 'environment', 'device')
    ),
    PRIMARY KEY (snapshot_id, feature_key),
    CHECK (
        missing = 1 OR
        numeric_value IS NOT NULL OR
        categorical_value IS NOT NULL OR
        boolean_value IS NOT NULL
    )
);

CREATE TABLE pomodoro_adaptive_context_snapshots (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    run_id TEXT REFERENCES pomodoro_runs(id) ON DELETE SET NULL,
    segment_id TEXT REFERENCES pomodoro_segments(id) ON DELETE SET NULL,
    local_started_at TEXT NOT NULL CHECK (trim(local_started_at) <> ''),
    time_of_day TEXT NOT NULL CHECK (time_of_day IN ('morning', 'midday', 'afternoon', 'evening', 'late')),
    session_position TEXT NOT NULL CHECK (session_position IN ('first', 'middle', 'late')),
    event_length TEXT NOT NULL CHECK (event_length IN ('short', 'medium', 'long')),
    workload TEXT NOT NULL CHECK (workload IN ('low', 'normal', 'high')),
    energy TEXT NOT NULL CHECK (energy IN ('low', 'normal', 'high', 'unknown')),
    environment_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pomodoro_adaptive_context_state_history (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    policy_id TEXT NOT NULL REFERENCES pomodoro_adaptive_policies(id) ON DELETE CASCADE,
    context_key TEXT NOT NULL CHECK (trim(context_key) <> ''),
    observed_at TEXT NOT NULL CHECK (trim(observed_at) <> ''),
    readiness REAL NOT NULL CHECK (readiness >= 0.0 AND readiness <= 1.0),
    strain REAL NOT NULL CHECK (strain >= 0.0 AND strain <= 1.0),
    recovery_debt REAL NOT NULL CHECK (recovery_debt >= 0.0 AND recovery_debt <= 1.0),
    avoidance_pressure REAL NOT NULL CHECK (avoidance_pressure >= 0.0 AND avoidance_pressure <= 1.0),
    momentum REAL NOT NULL CHECK (momentum >= 0.0 AND momentum <= 1.0),
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pomodoro_adaptive_context_states (
    policy_id TEXT NOT NULL REFERENCES pomodoro_adaptive_policies(id) ON DELETE CASCADE,
    context_key TEXT NOT NULL CHECK (trim(context_key) <> ''),
    readiness REAL NOT NULL CHECK (readiness >= 0.0 AND readiness <= 1.0),
    strain REAL NOT NULL CHECK (strain >= 0.0 AND strain <= 1.0),
    recovery_debt REAL NOT NULL CHECK (recovery_debt >= 0.0 AND recovery_debt <= 1.0),
    avoidance_pressure REAL NOT NULL CHECK (avoidance_pressure >= 0.0 AND avoidance_pressure <= 1.0),
    momentum REAL NOT NULL CHECK (momentum >= 0.0 AND momentum <= 1.0),
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    updated_at TEXT NOT NULL,
    PRIMARY KEY (policy_id, context_key)
);

CREATE TABLE pomodoro_adaptive_data_quality_flags (
    snapshot_id TEXT NOT NULL REFERENCES pomodoro_adaptive_context_snapshots(id) ON DELETE CASCADE,
    flag TEXT NOT NULL CHECK (
        flag IN (
            'extension_unavailable',
            'desktop_tracking_unavailable',
            'diary_missing',
            'idle_detection_disabled',
            'crash_recovered',
            'calendar_clipped'
        )
    ),
    PRIMARY KEY (snapshot_id, flag)
);

CREATE TABLE pomodoro_adaptive_decision_reasons (
    decision_id TEXT NOT NULL REFERENCES pomodoro_adaptive_decisions(id) ON DELETE CASCADE,
    reason_code TEXT NOT NULL CHECK (
        reason_code IN (
            'no_history',
            'low_confidence',
            'missing_extension_data',
            'missing_diary_data',
            'high_strain',
            'high_avoidance_pressure',
            'high_recovery_debt',
            'clean_momentum',
            'break_return_drift',
            'break_transition_pressure',
            'skipped_break_recovery',
            'focus_idle_pressure',
            'repeated_blocked_source_pressure',
            'capacity_rebuild',
            'experiment_assignment',
            'experiment_guardrail',
            'guardrail_recovery',
            'replay_candidate',
            'hold_current_rhythm'
        )
    ),
    PRIMARY KEY (decision_id, reason_code)
);

CREATE TABLE pomodoro_adaptive_decision_state_scores (
    decision_id TEXT PRIMARY KEY REFERENCES pomodoro_adaptive_decisions(id) ON DELETE CASCADE,
    readiness REAL NOT NULL CHECK (readiness >= 0.0 AND readiness <= 1.0),
    strain REAL NOT NULL CHECK (strain >= 0.0 AND strain <= 1.0),
    recovery_debt REAL NOT NULL CHECK (recovery_debt >= 0.0 AND recovery_debt <= 1.0),
    avoidance_pressure REAL NOT NULL CHECK (avoidance_pressure >= 0.0 AND avoidance_pressure <= 1.0),
    momentum REAL NOT NULL CHECK (momentum >= 0.0 AND momentum <= 1.0),
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0)
);

CREATE TABLE pomodoro_adaptive_decision_values (
    decision_id TEXT NOT NULL REFERENCES pomodoro_adaptive_decisions(id) ON DELETE CASCADE,
    value_key TEXT NOT NULL CHECK (
        value_key IN (
            'focus_duration_minutes',
            'short_break_minutes',
            'long_break_minutes',
            'long_break_after_focus_count'
        )
    ),
    previous_numeric_value REAL,
    selected_numeric_value REAL NOT NULL,
    value_unit TEXT NOT NULL CHECK (value_unit IN ('minutes', 'count')),
    PRIMARY KEY (decision_id, value_key)
);

CREATE TABLE pomodoro_adaptive_decisions (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    policy_id TEXT REFERENCES pomodoro_adaptive_policies(id) ON DELETE SET NULL,
    run_id TEXT REFERENCES pomodoro_runs(id) ON DELETE SET NULL,
    segment_id TEXT REFERENCES pomodoro_segments(id) ON DELETE SET NULL,
    context_snapshot_id TEXT REFERENCES pomodoro_adaptive_context_snapshots(id) ON DELETE SET NULL,
    opportunity_kind TEXT NOT NULL CHECK (
        opportunity_kind IN (
            'run_start',
            'focus_start',
            'break_start',
            'focus_tick',
            'break_overtime',
            'block_event',
            'idle_failure',
            'run_outcome'
        )
    ),
    candidate_id TEXT CHECK (candidate_id IS NULL OR trim(candidate_id) <> ''),
    decision_mode TEXT NOT NULL CHECK (
        decision_mode IN ('fallback', 'hold', 'recovery', 'guardrail', 'exploit', 'explore')
    ),
    policy_version INTEGER NOT NULL CHECK (policy_version > 0),
    model_version INTEGER NOT NULL CHECK (model_version > 0),
    occurred_at TEXT NOT NULL CHECK (trim(occurred_at) <> ''),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pomodoro_adaptive_experiment_variants (
    experiment_id TEXT NOT NULL REFERENCES pomodoro_adaptive_experiments(id) ON DELETE CASCADE,
    variant_key TEXT NOT NULL CHECK (trim(variant_key) <> ''),
    numeric_value REAL NOT NULL,
    is_control INTEGER NOT NULL DEFAULT 0 CHECK (is_control IN (0, 1)),
    PRIMARY KEY (experiment_id, variant_key)
);

CREATE TABLE pomodoro_adaptive_experiments (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    policy_id TEXT REFERENCES pomodoro_adaptive_policies(id) ON DELETE SET NULL,
    parameter_key TEXT NOT NULL CHECK (
        parameter_key IN (
            'focus_duration_minutes',
            'short_break_minutes',
            'long_break_minutes',
            'long_break_after_focus_count',
            'rhythm_bundle'
        )
    ),
    assignment_unit TEXT NOT NULL CHECK (assignment_unit IN ('phase', 'run', 'day', 'context')),
    status TEXT NOT NULL CHECK (status IN ('draft', 'active', 'paused', 'completed', 'abandoned')),
    started_at TEXT,
    ended_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (ended_at IS NULL OR started_at IS NOT NULL)
);

CREATE TABLE pomodoro_adaptive_outcomes (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    decision_id TEXT REFERENCES pomodoro_adaptive_decisions(id) ON DELETE SET NULL,
    assignment_id TEXT REFERENCES pomodoro_adaptive_assignments(id) ON DELETE SET NULL,
    outcome_window TEXT NOT NULL CHECK (outcome_window IN ('phase', 'run', 'day', 'next_day')),
    outcome_key TEXT NOT NULL CHECK (trim(outcome_key) <> ''),
    numeric_value REAL,
    boolean_value INTEGER CHECK (boolean_value IS NULL OR boolean_value IN (0, 1)),
    categorical_value TEXT,
    measured_at TEXT NOT NULL CHECK (trim(measured_at) <> ''),
    CHECK (
        numeric_value IS NOT NULL OR
        boolean_value IS NOT NULL OR
        categorical_value IS NOT NULL
    )
);

CREATE TABLE pomodoro_adaptive_planned_blocks (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    capture_run_id TEXT REFERENCES pomodoro_runs(id) ON DELETE SET NULL,
    event_date TEXT NOT NULL CHECK (trim(event_date) <> ''),
    event_id TEXT CHECK (event_id IS NULL OR trim(event_id) <> ''),
    original_event_id TEXT NOT NULL CHECK (trim(original_event_id) <> ''),
    planned_start TEXT NOT NULL CHECK (trim(planned_start) <> ''),
    planned_end TEXT NOT NULL CHECK (trim(planned_end) <> ''),
    source_kind TEXT NOT NULL CHECK (
        source_kind IN ('live_event', 'archived_event', 'scheduler_snapshot')
    ),
    captured_at TEXT NOT NULL CHECK (trim(captured_at) <> '')
);

CREATE TABLE pomodoro_adaptive_policies (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    status TEXT NOT NULL CHECK (status IN ('active', 'paused', 'archived')),
    policy_version INTEGER NOT NULL CHECK (policy_version > 0),
    model_version INTEGER NOT NULL CHECK (model_version > 0),
    exploration_budget_per_week INTEGER NOT NULL DEFAULT 2 CHECK (
        exploration_budget_per_week >= 0 AND exploration_budget_per_week <= 20
    ),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pomodoro_adaptive_policy_bounds (
    policy_id TEXT NOT NULL REFERENCES pomodoro_adaptive_policies(id) ON DELETE CASCADE,
    parameter_key TEXT NOT NULL CHECK (
        parameter_key IN (
            'focus_duration_minutes',
            'short_break_minutes',
            'long_break_minutes',
            'long_break_after_focus_count'
        )
    ),
    min_value REAL NOT NULL,
    max_value REAL NOT NULL,
    PRIMARY KEY (policy_id, parameter_key),
    CHECK (min_value <= max_value)
);

CREATE TABLE pomodoro_config_count_rhythms (
    event_id TEXT PRIMARY KEY REFERENCES pomodoro_configs(event_id) ON DELETE CASCADE,
    focus_duration_minutes INTEGER NOT NULL CHECK (focus_duration_minutes > 0),
    short_break_minutes INTEGER NOT NULL CHECK (short_break_minutes > 0),
    long_break_minutes INTEGER NOT NULL CHECK (long_break_minutes > 0),
    long_break_after_focus_count INTEGER NOT NULL CHECK (
        long_break_after_focus_count >= 1 AND long_break_after_focus_count <= 12
    )
);

CREATE TABLE pomodoro_config_sequence_steps (
    event_id TEXT NOT NULL REFERENCES pomodoro_configs(event_id) ON DELETE CASCADE,
    step_index INTEGER NOT NULL CHECK (step_index >= 0 AND step_index < 12),
    focus_duration_minutes INTEGER NOT NULL CHECK (focus_duration_minutes > 0),
    break_phase TEXT NOT NULL CHECK (break_phase IN ('short_break', 'long_break')),
    break_duration_minutes INTEGER NOT NULL CHECK (break_duration_minutes > 0),
    PRIMARY KEY (event_id, step_index)
);

CREATE TABLE pomodoro_configs (
    event_id TEXT PRIMARY KEY REFERENCES calendar_events(id) ON DELETE CASCADE,
    rhythm_kind TEXT NOT NULL CHECK (rhythm_kind IN ('count', 'sequence')),
    rhythm_source TEXT NOT NULL CHECK (rhythm_source IN ('preset', 'custom')),
    preset_key TEXT CHECK (
        preset_key IS NULL OR preset_key IN ('adaptive', 'creative', 'balanced', 'deep', 'extended')
    ),
    idle_timeout_minutes INTEGER CHECK (idle_timeout_minutes IS NULL OR idle_timeout_minutes > 0)
);

CREATE TABLE pomodoro_pauses (
    id TEXT PRIMARY KEY,
    segment_id TEXT NOT NULL REFERENCES pomodoro_segments(id) ON DELETE CASCADE,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    reason TEXT NOT NULL CHECK (reason IN ('idle', 'manual', 'suspend')),
    detected_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pomodoro_run_adaptive_snapshots (
    run_id TEXT PRIMARY KEY REFERENCES pomodoro_runs(id) ON DELETE CASCADE,
    policy_id TEXT REFERENCES pomodoro_adaptive_policies(id) ON DELETE SET NULL,
    policy_version INTEGER NOT NULL CHECK (policy_version > 0),
    model_version INTEGER NOT NULL CHECK (model_version > 0),
    context_snapshot_id TEXT REFERENCES pomodoro_adaptive_context_snapshots(id) ON DELETE SET NULL,
    decision_id TEXT REFERENCES pomodoro_adaptive_decisions(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pomodoro_run_count_rhythms (
    run_id TEXT PRIMARY KEY REFERENCES pomodoro_runs(id) ON DELETE CASCADE,
    focus_duration_minutes INTEGER NOT NULL CHECK (focus_duration_minutes > 0),
    short_break_minutes INTEGER NOT NULL CHECK (short_break_minutes > 0),
    long_break_minutes INTEGER NOT NULL CHECK (long_break_minutes > 0),
    long_break_after_focus_count INTEGER NOT NULL CHECK (
        long_break_after_focus_count >= 1 AND long_break_after_focus_count <= 12
    )
);

CREATE TABLE pomodoro_run_events (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES pomodoro_runs(id) ON DELETE CASCADE,
    segment_id TEXT REFERENCES pomodoro_segments(id) ON DELETE SET NULL,
    event_type TEXT NOT NULL CHECK (
        event_type IN (
            'start',
            'phase_start',
            'phase_complete',
            'pause_start',
            'pause_end',
            'idle_detected',
            'focus_failed',
            'suspend_detected',
            'skip_break',
            'extend_focus',
            'go_to_break_now',
            'start_focus_now',
            'reconfigure',
            'block_transition',
            'stop',
            'complete',
            'crash_recovery'
        )
    ),
    occurred_at TEXT NOT NULL,
    phase TEXT CHECK (phase IS NULL OR phase IN ('focus', 'short_break', 'long_break')),
    reason TEXT,
    duration_seconds INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pomodoro_run_sequence_steps (
    run_id TEXT NOT NULL REFERENCES pomodoro_runs(id) ON DELETE CASCADE,
    step_index INTEGER NOT NULL CHECK (step_index >= 0 AND step_index < 12),
    focus_duration_minutes INTEGER NOT NULL CHECK (focus_duration_minutes > 0),
    break_phase TEXT NOT NULL CHECK (break_phase IN ('short_break', 'long_break')),
    break_duration_minutes INTEGER NOT NULL CHECK (break_duration_minutes > 0),
    PRIMARY KEY (run_id, step_index)
);

CREATE TABLE pomodoro_runs (
    id TEXT PRIMARY KEY,
    event_id TEXT REFERENCES calendar_events(id) ON DELETE SET NULL,
    original_event_id TEXT NOT NULL,
    event_date TEXT NOT NULL,
    planned_start TEXT NOT NULL,
    planned_end TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    end_reason TEXT CHECK (end_reason IS NULL OR end_reason IN ('completed', 'stopped', 'interrupted', 'reconfigured', 'block_transition')),
    rhythm_kind TEXT NOT NULL CHECK (rhythm_kind IN ('count', 'sequence')),
    rhythm_source TEXT NOT NULL CHECK (rhythm_source IN ('preset', 'custom')),
    preset_key TEXT CHECK (
        preset_key IS NULL OR preset_key IN ('adaptive', 'creative', 'balanced', 'deep', 'extended')
    ),
    idle_timeout_minutes INTEGER,
    last_heartbeat TEXT NOT NULL,
    event_title_snapshot TEXT,
    inherited_focus_minutes INTEGER NOT NULL DEFAULT 0,
    inherited_rhythm_position INTEGER NOT NULL DEFAULT 1,
    inherited_from_run_id TEXT REFERENCES pomodoro_runs(id) ON DELETE SET NULL,
    start_trigger TEXT NOT NULL DEFAULT 'manual' CHECK (start_trigger IN ('manual', 'block_auto', 'block_transition', 'reconfigure', 'crash_recovery')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pomodoro_segments (
    id TEXT PRIMARY KEY,
    event_id TEXT REFERENCES calendar_events(id) ON DELETE SET NULL,
    event_date TEXT NOT NULL,
    run_id TEXT NOT NULL REFERENCES pomodoro_runs(id) ON DELETE CASCADE,
    rhythm_position INTEGER NOT NULL CHECK (rhythm_position > 0),
    phase TEXT NOT NULL CHECK (phase IN ('focus', 'short_break', 'long_break')),
    planned_start TEXT NOT NULL,
    planned_end TEXT NOT NULL,
    actual_start TEXT NOT NULL,
    actual_end TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'interrupted')),
    end_reason TEXT CHECK (end_reason IS NULL OR end_reason IN ('completed', 'stopped', 'skipped_by_user', 'event_expired', 'focus_failed', 'reconfigured', 'block_transition', 'crash_recovery')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE quick_note_tags (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    name TEXT NOT NULL COLLATE NOCASE CHECK (length(trim(name)) BETWEEN 1 AND 40),
    sort_order INTEGER NOT NULL CHECK (sort_order >= 0 AND sort_order < 9),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    UNIQUE (name),
    UNIQUE (sort_order)
);

CREATE TABLE quick_notes (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    title TEXT NOT NULL DEFAULT '' CHECK (length(title) <= 200),
    body_plain_text TEXT NOT NULL DEFAULT '' CHECK (length(body_plain_text) <= 65536),
    color INTEGER NOT NULL DEFAULT 30 CHECK (color >= 0 AND color < 32),
    tag_id TEXT REFERENCES quick_note_tags(id) ON DELETE SET NULL,
    pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1)),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    trashed_at TEXT CHECK (trashed_at IS NULL OR trim(trashed_at) <> ''),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    CHECK (pinned = 0 OR (archived = 0 AND trashed_at IS NULL))
);

CREATE TABLE quick_note_text_runs (
    note_id TEXT NOT NULL REFERENCES quick_notes(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL CHECK (sort_order >= 0),
    content TEXT NOT NULL CHECK (content <> ''),
    bold INTEGER NOT NULL DEFAULT 0 CHECK (bold IN (0, 1)),
    italic INTEGER NOT NULL DEFAULT 0 CHECK (italic IN (0, 1)),
    underline INTEGER NOT NULL DEFAULT 0 CHECK (underline IN (0, 1)),
    PRIMARY KEY (note_id, sort_order)
);

CREATE TABLE project_checklist_items (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    task_id TEXT NOT NULL REFERENCES project_tasks(id) ON DELETE CASCADE,
    title TEXT NOT NULL CHECK (trim(title) <> ''),
    completed_at TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE project_custom_emojis (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    asset_path TEXT NOT NULL CHECK (
        trim(asset_path) <> ''
        AND asset_path GLOB 'project-icons/*'
        AND instr(substr(asset_path, length('project-icons/') + 1), '/') = 0
        AND instr(asset_path, '..') = 0
        AND instr(asset_path, '\') = 0
    ),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE project_custom_field_option_values (
    task_id TEXT NOT NULL REFERENCES project_tasks(id) ON DELETE CASCADE,
    field_id TEXT NOT NULL REFERENCES project_custom_fields(id) ON DELETE CASCADE,
    option_id TEXT NOT NULL REFERENCES project_custom_field_options(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    PRIMARY KEY (task_id, field_id, option_id),
    FOREIGN KEY (field_id, option_id) REFERENCES project_custom_field_options(field_id, id) ON DELETE CASCADE
);

CREATE TABLE project_custom_field_options (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    field_id TEXT NOT NULL REFERENCES project_custom_fields(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    UNIQUE (field_id, id)
);

CREATE TABLE project_custom_field_values (
    task_id TEXT NOT NULL REFERENCES project_tasks(id) ON DELETE CASCADE,
    field_id TEXT NOT NULL REFERENCES project_custom_fields(id) ON DELETE CASCADE,
    text_value TEXT,
    number_value REAL,
    date_value TEXT,
    checkbox_value INTEGER CHECK (checkbox_value IS NULL OR checkbox_value IN (0, 1)),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    PRIMARY KEY (task_id, field_id),
    CHECK (
        (text_value IS NOT NULL AND number_value IS NULL AND date_value IS NULL AND checkbox_value IS NULL) OR
        (text_value IS NULL AND number_value IS NOT NULL AND date_value IS NULL AND checkbox_value IS NULL) OR
        (text_value IS NULL AND number_value IS NULL AND date_value IS NOT NULL AND checkbox_value IS NULL) OR
        (text_value IS NULL AND number_value IS NULL AND date_value IS NULL AND checkbox_value IS NOT NULL)
    )
);

CREATE TABLE "project_custom_fields" (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    field_type TEXT NOT NULL CHECK (
        field_type IN (
            'text',
            'number',
            'select',
            'multi_select',
            'status',
            'date',
            'person',
            'files',
            'checkbox',
            'url',
            'phone',
            'email'
        )
    ),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE project_groups (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    icon TEXT NOT NULL DEFAULT 'folder' CHECK (trim(icon) <> ''),
    color INTEGER CHECK (color IS NULL OR (color >= 0 AND color < 32)),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    collapsed INTEGER NOT NULL DEFAULT 0 CHECK (collapsed IN (0, 1)),
    hidden_at TEXT,
    archived_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE project_priorities (
    id TEXT NOT NULL CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    color INTEGER NOT NULL CHECK (color >= 0 AND color < 32),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    PRIMARY KEY (project_id, id)
);

CREATE TABLE project_sections (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    collapsed INTEGER NOT NULL DEFAULT 0 CHECK (collapsed IN (0, 1)),
    hidden_at TEXT,
    archived_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE project_statuses (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    category TEXT NOT NULL CHECK (category IN ('not_started', 'active', 'blocked', 'done')),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    terminal INTEGER NOT NULL DEFAULT 0 CHECK (terminal IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
, color INTEGER NOT NULL DEFAULT 30 CHECK (color >= 0 AND color < 32));

CREATE TABLE "project_tags" (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    color INTEGER CHECK (color IS NULL OR (color >= 0 AND color < 32)),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> '')
);

CREATE TABLE project_task_change_events (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    task_id TEXT NOT NULL REFERENCES project_tasks(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL CHECK (
        event_type IN (
            'created',
            'updated',
            'status_changed',
            'scheduled',
            'completed',
            'reopened',
            'archived',
            'event_unlinked',
            'dependency_added',
            'dependency_removed'
        )
    ),
    field_name TEXT,
    old_value TEXT,
    new_value TEXT,
    reason TEXT,
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(occurred_at) <> '')
);

CREATE TABLE project_task_dependencies (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    blocking_task_id TEXT NOT NULL REFERENCES project_tasks(id) ON DELETE CASCADE,
    blocked_task_id TEXT NOT NULL REFERENCES project_tasks(id) ON DELETE CASCADE,
    dependency_type TEXT NOT NULL DEFAULT 'blocks' CHECK (dependency_type IN ('blocks')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    CHECK (blocking_task_id <> blocked_task_id)
);

CREATE TABLE project_task_event_links (
    task_id TEXT NOT NULL REFERENCES project_tasks(id) ON DELETE CASCADE,
    event_id TEXT NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    link_kind TEXT NOT NULL DEFAULT 'scheduled' CHECK (link_kind IN ('scheduled', 'reference')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    PRIMARY KEY (task_id, event_id)
);

CREATE TABLE "project_task_tag_links" (
    task_id TEXT NOT NULL REFERENCES project_tasks(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES "project_tags"(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    PRIMARY KEY (task_id, tag_id)
);

CREATE TABLE project_tasks (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    section_id TEXT NOT NULL REFERENCES project_sections(id) ON DELETE RESTRICT,
    status_id TEXT NOT NULL REFERENCES project_statuses(id) ON DELETE RESTRICT,
    parent_task_id TEXT REFERENCES project_tasks(id) ON DELETE CASCADE,
    title TEXT NOT NULL CHECK (trim(title) <> ''),
    description TEXT NOT NULL DEFAULT '',
    priority TEXT NOT NULL DEFAULT 'normal' CHECK (trim(priority) <> ''),
    task_type TEXT NOT NULL DEFAULT 'task' CHECK (task_type IN ('task', 'milestone', 'bug', 'habit')),
    section_sort_order REAL NOT NULL DEFAULT 0,
    status_sort_order REAL NOT NULL DEFAULT 0,
    estimate_minutes INTEGER CHECK (estimate_minutes IS NULL OR estimate_minutes > 0),
    due_date TEXT,
    start_date TEXT,
    target_end_date TEXT,
    completed_at TEXT,
    archived_at TEXT,
    blocker_reason TEXT,
    milestone INTEGER NOT NULL DEFAULT 0 CHECK (milestone IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''), start_time TEXT CHECK (
    start_time IS NULL OR (
        start_date IS NOT NULL
        AND length(start_time) = 5
        AND start_time GLOB '[0-9][0-9]:[0-9][0-9]'
        AND substr(start_time, 3, 1) = ':'
        AND substr(start_time, 1, 2) BETWEEN '00' AND '23'
        AND substr(start_time, 4, 2) BETWEEN '00' AND '59'
    )
), due_time TEXT CHECK (
    due_time IS NULL OR (
        due_date IS NOT NULL
        AND length(due_time) = 5
        AND due_time GLOB '[0-9][0-9]:[0-9][0-9]'
        AND substr(due_time, 3, 1) = ':'
        AND substr(due_time, 1, 2) BETWEEN '00' AND '23'
        AND substr(due_time, 4, 2) BETWEEN '00' AND '59'
    )
),
    CHECK (parent_task_id IS NULL OR parent_task_id <> id),
    CHECK (start_date IS NULL OR target_end_date IS NULL OR start_date <= target_end_date)
);

CREATE TABLE "project_view_preferences" (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    view_id TEXT NOT NULL CHECK (view_id IN ('dashboard', 'list', 'kanban', 'calendar', 'gantt')),
    preference_key TEXT NOT NULL CHECK (trim(preference_key) <> ''),
    preference_value TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    PRIMARY KEY (project_id, view_id, preference_key)
);

CREATE TABLE projects (
    id TEXT PRIMARY KEY CHECK (trim(id) <> ''),
    group_id TEXT NOT NULL REFERENCES project_groups(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    icon TEXT NOT NULL DEFAULT 'folder' CHECK (trim(icon) <> ''),
    color INTEGER CHECK (color IS NULL OR (color >= 0 AND color < 32)),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'hidden', 'archived')),
    default_event_duration_minutes INTEGER CHECK (
        default_event_duration_minutes IS NULL
        OR (default_event_duration_minutes > 0 AND default_event_duration_minutes <= 1440)
    ),
    default_pomodoro_mode TEXT NOT NULL DEFAULT 'preset' CHECK (
        default_pomodoro_mode IN ('none', 'preset', 'custom')
    ),
    default_pomodoro_preset_key TEXT DEFAULT 'adaptive' CHECK (
        default_pomodoro_preset_key IS NULL
        OR default_pomodoro_preset_key IN ('adaptive', 'creative', 'balanced', 'deep', 'extended')
    ),
    default_pomodoro_focus_minutes INTEGER CHECK (
        default_pomodoro_focus_minutes IS NULL
        OR (default_pomodoro_focus_minutes >= 1 AND default_pomodoro_focus_minutes <= 120)
    ),
    default_pomodoro_short_break_minutes INTEGER CHECK (
        default_pomodoro_short_break_minutes IS NULL
        OR (default_pomodoro_short_break_minutes >= 1 AND default_pomodoro_short_break_minutes <= 30)
    ),
    default_pomodoro_long_break_minutes INTEGER CHECK (
        default_pomodoro_long_break_minutes IS NULL
        OR (default_pomodoro_long_break_minutes >= 1 AND default_pomodoro_long_break_minutes <= 60)
    ),
    default_pomodoro_long_break_after_focus_count INTEGER CHECK (
        default_pomodoro_long_break_after_focus_count IS NULL
        OR (
            default_pomodoro_long_break_after_focus_count >= 1
            AND default_pomodoro_long_break_after_focus_count <= 12
        )
    ),
    focus_playlist_id TEXT,
    break_playlist_id TEXT,
    work_environment_id TEXT,
    blocker_ruleset_id TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''), default_event_name TEXT CHECK (
    default_event_name IS NULL OR trim(default_event_name) <> ''
), default_idle_settings_source TEXT NOT NULL DEFAULT 'global'
CHECK (default_idle_settings_source IN ('global', 'custom')), default_idle_pause_enabled INTEGER NOT NULL DEFAULT 1
CHECK (default_idle_pause_enabled IN (0, 1)), default_idle_threshold_minutes INTEGER NOT NULL DEFAULT 3
CHECK (default_idle_threshold_minutes IN (1, 2, 3, 4, 5, 10, 15)), default_event_time_mode TEXT NOT NULL DEFAULT 'timed'
CHECK (default_event_time_mode IN ('timed', 'all_day')), notes_default_open_mode TEXT CHECK (
    notes_default_open_mode IS NULL
    OR notes_default_open_mode IN ('center', 'side', 'full')
), notes_history_retention_days INTEGER CHECK (
    notes_history_retention_days IS NULL
    OR notes_history_retention_days IN (0, 7, 30, 90, 180, 365)
),
    CHECK (
        (
            default_pomodoro_mode = 'none'
            AND default_pomodoro_preset_key IS NULL
            AND default_pomodoro_focus_minutes IS NULL
            AND default_pomodoro_short_break_minutes IS NULL
            AND default_pomodoro_long_break_minutes IS NULL
            AND default_pomodoro_long_break_after_focus_count IS NULL
        )
        OR (
            default_pomodoro_mode = 'preset'
            AND default_pomodoro_preset_key IS NOT NULL
            AND default_pomodoro_focus_minutes IS NULL
            AND default_pomodoro_short_break_minutes IS NULL
            AND default_pomodoro_long_break_minutes IS NULL
            AND default_pomodoro_long_break_after_focus_count IS NULL
        )
        OR (
            default_pomodoro_mode = 'custom'
            AND default_pomodoro_preset_key IS NULL
            AND default_pomodoro_focus_minutes IS NOT NULL
            AND default_pomodoro_short_break_minutes IS NOT NULL
            AND default_pomodoro_long_break_minutes IS NOT NULL
            AND default_pomodoro_long_break_after_focus_count IS NOT NULL
        )
    )
);

CREATE TABLE project_working_folders (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) BETWEEN 1 AND 1024),
    project_id TEXT NOT NULL REFERENCES projects(id) ON UPDATE CASCADE ON DELETE CASCADE,
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) BETWEEN 1 AND 240),
    kind TEXT NOT NULL CHECK (kind IN ('managed', 'external')),
    managed_relative_path TEXT CHECK (
        managed_relative_path IS NULL OR (
            length(managed_relative_path) BETWEEN 1 AND 2048
            AND managed_relative_path NOT LIKE '/%'
            AND managed_relative_path NOT LIKE '%/../%'
            AND managed_relative_path NOT LIKE '../%'
            AND managed_relative_path NOT LIKE '%/..'
            AND managed_relative_path NOT LIKE '%\\%'
        )
    ),
    repository_kind TEXT NOT NULL DEFAULT 'none' CHECK (repository_kind IN ('git', 'none')),
    repository_identity TEXT CHECK (
        repository_identity IS NULL OR length(repository_identity) BETWEEN 1 AND 1024
    ),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(created_at) <> ''),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) CHECK (trim(updated_at) <> ''),
    archived_at TEXT CHECK (archived_at IS NULL OR length(archived_at) >= 20),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    UNIQUE (id, project_id),
    CHECK (
        (kind = 'managed' AND managed_relative_path IS NOT NULL AND archived_at IS NULL)
        OR (kind = 'external' AND managed_relative_path IS NULL)
    ),
    CHECK (
        (repository_kind = 'git' AND repository_identity IS NOT NULL)
        OR (repository_kind = 'none' AND repository_identity IS NULL)
    )
) STRICT;

CREATE UNIQUE INDEX idx_project_working_folders_managed
ON project_working_folders(project_id)
WHERE kind = 'managed';

CREATE INDEX idx_project_working_folders_project_active
ON project_working_folders(project_id, archived_at, sort_order, display_name COLLATE NOCASE, id);

CREATE TRIGGER project_working_folders_protect_managed_delete
BEFORE DELETE ON project_working_folders
WHEN OLD.kind = 'managed' AND EXISTS (SELECT 1 FROM projects WHERE id = OLD.project_id)
BEGIN
    SELECT RAISE(ABORT, 'Managed project working folders cannot be deleted');
END;

CREATE TRIGGER project_working_folders_protect_managed_update
BEFORE UPDATE OF project_id, kind, managed_relative_path, archived_at ON project_working_folders
WHEN OLD.kind = 'managed' AND (
    NEW.project_id != OLD.project_id
    OR NEW.kind != 'managed'
    OR NEW.managed_relative_path != OLD.managed_relative_path
    OR NEW.archived_at IS NOT NULL
)
BEGIN
    SELECT RAISE(ABORT, 'Managed project working folders cannot change ownership or be archived');
END;

CREATE TABLE theme_event_palette (
    theme_id TEXT NOT NULL REFERENCES themes(id) ON DELETE CASCADE,
    slot INTEGER NOT NULL CHECK (slot >= 0 AND slot < 32),
    value TEXT NOT NULL,
    PRIMARY KEY (theme_id, slot)
);

CREATE TABLE theme_seed_event_palette (
    theme_id TEXT NOT NULL REFERENCES themes(id) ON DELETE CASCADE,
    slot INTEGER NOT NULL CHECK (slot >= 0 AND slot < 32),
    value TEXT NOT NULL,
    PRIMARY KEY (theme_id, slot)
);

CREATE TABLE theme_seed_tokens (
    theme_id TEXT NOT NULL REFERENCES themes(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('source', 'app', 'calendar')),
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    isolated INTEGER NOT NULL DEFAULT 0 CHECK (isolated IN (0, 1)),
    PRIMARY KEY (theme_id, kind, key)
);

CREATE TABLE theme_tokens (
    theme_id TEXT NOT NULL REFERENCES themes(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('source', 'app', 'calendar')),
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    isolated INTEGER NOT NULL DEFAULT 0 CHECK (isolated IN (0, 1)),
    PRIMARY KEY (theme_id, kind, key)
);

CREATE TABLE theme_upgrade_dismissals (
    theme_id TEXT NOT NULL REFERENCES themes(id) ON DELETE CASCADE,
    engine_version INTEGER NOT NULL,
    dismissed_at INTEGER NOT NULL,
    PRIMARY KEY (theme_id, engine_version)
);

CREATE TABLE themes (
    id TEXT PRIMARY KEY CHECK (id NOT IN ('light', 'dark')),
    display_name TEXT NOT NULL,
    blend_canvas TEXT NOT NULL,
    seed_blend_canvas TEXT NOT NULL,
    derivation_engine_version INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    icon_label TEXT NOT NULL CHECK (icon_label IN ('light', 'dark')),
    seed_icon_label TEXT NOT NULL CHECK (seed_icon_label IN ('light', 'dark')),
    calendar_default_mode TEXT NOT NULL DEFAULT 'app-canvas' CHECK (calendar_default_mode IN ('light', 'dark', 'app-canvas', 'custom')),
    calendar_default_custom TEXT NOT NULL DEFAULT '#27282A',
    seed_calendar_default_mode TEXT NOT NULL DEFAULT 'app-canvas' CHECK (seed_calendar_default_mode IN ('light', 'dark', 'app-canvas', 'custom')),
    seed_calendar_default_custom TEXT NOT NULL DEFAULT '#27282A'
);

CREATE VIRTUAL TABLE notes_search_fts USING fts5(
    index_id UNINDEXED,
    title,
    body,
    metadata,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE VIRTUAL TABLE quick_notes_search_fts USING fts5(
    note_id UNINDEXED,
    title,
    body,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE INDEX idx_alarms_event ON calendar_event_alarms(event_id);

CREATE INDEX idx_alarms_icalendar_component ON calendar_event_alarms(icalendar_component_id);

CREATE INDEX idx_attendees_event ON calendar_event_attendees(event_id);

CREATE INDEX idx_attendees_icalendar_component ON calendar_event_attendees(icalendar_component_id);

CREATE INDEX idx_calendar_event_archive_overrides_source ON calendar_event_archive_overrides(source_override_id);

CREATE INDEX idx_calendar_events_archive_calendar ON calendar_events_archive(calendar_id);

CREATE INDEX idx_calendar_events_archive_source ON calendar_events_archive(source_event_id);

CREATE INDEX idx_calendar_events_calendar ON calendar_events(calendar_id);

CREATE INDEX idx_calendar_events_end ON calendar_events(end_time);

CREATE INDEX idx_calendar_events_icalendar_component ON calendar_events(icalendar_component_id);

CREATE INDEX idx_calendar_events_project ON calendar_events(project_id, start_time);

CREATE UNIQUE INDEX idx_calendar_events_source_uid ON calendar_events(calendar_id, source_uid);

CREATE INDEX idx_calendar_events_start ON calendar_events(start_time);

CREATE INDEX idx_doomscrolling_block_events_run
ON doomscrolling_block_events(run_id, occurred_at);

CREATE INDEX idx_doomscrolling_block_events_source
ON doomscrolling_block_events(source_type, source_key, occurred_at);

CREATE INDEX idx_doomscrolling_usage_samples_date_source ON doomscrolling_usage_samples(local_date, source_type, source_key);

CREATE INDEX idx_doomscrolling_usage_samples_started ON doomscrolling_usage_samples(started_at);

CREATE INDEX idx_event_categories_event ON calendar_event_categories(event_id, sort_order);

CREATE UNIQUE INDEX idx_event_exdates_event_date ON calendar_event_exdates(event_id, occurrence_date);

CREATE UNIQUE INDEX idx_event_extended_properties_key ON calendar_event_extended_properties(event_id, property_key);

CREATE INDEX idx_event_notifications_event ON calendar_event_notifications(event_id, sort_order);

CREATE UNIQUE INDEX idx_event_rdates_event_start ON calendar_event_rdates(event_id, occurrence_start);

CREATE INDEX idx_icalendar_components_calendar_type ON icalendar_components(calendar_id, component_type);

CREATE INDEX idx_icalendar_components_object ON icalendar_components(object_id);

CREATE INDEX idx_icalendar_components_projection ON icalendar_components(projected_kind, projected_id);

CREATE INDEX idx_icalendar_components_status ON icalendar_components(preservation_status);

CREATE INDEX idx_icalendar_components_uid ON icalendar_components(calendar_id, uid);

CREATE INDEX idx_icalendar_components_uid_recurrence ON icalendar_components(calendar_id, uid, recurrence_id);

CREATE INDEX idx_icalendar_object_diagnostics_object ON icalendar_object_diagnostics(object_id, sort_order);

CREATE INDEX idx_icalendar_objects_calendar ON icalendar_objects(calendar_id);

CREATE INDEX idx_icalendar_objects_fingerprint ON icalendar_objects(source_fingerprint);

CREATE INDEX idx_icalendar_objects_source ON icalendar_objects(calendar_id, source_kind, source_name);

CREATE INDEX idx_icalendar_parameters_property ON icalendar_property_parameters(property_id, sort_order);

CREATE INDEX idx_icalendar_projection_warnings_component ON icalendar_component_projection_warnings(component_id, sort_order);

CREATE INDEX idx_icalendar_properties_component ON icalendar_component_properties(component_id, sort_order);

CREATE INDEX idx_icalendar_value_nodes_parameter ON icalendar_value_nodes(parameter_id, parent_node_id, sort_order);

CREATE INDEX idx_icalendar_value_nodes_parent ON icalendar_value_nodes(parent_node_id, sort_order);

CREATE INDEX idx_icalendar_value_nodes_property ON icalendar_value_nodes(property_id, parent_node_id, sort_order);

CREATE INDEX idx_music_playlist_tracks_playlist_position ON music_playlist_tracks(playlist_id, position);

CREATE INDEX idx_music_playlist_tracks_source_identity ON music_playlist_tracks(source_identity);

CREATE INDEX idx_music_track_skip_ranges_track ON music_track_skip_ranges(track_id, sort_order);

CREATE INDEX idx_notes_asset_references_block
    ON notes_asset_references(block_id, role, asset_id)
    WHERE block_id IS NOT NULL;

CREATE INDEX idx_notes_asset_references_comment
    ON notes_asset_references(comment_id, asset_id)
    WHERE comment_id IS NOT NULL;

CREATE INDEX idx_notes_asset_references_data_source
    ON notes_asset_references(data_source_id, property_id, asset_id)
    WHERE data_source_id IS NOT NULL;

CREATE INDEX idx_notes_asset_references_owner
    ON notes_asset_references(owner_type, owner_id, role, asset_id);

CREATE INDEX idx_notes_asset_references_page
    ON notes_asset_references(page_id, role, asset_id)
    WHERE page_id IS NOT NULL;

CREATE INDEX idx_notes_assets_hash ON notes_assets(sha256, byte_size, content_type);

CREATE INDEX idx_notes_assets_path ON notes_assets(asset_path);

CREATE INDEX idx_notes_assets_state ON notes_assets(storage_state, updated_at DESC, id);

CREATE INDEX idx_notes_backlink_index_source_block
    ON notes_backlink_index(source_block_id, reference_type, id)
    WHERE source_block_id IS NOT NULL;

CREATE INDEX idx_notes_backlink_index_source_comment
    ON notes_backlink_index(source_comment_id, reference_type, id)
    WHERE source_comment_id IS NOT NULL;

CREATE INDEX idx_notes_backlink_index_source_page
    ON notes_backlink_index(source_page_id, source_type, last_edited_time DESC, id);

CREATE INDEX idx_notes_backlink_index_target
    ON notes_backlink_index(target_type, target_id, last_edited_time DESC, id);

CREATE INDEX idx_notes_blocks_page ON notes_blocks(page_id, in_trash, sort_order, id);

CREATE INDEX idx_notes_blocks_parent_block ON notes_blocks(parent_block_id, in_trash, sort_order, id);

CREATE INDEX idx_notes_blocks_parent_page ON notes_blocks(parent_page_id, in_trash, sort_order, id);

CREATE INDEX idx_notes_blocks_source ON notes_blocks(source_provider, source_object_id);

CREATE INDEX idx_notes_collaboration_operations_entity
    ON notes_collaboration_operations(entity_type, entity_id, sequence);

CREATE INDEX idx_notes_collaboration_operations_page
    ON notes_collaboration_operations(page_id, sequence);

CREATE INDEX idx_notes_collaboration_operations_sync_state
    ON notes_collaboration_operations(sync_state, sequence);

CREATE INDEX idx_notes_comment_thread_anchors_block
ON notes_comment_thread_anchors(block_id, start_offset, end_offset);

CREATE INDEX idx_notes_comment_thread_anchors_page
ON notes_comment_thread_anchors(page_id, block_id);

CREATE INDEX idx_notes_comment_thread_reads_user
    ON notes_comment_thread_reads(user_id, read_at DESC, thread_id);

CREATE INDEX idx_notes_comment_threads_page ON notes_comment_threads(page_id, status, created_time, id);

CREATE INDEX idx_notes_comment_threads_parent_block ON notes_comment_threads(parent_block_id, status, created_time, id);

CREATE INDEX idx_notes_comment_threads_source
    ON notes_comment_threads(source_provider, source_workspace_id, source_object_id);

CREATE INDEX idx_notes_comments_source
    ON notes_comments(source_provider, source_workspace_id, source_object_id);

CREATE INDEX idx_notes_comments_thread ON notes_comments(thread_id, deleted_at, created_time, id);

CREATE INDEX idx_notes_data_source_template_blocks_parent
    ON notes_data_source_template_blocks(template_id, parent_type, parent_block_id, sort_order, id);

CREATE UNIQUE INDEX idx_notes_data_source_templates_default
    ON notes_data_source_templates(data_source_id)
    WHERE is_default = 1;

CREATE INDEX idx_notes_data_source_templates_source
    ON notes_data_source_templates(data_source_id, last_edited_time DESC, name);

CREATE INDEX idx_notes_data_sources_database ON notes_data_sources(database_id, in_trash, title);

CREATE INDEX idx_notes_data_sources_source ON notes_data_sources(source_provider, source_workspace_id, source_object_id);

CREATE INDEX idx_notes_database_views_data_source ON notes_database_views(data_source_id, sort_order, id);

CREATE INDEX idx_notes_database_views_database ON notes_database_views(database_id, sort_order, id);

CREATE INDEX idx_notes_database_views_source ON notes_database_views(source_provider, source_workspace_id, source_object_id);

CREATE INDEX idx_notes_databases_parent ON notes_databases(parent_type, parent_page_id, parent_block_id);

CREATE INDEX idx_notes_databases_source ON notes_databases(source_provider, source_workspace_id, source_object_id);

CREATE INDEX idx_notes_folders_project_parent
    ON notes_folders(project_id, parent_folder_id, name COLLATE NOCASE, id);

CREATE INDEX idx_notes_history_bundle_chunks_chunk
    ON notes_history_bundle_chunks(chunk_hash, parent_hash);

CREATE INDEX idx_notes_history_bundles_created
    ON notes_history_bundles(created_at, hash);

CREATE INDEX idx_notes_link_facts_source
    ON notes_link_facts(source_object_type, source_object_id, link_type, id);

CREATE INDEX idx_notes_link_facts_source_block
    ON notes_link_facts(source_block_id, link_type, target_object_type, id)
    WHERE source_block_id IS NOT NULL;

CREATE INDEX idx_notes_link_facts_source_comment
    ON notes_link_facts(source_comment_id, link_type, target_object_type, id)
    WHERE source_comment_id IS NOT NULL;

CREATE INDEX idx_notes_link_facts_source_page
    ON notes_link_facts(source_page_id, link_type, target_object_type, id)
    WHERE source_page_id IS NOT NULL;

CREATE INDEX idx_notes_link_facts_source_property
    ON notes_link_facts(source_data_source_id, source_property_id, target_object_type, id)
    WHERE source_data_source_id IS NOT NULL;

CREATE INDEX idx_notes_link_facts_target
    ON notes_link_facts(target_object_type, target_object_id, link_type, id);

CREATE INDEX idx_notes_link_facts_target_asset
    ON notes_link_facts(target_asset_id, source_object_type, link_type, id)
    WHERE target_asset_id IS NOT NULL;

CREATE INDEX idx_notes_link_facts_target_block
    ON notes_link_facts(target_block_id, source_object_type, link_type, id)
    WHERE target_block_id IS NOT NULL;

CREATE INDEX idx_notes_link_facts_target_page
    ON notes_link_facts(target_page_id, source_object_type, link_type, id)
    WHERE target_page_id IS NOT NULL;

CREATE INDEX idx_notes_link_facts_target_url
    ON notes_link_facts(target_url, source_object_type, id)
    WHERE target_url IS NOT NULL;

CREATE INDEX idx_notes_local_users_display_name
    ON notes_local_users(display_name);

CREATE INDEX idx_notes_mention_notifications_page
    ON notes_mention_notifications(page_id, status, created_time, id);

CREATE INDEX idx_notes_mention_notifications_source
    ON notes_mention_notifications(source_type, source_id);

CREATE INDEX idx_notes_mention_notifications_status
    ON notes_mention_notifications(status, kind, trigger_at, created_time, id);

CREATE UNIQUE INDEX idx_notes_page_aliases_normalized
    ON notes_page_aliases(normalized_alias);

CREATE INDEX idx_notes_page_aliases_page
    ON notes_page_aliases(page_id, alias COLLATE NOCASE, id);

CREATE INDEX idx_notes_page_cover_assets_recent ON notes_page_cover_assets(updated_at DESC, id);

CREATE INDEX idx_notes_page_history_snapshots_bundle
    ON notes_page_history_snapshots(block_bundle_hash)
    WHERE block_bundle_hash IS NOT NULL;

CREATE INDEX idx_notes_page_history_snapshots_created
    ON notes_page_history_snapshots(created_time);

CREATE INDEX idx_notes_page_history_snapshots_created_by
    ON notes_page_history_snapshots(created_by, created_time DESC, id);

CREATE INDEX idx_notes_page_history_snapshots_page
    ON notes_page_history_snapshots(page_id, created_time DESC, id);

CREATE INDEX idx_notes_page_icon_assets_recent ON notes_page_icon_assets(updated_at DESC, id);

CREATE INDEX idx_notes_page_template_blocks_parent
    ON notes_page_template_blocks(template_id, parent_block_id, sort_order, id);

CREATE INDEX idx_notes_page_template_blocks_root
    ON notes_page_template_blocks(template_id, parent_type, sort_order, id);

CREATE INDEX idx_notes_page_templates_recent ON notes_page_templates(last_edited_time DESC, name, id);

CREATE INDEX idx_notes_page_templates_source ON notes_page_templates(source_page_id);

CREATE INDEX idx_notes_pages_active
    ON notes_pages(in_trash, archived, last_edited_time DESC, title);

CREATE INDEX idx_notes_pages_data_source
    ON notes_pages(parent_data_source_id, in_trash, archived, last_edited_time DESC, title);

CREATE INDEX idx_notes_pages_folder
    ON notes_pages(folder_id, in_trash, archived, last_edited_time DESC, title);

CREATE INDEX idx_notes_pages_parent
    ON notes_pages(parent_type, parent_page_id, parent_block_id, parent_data_source_id);

CREATE INDEX idx_notes_pages_source
    ON notes_pages(source_provider, source_workspace_id, source_object_id);

CREATE INDEX idx_notes_pages_trash_retention
    ON notes_pages(in_trash, trashed_time);

CREATE INDEX idx_notes_pages_visible
    ON notes_pages(in_trash, last_edited_time DESC, title);

CREATE INDEX idx_notes_project_history_asset_pins_asset
    ON notes_project_history_asset_pins(asset_id, version_id);

CREATE INDEX idx_notes_project_history_bundle_references_hash
    ON notes_project_history_bundle_references(bundle_hash, version_id);

CREATE INDEX idx_notes_project_history_dirty_deadlines
    ON notes_project_history_dirty(force_checkpoint, first_dirty_at, last_dirty_at, project_id);

CREATE INDEX idx_notes_project_history_dirty_due
    ON notes_project_history_dirty(force_checkpoint DESC, last_dirty_at, first_dirty_at, project_id);

CREATE INDEX idx_notes_project_history_versions_project
    ON notes_project_history_versions(project_id, created_time DESC, id DESC);

CREATE INDEX idx_notes_relation_links_source
    ON notes_data_source_relation_links(source_data_source_id, source_property_id, source_page_id);

CREATE INDEX idx_notes_relation_links_target
    ON notes_data_source_relation_links(target_page_id, source_page_id);

CREATE INDEX idx_notes_relation_links_target_data_source
    ON notes_data_source_relation_links(target_data_source_id, target_page_id);

CREATE INDEX idx_notes_rollup_cache_source_data_source
    ON notes_data_source_rollup_cache(source_data_source_id);

CREATE INDEX idx_notes_rollup_cache_target_data_source
    ON notes_data_source_rollup_cache(target_data_source_id);

CREATE INDEX idx_notes_search_index_block
    ON notes_search_index(block_id, source_last_edited_time DESC, id);

CREATE INDEX idx_notes_search_index_comment
    ON notes_search_index(comment_id, source_last_edited_time DESC, id);

CREATE INDEX idx_notes_search_index_page
    ON notes_search_index(page_id, source_type, source_last_edited_time DESC, id);

CREATE INDEX idx_notes_suggestions_block_range
    ON notes_suggestions(block_id, range_start, range_end, status);

CREATE INDEX idx_notes_suggestions_page
    ON notes_suggestions(page_id, status, created_time, id);

CREATE INDEX idx_notes_undo_state_updated_at ON notes_undo_state(updated_at);

CREATE INDEX idx_notes_unresolved_link_block
    ON notes_unresolved_link_index(source_block_id, id)
    WHERE source_block_id IS NOT NULL;

CREATE INDEX idx_notes_unresolved_link_comment
    ON notes_unresolved_link_index(source_comment_id, id)
    WHERE source_comment_id IS NOT NULL;

CREATE INDEX idx_notes_unresolved_link_page
    ON notes_unresolved_link_index(source_page_id, last_edited_time DESC, id);

CREATE INDEX idx_notes_unresolved_link_target
    ON notes_unresolved_link_index(normalized_target, source_page_id, id);

CREATE UNIQUE INDEX idx_override_extended_properties_key ON calendar_event_override_extended_properties(override_id, property_key);

CREATE INDEX idx_overrides_icalendar_component ON calendar_event_overrides(icalendar_component_id);

CREATE UNIQUE INDEX idx_overrides_parent_recid ON calendar_event_overrides(parent_event_id, recurrence_id);

CREATE INDEX idx_pomodoro_adaptive_assignments_experiment
ON pomodoro_adaptive_assignments(experiment_id, assigned_at);

CREATE INDEX idx_pomodoro_adaptive_context_snapshots_run
ON pomodoro_adaptive_context_snapshots(run_id, created_at);

CREATE INDEX idx_pomodoro_adaptive_decisions_policy
ON pomodoro_adaptive_decisions(policy_id, occurred_at);

CREATE INDEX idx_pomodoro_adaptive_decisions_run
ON pomodoro_adaptive_decisions(run_id, occurred_at);

CREATE UNIQUE INDEX idx_pomodoro_adaptive_one_control_variant
ON pomodoro_adaptive_experiment_variants(experiment_id)
WHERE is_control = 1;

CREATE INDEX idx_pomodoro_adaptive_outcomes_assignment
ON pomodoro_adaptive_outcomes(assignment_id, measured_at);

CREATE INDEX idx_pomodoro_adaptive_outcomes_decision
ON pomodoro_adaptive_outcomes(decision_id, measured_at);

CREATE INDEX idx_pomodoro_adaptive_planned_blocks_date
ON pomodoro_adaptive_planned_blocks(event_date);

CREATE UNIQUE INDEX idx_pomodoro_adaptive_planned_blocks_unique
ON pomodoro_adaptive_planned_blocks(event_date, original_event_id, planned_start);

CREATE UNIQUE INDEX idx_pomodoro_adaptive_single_active_policy
ON pomodoro_adaptive_policies((1))
WHERE status = 'active';

CREATE INDEX idx_pomodoro_adaptive_state_history_policy
ON pomodoro_adaptive_context_state_history(policy_id, context_key, observed_at);

CREATE INDEX idx_pomodoro_pauses_reason ON pomodoro_pauses(reason, started_at);

CREATE INDEX idx_pomodoro_pauses_segment ON pomodoro_pauses(segment_id, started_at);

CREATE UNIQUE INDEX idx_pomodoro_pauses_single_open_per_segment ON pomodoro_pauses(segment_id) WHERE ended_at IS NULL;

CREATE INDEX idx_pomodoro_run_events_run ON pomodoro_run_events(run_id, occurred_at);

CREATE INDEX idx_pomodoro_runs_event_date ON pomodoro_runs(event_id, event_date);

CREATE INDEX idx_pomodoro_runs_open ON pomodoro_runs(ended_at);

CREATE INDEX idx_pomodoro_runs_original_event ON pomodoro_runs(original_event_id);

CREATE UNIQUE INDEX idx_pomodoro_runs_single_open ON pomodoro_runs((1)) WHERE ended_at IS NULL;

CREATE INDEX idx_pomodoro_segments_event ON pomodoro_segments(event_id, event_date);

CREATE INDEX idx_pomodoro_segments_run ON pomodoro_segments(run_id);

CREATE INDEX idx_pomodoro_segments_run_actual ON pomodoro_segments(run_id, actual_start);

CREATE UNIQUE INDEX idx_pomodoro_segments_single_active ON pomodoro_segments((1)) WHERE status = 'active';

CREATE INDEX idx_quick_note_text_runs_note ON quick_note_text_runs(note_id, sort_order);

CREATE INDEX idx_quick_notes_tag_active
    ON quick_notes(tag_id, pinned DESC, updated_at DESC, id)
    WHERE archived = 0 AND trashed_at IS NULL;

CREATE INDEX idx_quick_notes_active
    ON quick_notes(pinned DESC, updated_at DESC, id)
    WHERE archived = 0 AND trashed_at IS NULL;

CREATE INDEX idx_quick_notes_archive
    ON quick_notes(updated_at DESC, id)
    WHERE archived = 1 AND trashed_at IS NULL;

CREATE INDEX idx_quick_notes_trash
    ON quick_notes(trashed_at DESC, id)
    WHERE trashed_at IS NOT NULL;

CREATE TRIGGER quick_notes_search_insert
AFTER INSERT ON quick_notes
BEGIN
    INSERT INTO quick_notes_search_fts(note_id, title, body)
    VALUES (NEW.id, NEW.title, NEW.body_plain_text);
END;

CREATE TRIGGER quick_notes_search_update
AFTER UPDATE OF title, body_plain_text ON quick_notes
BEGIN
    DELETE FROM quick_notes_search_fts WHERE note_id = OLD.id;
    INSERT INTO quick_notes_search_fts(note_id, title, body)
    VALUES (NEW.id, NEW.title, NEW.body_plain_text);
END;

CREATE TRIGGER quick_notes_search_delete
AFTER DELETE ON quick_notes
BEGIN
    DELETE FROM quick_notes_search_fts WHERE note_id = OLD.id;
END;

CREATE INDEX idx_project_checklist_items_task ON project_checklist_items(task_id, sort_order);

CREATE INDEX idx_project_custom_emojis_sort ON project_custom_emojis(sort_order, name);

CREATE INDEX idx_project_custom_field_option_values_field ON project_custom_field_option_values(field_id, option_id);

CREATE UNIQUE INDEX idx_project_custom_field_options_field_name ON project_custom_field_options(field_id, lower(name));

CREATE INDEX idx_project_custom_field_options_field_sort ON project_custom_field_options(field_id, sort_order, name);

CREATE INDEX idx_project_custom_field_values_date ON project_custom_field_values(field_id, date_value);

CREATE INDEX idx_project_custom_field_values_field ON project_custom_field_values(field_id);

CREATE INDEX idx_project_custom_field_values_number ON project_custom_field_values(field_id, number_value);

CREATE UNIQUE INDEX idx_project_custom_fields_project_name ON project_custom_fields(project_id, lower(name));

CREATE INDEX idx_project_custom_fields_project_sort ON project_custom_fields(project_id, sort_order, name);

CREATE INDEX idx_project_groups_sort ON project_groups(sort_order, name);

CREATE INDEX idx_project_priorities_project_sort ON project_priorities(project_id, sort_order, name);

CREATE INDEX idx_project_sections_project_sort ON project_sections(project_id, sort_order, name);

CREATE INDEX idx_project_statuses_project_sort ON project_statuses(project_id, sort_order, name);

CREATE UNIQUE INDEX idx_project_tags_project_name ON project_tags(project_id, lower(name));

CREATE INDEX idx_project_tags_project_sort ON project_tags(project_id, sort_order, name);

CREATE INDEX idx_project_task_change_events_task ON project_task_change_events(task_id, occurred_at);

CREATE INDEX idx_project_task_dependencies_blocked ON project_task_dependencies(blocked_task_id);

CREATE UNIQUE INDEX idx_project_task_dependencies_unique
    ON project_task_dependencies(blocking_task_id, blocked_task_id, dependency_type);

CREATE INDEX idx_project_task_event_links_event ON project_task_event_links(event_id);

CREATE INDEX idx_project_task_tag_links_tag ON project_task_tag_links(tag_id);

CREATE INDEX idx_project_tasks_archived ON project_tasks(project_id, archived_at);

CREATE INDEX idx_project_tasks_due ON project_tasks(project_id, due_date);

CREATE INDEX idx_project_tasks_parent ON project_tasks(parent_task_id, section_sort_order);

CREATE INDEX idx_project_tasks_project_section ON project_tasks(project_id, section_id, section_sort_order);

CREATE INDEX idx_project_tasks_project_status ON project_tasks(project_id, status_id, status_sort_order);

CREATE INDEX idx_projects_group_sort ON projects(group_id, status, sort_order, name);

CREATE INDEX idx_projects_status ON projects(status);

CREATE INDEX idx_theme_seed_tokens_kind ON theme_seed_tokens(theme_id, kind);

CREATE INDEX idx_theme_tokens_kind ON theme_tokens(theme_id, kind);

CREATE TRIGGER notes_folders_validate_parent_insert
BEFORE INSERT ON notes_folders
WHEN NEW.parent_folder_id IS NOT NULL
BEGIN
    SELECT CASE
        WHEN NEW.parent_folder_id = NEW.id
        THEN RAISE(ABORT, 'notes folder cannot parent itself')
    END;
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM notes_folders AS parent
            WHERE parent.id = NEW.parent_folder_id
              AND parent.project_id = NEW.project_id
        )
        THEN RAISE(ABORT, 'notes folder parent must belong to the same project')
    END;
END;

CREATE TRIGGER notes_folders_validate_parent_update
BEFORE UPDATE OF parent_folder_id, project_id ON notes_folders
WHEN NEW.parent_folder_id IS NOT NULL
BEGIN
    SELECT CASE
        WHEN NEW.parent_folder_id = NEW.id
        THEN RAISE(ABORT, 'notes folder cannot parent itself')
    END;
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM notes_folders AS parent
            WHERE parent.id = NEW.parent_folder_id
              AND parent.project_id = NEW.project_id
        )
        THEN RAISE(ABORT, 'notes folder parent must belong to the same project')
    END;
    SELECT CASE
        WHEN EXISTS (
            WITH RECURSIVE descendants(id) AS (
                SELECT id
                FROM notes_folders
                WHERE parent_folder_id = OLD.id
                UNION
                SELECT child.id
                FROM notes_folders AS child
                JOIN descendants AS parent ON child.parent_folder_id = parent.id
            )
            SELECT 1
            FROM descendants
            WHERE id = NEW.parent_folder_id
        )
        THEN RAISE(ABORT, 'notes folder cannot be moved under its descendant')
    END;
END;

CREATE TRIGGER notes_folders_validate_project_update
BEFORE UPDATE OF project_id ON notes_folders
BEGIN
    SELECT CASE
        WHEN EXISTS (
            SELECT 1
            FROM notes_folders AS child
            WHERE child.parent_folder_id = OLD.id
              AND child.project_id <> NEW.project_id
        )
        THEN RAISE(ABORT, 'notes folder children must belong to the same project')
    END;
    SELECT CASE
        WHEN EXISTS (
            SELECT 1
            FROM notes_pages AS page
            WHERE page.folder_id = OLD.id
              AND (
                  json_type(page.properties, '$.__ganbaru_project_id') <> 'text'
                  OR trim(json_extract(page.properties, '$.__ganbaru_project_id')) <> NEW.project_id
              )
        )
        THEN RAISE(ABORT, 'notes folder pages must belong to the same project')
    END;
END;

CREATE TRIGGER notes_pages_validate_folder_insert
BEFORE INSERT ON notes_pages
WHEN NEW.folder_id IS NOT NULL
BEGIN
    SELECT CASE
        WHEN NEW.parent_type <> 'workspace'
        THEN RAISE(ABORT, 'notes folder pages must have a workspace parent')
    END;
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM notes_folders AS folder
            WHERE folder.id = NEW.folder_id
              AND json_type(NEW.properties, '$.__ganbaru_project_id') = 'text'
              AND folder.project_id = trim(json_extract(
                  NEW.properties,
                  '$.__ganbaru_project_id'
              ))
        )
        THEN RAISE(ABORT, 'notes folder page must belong to the same project')
    END;
END;

CREATE TRIGGER notes_pages_validate_folder_update
BEFORE UPDATE OF folder_id, parent_type, parent_page_id, parent_block_id, parent_data_source_id, properties
ON notes_pages
WHEN NEW.folder_id IS NOT NULL
BEGIN
    SELECT CASE
        WHEN NEW.parent_type <> 'workspace'
        THEN RAISE(ABORT, 'notes folder pages must have a workspace parent')
    END;
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM notes_folders AS folder
            WHERE folder.id = NEW.folder_id
              AND json_type(NEW.properties, '$.__ganbaru_project_id') = 'text'
              AND folder.project_id = trim(json_extract(
                  NEW.properties,
                  '$.__ganbaru_project_id'
              ))
        )
        THEN RAISE(ABORT, 'notes folder page must belong to the same project')
    END;
END;

INSERT INTO calendars (id, name, color, source, visible, read_only)
VALUES ('local', 'Ganbaru AI', '', 'local', 1, 0);

INSERT INTO project_groups (id, name, icon, color, sort_order)
VALUES ('group-routine', 'Routine', 'lucide:repeat', 0, 0);

INSERT INTO projects (
    id, group_id, name, icon, color, sort_order,
    default_pomodoro_mode, default_pomodoro_preset_key
)
VALUES
    ('project-routine-learning', 'group-routine', 'Learning', 'lucide:graduation-cap', 8, 0, 'preset', 'adaptive'),
    ('project-routine-reading', 'group-routine', 'Reading', 'lucide:book-open', 25, 10, 'none', NULL),
    ('project-routine-exercise', 'group-routine', 'Exercise', 'lucide:sport-shoe', 0, 20, 'none', NULL),
    ('project-routine-hygiene', 'group-routine', 'Hygiene', 'lucide:bath', 15, 30, 'none', NULL),
    ('project-routine-eat', 'group-routine', 'Eating', 'lucide:apple', 13, 40, 'none', NULL),
    ('project-routine-commute', 'group-routine', 'Commute', 'lucide:bike', 17, 50, 'none', NULL),
    ('project-routine-social', 'group-routine', 'Social', 'lucide:heart', 21, 60, 'none', NULL),
    ('project-routine-chores', 'group-routine', 'Chores', 'lucide:shopping-cart', 4, 70, 'none', NULL),
    ('project-routine-leisure', 'group-routine', 'Leisure', 'lucide:clapperboard', 31, 80, 'none', NULL),
    ('project-routine-meditate', 'group-routine', 'Meditate', 'lucide:smile', 23, 90, 'none', NULL),
    ('project-routine-health', 'group-routine', 'Health', 'lucide:pill', 3, 100, 'none', NULL),
    ('project-routine-sleep', 'group-routine', 'Sleep', 'lucide:bed', 30, 110, 'none', NULL);

INSERT INTO project_working_folders (
    id, project_id, display_name, kind, managed_relative_path, sort_order
)
SELECT 'working-folder-' || substr(id, length('project-') + 1),
       id,
       name,
       'managed',
       'projects/' || id,
       0
FROM projects;

INSERT INTO project_sections (id, project_id, name, sort_order)
SELECT 'section-' || substr(id, length('project-') + 1) || '-general', id, 'General', 0
FROM projects;

INSERT INTO project_statuses (
    id, project_id, name, category, color, sort_order, terminal
)
SELECT 'status-' || substr(id, length('project-') + 1) || '-' || status.slug,
       id, status.name, status.category, status.color, status.sort_order, status.terminal
FROM projects
CROSS JOIN (
    SELECT 'backlog' AS slug, 'Backlog' AS name, 'not_started' AS category, 30 AS color, 0 AS sort_order, 0 AS terminal
    UNION ALL SELECT 'todo', 'To do', 'not_started', 31, 10, 0
    UNION ALL SELECT 'in-progress', 'In progress', 'active', 19, 20, 0
    UNION ALL SELECT 'in-review', 'In review', 'active', 23, 30, 0
    UNION ALL SELECT 'blocked', 'Blocked', 'blocked', 2, 40, 0
    UNION ALL SELECT 'done', 'Done', 'done', 13, 50, 1
) AS status;

INSERT INTO project_priorities (
    id, project_id, name, color, sort_order
)
SELECT priority.id, project.id, priority.name, priority.color, priority.sort_order
FROM projects AS project
CROSS JOIN (
    SELECT 'urgent' AS id, 'Urgent' AS name, 2 AS color, 0 AS sort_order
    UNION ALL SELECT 'high', 'High', 7, 10
    UNION ALL SELECT 'normal', 'Normal', 19, 20
    UNION ALL SELECT 'low', 'Low', 30, 30
) AS priority;

INSERT INTO notes_local_users (id, display_name)
VALUES ('69bb434b-3734-3467-574f-fb8175def649', 'You');

INSERT INTO notes_page_history_settings (id, retention_days)
VALUES (1, 30);

INSERT INTO notes_history_maintenance_state (id, last_run_at)
VALUES (1, NULL);
