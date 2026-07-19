ALTER TABLE music_playlists
ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0);

WITH ordered_playlists AS (
    SELECT
        id,
        ROW_NUMBER() OVER (
            ORDER BY
                CASE id
                    WHEN 'playlist-default-start-of-day' THEN 0
                    WHEN 'playlist-default-work-focus' THEN 1
                    WHEN 'playlist-default-work-ganbare' THEN 2
                    WHEN 'playlist-default-break-calm' THEN 3
                    WHEN 'playlist-default-break-active' THEN 4
                    WHEN 'playlist-default-meditate' THEN 5
                    WHEN 'playlist-default-exercise' THEN 6
                    WHEN 'playlist-default-hygiene' THEN 7
                    WHEN 'playlist-default-chores' THEN 8
                    WHEN 'playlist-default-cooking' THEN 9
                    WHEN 'playlist-default-commute' THEN 10
                    ELSE 1000
                END,
                created_at,
                name COLLATE NOCASE,
                id
        ) - 1 AS position
    FROM music_playlists
)
UPDATE music_playlists
SET sort_order = (
    SELECT position
    FROM ordered_playlists
    WHERE ordered_playlists.id = music_playlists.id
);

CREATE INDEX idx_music_playlists_sort
ON music_playlists(sort_order, name COLLATE NOCASE, id);
