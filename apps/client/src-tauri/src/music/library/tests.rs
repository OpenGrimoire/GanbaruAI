use super::*;
use sqlx::{Row, SqlitePool};

async fn pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::raw_sql("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .unwrap();
    crate::db::run_migrations(&pool).await.unwrap();
    pool
}

async fn seed_item(pool: &SqlitePool, id: &str, identity: &str) {
    sqlx::query(
        "INSERT INTO music_library_items
            (id, identity_key, source_kind, original_title, availability, review_state,
             discovered_at, updated_at)
         VALUES (?, ?, 'local-file', ?, 'available', 'unreviewed',
             1700000000000, 1700000000000)",
    )
    .bind(id)
    .bind(identity)
    .bind(id)
    .execute(pool)
    .await
    .unwrap();
}

fn playlist(id: &str) -> MusicPlaylistCreate {
    MusicPlaylistCreate {
        id: id.to_string(),
        name: "Focus".to_string(),
        description: "Quiet soundtrack".to_string(),
        shuffle_enabled: true,
        repeat_mode: MusicRepeatMode::All,
        intended_uses: vec![MusicIntendedUse::Focus],
        created_at: 1_700_000_000_000,
    }
}

fn library_window() -> MusicItemWindowRequest {
    MusicItemWindowRequest {
        destination: MusicListDestination::Library,
        playlist_id: None,
        search: String::new(),
        source_kind: None,
        availability: None,
        review_state: None,
        source_collection_id: None,
        snoozed: None,
        sort: MusicItemSort::Title,
        direction: MusicSortDirection::Ascending,
        group_by: MusicGroupBy::None,
        now_ms: 1_700_000_100_000,
        offset: 0,
        limit: 50,
    }
}

fn membership(index: usize) -> MusicMembershipWrite {
    MusicMembershipWrite {
        id: format!("membership-{index}"),
        playlist_id: "playlist-1".to_string(),
        item_id: format!("item-{index}"),
        position: index as i64,
        weight: MusicWeight::Normal,
        enabled: true,
        focus_fit: MusicFocusFit::Unknown,
        start_ms: None,
        end_ms: None,
        volume: None,
        rate: None,
        expected_version: None,
        updated_at: 1_700_000_000_000,
    }
}

#[test]
fn typed_enums_reject_unknown_external_values() {
    let error = serde_json::from_str::<MusicWeight>(r#""always""#).unwrap_err();
    assert!(error.to_string().contains("unknown variant"));
    assert!(MusicReviewState::try_from("maybe-reviewed").is_err());
}

#[test]
fn library_items_require_source_specific_identity() {
    let item = MusicLibraryItemWrite {
        id: "item-1".to_string(),
        identity_key: "youtube:item-1".to_string(),
        source_kind: MusicLibrarySourceKind::YouTubeVideo,
        media_kind: MusicMediaKind::Video,
        youtube_video_id: None,
        original_title: "Video".to_string(),
        original_artist: String::new(),
        original_album: String::new(),
        original_track_number: None,
        original_artwork_identity: None,
        youtube_resolution_state: None,
        duration_ms: None,
        availability: MusicItemAvailability::Unknown,
        discovered_at: 1_700_000_000_000,
        updated_at: 1_700_000_000_000,
    };

    let error = validate_library_item_write(&item).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("youtubeVideoId"));
}

