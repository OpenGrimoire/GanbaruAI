use super::*;
use sqlx::SqlitePool;

pub(super) async fn pool() -> SqlitePool {
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

pub(super) async fn seed_item(pool: &SqlitePool, id: &str, identity: &str) {
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

pub(super) fn playlist(id: &str) -> MusicPlaylistCreate {
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

pub(super) fn library_window() -> MusicItemWindowRequest {
    MusicItemWindowRequest {
        destination: MusicListDestination::Library,
        playlist_id: None,
        search: String::new(),
        source_kind: None,
        availability: None,
        review_state: None,
        source_collection_id: None,
        membership_playlist_id: None,
        snoozed: None,
        sort: MusicItemSort::Title,
        direction: MusicSortDirection::Ascending,
        group_by: MusicGroupBy::None,
        now_ms: 1_700_000_100_000,
        offset: 0,
        limit: 50,
    }
}

pub(super) fn membership(index: usize) -> MusicMembershipWrite {
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
fn local_root_creation_is_atomic_and_rejects_duplicate_identity() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        let request = MusicLocalRootCreate {
            root_id: "root-soundtracks".to_string(),
            collection_id: "source-soundtracks".to_string(),
            identity_key: "local-root:root-soundtracks".to_string(),
            name: "Soundtracks".to_string(),
            created_at: 1_700_000_000_000,
        };
        let receipt = writes::create_local_root(&pool, request.clone())
            .await
            .unwrap();
        assert_eq!(receipt.id, "source-soundtracks");
        assert_eq!(queries::local_roots(&pool, 0, 10).await.unwrap().len(), 1);
        assert_eq!(
            queries::source_collections(&pool, 0, 10)
                .await
                .unwrap()
                .len(),
            1
        );

        let duplicate = MusicLocalRootCreate {
            root_id: "root-other".to_string(),
            collection_id: "source-other".to_string(),
            ..request
        };
        assert!(writes::create_local_root(&pool, duplicate).await.is_err());
        assert_eq!(queries::local_roots(&pool, 0, 10).await.unwrap().len(), 1);
    });
}

#[test]
fn item_location_repair_requires_weak_match_confirmation_and_can_be_undone() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        seed_item(&pool, "repair-item", "local:repair-item").await;
        let folder = std::env::temp_dir().join(format!(
            "ganbaru-music-repair-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&folder).unwrap();
        let file = folder.join("replacement.wav");
        std::fs::write(&file, b"RIFF\x04\x00\x00\x00WAVE").unwrap();

        let preview = item_repair::preview(&pool, "repair-item", &file.to_string_lossy())
            .await
            .unwrap();
        assert_eq!(preview.match_strength, MusicRepairMatchStrength::Weak);
        let mut request = MusicItemRepairApply {
            item_id: "repair-item".to_string(),
            root_id: "repair-root".to_string(),
            location_id: "repair-location".to_string(),
            root_name: "Recovered".to_string(),
            folder_path: preview.folder_path.clone(),
            relative_path: preview.relative_path.clone(),
            expected_strong_fingerprint: preview.strong_fingerprint.clone(),
            accept_weak_mismatch: false,
            applied_at: 1_700_000_000_000,
        };
        assert!(item_repair::apply(&pool, request.clone()).await.is_err());
        request.accept_weak_mismatch = true;
        item_repair::apply(&pool, request).await.unwrap();
        let available: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM music_local_locations
             WHERE item_id = 'repair-item' AND availability = 'available'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(available, 1);

        item_repair::undo(&pool, "repair-location", "repair-root")
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM music_local_locations WHERE id = 'repair-location'",
            )
            .fetch_one(&pool)
            .await
            .unwrap(),
            0
        );
        std::fs::remove_dir_all(folder).unwrap();
    });
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
fn deletion_requires_current_impact_and_repairs_assignments_atomically() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        super::writes::create_playlist(&pool, playlist("playlist-1"))
            .await
            .unwrap();
        super::writes::create_playlist(&pool, playlist("playlist-2"))
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
                replacement_playlist_id: Some("playlist-2".to_string()),
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
                replacement_playlist_id: Some("playlist-2".to_string()),
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
        assert_eq!(assignment.as_deref(), Some("playlist-2"));
        let playlist_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM music_playlists")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(playlist_count, 1);
    });
}

