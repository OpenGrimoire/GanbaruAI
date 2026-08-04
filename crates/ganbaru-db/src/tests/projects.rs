use super::helpers::migrated_memory_pool;

#[test]
fn schema_creates_project_custom_emojis() {
    super::block_on(async {
        let pool = migrated_memory_pool().await;

        sqlx::query(
            "INSERT INTO project_custom_emojis (id, name, asset_path, sort_order)
             VALUES ('emoji-1', 'Rocket',
                     'project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png',
                     1000)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let asset_path: String =
            sqlx::query_scalar("SELECT asset_path FROM project_custom_emojis WHERE id = 'emoji-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            asset_path,
            "project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
        );

        for (id, name, asset_path, sort_order) in [
            (
                "bad-name",
                "",
                "project-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                0,
            ),
            (
                "bad-prefix",
                "Bad",
                "other/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                0,
            ),
            (
                "bad-parent",
                "Bad",
                "project-icons/../bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                0,
            ),
            (
                "bad-nested",
                "Bad",
                "project-icons/nested/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                0,
            ),
            (
                "bad-order",
                "Bad",
                "project-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                -1,
            ),
        ] {
            assert!(
                sqlx::query(
                    "INSERT INTO project_custom_emojis (id, name, asset_path, sort_order)
                     VALUES (?, ?, ?, ?)",
                )
                .bind(id)
                .bind(name)
                .bind(asset_path)
                .bind(sort_order)
                .execute(&pool)
                .await
                .is_err(),
                "{id} should fail"
            );
        }
    });
}