#[test]
fn item_and_location_upserts_preserve_canonical_identity_and_refresh_search() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        sqlx::query(
            "INSERT INTO music_local_roots (id, name, created_at, updated_at)
             VALUES ('root-1', 'Soundtracks', 1700000000000, 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();
        let item = MusicLibraryItemWrite {
            id: "item-1".to_string(),
            identity_key: "local:fingerprint-1".to_string(),
            source_kind: MusicLibrarySourceKind::LocalFile,
            media_kind: MusicMediaKind::Audio,
            youtube_video_id: None,
            original_title: "First title".to_string(),
            original_artist: "Composer".to_string(),
            original_album: "Album".to_string(),
            original_track_number: Some(3),
            original_artwork_identity: None,
            youtube_resolution_state: None,
            duration_ms: Some(120_000),
            availability: MusicItemAvailability::Available,
            discovered_at: 1_700_000_000_000,
            updated_at: 1_700_000_000_000,
        };
        let created = writes::upsert_library_item(&pool, item.clone())
            .await
            .unwrap();
        assert_eq!(created.version, 1);

        let mut updated = item.clone();
        updated.original_title = "Updated title".to_string();
        updated.updated_at += 1;
        let receipt = writes::upsert_library_item(&pool, updated).await.unwrap();
        assert_eq!(receipt.version, 2);
        let stored_review: String =
            sqlx::query_scalar("SELECT review_state FROM music_library_items WHERE id = 'item-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stored_review, "unreviewed");

        writes::upsert_local_location(
            &pool,
            MusicLocalLocationWrite {
                id: "location-1".to_string(),
                item_id: "item-1".to_string(),
                root_id: "root-1".to_string(),
                relative_path: "Album/Updated title.flac".to_string(),
                file_size_bytes: Some(2_000_000),
                modified_at_ms: Some(1_700_000_000_000),
                lightweight_fingerprint: Some("light-1".to_string()),
                strong_fingerprint: None,
                availability: MusicLocationAvailability::Available,
                last_seen_generation: Some(1),
                first_seen_at: 1_700_000_000_000,
                updated_at: 1_700_000_000_001,
            },
        )
        .await
        .unwrap();
        let search_title: String =
            sqlx::query_scalar("SELECT title FROM music_search_fts WHERE item_id = 'item-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(search_title, "Updated title");

        let mut conflicting = item;
        conflicting.identity_key = "local:different".to_string();
        let error = writes::upsert_library_item(&pool, conflicting)
            .await
            .unwrap_err();
        assert_eq!(error.code, MusicLibraryErrorCode::Conflict);
    });
}

#[test]
fn local_locations_reject_paths_that_escape_their_root() {
    let location = MusicLocalLocationWrite {
        id: "location-1".to_string(),
        item_id: "item-1".to_string(),
        root_id: "root-1".to_string(),
        relative_path: "../outside.mp3".to_string(),
        file_size_bytes: Some(100),
        modified_at_ms: None,
        lightweight_fingerprint: None,
        strong_fingerprint: None,
        availability: MusicLocationAvailability::Available,
        last_seen_generation: Some(1),
        first_seen_at: 1_700_000_000_000,
        updated_at: 1_700_000_000_000,
    };

    let error = validate_local_location_write(&location).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("relativePath"));
}

#[test]
fn memberships_reject_invalid_ranges_and_duplicate_bulk_identity() {
    let mut invalid = membership(0);
    invalid.start_ms = Some(5_000);
    invalid.end_ms = Some(4_000);
    assert_eq!(
        validate_membership_write(&invalid)
            .unwrap_err()
            .field
            .as_deref(),
        Some("endMs"),
    );

    let first = membership(1);
    let mut duplicate = membership(2);
    duplicate.item_id = first.item_id.clone();
    let error = validate_bulk_membership_write(&MusicBulkMembershipWrite {
        memberships: vec![first, duplicate],
    })
    .unwrap_err();
    assert!(error.message.contains("duplicate item"));
}

#[test]
fn bulk_memberships_have_an_explicit_request_bound() {
    let request = MusicBulkMembershipWrite {
        memberships: (0..=MAX_BULK_MEMBERSHIPS).map(membership).collect(),
    };
    let error = validate_bulk_membership_write(&request).unwrap_err();
    assert!(error.message.contains("500 item limit"));
}

#[test]
fn snooze_scope_and_timestamp_are_validated_together() {
    let snooze = MusicSnoozeWrite {
        id: "snooze-1".to_string(),
        item_id: "item-1".to_string(),
        scope: MusicSnoozeScope::Playlist,
        playlist_id: None,
        starts_at: 1_700_000_000_000,
        ends_at: Some(1_699_999_999_999),
        reason: String::new(),
        created_at: 1_700_000_000_000,
    };
    let error = validate_snooze_write(&snooze).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("endsAt"));
}

#[test]
fn row_mapping_rejects_unknown_persisted_enums_with_field_context() {
    let row = MusicMembershipRow {
        id: "membership-1".to_string(),
        playlist_id: "playlist-1".to_string(),
        item_id: "item-1".to_string(),
        position: 0,
        weight: "sometimes-ish".to_string(),
        enabled: 1,
        focus_fit: "unknown".to_string(),
        start_ms: None,
        end_ms: None,
        volume: None,
        rate: None,
        created_at: 1_700_000_000_000,
        updated_at: 1_700_000_000_000,
        version: 1,
    };

    let error = MusicPlaylistMembership::try_from(row).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("weight"));
    assert!(error.message.contains("sometimes-ish"));
}