#[test]
fn deferred_review_items_return_after_their_optional_date() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        seed_item(&pool, "item-1", "local:item-1").await;
        super::writes::set_review_state(
            &pool,
            MusicReviewWrite {
                item_id: "item-1".to_string(),
                review_state: MusicReviewState::Deferred,
                deferred_until: Some(1_700_000_200_000),
                expected_version: 1,
                updated_at: 1_700_000_100_000,
            },
        )
        .await
        .unwrap();

        let mut request = library_window();
        request.destination = MusicListDestination::Review;
        request.now_ms = 1_700_000_150_000;
        assert_eq!(
            super::queries::item_window(&pool, request.clone())
                .await
                .unwrap()
                .total_count,
            0
        );
        request.now_ms = 1_700_000_250_000;
        assert_eq!(
            super::queries::item_window(&pool, request)
                .await
                .unwrap()
                .total_count,
            1
        );
    });
}

#[test]
fn metadata_overrides_preserve_original_values_and_refresh_search() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        seed_item(&pool, "item-1", "local:item-1").await;
        let receipt = super::writes::set_metadata_overrides(
            &pool,
            MusicMetadataOverrideWrite {
                item_id: "item-1".to_string(),
                title_override: Some("Quiet focus".to_string()),
                artist_override: Some("Composer".to_string()),
                album_override: None,
                artwork_override: None,
                expected_version: 1,
                updated_at: 1_700_000_100_000,
            },
        )
        .await
        .unwrap();
        assert_eq!(receipt.version, 2);

        let detail = super::queries::inspector_detail(&pool, "item-1")
            .await
            .unwrap();
        assert_eq!(detail.item.original_title, "item-1");
        assert_eq!(detail.item.title_override.as_deref(), Some("Quiet focus"));

        let mut request = library_window();
        request.search = "Quiet".to_string();
        let window = super::queries::item_window(&pool, request).await.unwrap();
        assert_eq!(window.total_count, 1);
        assert_eq!(window.items[0].title, "Quiet focus");
    });
}

#[test]
fn advanced_membership_settings_replace_validated_skip_ranges_atomically() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        super::writes::create_playlist(&pool, playlist("playlist-1"))
            .await
            .unwrap();
        seed_item(&pool, "item-1", "local:item-1").await;
        let initial = membership(1);
        super::writes::upsert_memberships(
            &pool,
            MusicBulkMembershipWrite {
                memberships: vec![initial.clone()],
            },
        )
        .await
        .unwrap();
        let mut updated = initial;
        updated.start_ms = Some(1_000);
        updated.end_ms = Some(90_000);
        updated.volume = Some(0.6);
        updated.rate = Some(1.25);
        updated.expected_version = Some(1);
        let receipt = super::writes::save_advanced_membership(
            &pool,
            MusicAdvancedMembershipWrite {
                membership: updated,
                skip_ranges: vec![
                    MusicMembershipSkipRange {
                        id: "skip-1".to_string(),
                        membership_id: "membership-1".to_string(),
                        start_ms: 5_000,
                        end_ms: 10_000,
                        sort_order: 0,
                    },
                    MusicMembershipSkipRange {
                        id: "skip-2".to_string(),
                        membership_id: "membership-1".to_string(),
                        start_ms: 20_000,
                        end_ms: 25_000,
                        sort_order: 1,
                    },
                ],
            },
        )
        .await
        .unwrap();
        assert_eq!(receipt.version, 2);
        let detail = super::queries::inspector_detail(&pool, "item-1")
            .await
            .unwrap();
        assert_eq!(detail.memberships[0].start_ms, Some(1_000));
        assert_eq!(detail.membership_skip_ranges.len(), 2);
        assert_eq!(detail.membership_skip_ranges[1].start_ms, 20_000);
    });
}

