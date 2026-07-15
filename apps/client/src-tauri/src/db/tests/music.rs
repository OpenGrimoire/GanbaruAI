use super::helpers::migrated_memory_pool;
use crate::db::run_migrations;
use sqlx::{Row, SqlitePool};

const BASELINE_SCHEMA: &str =
    include_str!("../../../migrations/20260713024120_baseline_schema.sql");
const CANONICAL_MUSIC_SCHEMA: &str =
    include_str!("../../../migrations/20260715042907_add_canonical_music_library.sql");
const LEGACY_MUSIC_MIGRATION: &str =
    include_str!("../../../migrations/20260715043240_migrate_legacy_music_playlists.sql");

async fn pre_legacy_music_migration_pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::raw_sql("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(BASELINE_SCHEMA).execute(&pool).await.unwrap();
    sqlx::raw_sql(CANONICAL_MUSIC_SCHEMA)
        .execute(&pool)
        .await
        .unwrap();
    pool
}

#[test]
fn repeated_migration_startup_preserves_music_data() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        sqlx::query(
            "INSERT INTO music_playlists (id, name, description, created_at, updated_at)
             VALUES ('playlist-1', 'Deep focus', 'User-authored description', 1, 1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        let migration_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();

        run_migrations(&pool).await.unwrap();

        let playlist: (String, Option<String>) =
            sqlx::query_as("SELECT name, description FROM music_playlists WHERE id = 'playlist-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let repeated_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(
            playlist,
            (
                "Deep focus".into(),
                Some("User-authored description".into())
            )
        );
        assert_eq!(repeated_count, migration_count);
    });
}

#[test]
fn canonical_music_schema_keeps_device_paths_out_of_logical_roots() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        for object in [
            "music_library_items",
            "music_local_roots",
            "music_local_locations",
            "music_source_collections",
            "music_source_collection_items",
            "music_item_signals",
            "music_playlist_intended_uses",
            "music_playlist_memberships",
            "music_membership_skip_ranges",
            "music_snoozes",
            "music_listening_statistics",
            "music_recent_selections",
            "music_search_index_state",
            "music_search_fts",
            "idx_music_library_items_review",
            "idx_music_local_locations_root_availability",
            "idx_music_source_collection_items_order",
            "idx_music_playlist_memberships_order",
            "idx_music_snoozes_active_item",
            "idx_music_recent_selections_playlist",
            "music_library_items_review_deferred_until_idx",
        ] {
            let exists: Option<i64> =
                sqlx::query_scalar("SELECT 1 FROM sqlite_schema WHERE name = ?")
                    .bind(object)
                    .fetch_optional(&pool)
                    .await
                    .unwrap();
            assert_eq!(exists, Some(1), "{object} should exist");
        }

        let root_columns = sqlx::query("SELECT name FROM pragma_table_info('music_local_roots')")
            .fetch_all(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect::<Vec<_>>();
        assert!(!root_columns.iter().any(|column| {
            column.contains("path") || column.contains("folder") || column.contains("directory")
        }));
        let item_columns = sqlx::query("SELECT name FROM pragma_table_info('music_library_items')")
            .fetch_all(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect::<Vec<_>>();
        assert!(item_columns
            .iter()
            .any(|column| column == "review_deferred_until"));
    });
}