#[test]
fn playlist_create_update_and_stale_detection_are_transactional() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        let created = super::writes::create_playlist(&pool, playlist("playlist-1"))
            .await
            .unwrap();
        assert_eq!(created.version, 1);

        let updated = super::writes::update_playlist(
            &pool,
            MusicPlaylistUpdate {
                id: "playlist-1".to_string(),
                name: "Deep focus".to_string(),
                description: "No vocals".to_string(),
                shuffle_enabled: false,
                repeat_mode: MusicRepeatMode::Off,
                intended_uses: vec![MusicIntendedUse::Focus, MusicIntendedUse::Reading],
                expected_version: 1,
                updated_at: 1_700_000_000_100,
            },
        )
        .await
        .unwrap();
        assert_eq!(updated.version, 2);

        let stale = super::writes::update_playlist(
            &pool,
            MusicPlaylistUpdate {
                id: "playlist-1".to_string(),
                name: "Stale edit".to_string(),
                description: String::new(),
                shuffle_enabled: false,
                repeat_mode: MusicRepeatMode::All,
                intended_uses: Vec::new(),
                expected_version: 1,
                updated_at: 1_700_000_000_200,
            },
        )
        .await
        .unwrap_err();
        assert_eq!(stale.code, MusicLibraryErrorCode::StaleWrite);

        let uses: Vec<String> = sqlx::query_scalar(
            "SELECT intended_use FROM music_playlist_intended_uses
             WHERE playlist_id = 'playlist-1' ORDER BY intended_use",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(uses, vec!["focus", "reading"]);
    });
}

#[test]
fn bulk_membership_failure_rolls_back_earlier_rows() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        super::writes::create_playlist(&pool, playlist("playlist-1"))
            .await
            .unwrap();
        seed_item(&pool, "item-1", "local:item-1").await;
        let first = membership(1);
        let mut missing = membership(2);
        missing.item_id = "missing-item".to_string();

        let error = super::writes::upsert_memberships(
            &pool,
            MusicBulkMembershipWrite {
                memberships: vec![first, missing],
            },
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, MusicLibraryErrorCode::Database);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM music_playlist_memberships")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    });
}

#[test]
fn duplicate_playlist_preserves_membership_details_and_ranges() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        super::writes::create_playlist(&pool, playlist("playlist-1"))
            .await
            .unwrap();
        seed_item(&pool, "item-1", "local:item-1").await;
        let mut source_membership = membership(1);
        source_membership.item_id = "item-1".to_string();
        source_membership.weight = MusicWeight::MoreOften;
        source_membership.start_ms = Some(1_000);
        source_membership.end_ms = Some(90_000);
        let receipt = super::writes::upsert_memberships(
            &pool,
            MusicBulkMembershipWrite {
                memberships: vec![source_membership],
            },
        )
        .await
        .unwrap()
        .remove(0);
        sqlx::query(
            "INSERT INTO music_membership_skip_ranges
                (id, membership_id, start_ms, end_ms, sort_order)
             VALUES ('skip-1', ?, 10000, 12000, 0)",
        )
        .bind(&receipt.id)
        .execute(&pool)
        .await
        .unwrap();

        super::writes::duplicate_playlist(
            &pool,
            MusicPlaylistDuplicate {
                source_playlist_id: "playlist-1".to_string(),
                new_playlist_id: "playlist-2".to_string(),
                name: "Focus copy".to_string(),
                created_at: 1_700_000_001_000,
            },
        )
        .await
        .unwrap();

        let copied: (String, Option<i64>, Option<i64>) = sqlx::query_as(
            "SELECT weight, start_ms, end_ms FROM music_playlist_memberships
             WHERE playlist_id = 'playlist-2'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            copied,
            ("more-often".to_string(), Some(1_000), Some(90_000))
        );
        let skip_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM music_membership_skip_ranges AS skip
             JOIN music_playlist_memberships AS membership ON membership.id = skip.membership_id
             WHERE membership.playlist_id = 'playlist-2'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(skip_count, 1);
    });
}

