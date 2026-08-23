use super::fixtures::*;

#[test]
fn batch_delete_archive_and_cap_executes_in_one_transaction() {
    tauri::async_runtime::block_on(async {
        let pool = in_memory_pool().await;
        insert_test_event_at(
            &pool,
            "delete-me",
            "2099-05-09T10:00:00Z",
            "2099-05-09T11:00:00Z",
            None,
        )
        .await;
        insert_test_event_at(
            &pool,
            "archive-me",
            "2000-05-09T10:00:00Z",
            "2000-05-09T11:00:00Z",
            None,
        )
        .await;
        insert_test_event_at(
            &pool,
            "series-1",
            "2099-05-09T10:00:00Z",
            "2099-05-09T11:00:00Z",
            Some("FREQ=DAILY"),
        )
        .await;

        let mut tx = pool.begin().await.unwrap();
        apply_delete_archive_operations_tx(
            &mut tx,
            vec![
                CalendarDeleteArchiveOperation::DeleteEvent {
                    target: CalendarEventMutationTarget {
                        id: "delete-me".to_string(),
                        occurrence_start: None,
                        occurrence_end: None,
                    },
                },
                CalendarDeleteArchiveOperation::ArchiveEvent {
                    target: CalendarEventMutationTarget {
                        id: "archive-me".to_string(),
                        occurrence_start: None,
                        occurrence_end: None,
                    },
                },
                CalendarDeleteArchiveOperation::CapSeries {
                    event_id: "series-1".to_string(),
                    repeat_until: "2099-05-08".to_string(),
                    rrule: "FREQ=DAILY;UNTIL=20990508T235959Z".to_string(),
                },
            ],
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        let delete_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM calendar_events WHERE id = 'delete-me'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let archived_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM calendar_events_archive WHERE id = 'archive-me'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let series: (Option<String>, Option<String>) =
            sqlx::query_as("SELECT repeat_until, rrule FROM calendar_events WHERE id = 'series-1'")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(delete_count, 0);
        assert_eq!(archived_count, 1);
        assert_eq!(series.0, Some("2099-05-08".to_string()));
        assert_eq!(
            series.1,
            Some("FREQ=DAILY;UNTIL=20990508T235959Z".to_string())
        );
    });
}

#[test]
fn batch_rolls_back_when_later_operation_fails() {
    tauri::async_runtime::block_on(async {
        let pool = in_memory_pool().await;
        insert_test_event_at(
            &pool,
            "delete-me",
            "2099-05-09T10:00:00Z",
            "2099-05-09T11:00:00Z",
            None,
        )
        .await;

        let mut tx = pool.begin().await.unwrap();
        let err = apply_delete_archive_operations_tx(
            &mut tx,
            vec![
                CalendarDeleteArchiveOperation::DeleteEvent {
                    target: CalendarEventMutationTarget {
                        id: "delete-me".to_string(),
                        occurrence_start: None,
                        occurrence_end: None,
                    },
                },
                CalendarDeleteArchiveOperation::CapSeries {
                    event_id: "missing-series".to_string(),
                    repeat_until: "2099-05-08".to_string(),
                    rrule: "FREQ=DAILY;UNTIL=20990508T235959Z".to_string(),
                },
            ],
        )
        .await
        .unwrap_err();
        tx.rollback().await.unwrap();
        assert!(err.contains("missing-series"));

        let live_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM calendar_events WHERE id = 'delete-me'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(live_count, 1);
    });
}

#[test]
fn recurrence_commit_batch_updates_event_and_active_run_in_one_transaction() {
    tauri::async_runtime::block_on(async {
        let pool = in_memory_pool().await;
        insert_test_event(&pool, "event-1", "").await;
        insert_test_event_at(
            &pool,
            "event-2",
            "2026-05-10T10:00:00Z",
            "2026-05-10T11:00:00Z",
            None,
        )
        .await;
        insert_test_open_pomodoro_run(&pool).await;
        insert_test_active_pomodoro_segment(&pool).await;

        let mut tx = pool.begin().await.unwrap();
        apply_recurrence_commit_operations_tx(
            &mut tx,
            vec![
                CalendarRecurrenceCommitOperation::UpdateEvent {
                    patch: Box::new(CalendarEventUpdate {
                        id: "event-1".to_string(),
                        updated_at: "2026-05-09T10:30:00Z".to_string(),
                        fields: vec![CalendarEventUpdateField::Title("Changed".to_string())],
                        attendees: None,
                        alarms: None,
                        pomodoro_config: None,
                    }),
                },
                CalendarRecurrenceCommitOperation::TransferActiveEventReference {
                    transfer: CalendarActiveEventReferenceTransfer {
                        new_event_id: "event-2".to_string(),
                        new_event_date: Some("2026-05-10".to_string()),
                        planned_end: Some("2026-05-10T11:00:00Z".to_string()),
                    },
                },
            ],
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        let title: String =
            sqlx::query_scalar("SELECT title FROM calendar_events WHERE id = 'event-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let run: (String, String, String) = sqlx::query_as(
            "SELECT event_id, original_event_id, event_date FROM pomodoro_runs WHERE id = 'run-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let segment: (String, String) = sqlx::query_as(
            "SELECT event_id, event_date FROM pomodoro_segments WHERE id = 'segment-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(title, "Changed");
        assert_eq!(
            run,
            (
                "event-2".to_string(),
                "event-2".to_string(),
                "2026-05-10".to_string()
            )
        );
        assert_eq!(segment, ("event-2".to_string(), "2026-05-10".to_string()));
    });
}

#[test]
fn recurrence_commit_batch_rolls_back_when_later_operation_fails() {
    tauri::async_runtime::block_on(async {
        let pool = in_memory_pool().await;
        insert_test_event(&pool, "event-1", "").await;

        let mut tx = pool.begin().await.unwrap();
        let err = apply_recurrence_commit_operations_tx(
            &mut tx,
            vec![
                CalendarRecurrenceCommitOperation::UpdateEvent {
                    patch: Box::new(CalendarEventUpdate {
                        id: "event-1".to_string(),
                        updated_at: "2026-05-09T10:30:00Z".to_string(),
                        fields: vec![CalendarEventUpdateField::Title("Changed".to_string())],
                        attendees: None,
                        alarms: None,
                        pomodoro_config: None,
                    }),
                },
                CalendarRecurrenceCommitOperation::DetachInstance {
                    input: Box::new(CalendarDetachInstance {
                        parent_id: "missing-parent".to_string(),
                        instance_date: "2026-05-10".to_string(),
                        exceptions: "[\"2026-05-10\"]".to_string(),
                        new_id: "detached-1".to_string(),
                        title: "Detached".to_string(),
                        start_time: "2026-05-10T10:00:00Z".to_string(),
                        end_time: "2026-05-10T11:00:00Z".to_string(),
                        timezone: "America/Monterrey".to_string(),
                        calendar_id: "local".to_string(),
                        project_id: None,
                        environment_id: None,
                        playlist_id: None,
                        color: None,
                        notifications: None,
                        all_day: false,
                        location: String::new(),
                        transparency: "opaque".to_string(),
                        status: "confirmed".to_string(),
                        now: "2026-05-09T10:30:00Z".to_string(),
                        music_snapshot_assignments: Vec::new(),
                        music_override_assignments: Vec::new(),
                    }),
                },
            ],
        )
        .await
        .unwrap_err();
        tx.rollback().await.unwrap();
        assert!(!err.is_empty());

        let title: String =
            sqlx::query_scalar("SELECT title FROM calendar_events WHERE id = 'event-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(title, "");
    });
}

#[test]
fn detached_recurrence_preserves_scoped_music_assignments() {
    tauri::async_runtime::block_on(async {
        let pool = in_memory_pool().await;
        insert_test_event_at(
            &pool,
            "series-1",
            "2099-05-09T10:00:00Z",
            "2099-05-09T11:00:00Z",
            Some("FREQ=DAILY"),
        )
        .await;
        sqlx::query(
            "INSERT INTO music_playlists (id, name, created_at, updated_at)
             VALUES ('focus-playlist', 'Focus', 1, 1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let snapshot = crate::music::library::MusicContextAssignmentDraft {
            phase: crate::music::library::MusicActivityPhase::Focus,
            behavior: crate::music::library::MusicAssignmentBehavior::PlayAutomatically,
            playlist_id: Some("focus-playlist".to_string()),
            soundscape_id: None,
            soundscape_behavior: crate::music::library::MusicSoundscapeBehavior::Inherit,
            provenance_kind: crate::music::library::MusicAssignmentProvenanceKind::CopiedProject,
            provenance_id: Some("project-1".to_string()),
        };
        let override_assignment = crate::music::library::MusicContextAssignmentDraft {
            phase: crate::music::library::MusicActivityPhase::ShortBreak,
            behavior: crate::music::library::MusicAssignmentBehavior::PauseMusic,
            playlist_id: None,
            soundscape_id: None,
            soundscape_behavior: crate::music::library::MusicSoundscapeBehavior::Inherit,
            provenance_kind: crate::music::library::MusicAssignmentProvenanceKind::Explicit,
            provenance_id: None,
        };
        let mut tx = pool.begin().await.unwrap();
        apply_recurrence_commit_operations_tx(
            &mut tx,
            vec![CalendarRecurrenceCommitOperation::DetachInstance {
                input: Box::new(CalendarDetachInstance {
                    parent_id: "series-1".to_string(),
                    instance_date: "2099-05-10".to_string(),
                    exceptions: "[\"2099-05-10\"]".to_string(),
                    new_id: "detached-1".to_string(),
                    title: "Detached focus".to_string(),
                    start_time: "2099-05-10T10:00:00Z".to_string(),
                    end_time: "2099-05-10T11:00:00Z".to_string(),
                    timezone: "America/Monterrey".to_string(),
                    calendar_id: "local".to_string(),
                    project_id: None,
                    environment_id: None,
                    playlist_id: None,
                    color: None,
                    notifications: None,
                    all_day: false,
                    location: String::new(),
                    transparency: "opaque".to_string(),
                    status: "confirmed".to_string(),
                    now: "2026-07-15T10:00:00Z".to_string(),
                    music_snapshot_assignments: vec![snapshot],
                    music_override_assignments: vec![override_assignment],
                }),
            }],
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        let assignments: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT owner_kind, phase, behavior, playlist_id
             FROM music_context_assignments
             WHERE owner_id = 'detached-1'
             ORDER BY owner_kind, phase",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            assignments,
            vec![
                (
                    "event-override".to_string(),
                    "short-break".to_string(),
                    "pause-music".to_string(),
                    None,
                ),
                (
                    "event-snapshot".to_string(),
                    "focus".to_string(),
                    "play-automatically".to_string(),
                    Some("focus-playlist".to_string()),
                ),
            ]
        );
        let parent_assignment_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM music_context_assignments WHERE owner_id = 'series-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(parent_assignment_count, 0);
    });
}

#[test]
fn batch_hard_delete_rejects_protected_rows() {
    tauri::async_runtime::block_on(async {
        let pool = in_memory_pool().await;
        insert_test_event_at(
            &pool,
            "event-1",
            "2000-05-09T10:00:00Z",
            "2000-05-09T11:00:00Z",
            None,
        )
        .await;

        let mut tx = pool.begin().await.unwrap();
        let err = apply_delete_archive_operations_tx(
            &mut tx,
            vec![CalendarDeleteArchiveOperation::DeleteEvent {
                target: CalendarEventMutationTarget {
                    id: "event-1".to_string(),
                    occurrence_start: None,
                    occurrence_end: None,
                },
            }],
        )
        .await
        .unwrap_err();
        tx.rollback().await.unwrap();
        assert!(err.contains("archive it instead"));
    });
}

#[test]
fn batch_archive_rejects_active_pomodoro_rows() {
    tauri::async_runtime::block_on(async {
        let pool = in_memory_pool().await;
        insert_test_event(&pool, "event-1", "").await;
        insert_test_open_pomodoro_run(&pool).await;
        insert_test_active_pomodoro_segment(&pool).await;

        let mut tx = pool.begin().await.unwrap();
        let err = apply_delete_archive_operations_tx(
            &mut tx,
            vec![CalendarDeleteArchiveOperation::ArchiveEvent {
                target: CalendarEventMutationTarget {
                    id: "event-1".to_string(),
                    occurrence_start: None,
                    occurrence_end: None,
                },
            }],
        )
        .await
        .unwrap_err();
        tx.rollback().await.unwrap();
        assert!(err.contains("active pomodoro run"));
    });
}

#[test]
fn batch_synthetic_archive_preserves_original_event_id() {
    tauri::async_runtime::block_on(async {
        let pool = in_memory_pool().await;
        insert_test_event_at(
            &pool,
            "event-1",
            "2000-05-09T10:00:00Z",
            "2000-05-09T11:00:00Z",
            Some("FREQ=DAILY"),
        )
        .await;
        insert_test_completed_pomodoro_history(
            &pool,
            "event-1",
            "event-1::2000-05-10",
            "2000-05-10",
        )
        .await;

        let mut tx = pool.begin().await.unwrap();
        apply_delete_archive_operations_tx(
            &mut tx,
            vec![CalendarDeleteArchiveOperation::ArchiveEvent {
                target: CalendarEventMutationTarget {
                    id: "event-1::2000-05-10".to_string(),
                    occurrence_start: Some("2000-05-10T10:00:00Z".to_string()),
                    occurrence_end: Some("2000-05-10T11:00:00Z".to_string()),
                },
            }],
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        let archived_source: String = sqlx::query_scalar(
            "SELECT source_event_id FROM calendar_events_archive WHERE id = 'event-1::2000-05-10'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let exdate_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM calendar_event_exdates
             WHERE event_id = 'event-1' AND occurrence_date = '2000-05-10'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let run: (Option<String>, String) = sqlx::query_as(
            "SELECT event_id, original_event_id FROM pomodoro_runs WHERE id = 'run-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(archived_source, "event-1");
        assert_eq!(exdate_count, 1);
        assert_eq!(run.0, None);
        assert_eq!(run.1, "event-1::2000-05-10");
    });
}

#[test]
fn batch_cap_updates_repeat_rule_and_timestamp() {
    tauri::async_runtime::block_on(async {
        let pool = in_memory_pool().await;
        insert_test_event_at(
            &pool,
            "series-1",
            "2099-05-09T10:00:00Z",
            "2099-05-09T11:00:00Z",
            Some("FREQ=DAILY"),
        )
        .await;

        let before: String =
            sqlx::query_scalar("SELECT updated_at FROM calendar_events WHERE id = 'series-1'")
                .fetch_one(&pool)
                .await
                .unwrap();

        let mut tx = pool.begin().await.unwrap();
        cap_calendar_series_tx(
            &mut tx,
            "series-1",
            "2099-05-08",
            "FREQ=DAILY;UNTIL=20990508T235959Z",
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        let row: (Option<String>, Option<String>, String) = sqlx::query_as(
            "SELECT repeat_until, rrule, updated_at FROM calendar_events WHERE id = 'series-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, Some("2099-05-08".to_string()));
        assert_eq!(row.1, Some("FREQ=DAILY;UNTIL=20990508T235959Z".to_string()));
        assert_ne!(row.2, before);
    });
}