#[test]
fn bulk_membership_edits_preserve_existing_settings_and_commit_as_one_change() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        for index in 1..=3 {
            seed_item(
                &pool,
                &format!("item-{index}"),
                &format!("local:item-{index}"),
            )
            .await;
        }
        super::writes::create_playlist(&pool, playlist("playlist-1"))
            .await
            .unwrap();
        let mut existing = membership(1);
        existing.weight = MusicWeight::Rarely;
        existing.start_ms = Some(5_000);
        super::writes::upsert_memberships(
            &pool,
            MusicBulkMembershipWrite {
                memberships: vec![existing],
            },
        )
        .await
        .unwrap();

        let added = super::playlist_edits::bulk_edit_memberships(
            &pool,
            MusicBulkMembershipEdit {
                action_id: "bulk-add".to_string(),
                item_ids: vec!["item-1".to_string(), "item-2".to_string()],
                add_playlist_ids: vec!["playlist-1".to_string()],
                remove_playlist_ids: vec![],
                weight_playlist_ids: vec![],
                weight: None,
                updated_at: 1_700_000_000_100,
            },
        )
        .await
        .unwrap();
        assert_eq!(added.changed_count, 1);
        let preserved: (String, Option<i64>) = sqlx::query_as(
            "SELECT weight, start_ms FROM music_playlist_memberships WHERE playlist_id = 'playlist-1' AND item_id = 'item-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(preserved, ("rarely".to_string(), Some(5_000)));

        let weighted = super::playlist_edits::bulk_edit_memberships(
            &pool,
            MusicBulkMembershipEdit {
                action_id: "bulk-weight".to_string(),
                item_ids: vec!["item-1".to_string(), "item-2".to_string()],
                add_playlist_ids: vec![],
                remove_playlist_ids: vec![],
                weight_playlist_ids: vec!["playlist-1".to_string()],
                weight: Some(MusicWeight::MoreOften),
                updated_at: 1_700_000_000_200,
            },
        )
        .await
        .unwrap();
        assert_eq!(weighted.changed_count, 2);

        let removed = super::playlist_edits::bulk_edit_memberships(
            &pool,
            MusicBulkMembershipEdit {
                action_id: "bulk-remove".to_string(),
                item_ids: vec!["item-1".to_string(), "item-3".to_string()],
                add_playlist_ids: vec![],
                remove_playlist_ids: vec!["playlist-1".to_string()],
                weight_playlist_ids: vec![],
                weight: None,
                updated_at: 1_700_000_000_300,
            },
        )
        .await
        .unwrap();
        assert_eq!(removed.changed_count, 1);
        let remaining: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM music_playlist_memberships WHERE playlist_id = 'playlist-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(remaining, 1);
    });
}

#[test]
fn playlist_reorder_and_playback_projection_share_canonical_memberships() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        super::writes::create_playlist(&pool, playlist("playlist-1"))
            .await
            .unwrap();
        for index in 1..=3 {
            seed_item(
                &pool,
                &format!("item-{index}"),
                &format!("local:item-{index}"),
            )
            .await;
            super::writes::upsert_memberships(
                &pool,
                MusicBulkMembershipWrite {
                    memberships: vec![membership(index)],
                },
            )
            .await
            .unwrap();
        }
        let reordered = super::playlist_edits::reorder_playlist(
            &pool,
            MusicPlaylistReorder {
                playlist_id: "playlist-1".to_string(),
                item_id: "item-3".to_string(),
                target_index: 0,
                updated_at: 1_700_000_000_100,
            },
        )
        .await
        .unwrap();
        assert_eq!(reordered.item_ids, vec!["item-3", "item-1", "item-2"]);

        let entries =
            super::queries::playlist_playback_entries(&pool, "playlist-1", 1_700_000_000_200)
                .await
                .unwrap();
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.item_id.as_str())
                .collect::<Vec<_>>(),
            vec!["item-3", "item-1", "item-2"]
        );
        assert!(entries.iter().all(|entry| entry.enabled && !entry.snoozed));
    });
}

#[test]
fn bulk_review_and_snooze_updates_are_atomic() {
    tauri::async_runtime::block_on(async {
        let pool = pool().await;
        for index in 1..=2 {
            seed_item(
                &pool,
                &format!("item-{index}"),
                &format!("local:item-{index}"),
            )
            .await;
        }
        let result = super::playlist_edits::bulk_set_review_state(
            &pool,
            MusicBulkReviewWrite {
                items: vec![
                    MusicVersionedItem {
                        item_id: "item-1".to_string(),
                        expected_version: 1,
                    },
                    MusicVersionedItem {
                        item_id: "item-2".to_string(),
                        expected_version: 1,
                    },
                ],
                review_state: MusicReviewState::Reviewed,
                deferred_until: None,
                updated_at: 1_700_000_000_100,
            },
        )
        .await
        .unwrap();
        assert_eq!(result.changed_count, 2);
        let snoozed = super::playlist_edits::bulk_snooze(
            &pool,
            MusicBulkSnoozeWrite {
                action_id: "snooze-action".to_string(),
                item_ids: vec!["item-1".to_string(), "item-2".to_string()],
                scope: MusicSnoozeScope::AllPlaylists,
                playlist_id: None,
                starts_at: 1_700_000_000_100,
                ends_at: Some(1_700_086_400_100),
                reason: String::new(),
                created_at: 1_700_000_000_100,
            },
        )
        .await
        .unwrap();
        assert_eq!(snoozed.changed_count, 2);
        let counts: (i64, i64) = sqlx::query_as(
            "SELECT
                (SELECT COUNT(*) FROM music_library_items WHERE review_state = 'reviewed'),
                (SELECT COUNT(*) FROM music_snoozes)",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(counts, (2, 2));
    });
}