#[test]
fn deletion_requires_current_impact_and_clears_assignments_atomically() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        super::writes::create_playlist(&pool, playlist("playlist-1"))
            .await
            .unwrap();
        seed_item(&pool, "item-1", "local:item-1").await;
        let mut linked = membership(1);
        linked.item_id = "item-1".to_string();
        super::writes::upsert_memberships(
            &pool,
            MusicBulkMembershipWrite {
                memberships: vec![linked],
            },
        )
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO calendar_events (id, title, start_time, end_time, playlist_id)
             VALUES ('event-1', 'Focus', '2026-07-15T09:00:00Z',
                 '2026-07-15T10:00:00Z', 'playlist-1')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let impact = super::writes::playlist_delete_impact(&pool, "playlist-1")
            .await
            .unwrap();
        assert_eq!(impact.membership_count, 1);
        assert_eq!(impact.calendar_assignment_count, 1);
        let mut stale_impact = impact.clone();
        stale_impact.calendar_assignment_count = 0;
        let conflict = super::writes::delete_playlist(
            &pool,
            MusicPlaylistDelete {
                playlist_id: "playlist-1".to_string(),
                expected_version: 1,
                expected_impact: stale_impact,
            },
        )
        .await
        .unwrap_err();
        assert_eq!(conflict.code, MusicLibraryErrorCode::Conflict);

        super::writes::delete_playlist(
            &pool,
            MusicPlaylistDelete {
                playlist_id: "playlist-1".to_string(),
                expected_version: 1,
                expected_impact: impact,
            },
        )
        .await
        .unwrap();
        let assignment: Option<String> =
            sqlx::query_scalar("SELECT playlist_id FROM calendar_events WHERE id = 'event-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(assignment, None);
        let playlist_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM music_playlists")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(playlist_count, 0);
    });
}

#[test]
fn review_snooze_and_statistics_commands_preserve_independent_scopes() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        seed_item(&pool, "item-1", "local:item-1").await;
        super::writes::set_review_state(
            &pool,
            MusicReviewWrite {
                item_id: "item-1".to_string(),
                review_state: MusicReviewState::Ignored,
                expected_version: 1,
                updated_at: 1_700_000_000_100,
            },
        )
        .await
        .unwrap();
        super::writes::upsert_snooze(
            &pool,
            MusicSnoozeWrite {
                id: "snooze-1".to_string(),
                item_id: "item-1".to_string(),
                scope: MusicSnoozeScope::AllPlaylists,
                playlist_id: None,
                starts_at: 1_700_000_000_100,
                ends_at: None,
                reason: "Rest".to_string(),
                created_at: 1_700_000_000_100,
            },
        )
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO music_listening_statistics
                (item_id, play_count, completion_count, skip_count, updated_at)
             VALUES ('item-1', 3, 2, 1, 1700000000200)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO music_recent_selections (item_id, selection_kind, selected_at)
             VALUES ('item-1', 'automatic', 1700000000200)",
        )
        .execute(&pool)
        .await
        .unwrap();
        super::writes::reset_statistics(
            &pool,
            MusicStatisticsReset {
                item_ids: vec!["item-1".to_string()],
                reset_recent_selections: false,
            },
        )
        .await
        .unwrap();

        let recent_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM music_recent_selections")
            .fetch_one(&pool)
            .await
            .unwrap();
        let snooze_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM music_snoozes")
            .fetch_one(&pool)
            .await
            .unwrap();
        let review_state: String =
            sqlx::query_scalar("SELECT review_state FROM music_library_items WHERE id = 'item-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(recent_count, 1);
        assert_eq!(snooze_count, 1);
        assert_eq!(review_state, "ignored");
    });
}

#[test]
fn item_windows_are_bounded_stable_filterable_and_grouped() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        let mut transaction = pool.begin().await.unwrap();
        for index in 0..600 {
            sqlx::query(
                "INSERT INTO music_library_items
                    (id, identity_key, source_kind, media_kind, youtube_video_id,
                     original_title, original_artist,
                     original_album, availability, review_state, discovered_at, updated_at)
                 VALUES (?, ?, ?, 'audio', ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(format!("item-{index:04}"))
            .bind(format!("identity-{index:04}"))
            .bind(if index % 3 == 0 {
                "youtube-video"
            } else {
                "local-file"
            })
            .bind(if index % 3 == 0 {
                Some(format!("video-{index:04}"))
            } else {
                None
            })
            .bind(format!("Track {index:04}"))
            .bind(if index % 2 == 0 {
                "Alpha Composer"
            } else {
                "Beta Composer"
            })
            .bind(format!("Album {:02}", index % 12))
            .bind(if index % 17 == 0 {
                "missing"
            } else {
                "available"
            })
            .bind(if index % 5 == 0 {
                "unreviewed"
            } else {
                "reviewed"
            })
            .bind(1_700_000_000_000_i64 + index)
            .bind(1_700_000_000_000_i64 + index)
            .execute(&mut *transaction)
            .await
            .unwrap();
        }
        transaction.commit().await.unwrap();

        let first = super::queries::item_window(&pool, library_window())
            .await
            .unwrap();
        assert_eq!(first.total_count, 600);
        assert_eq!(first.items.len(), 50);
        assert_eq!(first.items.first().unwrap().title, "Track 0000");
        assert_eq!(first.items.last().unwrap().title, "Track 0049");

        let mut filtered = library_window();
        filtered.search = "Alpha".to_string();
        filtered.availability = Some(MusicItemAvailability::Available);
        filtered.group_by = MusicGroupBy::SourceKind;
        filtered.limit = 25;
        let result = super::queries::item_window(&pool, filtered).await.unwrap();
        assert_eq!(result.items.len(), 25);
        assert!(result.total_count < 300);
        assert_eq!(
            result.groups.iter().map(|group| group.count).sum::<i64>(),
            result.total_count,
        );
        assert!(result
            .items
            .iter()
            .all(|item| item.artist == "Alpha Composer"
                && item.availability == MusicItemAvailability::Available));
    });
}

