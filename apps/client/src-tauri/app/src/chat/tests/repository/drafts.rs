use super::*;

#[test]
fn durable_draft_preserves_unknown_json_and_attachment_references() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let vault_root = std::env::temp_dir().join(format!(
            "ganbaru-chat-draft-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&vault_root).unwrap();
        let source = vault_root.join("source.txt");
        fs::write(&source, b"explicit context").unwrap();
        let attachment_id = ChatAttachmentId::new("attachment-draft").unwrap();
        import_attachment(
            &pool,
            &vault_root,
            &ProjectWorkingFolderId::new("workspace-1").unwrap(),
            attachment_id.clone(),
            &source,
            ChatAttachmentKind::TextSnippet,
            &UtcTimestamp::new(NOW).unwrap(),
        )
        .await
        .unwrap();
        let draft = ChatDraftWrite {
            id: "draft-1".to_string(),
            working_folder_id: ProjectWorkingFolderId::new("workspace-1").unwrap(),
            thread_id: Some(ChatThreadId::new("thread-1").unwrap()),
            text: "Keep this".to_string(),
            rich_content: Some(VersionedJson {
                schema_version: 1,
                value: serde_json::json!({
                    "lines": [{ "runs": [{ "text": "Keep this", "marks": ["bold"] }] }]
                }),
            }),
            attachment_ids: vec![attachment_id],
            mentions: VersionedJson {
                schema_version: 88,
                value: serde_json::json!([{ "futureMention": true }]),
            },
            provider_instance_id: Some(ProviderInstanceId::new("codex-personal").unwrap()),
            model_selection: Some(VersionedJson {
                schema_version: 91,
                value: serde_json::json!({ "futureModel": "x" }),
            }),
            safety_mode: Some(SafetyMode::AskForApproval),
            interaction_mode: Some(InteractionMode::Build),
            sent_snapshot: None,
            updated_at: UtcTimestamp::new(NOW).unwrap(),
        };
        assert_eq!(save_draft(&pool, &draft).await.unwrap(), draft);
        assert_eq!(read_draft(&pool, "draft-1").await.unwrap(), Some(draft));
        delete_draft(&pool, "draft-1", &UtcTimestamp::new(NOW).unwrap())
            .await
            .unwrap();
        let unreferenced: String = sqlx::query_scalar(
            "SELECT unreferenced_at FROM chat_attachments WHERE id = 'attachment-draft'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(unreferenced, NOW);
        fs::remove_dir_all(vault_root).unwrap();
    });
}
