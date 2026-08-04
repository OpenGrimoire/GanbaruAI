use super::*;
use crate::chat::models::ChatThreadId;
use crate::chat::repository::reads::{read_timeline_page, search_thread_titles};
use std::io::Read;

#[test]
fn dense_fixture_is_deterministic_paged_and_searchable() {
    tauri::async_runtime::block_on(async {
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

        let first = seed_dense_chat_fixture(&pool).await.unwrap();
        let second = seed_dense_chat_fixture(&pool).await.unwrap();
        assert_eq!(first, second);
        assert_eq!(second.event_count, 10_000);
        assert_eq!(count(&pool, "chat_events").await, 10_000);
        assert_eq!(count(&pool, "chat_turns").await, 2_000);
        assert_eq!(count(&pool, "chat_messages").await, 4_000);
        assert_eq!(count(&pool, "chat_activities").await, 4_000);
        assert_eq!(count(&pool, "chat_attachments").await, 500);
        let channel_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chat_channels WHERE project_id LIKE 'benchmark-chat-%'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let organizational_message_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chat_communication_messages
             WHERE item_id LIKE 'benchmark-chat-organization-%'
                OR item_id LIKE 'benchmark-chat-reply-%'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(channel_count, 80);
        assert_eq!(organizational_message_count, 10_400);
        let fts_match_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chat_communication_search_fts
             WHERE chat_communication_search_fts MATCH 'planning'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(fts_match_count, 8_600);

        let thread = ChatThreadId::new(thread_id(99)).unwrap();
        let latest = read_timeline_page(&pool, &thread, None, 50).await.unwrap();
        assert_eq!(latest.items.len(), 50);
        assert!(latest.previous_cursor.is_some());
        assert!(latest
            .items
            .windows(2)
            .all(|pair| pair[0].sequence_anchor <= pair[1].sequence_anchor));

        let matches = search_thread_titles(&pool, "conversation 100", Some(false), 20)
            .await
            .unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, thread);
    });
}

#[test]
fn benchmark_owned_process_stops_within_normal_deadline() {
    tauri::async_runtime::block_on(async {
        let elapsed = measure_provider_stop().await.unwrap();
        assert!(
            elapsed < 2_000.0,
            "owned benchmark process stop took {elapsed} ms"
        );
    });
}

#[test]
#[ignore = "launched only by the owned process stop benchmark"]
fn benchmark_child_fixture() {
    assert_eq!(std::env::var(BENCHMARK_CHILD_ENV).as_deref(), Ok("1"));
    let mut buffer = [0_u8; 1024];
    while std::io::stdin()
        .read(&mut buffer)
        .is_ok_and(|read| read > 0)
    {}
}

#[test]
fn process_cpu_counter_is_monotonic() {
    let before = process_tree_cpu_time_ms().unwrap();
    std::hint::black_box((0..100_000).fold(0_u64, |sum, value| sum.wrapping_add(value)));
    let after = process_tree_cpu_time_ms().unwrap();
    assert!(after >= before);
}

async fn count(pool: &SqlitePool, table: &str) -> i64 {
    let query = format!("SELECT COUNT(*) FROM {table} WHERE id LIKE '{FIXTURE_PREFIX}%'");
    sqlx::query_scalar(&query).fetch_one(pool).await.unwrap()
}