#[test]
fn playlist_window_uses_manual_order_and_returns_membership_state() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        super::writes::create_playlist(&pool, playlist("playlist-1"))
            .await
            .unwrap();
        for index in 0..3 {
            seed_item(
                &pool,
                &format!("item-{index}"),
                &format!("local:item-{index}"),
            )
            .await;
            let mut entry = membership(index);
            entry.item_id = format!("item-{index}");
            entry.position = 2 - index as i64;
            super::writes::upsert_memberships(
                &pool,
                MusicBulkMembershipWrite {
                    memberships: vec![entry],
                },
            )
            .await
            .unwrap();
        }
        let mut request = library_window();
        request.destination = MusicListDestination::Playlist;
        request.playlist_id = Some("playlist-1".to_string());
        request.sort = MusicItemSort::ManualPosition;
        let result = super::queries::item_window(&pool, request).await.unwrap();

        assert_eq!(
            result
                .items
                .iter()
                .map(|item| item.membership_position)
                .collect::<Vec<_>>(),
            vec![Some(0), Some(1), Some(2)],
        );
        assert!(result.items.iter().all(|item| item.membership_id.is_some()));
    });
}

#[test]
fn summaries_issues_and_inspector_return_composed_data_without_row_queries() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        super::writes::create_playlist(&pool, playlist("playlist-1"))
            .await
            .unwrap();
        seed_item(&pool, "item-1", "local:item-1").await;
        let linked = membership(1);
        super::writes::upsert_memberships(
            &pool,
            MusicBulkMembershipWrite {
                memberships: vec![linked],
            },
        )
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO music_local_roots (id, name, created_at, updated_at)
             VALUES ('root-1', 'Soundtracks', 1700000000000, 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO music_local_locations
                (id, item_id, root_id, relative_path, availability, first_seen_at, updated_at)
             VALUES ('location-1', 'item-1', 'root-1', 'Album/track.flac',
                 'available', 1700000000000, 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO music_source_collections
                (id, kind, identity_key, name, local_root_id, created_at, updated_at)
             VALUES ('source-1', 'local-root', 'root:root-1', 'Soundtracks', 'root-1',
                 1700000000000, 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO music_source_collection_items
                (collection_id, item_id, first_discovered_at)
             VALUES ('source-1', 'item-1', 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO music_item_signals (item_id, signal, created_at)
             VALUES ('item-1', 'calm', 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO music_library_repair_issues
                (id, issue_kind, item_id, message, created_at)
             VALUES ('issue-1', 'legacy-local-root-required', 'item-1',
                 'Choose a root.', 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let playlists = super::queries::playlist_summaries(&pool, 1_700_000_100_000, 0, 20)
            .await
            .unwrap();
        let sources = super::queries::source_summaries(&pool, 1_700_000_000_000, 0, 20)
            .await
            .unwrap();
        let roots = super::queries::local_roots(&pool, 0, 20).await.unwrap();
        let collections = super::queries::source_collections(&pool, 0, 20)
            .await
            .unwrap();
        let playlist_detail = super::queries::playlist_detail(&pool, "playlist-1")
            .await
            .unwrap();
        let issues = super::queries::issues(&pool, 0, 20).await.unwrap();
        let detail = super::queries::inspector_detail(&pool, "item-1")
            .await
            .unwrap();

        assert_eq!(playlists[0].total_count, 1);
        assert_eq!(sources[0].item_count, 1);
        assert_eq!(sources[0].open_issue_count, 1);
        assert_eq!(roots[0].name, "Soundtracks");
        assert_eq!(collections[0].kind, MusicCollectionKind::LocalRoot);
        assert_eq!(playlist_detail.intended_uses, vec![MusicIntendedUse::Focus]);
        assert_eq!(issues[0].id, "issue-1");
        assert_eq!(detail.locations.len(), 1);
        assert_eq!(detail.memberships.len(), 1);
        assert_eq!(detail.signals, vec![MusicItemSignal::Calm]);
        assert_eq!(detail.source_collection_ids, vec!["source-1"]);
    });
}

#[test]
fn review_and_membership_queries_use_purpose_built_indexes() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        let review_plan = sqlx::query(
            "EXPLAIN QUERY PLAN
             SELECT id FROM music_library_items
             WHERE review_state = 'unreviewed'
             ORDER BY discovered_at, id LIMIT 50",
        )
        .fetch_all(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get::<String, _>("detail"))
        .collect::<Vec<_>>()
        .join("\n");
        let membership_plan = sqlx::query(
            "EXPLAIN QUERY PLAN
             SELECT id FROM music_playlist_memberships
             WHERE playlist_id = 'playlist-1'
             ORDER BY position, id LIMIT 50",
        )
        .fetch_all(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get::<String, _>("detail"))
        .collect::<Vec<_>>()
        .join("\n");

        assert!(review_plan.contains("idx_music_library_items_review"));
        assert!(membership_plan.contains("idx_music_playlist_memberships_order"));
    });
}

