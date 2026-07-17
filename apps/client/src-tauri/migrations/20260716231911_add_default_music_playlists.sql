INSERT OR IGNORE INTO music_playlists
    (id, name, description, shuffle_enabled, repeat_mode, created_at, updated_at, version)
VALUES
    ('playlist-default-reading', 'Reading', '', 1, 'all', 1, 1, 1),
    ('playlist-default-exercise', 'Exercise', '', 1, 'all', 1, 1, 1),
    ('playlist-default-hygiene', 'Hygiene', '', 1, 'all', 1, 1, 1),
    ('playlist-default-commute', 'Commute', '', 1, 'all', 1, 1, 1),
    ('playlist-default-chores', 'Chores', '', 1, 'all', 1, 1, 1),
    ('playlist-default-meditate', 'Meditate', '', 1, 'all', 1, 1, 1),
    ('playlist-default-start-of-day', 'Start of the day!', '', 1, 'all', 1, 1, 1),
    ('playlist-default-working', 'Working', '', 1, 'all', 1, 1, 1),
    ('playlist-default-short-breaks', 'Short breaks', '', 1, 'all', 1, 1, 1),
    ('playlist-default-long-breaks', 'Long breaks', '', 1, 'all', 1, 1, 1);
