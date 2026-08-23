//! Deterministic Chat data used by the isolated performance harness.

use serde::Serialize;
use sqlx::{Sqlite, SqlitePool, Transaction};

mod process_metrics;

#[cfg(test)]
use process_metrics::BENCHMARK_CHILD_ENV;
pub use process_metrics::{
    measure_provider_stop, process_tree_cpu_time_ms, run_benchmark_child_if_requested,
};

pub const DENSE_CHAT_FIXTURE_PROFILE: &str = "dense-chat-v1";
const FIXTURE_PREFIX: &str = "benchmark-chat-";
const GROUP_COUNT: usize = 4;
const PROJECT_COUNT: usize = 20;
const CHANNELS_PER_PROJECT: usize = 4;
const THREAD_COUNT: usize = 100;
const TURNS_PER_THREAD: usize = 20;
const MESSAGES_PER_THREAD: usize = 40;
const ACTIVITIES_PER_THREAD: usize = 40;
const PLANS_PER_THREAD: usize = 10;
const ATTACHMENTS_PER_THREAD: usize = 5;
const CHECKPOINTS_PER_THREAD: usize = 2;
const EVENTS_PER_THREAD: usize = 100;
const ORGANIZATIONAL_MESSAGES_PER_CHANNEL: usize = 100;
const ORGANIZATIONAL_REPLY_THREADS_PER_CHANNEL: usize = 10;
const ORGANIZATIONAL_REPLIES_PER_THREAD: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DenseChatFixtureSummary {
    pub profile: &'static str,
    pub group_count: usize,
    pub project_count: usize,
    pub workspace_count: usize,
    pub channel_count: usize,
    pub thread_count: usize,
    pub turn_count: usize,
    pub message_count: usize,
    pub activity_count: usize,
    pub diff_activity_count: usize,
    pub plan_count: usize,
    pub attachment_count: usize,
    pub checkpoint_count: usize,
    pub event_count: usize,
}

/// Seeds the fixed dense Chat workload in a migrated benchmark database.
pub async fn seed_dense_chat_fixture(
    pool: &SqlitePool,
) -> Result<DenseChatFixtureSummary, sqlx::Error> {
    let mut tx = pool.begin().await?;
    clear_previous_fixture(&mut tx).await?;
    seed_projects_and_workspaces(&mut tx).await?;
    seed_organizational_messages(&mut tx).await?;
    for thread_index in 0..THREAD_COUNT {
        seed_thread(&mut tx, thread_index).await?;
        seed_turns(&mut tx, thread_index).await?;
        seed_messages(&mut tx, thread_index).await?;
        seed_activities(&mut tx, thread_index).await?;
        seed_plans(&mut tx, thread_index).await?;
        seed_attachments(&mut tx, thread_index).await?;
        seed_checkpoints(&mut tx, thread_index).await?;
        seed_events(&mut tx, thread_index).await?;
    }
    tx.commit().await?;

    Ok(DenseChatFixtureSummary {
        profile: DENSE_CHAT_FIXTURE_PROFILE,
        group_count: GROUP_COUNT,
        project_count: PROJECT_COUNT,
        workspace_count: PROJECT_COUNT,
        channel_count: PROJECT_COUNT * CHANNELS_PER_PROJECT,
        thread_count: THREAD_COUNT,
        turn_count: THREAD_COUNT * TURNS_PER_THREAD,
        message_count: THREAD_COUNT * MESSAGES_PER_THREAD,
        activity_count: THREAD_COUNT * ACTIVITIES_PER_THREAD,
        diff_activity_count: THREAD_COUNT * (ACTIVITIES_PER_THREAD / 4),
        plan_count: THREAD_COUNT * PLANS_PER_THREAD,
        attachment_count: THREAD_COUNT * ATTACHMENTS_PER_THREAD,
        checkpoint_count: THREAD_COUNT * CHECKPOINTS_PER_THREAD,
        event_count: THREAD_COUNT * EVENTS_PER_THREAD,
    })
}

