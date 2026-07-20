INSERT OR IGNORE INTO music_playlists
    (id, name, icon, shuffle_enabled, repeat_mode, created_at, updated_at, version)
VALUES
    ('playlist-default-start-of-day', 'Start of the day!', 'lucide:sunrise', 1, 'all', 1, 1, 1),
    ('playlist-default-work-focus', 'Work (focus)', 'lucide:laptop', 1, 'all', 1, 1, 1),
    ('playlist-default-work-ganbare', 'Work (ganbare!)', 'lucide:coffee', 1, 'all', 1, 1, 1),
    ('playlist-default-break-calm', 'Break (calm)', 'lucide:tree-pine', 1, 'all', 1, 1, 1),
    ('playlist-default-break-active', 'Break (active)', 'lucide:footprints', 1, 'all', 1, 1, 1),
    ('playlist-default-meditate', 'Meditate', 'lucide:smile', 1, 'all', 1, 1, 1),
    ('playlist-default-exercise', 'Exercise', 'lucide:sport-shoe', 1, 'all', 1, 1, 1),
    ('playlist-default-hygiene', 'Hygiene', 'lucide:bath', 1, 'all', 1, 1, 1),
    ('playlist-default-chores', 'Chores', 'lucide:shopping-cart', 1, 'all', 1, 1, 1),
    ('playlist-default-cooking', 'Cooking', 'lucide:apple', 1, 'all', 1, 1, 1),
    ('playlist-default-commute', 'Commute', 'lucide:bike', 1, 'all', 1, 1, 1);
