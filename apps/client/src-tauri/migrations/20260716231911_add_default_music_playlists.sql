INSERT OR IGNORE INTO music_playlists
    (id, name, description, shuffle_enabled, repeat_mode, created_at, updated_at, version)
VALUES
    ('playlist-default-start-of-day', 'Start of the day!', '', 1, 'all', 1, 1, 1),
    ('playlist-default-work-focus', 'Work (focus)', '', 1, 'all', 1, 1, 1),
    ('playlist-default-work-ganbare', 'Work (ganbare!)', '', 1, 'all', 1, 1, 1),
    ('playlist-default-break-calm', 'Break (calm)', '', 1, 'all', 1, 1, 1),
    ('playlist-default-break-active', 'Break (active)', '', 1, 'all', 1, 1, 1),
    ('playlist-default-meditate', 'Meditate', '', 1, 'all', 1, 1, 1),
    ('playlist-default-exercise', 'Exercise', '', 1, 'all', 1, 1, 1),
    ('playlist-default-hygiene', 'Hygiene', '', 1, 'all', 1, 1, 1),
    ('playlist-default-chores', 'Chores', '', 1, 'all', 1, 1, 1),
    ('playlist-default-cooking', 'Cooking', '', 1, 'all', 1, 1, 1),
    ('playlist-default-commute', 'Commute', '', 1, 'all', 1, 1, 1);
