//! Deterministic Chat data used by the isolated performance harness.

use serde::Serialize;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::BTreeMap;
use std::io::Read;
use std::time::{Duration, Instant};

use crate::chat::models::ChatResult;
use crate::chat::process::{spawn_provider_process, ProviderProcessConfig};

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
const BENCHMARK_CHILD_ENV: &str = "GANBARU_CHAT_BENCHMARK_CHILD";

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

/// Runs the inert child mode used to measure supervised process shutdown.
pub fn run_benchmark_child_if_requested() -> bool {
    if std::env::var(BENCHMARK_CHILD_ENV).as_deref() != Ok("1") {
        return false;
    }
    let mut buffer = [0_u8; 1024];
    while std::io::stdin()
        .read(&mut buffer)
        .is_ok_and(|read| read > 0)
    {}
    true
}

/// Measures graceful shutdown of an owned shell-free process.
pub async fn measure_provider_stop() -> ChatResult<f64> {
    let executable = std::env::current_exe().map_err(|error| {
        crate::chat::models::ChatError::driver_unavailable(format!(
            "Chat benchmark executable is unavailable: {error}"
        ))
    })?;
    let mut environment = BTreeMap::new();
    environment.insert(BENCHMARK_CHILD_ENV.to_string(), "1".to_string());
    #[cfg(test)]
    let arguments = vec![
        "--ignored".to_string(),
        "--exact".to_string(),
        "chat::benchmark::tests::benchmark_child_fixture".to_string(),
        "--nocapture".to_string(),
    ];
    #[cfg(not(test))]
    let arguments = Vec::new();
    let mut process = spawn_provider_process(ProviderProcessConfig {
        executable,
        arguments,
        working_directory: std::env::temp_dir(),
        environment,
        stderr_limit_bytes: 8 * 1024,
    })?;
    tokio::time::sleep(Duration::from_millis(25)).await;
    let started = Instant::now();
    process
        .stop(Duration::from_secs(2), Duration::from_secs(1))
        .await?;
    Ok(started.elapsed().as_secs_f64() * 1_000.0)
}

/// Returns cumulative CPU milliseconds for the app process tree.
pub fn process_tree_cpu_time_ms() -> Result<f64, String> {
    platform_process_tree_cpu_time_ms()
}

#[cfg(target_os = "linux")]
fn platform_process_tree_cpu_time_ms() -> Result<f64, String> {
    use std::collections::{HashMap, HashSet};

    let mut processes = HashMap::<u32, (u32, u64)>::new();
    let entries =
        std::fs::read_dir("/proc").map_err(|error| format!("read process list: {error}"))?;
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|value| value.parse::<u32>().ok())
        else {
            continue;
        };
        let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
            continue;
        };
        let Some(after_name) = stat.rfind(')') else {
            continue;
        };
        let fields = stat[after_name + 2..]
            .split_whitespace()
            .collect::<Vec<_>>();
        let (Some(parent), Some(user), Some(system)) =
            (fields.get(1), fields.get(11), fields.get(12))
        else {
            continue;
        };
        let (Ok(parent), Ok(user), Ok(system)) =
            (parent.parse(), user.parse::<u64>(), system.parse::<u64>())
        else {
            continue;
        };
        processes.insert(pid, (parent, user.saturating_add(system)));
    }
    let root = std::process::id();
    let mut included = HashSet::from([root]);
    loop {
        let before = included.len();
        for (&pid, &(parent, _)) in &processes {
            if included.contains(&parent) {
                included.insert(pid);
            }
        }
        if included.len() == before {
            break;
        }
    }
    let ticks = included
        .iter()
        .filter_map(|pid| processes.get(pid).map(|(_, ticks)| *ticks))
        .sum::<u64>();
    let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if ticks_per_second <= 0 {
        return Err("read process clock rate: unavailable".to_string());
    }
    Ok(ticks as f64 * 1_000.0 / ticks_per_second as f64)
}

#[cfg(target_os = "windows")]
fn platform_process_tree_cpu_time_ms() -> Result<f64, String> {
    use std::collections::{HashMap, HashSet};
    use windows::Win32::Foundation::{CloseHandle, FILETIME};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    fn filetime_ticks(value: FILETIME) -> u64 {
        (u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime)
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
        .map_err(|error| format!("snapshot process list: {error}"))?;
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut parents = HashMap::<u32, u32>::new();
    if unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok() {
        loop {
            parents.insert(entry.th32ProcessID, entry.th32ParentProcessID);
            if unsafe { Process32NextW(snapshot, &mut entry) }.is_err() {
                break;
            }
        }
    }
    unsafe { CloseHandle(snapshot) }.ok();
    let root = std::process::id();
    let mut included = HashSet::from([root]);
    loop {
        let before = included.len();
        for (&pid, &parent) in &parents {
            if included.contains(&parent) {
                included.insert(pid);
            }
        }
        if included.len() == before {
            break;
        }
    }
    let mut total_ticks = 0_u64;
    for pid in included {
        let Ok(process) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) })
        else {
            continue;
        };
        let mut creation = FILETIME::default();
        let mut exit = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        if unsafe { GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user) }
            .is_ok()
        {
            total_ticks = total_ticks
                .saturating_add(filetime_ticks(kernel))
                .saturating_add(filetime_ticks(user));
        }
        unsafe { CloseHandle(process) }.ok();
    }
    Ok(total_ticks as f64 / 10_000.0)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn platform_process_tree_cpu_time_ms() -> Result<f64, String> {
    Err("Chat CPU benchmark is supported on Linux and Windows".to_string())
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
                    (conversation_id, participant_id, membership_role, addressable,
                     approval_policy, created_at, updated_at)
                 VALUES (?, 'participant:local-owner', 'owner', 0,
                         'ask_for_approval', ?, ?)",
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
            (item_id, author_participant_id, created_at)
         VALUES (?, 'participant:local-owner', ?)",
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
mod tests {
    use super::*;
    use crate::chat::models::ChatThreadId;
    use crate::chat::repository::reads::{read_timeline_page, search_thread_titles};

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
}