#[test]
fn canonical_music_schema_enforces_identity_membership_and_snooze_invariants() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        sqlx::query(
            "INSERT INTO music_playlists (id, name, created_at, updated_at)
             VALUES ('playlist-1', 'Focus', 1700000000000, 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO music_library_items
                (id, identity_key, source_kind, youtube_video_id, original_title,
                 availability, review_state, discovered_at, updated_at)
             VALUES
                ('item-1', 'local:item-1', 'local-file', NULL, 'Focus',
                 'available', 'unreviewed', 1700000000000, 1700000000000),
                ('item-2', 'youtube:video-2', 'youtube-video', 'video-2', 'Online',
                 'available', 'reviewed', 1700000000000, 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO music_playlist_memberships
                (id, playlist_id, item_id, position, created_at, updated_at)
             VALUES ('membership-1', 'playlist-1', 'item-1', 0, 1700000000000, 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let duplicate_membership = sqlx::query(
            "INSERT INTO music_playlist_memberships
                (id, playlist_id, item_id, position, created_at, updated_at)
             VALUES ('membership-2', 'playlist-1', 'item-1', 1, 1700000000000, 1700000000000)",
        )
        .execute(&pool)
        .await;
        assert!(duplicate_membership.is_err());

        let local_item_with_youtube_id = sqlx::query(
            "INSERT INTO music_library_items
                (id, identity_key, source_kind, youtube_video_id, discovered_at, updated_at)
             VALUES ('bad-item', 'local:bad', 'local-file', 'video-bad', 1700000000000, 1700000000000)",
        )
        .execute(&pool)
        .await;
        assert!(local_item_with_youtube_id.is_err());

        let playlist_snooze_without_playlist = sqlx::query(
            "INSERT INTO music_snoozes
                (id, item_id, scope, playlist_id, starts_at, created_at)
             VALUES ('bad-snooze', 'item-1', 'playlist', NULL, 1700000000000, 1700000000000)",
        )
        .execute(&pool)
        .await;
        assert!(playlist_snooze_without_playlist.is_err());

        let invalid_weight = sqlx::query(
            "UPDATE music_playlist_memberships SET weight = 'always' WHERE id = 'membership-1'",
        )
        .execute(&pool)
        .await;
        assert!(invalid_weight.is_err());
    });
}

#[test]
fn legacy_music_rows_migrate_without_discarding_conflicts() {
    tauri::async_runtime::block_on(async {
        let pool = pre_legacy_music_migration_pool().await;
        sqlx::query(
            "INSERT INTO music_playlists (id, name, created_at, updated_at)
             VALUES ('legacy-playlist', 'Legacy focus', 1700000000000, 1700000000100)",
        )
        .execute(&pool)
        .await
        .unwrap();
        for statement in [
            "INSERT INTO music_playlist_tracks
                (id, playlist_id, position, source_kind, source_uri, source_identity, title,
                 start_ms, end_ms, volume, rate, created_at, updated_at)
             VALUES
                ('local-primary', 'legacy-playlist', 0, 'local-file', '/old/Music/stale.flac',
                 'local:/old/Music/stale.flac', 'Stale local', 1000, 90000, 0.7, 1.25,
                 1700000000000, 1700000000100),
                ('local-duplicate', 'legacy-playlist', 1, 'local-file', '/old/Music/stale.flac',
                 'local:/old/Music/stale.flac', 'Duplicate edit', 2000, 80000, 0.5, 1.0,
                 1700000000001, 1700000000200),
                ('youtube-direct', 'legacy-playlist', 2, 'youtube-video',
                 'https://www.youtube.com/watch?v=video-direct', 'youtube:video:video-direct',
                 'Direct video', NULL, NULL, NULL, NULL, 1700000000002, 1700000000200),
                ('youtube-playlist-entry', 'legacy-playlist', 3, 'youtube-playlist',
                 'https://www.youtube.com/watch?v=video-list&list=playlist-1',
                 'youtube:playlist:playlist-1:item:3:video:video-list', 'Playlist video',
                 3000, NULL, 0.8, 1.0, 1700000000003, 1700000000200)",
            "INSERT INTO music_track_skip_ranges
                (id, track_id, start_ms, end_ms, sort_order)
             VALUES ('skip-1', 'local-primary', 10000, 12000, 0)",
            "INSERT INTO music_track_break_sources
                (track_id, source_kind, source_uri, source_identity, title, start_ms, end_ms, volume, rate)
             VALUES ('local-primary', 'youtube-video',
                 'https://www.youtube.com/watch?v=break-video', 'youtube:video:break-video',
                 'Break video', 500, 30000, 0.6, 1.0)",
        ] {
            sqlx::raw_sql(statement).execute(&pool).await.unwrap();
        }

        sqlx::raw_sql(LEGACY_MUSIC_MIGRATION)
            .execute(&pool)
            .await
            .unwrap();

        let item_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM music_library_items")
            .fetch_one(&pool)
            .await
            .unwrap();
        let membership_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM music_playlist_memberships")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(item_count, 4);
        assert_eq!(membership_count, 3);

        let membership: (i64, Option<i64>, Option<i64>, Option<f64>, Option<f64>) = sqlx::query_as(
            "SELECT position, start_ms, end_ms, volume, rate
                 FROM music_playlist_memberships
                 WHERE id = 'legacy-membership:local-primary'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            membership,
            (0, Some(1_000), Some(90_000), Some(0.7), Some(1.25))
        );

        let skip: (i64, i64) = sqlx::query_as(
            "SELECT start_ms, end_ms FROM music_membership_skip_ranges
             WHERE membership_id = 'legacy-membership:local-primary'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(skip, (10_000, 12_000));

        let break_identity: String = sqlx::query_scalar(
            "SELECT item.identity_key
             FROM music_membership_break_items AS break_item
             JOIN music_library_items AS item ON item.id = break_item.item_id
             WHERE break_item.membership_id = 'legacy-membership:local-primary'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(break_identity, "youtube:video:break-video");

        let duplicate_issue: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM music_library_repair_issues
             WHERE issue_kind = 'duplicate-playlist-membership'
               AND legacy_track_id = 'local-duplicate'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(duplicate_issue, 1);

        let retained_legacy_uri: String = sqlx::query_scalar(
            "SELECT source_uri FROM music_playlist_tracks WHERE id = 'youtube-playlist-entry'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            retained_legacy_uri,
            "https://www.youtube.com/watch?v=video-list&list=playlist-1",
        );
    });
}

#[test]
fn failed_legacy_music_conversion_rolls_back_when_migration_is_transactional() {
    tauri::async_runtime::block_on(async {
        let pool = pre_legacy_music_migration_pool().await;
        sqlx::query(
            "INSERT INTO music_playlists (id, name, created_at, updated_at)
             VALUES ('legacy-playlist', 'Legacy focus', 1700000000000, 1700000000100)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::raw_sql("PRAGMA ignore_check_constraints=ON")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO music_playlist_tracks
                (id, playlist_id, position, source_kind, source_uri, source_identity,
                 created_at, updated_at)
             VALUES ('invalid-position', 'legacy-playlist', -1, 'local-file',
                 '/old/Music/invalid.flac', 'local:/old/Music/invalid.flac',
                 1700000000000, 1700000000100)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::raw_sql("PRAGMA ignore_check_constraints=OFF")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::raw_sql("BEGIN").execute(&pool).await.unwrap();
        let result = sqlx::raw_sql(LEGACY_MUSIC_MIGRATION).execute(&pool).await;
        assert!(result.is_err());
        sqlx::raw_sql("ROLLBACK").execute(&pool).await.unwrap();

        let canonical_items: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM music_library_items")
            .fetch_one(&pool)
            .await
            .unwrap();
        let repair_table: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM sqlite_schema WHERE name = 'music_library_repair_issues'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert_eq!(canonical_items, 0);
        assert_eq!(repair_table, None);
    });
}

#[test]
fn empty_playlist_tables_do_not_block_playback_state_persistence() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        let playlists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM music_playlists")
            .fetch_one(&pool)
            .await
            .unwrap();
        let tracks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM music_playlist_tracks")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(playlists, 0);
        assert_eq!(tracks, 0);

        sqlx::query(
            "INSERT INTO music_playback_states
                (source_identity, source_kind, position_ms, duration_ms, status, updated_at)
             VALUES ('local:/music/focus.flac', 'local-file', 42000, 180000, 'paused', 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let restored: (i64, Option<i64>, String) = sqlx::query_as(
            "SELECT position_ms, duration_ms, status
             FROM music_playback_states
             WHERE source_identity = 'local:/music/focus.flac'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(restored, (42_000, Some(180_000), "paused".to_string()));

        let playlists_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM music_playlists")
            .fetch_one(&pool)
            .await
            .unwrap();
        let tracks_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM music_playlist_tracks")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(playlists_after, 0);
        assert_eq!(tracks_after, 0);
    });
}