#[test]
fn search_rebuild_repairs_stale_rows_and_incremental_membership_metadata() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        sqlx::query(
            "INSERT INTO music_library_items
                (id, identity_key, source_kind, media_kind, original_title, original_artist,
                 original_album, availability, review_state, discovered_at, updated_at)
             VALUES ('item-1', 'local:item-1', 'local-file', 'audio',
                 'Café de la pluie 雨', 'Artista', 'Lectura tranquila',
                 'available', 'unreviewed', 1700000000000, 1700000000000)",
        )
        .execute(&pool)
        .await
        .unwrap();
        let rebuilt = super::search::rebuild(&pool, 1_700_000_100_000)
            .await
            .unwrap();
        assert_eq!(rebuilt.indexed_item_count, 1);

        let mut by_diacritic = library_window();
        by_diacritic.search = "cafe".to_string();
        assert_eq!(
            super::queries::item_window(&pool, by_diacritic)
                .await
                .unwrap()
                .total_count,
            1,
        );
        let mut by_cjk = library_window();
        by_cjk.search = "雨".to_string();
        assert_eq!(
            super::queries::item_window(&pool, by_cjk)
                .await
                .unwrap()
                .total_count,
            1,
        );

        sqlx::query("DELETE FROM music_search_fts")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE music_search_index_state SET fingerprint = 'stale' WHERE singleton = 1",
        )
        .execute(&pool)
        .await
        .unwrap();
        let mut repaired = library_window();
        repaired.search = "tranquila".to_string();
        assert_eq!(
            super::queries::item_window(&pool, repaired)
                .await
                .unwrap()
                .total_count,
            1,
        );

        super::writes::create_playlist(
            &pool,
            MusicPlaylistCreate {
                name: "Morning flow".to_string(),
                ..playlist("playlist-1")
            },
        )
        .await
        .unwrap();
        let linked = membership(1);
        super::writes::upsert_memberships(
            &pool,
            MusicBulkMembershipWrite {
                memberships: vec![linked],
            },
        )
        .await
        .unwrap();
        let mut by_playlist = library_window();
        by_playlist.search = "Morning".to_string();
        assert_eq!(
            super::queries::item_window(&pool, by_playlist)
                .await
                .unwrap()
                .total_count,
            1,
        );

        super::writes::update_playlist(
            &pool,
            MusicPlaylistUpdate {
                id: "playlist-1".to_string(),
                name: "Dawn routine".to_string(),
                description: String::new(),
                shuffle_enabled: true,
                repeat_mode: MusicRepeatMode::All,
                intended_uses: vec![MusicIntendedUse::General],
                expected_version: 1,
                updated_at: 1_700_000_200_000,
            },
        )
        .await
        .unwrap();
        let mut by_renamed_playlist = library_window();
        by_renamed_playlist.search = "Dawn".to_string();
        assert_eq!(
            super::queries::item_window(&pool, by_renamed_playlist)
                .await
                .unwrap()
                .total_count,
            1,
        );
    });
}
