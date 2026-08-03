use super::*;

#[test]
fn shell_search_timeline_and_archive_reads_stay_lightweight() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        append_canonical_event(&pool, content_event("event-shell", "Searchable response"))
            .await
            .unwrap();
        let projects = read_project_shells(&pool).await.unwrap();
        let project = projects
            .iter()
            .find(|project| project.project_id == "project-chat")
            .expect("Chat project shell");
        assert_eq!(project.active_thread_count, 1);
        let shells = read_thread_shells(&pool, None, false).await.unwrap();
        assert_eq!(shells.len(), 1);
        assert_eq!(shells[0].message_count, 1);
        let search = search_thread_titles(&pool, "cha", Some(false), 20)
            .await
            .unwrap();
        assert_eq!(search.len(), 1);
        let page = read_timeline_page(&pool, &ChatThreadId::new("thread-1").unwrap(), None, 20)
            .await
            .unwrap();
        assert_eq!(page.items.len(), 1);

        let revision = set_thread_read(
            &pool,
            &ChatThreadId::new("thread-1").unwrap(),
            false,
            2,
            &UtcTimestamp::new(NOW).unwrap(),
        )
        .await
        .unwrap();
        let revision = set_thread_archived(
            &pool,
            &ChatThreadId::new("thread-1").unwrap(),
            true,
            revision,
            &UtcTimestamp::new(NOW).unwrap(),
        )
        .await
        .unwrap();
        assert!(read_thread_shells(&pool, None, false)
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            read_thread_shells(&pool, None, true).await.unwrap().len(),
            1
        );
        set_thread_archived(
            &pool,
            &ChatThreadId::new("thread-1").unwrap(),
            false,
            revision,
            &UtcTimestamp::new(NOW).unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(
            read_thread_shells(&pool, None, false).await.unwrap().len(),
            1
        );
    });
}

#[test]
fn thread_shell_window_limits_recent_navigation_rows() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        sqlx::query(
            "INSERT INTO chat_threads
                (id, project_id, working_folder_id, title, provider_family_id,
                 provider_instance_id, continuation_group_id, safety_mode, interaction_mode, state,
                 last_activity_at, created_at, updated_at)
             VALUES ('thread-recent', 'project-chat', 'workspace-1', 'Recent', 'codex',
                     'codex-personal', 'continuation-1', 'ask_for_approval', 'build', 'idle',
                     '2026-07-21T00:00:00.000Z', ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();

        let threads = read_thread_shell_window(&pool, None, false, 1)
            .await
            .unwrap();
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].id.as_str(), "thread-recent");
    });
}

#[test]
fn timeline_cursor_does_not_skip_rows_that_share_a_sequence() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        append_canonical_event(&pool, content_event("event-page", "Answer"))
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO chat_activities
                (id, thread_id, sequence_anchor, item_kind, status, title,
                 source_event_type, created_at, updated_at)
             VALUES ('activity-same-sequence', 'thread-1', 1, 'web_search',
                     'completed', 'Search', 'item_completed', ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();

        let thread_id = ChatThreadId::new("thread-1").unwrap();
        let newest = read_timeline_page(&pool, &thread_id, None, 1)
            .await
            .unwrap();
        let cursor = parse_timeline_cursor(newest.previous_cursor.as_deref().unwrap()).unwrap();
        let older = read_timeline_page(&pool, &thread_id, Some(&cursor), 1)
            .await
            .unwrap();

        let ids = [
            newest.items[0].activity_id.as_str(),
            older.items[0].activity_id.as_str(),
        ]
        .into_iter()
        .collect::<HashSet<_>>();
        assert_eq!(ids.len(), 2);
        assert!(older.previous_cursor.is_none());
    });
}