async fn clear_previous_fixture(tx: &mut Transaction<'_, Sqlite>) -> Result<(), sqlx::Error> {
    for statement in [
        "DELETE FROM chat_cleanup_queue WHERE id LIKE 'benchmark-chat-%'",
        "DELETE FROM chat_attachment_references WHERE id LIKE 'benchmark-chat-%'",
        "DELETE FROM chat_attachments WHERE id LIKE 'benchmark-chat-%'",
        "DELETE FROM chat_channels WHERE project_id LIKE 'benchmark-chat-%'",
        "DELETE FROM chat_threads WHERE id LIKE 'benchmark-chat-%'",
        "DELETE FROM projects WHERE id LIKE 'benchmark-chat-%'",
        "DELETE FROM project_working_folders WHERE id LIKE 'benchmark-chat-%'",
        "DELETE FROM project_groups WHERE id LIKE 'benchmark-chat-%'",
    ] {
        sqlx::query(statement).execute(&mut **tx).await?;
    }
    Ok(())
}

async fn seed_projects_and_workspaces(tx: &mut Transaction<'_, Sqlite>) -> Result<(), sqlx::Error> {
    for group_index in 0..GROUP_COUNT {
        sqlx::query(
            "INSERT INTO project_groups (id, name, icon, sort_order, created_at, updated_at)
             VALUES (?, ?, 'folder', ?, ?, ?)",
        )
        .bind(group_id(group_index))
        .bind(format!("Benchmark group {:02}", group_index + 1))
        .bind(group_index as i64)
        .bind(timestamp(group_index))
        .bind(timestamp(group_index))
        .execute(&mut **tx)
        .await?;
    }

    for project_index in 0..PROJECT_COUNT {
        let project_id = project_id(project_index);
        let working_folder_id = working_folder_id(project_index);
        let created_at = timestamp(100 + project_index);
        sqlx::query(
            "INSERT INTO projects (id, group_id, name, icon, sort_order, created_at, updated_at)
             VALUES (?, ?, ?, 'folder', ?, ?, ?)",
        )
        .bind(&project_id)
        .bind(group_id(project_index % GROUP_COUNT))
        .bind(format!("Benchmark project {:02}", project_index + 1))
        .bind(project_index as i64)
        .bind(&created_at)
        .bind(&created_at)
        .execute(&mut **tx)
        .await?;
        sqlx::query(
            "INSERT INTO project_working_folders
                (id, project_id, display_name, kind, managed_relative_path,
                 repository_kind, repository_identity, created_at, updated_at)
             VALUES (?, ?, ?, 'managed', ?, 'git', ?, ?, ?)",
        )
        .bind(&working_folder_id)
        .bind(&project_id)
        .bind(format!("Project files {:02}", project_index + 1))
        .bind(format!("projects/{project_id}"))
        .bind(format!("benchmark-repository-{project_index:02}"))
        .bind(&created_at)
        .bind(&created_at)
        .execute(&mut **tx)
        .await?;
        for channel_slot in 1..CHANNELS_PER_PROJECT {
            let conversation_id = conversation_id(project_index, channel_slot);
            sqlx::query(
                "INSERT INTO chat_conversations
                    (id, project_id, conversation_kind, last_activity_at, created_at, updated_at)
                 VALUES (?, ?, 'channel', ?, ?, ?)",
            )
            .bind(&conversation_id)
            .bind(&project_id)
            .bind(&created_at)
            .bind(&created_at)
            .bind(&created_at)
            .execute(&mut **tx)
            .await?;
            sqlx::query(
                "INSERT INTO chat_channels
                    (id, project_id, conversation_id, name, topic, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(channel_id(project_index, channel_slot))
            .bind(&project_id)
            .bind(&conversation_id)
            .bind(channel_name(channel_slot))
            .bind(format!(
                "Dense benchmark {} coordination",
                channel_name(channel_slot)
            ))
            .bind(&created_at)
            .bind(&created_at)
            .execute(&mut **tx)
            .await?;
            sqlx::query(
                "INSERT INTO chat_conversation_memberships
                    (conversation_id, participant_id, membership_role, created_at, updated_at)
                 VALUES (?, 'participant:local-owner', 'owner', ?, ?)",
            )
            .bind(&conversation_id)
            .bind(&created_at)
            .bind(&created_at)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

async fn seed_organizational_messages(tx: &mut Transaction<'_, Sqlite>) -> Result<(), sqlx::Error> {
    for project_index in 0..PROJECT_COUNT {
        let channels = sqlx::query_as::<_, (String, String)>(
            "SELECT conversation_id, name FROM chat_channels
             WHERE project_id = ? ORDER BY is_default DESC, name",
        )
        .bind(project_id(project_index))
        .fetch_all(&mut **tx)
        .await?;
        for (conversation_id, name) in channels {
            let channel_slot = match name.as_str() {
                "planning" => 1,
                "implementation" => 2,
                "review" => 3,
                _ => 0,
            };
            for message_index in 0..ORGANIZATIONAL_MESSAGES_PER_CHANNEL {
                let created_at = timestamp(
                    30_000
                        + project_index
                            * CHANNELS_PER_PROJECT
                            * ORGANIZATIONAL_MESSAGES_PER_CHANNEL
                        + channel_slot * ORGANIZATIONAL_MESSAGES_PER_CHANNEL
                        + message_index,
                );
                let item_id = organizational_message_id(project_index, channel_slot, message_index);
                insert_organizational_message(
                    tx,
                    &conversation_id,
                    None,
                    &item_id,
                    i64::try_from(message_index + 1).unwrap_or(i64::MAX),
                    &format!(
                        "Benchmark {name} message {:03} covers general planning implementation review.",
                        message_index + 1
                    ),
                    &created_at,
                )
                .await?;
                if message_index
                    % (ORGANIZATIONAL_MESSAGES_PER_CHANNEL
                        / ORGANIZATIONAL_REPLY_THREADS_PER_CHANNEL)
                    != 0
                {
                    continue;
                }
                let reply_thread_id =
                    organizational_reply_thread_id(project_index, channel_slot, message_index);
                sqlx::query(
                    "INSERT INTO chat_reply_threads
                        (id, conversation_id, root_item_id, reply_count,
                         last_activity_at, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&reply_thread_id)
                .bind(&conversation_id)
                .bind(&item_id)
                .bind(i64::try_from(ORGANIZATIONAL_REPLIES_PER_THREAD).unwrap_or(i64::MAX))
                .bind(&created_at)
                .bind(&created_at)
                .bind(&created_at)
                .execute(&mut **tx)
                .await?;
                for reply_index in 0..ORGANIZATIONAL_REPLIES_PER_THREAD {
                    insert_organizational_message(
                        tx,
                        &conversation_id,
                        Some(&reply_thread_id),
                        &organizational_reply_id(
                            project_index,
                            channel_slot,
                            message_index,
                            reply_index,
                        ),
                        i64::try_from(reply_index + 1).unwrap_or(i64::MAX),
                        &format!("Benchmark reply {} in the {name} thread.", reply_index + 1),
                        &created_at,
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}

async fn insert_organizational_message(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    reply_thread_id: Option<&str>,
    item_id: &str,
    ordinal: i64,
    markdown: &str,
    created_at: &str,
) -> Result<(), sqlx::Error> {
    let revision_id = format!("{item_id}:revision:1");
    sqlx::query(
        "INSERT INTO chat_conversation_items
            (id, conversation_id, reply_thread_id, item_kind, ordinal, created_at)
         VALUES (?, ?, ?, 'message', ?, ?)",
    )
    .bind(item_id)
    .bind(conversation_id)
    .bind(reply_thread_id)
    .bind(ordinal)
    .bind(created_at)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "INSERT INTO chat_communication_messages
            (item_id, author_participant_id, author_label_snapshot, created_at)
         VALUES (
            ?,
            'participant:local-owner',
            (SELECT display_name FROM chat_participants
             WHERE id = 'participant:local-owner'),
            ?
         )",
    )
    .bind(item_id)
    .bind(created_at)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "INSERT INTO chat_communication_message_revisions
            (id, message_item_id, revision, normalized_markdown, created_at)
         VALUES (?, ?, 1, ?, ?)",
    )
    .bind(&revision_id)
    .bind(item_id)
    .bind(markdown)
    .bind(created_at)
    .execute(&mut **tx)
    .await?;
    sqlx::query("UPDATE chat_communication_messages SET current_revision_id = ? WHERE item_id = ?")
        .bind(&revision_id)
        .bind(item_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn seed_thread(
    tx: &mut Transaction<'_, Sqlite>,
    thread_index: usize,
) -> Result<(), sqlx::Error> {
    let project_index = thread_index % PROJECT_COUNT;
    let provider = provider_family(thread_index);
    let timestamp = timestamp(1_000 + thread_index);
    sqlx::query(
        "INSERT INTO chat_threads
            (id, working_folder_id, project_id, title, provider_family_id,
             provider_instance_id, continuation_group_id, provider_thread_id,
             model_selection_data, safety_mode, interaction_mode, state,
             latest_turn_state, latest_preview, message_count, revision,
             last_activity_at, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'ask_for_approval', ?, 'idle',
                 'completed', ?, ?, ?, ?, ?, ?)",
    )
    .bind(thread_id(thread_index))
    .bind(working_folder_id(project_index))
    .bind(project_id(project_index))
    .bind(format!(
        "Dense benchmark conversation {:03}",
        thread_index + 1
    ))
    .bind(provider)
    .bind(format!("benchmark-{provider}-instance"))
    .bind(format!("benchmark-continuation-{thread_index:03}"))
    .bind(format!("benchmark-provider-thread-{thread_index:03}"))
    .bind(format!(
        r#"{{"modelId":"benchmark-{provider}-model","modelOptions":[]}}"#
    ))
    .bind(if thread_index.is_multiple_of(5) {
        "plan"
    } else {
        "build"
    })
    .bind(format!(
        "Completed benchmark response for thread {:03}",
        thread_index + 1
    ))
    .bind(MESSAGES_PER_THREAD as i64)
    .bind((EVENTS_PER_THREAD + 1) as i64)
    .bind(&timestamp)
    .bind(&timestamp)
    .bind(&timestamp)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn seed_turns(
    tx: &mut Transaction<'_, Sqlite>,
    thread_index: usize,
) -> Result<(), sqlx::Error> {
    for turn_index in 0..TURNS_PER_THREAD {
        let created_at = timestamp(2_000 + thread_index * TURNS_PER_THREAD + turn_index);
        let changed_files = format!(
            r#"[{{"relativePath":"src/feature-{turn_index:02}.ts","previousRelativePath":null,"additions":{},"deletions":{},"binary":false,"status":"modified"}}]"#,
            turn_index + 1,
            turn_index % 7
        );
        sqlx::query(
            "INSERT INTO chat_turns
                (id, thread_id, ordinal, provider_turn_id, state, started_at, completed_at,
                 model_selection_data, safety_mode, interaction_mode, usage_schema_version,
                 usage_data, changed_file_summary_schema_version, changed_file_summary_data,
                 created_at, updated_at)
             VALUES (?, ?, ?, ?, 'completed', ?, ?, ?, 'ask_for_approval', ?, 1, ?, 1, ?, ?, ?)",
        )
        .bind(turn_id(thread_index, turn_index))
        .bind(thread_id(thread_index))
        .bind(turn_index as i64)
        .bind(format!("benchmark-provider-turn-{thread_index:03}-{turn_index:02}"))
        .bind(&created_at)
        .bind(&created_at)
        .bind(format!(r#"{{"modelId":"benchmark-{}-model","modelOptions":[]}}"#, provider_family(thread_index)))
        .bind(if thread_index.is_multiple_of(5) { "plan" } else { "build" })
        .bind(r#"{"inputTokens":1200,"cachedInputTokens":400,"outputTokens":300,"reasoningTokens":80,"totalTokens":1500,"costUsd":0.012}"#)
        .bind(changed_files)
        .bind(&created_at)
        .bind(&created_at)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn seed_messages(
    tx: &mut Transaction<'_, Sqlite>,
    thread_index: usize,
) -> Result<(), sqlx::Error> {
    for message_index in 0..MESSAGES_PER_THREAD {
        let turn_index = message_index / 2;
        let role = if message_index.is_multiple_of(2) {
            "user"
        } else {
            "assistant"
        };
        let created_at = timestamp(4_000 + thread_index * MESSAGES_PER_THREAD + message_index);
        let body = if role == "user" {
            format!("Please implement benchmark task {turn_index:02} with focused tests.")
        } else {
            format!("Implemented benchmark task {turn_index:02}.\n\n```ts\nconst result = {message_index};\n```\n\nThe change is complete.")
        };
        sqlx::query(
            "INSERT INTO chat_messages
                (id, thread_id, turn_id, sequence_anchor, role, normalized_markdown,
                 streaming_state, provider_item_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, 'complete', ?, ?, ?)",
        )
        .bind(message_id(thread_index, message_index))
        .bind(thread_id(thread_index))
        .bind(turn_id(thread_index, turn_index))
        .bind((message_index * 2 + 1) as i64)
        .bind(role)
        .bind(body)
        .bind(format!(
            "benchmark-message-item-{thread_index:03}-{message_index:02}"
        ))
        .bind(&created_at)
        .bind(&created_at)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn seed_activities(
    tx: &mut Transaction<'_, Sqlite>,
    thread_index: usize,
) -> Result<(), sqlx::Error> {
    for activity_index in 0..ACTIVITIES_PER_THREAD {
        let turn_index = activity_index / 2;
        let is_diff = activity_index.is_multiple_of(4);
        let kind = if is_diff {
            "file_change"
        } else {
            ["command", "reasoning", "tool"][activity_index % 3]
        };
        let created_at = timestamp(8_000 + thread_index * ACTIVITIES_PER_THREAD + activity_index);
        let metadata = if is_diff {
            format!(
                r#"{{"relativePath":"src/feature-{turn_index:02}.ts","additions":{},"deletions":{}}}"#,
                activity_index + 1,
                activity_index % 5
            )
        } else {
            format!(r#"{{"benchmarkIndex":{activity_index},"bounded":true}}"#)
        };
        sqlx::query(
            "INSERT INTO chat_activities
                (id, thread_id, turn_id, sequence_anchor, item_kind, status, title, detail,
                 provider_item_id, safe_metadata_data, started_at, completed_at,
                 source_event_type, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 'completed', ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(activity_id(thread_index, activity_index))
        .bind(thread_id(thread_index))
        .bind(turn_id(thread_index, turn_index))
        .bind((activity_index * 2 + 2) as i64)
        .bind(kind)
        .bind(if is_diff {
            "Modified benchmark file"
        } else {
            "Completed benchmark work"
        })
        .bind(format!(
            "Deterministic detail line {activity_index:02} for dense timeline measurement."
        ))
        .bind(format!(
            "benchmark-activity-item-{thread_index:03}-{activity_index:02}"
        ))
        .bind(metadata)
        .bind(&created_at)
        .bind(&created_at)
        .bind(if is_diff {
            "diff_updated"
        } else {
            "item_completed"
        })
        .bind(&created_at)
        .bind(&created_at)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn seed_plans(
    tx: &mut Transaction<'_, Sqlite>,
    thread_index: usize,
) -> Result<(), sqlx::Error> {
    for plan_index in 0..PLANS_PER_THREAD {
        let turn_index = plan_index * 2;
        let created_at = timestamp(12_000 + thread_index * PLANS_PER_THREAD + plan_index);
        sqlx::query(
            "INSERT INTO chat_plans
                (id, thread_id, origin_turn_id, sequence_anchor, markdown, steps_data,
                 state, implementation_turn_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, 'implemented', ?, ?, ?)",
        )
        .bind(plan_id(thread_index, plan_index))
        .bind(thread_id(thread_index))
        .bind(turn_id(thread_index, turn_index))
        .bind((81 + plan_index) as i64)
        .bind(format!("1. Inspect benchmark task {plan_index}.\n2. Implement it.\n3. Verify it."))
        .bind(format!(r#"[{{"id":"step-{plan_index}-1","text":"Inspect","status":"completed"}},{{"id":"step-{plan_index}-2","text":"Implement","status":"completed"}}]"#))
        .bind(turn_id(thread_index, turn_index))
        .bind(&created_at)
        .bind(&created_at)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn seed_attachments(
    tx: &mut Transaction<'_, Sqlite>,
    thread_index: usize,
) -> Result<(), sqlx::Error> {
    let workspace = working_folder_id(thread_index % PROJECT_COUNT);
    for attachment_index in 0..ATTACHMENTS_PER_THREAD {
        let attachment = attachment_id(thread_index, attachment_index);
        let created_at =
            timestamp(14_000 + thread_index * ATTACHMENTS_PER_THREAD + attachment_index);
        sqlx::query(
            "INSERT INTO chat_attachments
                (id, working_folder_id, kind, original_display_name, mime_type, byte_size,
                 sha256, managed_relative_path, signature_kind, created_at)
             VALUES (?, ?, 'text_snippet', ?, 'text/plain', ?, ?, ?, 'utf8-text', ?)",
        )
        .bind(&attachment)
        .bind(&workspace)
        .bind(format!("benchmark-context-{attachment_index:02}.txt"))
        .bind(512_i64 + attachment_index as i64)
        .bind(format!(
            "{thread_index:02x}{attachment_index:02x}{:060x}",
            thread_index * ATTACHMENTS_PER_THREAD + attachment_index
        ))
        .bind(format!(
            "chat/benchmark/{thread_index:03}/{attachment_index:02}.txt"
        ))
        .bind(&created_at)
        .execute(&mut **tx)
        .await?;
        sqlx::query(
            "INSERT INTO chat_attachment_references
                (id, attachment_id, message_id, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(format!(
            "{FIXTURE_PREFIX}attachment-ref-{thread_index:03}-{attachment_index:02}"
        ))
        .bind(attachment)
        .bind(message_id(thread_index, attachment_index * 2))
        .bind(created_at)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn seed_checkpoints(
    tx: &mut Transaction<'_, Sqlite>,
    thread_index: usize,
) -> Result<(), sqlx::Error> {
    for checkpoint_index in 0..CHECKPOINTS_PER_THREAD {
        let object_id = format!(
            "{thread_index:03x}{checkpoint_index:x}{:036x}",
            thread_index * 2 + checkpoint_index + 1
        );
        let created_at =
            timestamp(15_000 + thread_index * CHECKPOINTS_PER_THREAD + checkpoint_index);
        sqlx::query(
            "INSERT INTO chat_checkpoints
                (id, thread_id, turn_count, repository_identity, hidden_ref_name,
                 git_object_id, status, changed_files_data, created_at, checkpoint_kind,
                 index_commit_oid, index_tree_oid, worktree_tree_oid, index_fingerprint)
             VALUES (?, ?, ?, ?, ?, ?, 'available', ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(checkpoint_id(thread_index, checkpoint_index))
        .bind(thread_id(thread_index))
        .bind(((checkpoint_index + 1) * 10) as i64)
        .bind(format!(
            "benchmark-repository-{:02}",
            thread_index % PROJECT_COUNT
        ))
        .bind(format!(
            "refs/ganbaru-ai/chat/benchmark-{thread_index:03}-{checkpoint_index}"
        ))
        .bind(&object_id)
        .bind(format!(
            r#"[{{"relativePath":"src/checkpoint-{checkpoint_index}.ts","status":"modified"}}]"#
        ))
        .bind(created_at)
        .bind(if checkpoint_index == 0 {
            "initial"
        } else {
            "post_turn"
        })
        .bind(&object_id)
        .bind(&object_id)
        .bind(&object_id)
        .bind(format!(
            "benchmark-fingerprint-{thread_index:03}-{checkpoint_index}"
        ))
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn seed_events(
    tx: &mut Transaction<'_, Sqlite>,
    thread_index: usize,
) -> Result<(), sqlx::Error> {
    let provider = provider_family(thread_index);
    for sequence in 1..=EVENTS_PER_THREAD {
        let turn_index = ((sequence - 1) / 5).min(TURNS_PER_THREAD - 1);
        let created_at = timestamp(16_000 + thread_index * EVENTS_PER_THREAD + sequence);
        sqlx::query(
            "INSERT INTO chat_events
                (id, thread_id, sequence, turn_id, provider_turn_id, provider_item_id,
                 provider_family_id, provider_instance_id, event_type, payload_schema_version,
                 payload_data, created_at, ingested_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'benchmark_history', 1, ?, ?, ?)",
        )
        .bind(event_id(thread_index, sequence))
        .bind(thread_id(thread_index))
        .bind(sequence as i64)
        .bind(turn_id(thread_index, turn_index))
        .bind(format!("benchmark-provider-turn-{thread_index:03}-{turn_index:02}"))
        .bind(format!("benchmark-event-item-{thread_index:03}-{sequence:03}"))
        .bind(provider)
        .bind(format!("benchmark-{provider}-instance"))
        .bind(format!(r#"{{"kind":"benchmark","sequence":{sequence},"text":"Bounded history row {sequence}"}}"#))
        .bind(&created_at)
        .bind(&created_at)
        .execute(&mut **tx)
        .await?;
        sqlx::query(
            "UPDATE chat_threads SET last_event_sequence = ?, last_projected_sequence = ? WHERE id = ?",
        )
        .bind(sequence as i64)
        .bind(sequence as i64)
        .bind(thread_id(thread_index))
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

fn provider_family(index: usize) -> &'static str {
    ["codex", "claude", "cursor", "opencode"][index % 4]
}

fn timestamp(index: usize) -> String {
    let day = index / 86_400 + 1;
    let second = index % 86_400;
    format!(
        "2026-01-{day:02}T{:02}:{:02}:{:02}.000Z",
        second / 3_600,
        second / 60 % 60,
        second % 60
    )
}

fn group_id(index: usize) -> String {
    format!("{FIXTURE_PREFIX}group-{index:02}")
}
fn project_id(index: usize) -> String {
    format!("{FIXTURE_PREFIX}project-{index:02}")
}
fn working_folder_id(index: usize) -> String {
    format!("{FIXTURE_PREFIX}workspace-{index:02}")
}
fn thread_id(index: usize) -> String {
    format!("{FIXTURE_PREFIX}thread-{index:03}")
}
fn channel_id(project: usize, slot: usize) -> String {
    format!("{FIXTURE_PREFIX}channel-{project:02}-{slot}")
}
fn conversation_id(project: usize, slot: usize) -> String {
    format!("{FIXTURE_PREFIX}conversation-{project:02}-{slot}")
}
fn channel_name(slot: usize) -> &'static str {
    match slot {
        1 => "planning",
        2 => "implementation",
        3 => "review",
        _ => "general",
    }
}
fn turn_id(thread: usize, turn: usize) -> String {
    format!("{FIXTURE_PREFIX}turn-{thread:03}-{turn:02}")
}
fn message_id(thread: usize, message: usize) -> String {
    format!("{FIXTURE_PREFIX}message-{thread:03}-{message:02}")
}
fn organizational_message_id(project: usize, channel: usize, message: usize) -> String {
    format!("{FIXTURE_PREFIX}organization-{project:02}-{channel}-{message:03}")
}
fn organizational_reply_thread_id(project: usize, channel: usize, message: usize) -> String {
    format!("{FIXTURE_PREFIX}reply-thread-{project:02}-{channel}-{message:03}")
}
fn organizational_reply_id(project: usize, channel: usize, message: usize, reply: usize) -> String {
    format!("{FIXTURE_PREFIX}reply-{project:02}-{channel}-{message:03}-{reply}")
}
fn activity_id(thread: usize, activity: usize) -> String {
    format!("{FIXTURE_PREFIX}activity-{thread:03}-{activity:02}")
}
fn plan_id(thread: usize, plan: usize) -> String {
    format!("{FIXTURE_PREFIX}plan-{thread:03}-{plan:02}")
}
fn attachment_id(thread: usize, attachment: usize) -> String {
    format!("{FIXTURE_PREFIX}attachment-{thread:03}-{attachment:02}")
}
fn checkpoint_id(thread: usize, checkpoint: usize) -> String {
    format!("{FIXTURE_PREFIX}checkpoint-{thread:03}-{checkpoint}")
}
fn event_id(thread: usize, sequence: usize) -> String {
    format!("{FIXTURE_PREFIX}event-{thread:03}-{sequence:03}")
}

#[cfg(test)]
mod tests;
