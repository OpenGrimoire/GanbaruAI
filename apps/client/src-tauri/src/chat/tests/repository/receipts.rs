use super::*;

#[test]
fn command_receipts_replay_every_idempotent_command_kind() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let thread_id = ChatThreadId::new("thread-1").unwrap();
        let now = UtcTimestamp::new(NOW).unwrap();
        for command_kind in [
            "send",
            "approval",
            "answer",
            "interrupt",
            "restore",
            "delete",
        ] {
            let command_id = ChatCommandId::new(format!("command-{command_kind}")).unwrap();
            let claim =
                claim_command_receipt(&pool, &command_id, &thread_id, command_kind, Some(1), &now)
                    .await
                    .unwrap();
            assert!(matches!(claim, CommandReceiptClaim::Claimed(_)));
            let result = VersionedJson {
                schema_version: 7,
                value: serde_json::json!({ "commandKind": command_kind, "future": true }),
            };
            complete_command_receipt(
                &pool,
                &command_id,
                CommandReceiptState::Completed,
                Some(&result),
                None,
                &now,
            )
            .await
            .unwrap();
            let replay =
                claim_command_receipt(&pool, &command_id, &thread_id, command_kind, Some(1), &now)
                    .await
                    .unwrap();
            let CommandReceiptClaim::Replay(receipt) = replay else {
                panic!("duplicate command should replay its original receipt");
            };
            assert_eq!(receipt.result, Some(result));
        }
    });
}

#[test]
fn command_receipts_reject_cross_thread_and_changed_command_reuse() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        sqlx::query(
            "INSERT INTO chat_threads
                (id, project_id, working_folder_id, title, provider_family_id, provider_instance_id,
                 continuation_group_id, safety_mode, interaction_mode, state,
                 last_activity_at, created_at, updated_at)
             VALUES ('thread-2', 'project-chat', 'workspace-1', 'Other', 'codex', 'codex-personal',
                     'continuation-1', 'ask_for_approval', 'build', 'idle', ?, ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        let command_id = ChatCommandId::new("command-boundary").unwrap();
        let thread_1 = ChatThreadId::new("thread-1").unwrap();
        let thread_2 = ChatThreadId::new("thread-2").unwrap();
        let now = UtcTimestamp::new(NOW).unwrap();
        claim_command_receipt(&pool, &command_id, &thread_1, "approval", Some(1), &now)
            .await
            .unwrap();

        assert!(
            claim_command_receipt(&pool, &command_id, &thread_2, "approval", Some(1), &now,)
                .await
                .is_err()
        );
        assert!(
            claim_command_receipt(&pool, &command_id, &thread_1, "answer", Some(1), &now,)
                .await
                .is_err()
        );
    });
}
