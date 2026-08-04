mod calendar;
mod chat;
mod core;
mod doomscrolling;
mod helpers;
mod music;
mod notes;
mod pomodoro;
mod projects;
mod query_plans;
mod quick_notes;

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("database test runtime should initialize")
        .block_on(future)
}

#[test]
fn pool_registry_reuses_and_closes_authorized_path() {
    block_on(async {
        let directory = std::env::temp_dir().join(format!(
            "ganbaru-db-registry-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("ganbaru-ai.sqlite");
        let registry = crate::DatabasePoolRegistry::default();

        let first = registry.connect_path(&path).await.unwrap();
        sqlx::query("CREATE TABLE registry_test (value TEXT NOT NULL)")
            .execute(&first)
            .await
            .unwrap();
        let second = registry.connect_path(&path).await.unwrap();
        let table_exists: i64 = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name = 'registry_test')",
        )
        .fetch_one(&second)
        .await
        .unwrap();
        assert_eq!(table_exists, 1);

        registry.close_path(&path).await.unwrap();
        assert!(first.is_closed());
        assert!(second.is_closed());
        registry.close_all().await.unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    });
}
