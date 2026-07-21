//! Git checkpoint capture and bounded diff plumbing.

use super::events::ChangedFileSummary;
use super::models::{
    ChatCheckpointId, ChatError, ChatErrorCode, ChatResult, ChatThreadId, ChatTurnId, UtcTimestamp,
};
use super::workspace::AuthorizedWorkspace;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_GIT_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
const MAX_DIFF_BYTES: usize = 2 * 1024 * 1024;
const REF_PREFIX: &str = "refs/ganbaru-ai/chat/";
type DiffLineCounts = BTreeMap<String, (Option<u64>, Option<u64>)>;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointKind {
    Initial,
    PreTurn,
    PostTurn,
    Recovery,
}

impl CheckpointKind {
    fn wire(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::PreTurn => "pre_turn",
            Self::PostTurn => "post_turn",
            Self::Recovery => "recovery",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CapturedCheckpoint {
    pub id: ChatCheckpointId,
    pub hidden_ref_name: String,
    pub git_object_id: String,
    pub index_commit_oid: String,
    pub index_tree_oid: String,
    pub worktree_tree_oid: String,
    pub head_oid: Option<String>,
    pub head_ref: Option<String>,
    pub index_fingerprint: String,
    pub changed_files: Vec<ChangedFileSummary>,
}

#[derive(Clone, Debug)]
pub struct StoredCheckpoint {
    pub id: ChatCheckpointId,
    pub thread_id: ChatThreadId,
    pub turn_count: u64,
    pub repository_identity: String,
    pub hidden_ref_name: String,
    pub git_object_id: String,
    pub index_tree_oid: String,
    pub worktree_tree_oid: String,
    pub head_oid: Option<String>,
    pub head_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatChangedFileRead {
    pub relative_path: String,
    pub previous_relative_path: Option<String>,
    pub status: String,
    pub additions: Option<u64>,
    pub deletions: Option<u64>,
    pub binary: bool,
    pub provider_reported: bool,
    pub git_observed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCheckpointFileDiffRead {
    pub relative_path: String,
    pub patch: Option<String>,
    pub binary: bool,
    pub truncated: bool,
    pub byte_size: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrentGitSnapshot {
    pub worktree_commit_oid: String,
    pub worktree_tree_oid: String,
    pub index_tree_oid: String,
    pub index_fingerprint: String,
    pub head_oid: Option<String>,
    pub head_ref: Option<String>,
}

pub async fn capture_and_store(
    pool: &SqlitePool,
    authorized: &AuthorizedWorkspace,
    thread_id: &ChatThreadId,
    turn_id: Option<&ChatTurnId>,
    turn_count: u64,
    kind: CheckpointKind,
    now: &UtcTimestamp,
) -> ChatResult<CapturedCheckpoint> {
    let repository_identity = authorized.repository_identity.as_deref().ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::CapabilityUnsupported,
            "Git checkpoints are unavailable for this workspace",
            true,
        )
    })?;
    let existing = sqlx::query(
        "SELECT id FROM chat_checkpoints
         WHERE thread_id = ? AND turn_count = ? AND status = 'available'",
    )
    .bind(thread_id.as_str())
    .bind(i64_value(turn_count)?)
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?;
    if existing.is_some() {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "This Chat checkpoint already exists",
            true,
        ));
    }
    let checkpoint_id = checkpoint_id(thread_id, turn_id, turn_count, now)?;
    let previous_oid = latest_checkpoint_oid(pool, thread_id).await?;
    let root = authorized.canonical_path.clone();
    let thread_for_git = thread_id.clone();
    let checkpoint_for_git = checkpoint_id.clone();
    let captured = tauri::async_runtime::spawn_blocking(move || {
        capture_git(
            &root,
            &thread_for_git,
            &checkpoint_for_git,
            previous_oid.as_deref(),
        )
    })
    .await
    .map_err(|_| checkpoint_error("Git checkpoint worker stopped"))??;
    let changed_files_data =
        serde_json::to_string(&captured.changed_files).map_err(serialization_error)?;
    let inserted = sqlx::query(
        "INSERT INTO chat_checkpoints
            (id, thread_id, turn_count, repository_identity, hidden_ref_name, git_object_id,
             status, changed_files_data, created_at, checkpoint_kind, turn_id,
             index_commit_oid, index_tree_oid, worktree_tree_oid, head_oid, head_ref,
             index_fingerprint)
         VALUES (?, ?, ?, ?, ?, ?, 'available', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(captured.id.as_str())
    .bind(thread_id.as_str())
    .bind(i64_value(turn_count)?)
    .bind(repository_identity)
    .bind(&captured.hidden_ref_name)
    .bind(&captured.git_object_id)
    .bind(changed_files_data)
    .bind(now.as_str())
    .bind(kind.wire())
    .bind(turn_id.map(ChatTurnId::as_str))
    .bind(&captured.index_commit_oid)
    .bind(&captured.index_tree_oid)
    .bind(&captured.worktree_tree_oid)
    .bind(captured.head_oid.as_deref())
    .bind(captured.head_ref.as_deref())
    .bind(&captured.index_fingerprint)
    .execute(pool)
    .await;
    if let Err(error) = inserted {
        let _ = delete_exact_ref(
            &authorized.canonical_path,
            &captured.hidden_ref_name,
            &captured.git_object_id,
        );
        return Err(persistence_error(error));
    }
    if let Some(turn_id) = turn_id {
        let column = if matches!(kind, CheckpointKind::Initial | CheckpointKind::PreTurn) {
            "pre_checkpoint_id"
        } else {
            "post_checkpoint_id"
        };
        let statement =
            format!("UPDATE chat_turns SET {column} = ? WHERE id = ? AND thread_id = ?");
        sqlx::query(&statement)
            .bind(captured.id.as_str())
            .bind(turn_id.as_str())
            .bind(thread_id.as_str())
            .execute(pool)
            .await
            .map_err(persistence_error)?;
    }
    Ok(captured)
}

pub async fn read_stored_checkpoint(
    pool: &SqlitePool,
    checkpoint_id: &ChatCheckpointId,
) -> ChatResult<StoredCheckpoint> {
    let row = sqlx::query(
        "SELECT id, thread_id, turn_count, repository_identity, hidden_ref_name,
                git_object_id, index_tree_oid, worktree_tree_oid, head_oid, head_ref
         FROM chat_checkpoints
         WHERE id = ? AND status = 'available' AND invalidated_at IS NULL",
    )
    .bind(checkpoint_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Chat checkpoint is unavailable",
            true,
        )
    })?;
    Ok(StoredCheckpoint {
        id: parse_id(row.try_get("id").map_err(persistence_error)?)?,
        thread_id: parse_thread_id(row.try_get("thread_id").map_err(persistence_error)?)?,
        turn_count: u64_column(&row, "turn_count")?,
        repository_identity: row
            .try_get("repository_identity")
            .map_err(persistence_error)?,
        hidden_ref_name: row.try_get("hidden_ref_name").map_err(persistence_error)?,
        git_object_id: row.try_get("git_object_id").map_err(persistence_error)?,
        index_tree_oid: required_string(&row, "index_tree_oid")?,
        worktree_tree_oid: required_string(&row, "worktree_tree_oid")?,
        head_oid: row.try_get("head_oid").map_err(persistence_error)?,
        head_ref: row.try_get("head_ref").map_err(persistence_error)?,
    })
}

pub fn verify_checkpoint(
    authorized: &AuthorizedWorkspace,
    checkpoint: &StoredCheckpoint,
) -> ChatResult<()> {
    if authorized.repository_identity.as_deref() != Some(&checkpoint.repository_identity)
        || !valid_hidden_ref(&checkpoint.hidden_ref_name)
    {
        return Err(ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "Checkpoint repository identity no longer matches",
            true,
        ));
    }
    let resolved = optional_git_text(
        &authorized.canonical_path,
        &[
            "rev-parse",
            "-q",
            "--verify",
            &format!("{}^{{commit}}", checkpoint.hidden_ref_name),
        ],
    )?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Checkpoint Git ref is missing or changed",
            true,
        )
    })?;
    if resolved != checkpoint.git_object_id {
        return Err(ChatError::new(
            ChatErrorCode::NotFound,
            "Checkpoint Git ref is missing or changed",
            true,
        ));
    }
    Ok(())
}

pub fn diff_files(
    root: &Path,
    pre: &StoredCheckpoint,
    post: &StoredCheckpoint,
) -> ChatResult<Vec<ChatChangedFileRead>> {
    let status = git_output(
        root,
        &[
            "diff-tree",
            "-r",
            "-z",
            "--no-commit-id",
            "--find-renames",
            "--name-status",
            &pre.git_object_id,
            &post.git_object_id,
        ],
        None,
        None,
    )?;
    let stats = diff_numstat(root, &pre.git_object_id, &post.git_object_id)?;
    parse_name_status(&status.stdout, &stats)
}

pub fn file_diff(
    root: &Path,
    pre: &StoredCheckpoint,
    post: &StoredCheckpoint,
    relative_path: &str,
    ignore_whitespace: bool,
) -> ChatResult<ChatCheckpointFileDiffRead> {
    validate_diff_path(relative_path)?;
    let mut arguments = vec![
        "diff",
        "--no-ext-diff",
        "--no-textconv",
        "--find-renames",
        "--unified=3",
    ];
    if ignore_whitespace {
        arguments.push("-w");
    }
    arguments.extend([
        pre.git_object_id.as_str(),
        post.git_object_id.as_str(),
        "--",
        relative_path,
    ]);
    let output = git_output(root, &arguments, None, None)?;
    let byte_size = output.stdout.len() as u64;
    let truncated = output.stdout.len() > MAX_DIFF_BYTES;
    let bounded = &output.stdout[..output.stdout.len().min(MAX_DIFF_BYTES)];
    let binary = bounded.windows(16).any(|window| window == b"Binary files ");
    Ok(ChatCheckpointFileDiffRead {
        relative_path: relative_path.to_string(),
        patch: (!binary).then(|| String::from_utf8_lossy(bounded).into_owned()),
        binary,
        truncated,
        byte_size,
    })
}

pub fn delete_exact_ref(root: &Path, reference: &str, expected_oid: &str) -> ChatResult<()> {
    if !valid_hidden_ref(reference)
        || !(40..=64).contains(&expected_oid.len())
        || !expected_oid.bytes().all(|byte| byte.is_ascii_hexdigit())
        || expected_oid.bytes().all(|byte| byte == b'0')
    {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Checkpoint cleanup target is invalid",
            false,
        ));
    }
    let Some(actual_oid) = optional_git_text(root, &["rev-parse", "-q", "--verify", reference])?
    else {
        return Ok(());
    };
    if actual_oid != expected_oid {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Checkpoint ref changed before cleanup",
            true,
        ));
    }
    git_output(
        root,
        &["update-ref", "-d", reference, expected_oid],
        None,
        None,
    )
    .map(|_| ())
}

pub fn current_git_snapshot(root: &Path) -> ChatResult<CurrentGitSnapshot> {
    verify_git_root(root)?;
    let head_ref = optional_git_text(root, &["symbolic-ref", "-q", "HEAD"])?;
    let head_oid = optional_git_text(root, &["rev-parse", "-q", "--verify", "HEAD^{commit}"])?;
    let index_tree_oid = git_text(root, &["write-tree"], None)?;
    let index_fingerprint =
        hash_bytes(&git_output(root, &["ls-files", "--stage", "-z"], None, None)?.stdout);
    let temp_index = TemporaryIndex::new()?;
    if let Some(head_oid) = head_oid.as_deref() {
        git_output(
            root,
            &["read-tree", head_oid],
            Some(temp_index.path()),
            None,
        )?;
    } else {
        git_output(
            root,
            &["read-tree", "--empty"],
            Some(temp_index.path()),
            None,
        )?;
    }
    git_output(
        root,
        &["add", "-A", "--", "."],
        Some(temp_index.path()),
        None,
    )?;
    let worktree_tree_oid = git_text(root, &["write-tree"], Some(temp_index.path()))?;
    let index_commit_oid =
        git_commit_tree(root, &index_tree_oid, None, "Ganbaru Chat preview index")?;
    let worktree_commit_oid = git_commit_tree(
        root,
        &worktree_tree_oid,
        Some(&index_commit_oid),
        "Ganbaru Chat restore preview",
    )?;
    Ok(CurrentGitSnapshot {
        worktree_commit_oid,
        worktree_tree_oid,
        index_tree_oid,
        index_fingerprint,
        head_oid,
        head_ref,
    })
}

pub fn restore_git_snapshot(
    root: &Path,
    current: &CurrentGitSnapshot,
    target: &StoredCheckpoint,
) -> ChatResult<()> {
    verify_git_root(root)?;
    let before = current_git_snapshot(root)?;
    if &before != current {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "Workspace changed after the restore preview",
            true,
        ));
    }
    let restore_index = TemporaryIndex::new()?;
    git_output(
        root,
        &["read-tree", &current.worktree_tree_oid],
        Some(restore_index.path()),
        None,
    )?;
    let worktree_result = git_output(
        root,
        &["read-tree", "--reset", "-u", &target.worktree_tree_oid],
        Some(restore_index.path()),
        None,
    );
    if worktree_result.is_err() {
        return Err(checkpoint_error("Workspace checkpoint restore failed"));
    }
    if let Err(error) = git_output(
        root,
        &["read-tree", "--reset", &target.index_tree_oid],
        None,
        None,
    ) {
        let _ = recover_git_snapshot(root, current);
        return Err(error);
    }
    let restored = current_git_snapshot(root)?;
    if restored.worktree_tree_oid != target.worktree_tree_oid
        || restored.index_tree_oid != target.index_tree_oid
        || restored.head_oid != current.head_oid
        || restored.head_ref != current.head_ref
    {
        let _ = recover_git_snapshot(root, current);
        return Err(checkpoint_error(
            "Checkpoint restore verification failed and recovery was attempted",
        ));
    }
    Ok(())
}

fn recover_git_snapshot(root: &Path, snapshot: &CurrentGitSnapshot) -> ChatResult<()> {
    let recovery_index = TemporaryIndex::new()?;
    if let Some(head_oid) = snapshot.head_oid.as_deref() {
        git_output(
            root,
            &["read-tree", head_oid],
            Some(recovery_index.path()),
            None,
        )?;
    } else {
        git_output(
            root,
            &["read-tree", "--empty"],
            Some(recovery_index.path()),
            None,
        )?;
    }
    git_output(
        root,
        &["add", "-A", "--", "."],
        Some(recovery_index.path()),
        None,
    )?;
    git_output(
        root,
        &["read-tree", "--reset", "-u", &snapshot.worktree_tree_oid],
        Some(recovery_index.path()),
        None,
    )?;
    git_output(
        root,
        &["read-tree", "--reset", &snapshot.index_tree_oid],
        None,
        None,
    )?;
    Ok(())
}

fn capture_git(
    root: &Path,
    thread_id: &ChatThreadId,
    checkpoint_id: &ChatCheckpointId,
    previous_oid: Option<&str>,
) -> ChatResult<CapturedCheckpoint> {
    verify_git_root(root)?;
    let before = repository_fingerprint(root)?;
    let head_ref = optional_git_text(root, &["symbolic-ref", "-q", "HEAD"])?;
    let head_oid = optional_git_text(root, &["rev-parse", "-q", "--verify", "HEAD^{commit}"])?;
    let index_tree_oid = git_text(root, &["write-tree"], None)?;
    let index_fingerprint =
        hash_bytes(&git_output(root, &["ls-files", "--stage", "-z"], None, None)?.stdout);
    let temp_index = TemporaryIndex::new()?;
    if let Some(head_oid) = head_oid.as_deref() {
        git_output(
            root,
            &["read-tree", head_oid],
            Some(temp_index.path()),
            None,
        )?;
    } else {
        git_output(
            root,
            &["read-tree", "--empty"],
            Some(temp_index.path()),
            None,
        )?;
    }
    git_output(
        root,
        &["add", "-A", "--", "."],
        Some(temp_index.path()),
        None,
    )?;
    let worktree_tree_oid = git_text(root, &["write-tree"], Some(temp_index.path()))?;
    let index_commit_oid =
        git_commit_tree(root, &index_tree_oid, None, "Ganbaru Chat index checkpoint")?;
    let git_object_id = git_commit_tree(
        root,
        &worktree_tree_oid,
        Some(&index_commit_oid),
        "Ganbaru Chat worktree checkpoint",
    )?;
    let hidden_ref_name = checkpoint_ref(thread_id, checkpoint_id);
    git_output(
        root,
        &["update-ref", &hidden_ref_name, &git_object_id],
        None,
        None,
    )?;
    let after = repository_fingerprint(root)?;
    if before != after {
        let _ = delete_exact_ref(root, &hidden_ref_name, &git_object_id);
        return Err(checkpoint_error(
            "Git checkpoint capture changed the branch, index, or worktree",
        ));
    }
    let changed_files = previous_oid
        .map(|previous| changed_file_summaries(root, previous, &git_object_id))
        .transpose()?
        .unwrap_or_default();
    Ok(CapturedCheckpoint {
        id: checkpoint_id.clone(),
        hidden_ref_name,
        git_object_id,
        index_commit_oid,
        index_tree_oid,
        worktree_tree_oid,
        head_oid,
        head_ref,
        index_fingerprint,
        changed_files,
    })
}

fn changed_file_summaries(
    root: &Path,
    pre_oid: &str,
    post_oid: &str,
) -> ChatResult<Vec<ChangedFileSummary>> {
    let pre = StoredCheckpoint {
        id: ChatCheckpointId::new("checkpoint:pre").map_err(checkpoint_error)?,
        thread_id: ChatThreadId::new("thread:pre").map_err(checkpoint_error)?,
        turn_count: 0,
        repository_identity: String::new(),
        hidden_ref_name: String::new(),
        git_object_id: pre_oid.to_string(),
        index_tree_oid: String::new(),
        worktree_tree_oid: String::new(),
        head_oid: None,
        head_ref: None,
    };
    let post = StoredCheckpoint {
        git_object_id: post_oid.to_string(),
        ..pre.clone()
    };
    Ok(diff_files(root, &pre, &post)?
        .into_iter()
        .map(|file| ChangedFileSummary {
            relative_path: file.relative_path,
            previous_relative_path: file.previous_relative_path,
            additions: file.additions,
            deletions: file.deletions,
            binary: file.binary,
            status: file.status,
        })
        .collect())
}

fn diff_numstat(root: &Path, pre_oid: &str, post_oid: &str) -> ChatResult<DiffLineCounts> {
    let output = git_output(
        root,
        &[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--numstat",
            "-z",
            "--find-renames",
            pre_oid,
            post_oid,
        ],
        None,
        None,
    )?;
    let mut result = BTreeMap::new();
    let mut fields = output.stdout.split(|byte| *byte == 0).peekable();
    while let Some(field) = fields.next() {
        if field.is_empty() {
            continue;
        }
        let text = String::from_utf8_lossy(field);
        let mut parts = text.splitn(3, '\t');
        let additions = parse_stat(parts.next());
        let deletions = parse_stat(parts.next());
        let path = parts.next().unwrap_or_default();
        let final_path = if path.is_empty() {
            let _old = fields.next();
            fields
                .next()
                .and_then(|value| std::str::from_utf8(value).ok())
                .unwrap_or_default()
        } else {
            path
        };
        if !final_path.is_empty() {
            result.insert(final_path.to_string(), (additions, deletions));
        }
    }
    Ok(result)
}

fn parse_name_status(bytes: &[u8], stats: &DiffLineCounts) -> ChatResult<Vec<ChatChangedFileRead>> {
    let fields = bytes
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let mut index = 0;
    let mut result = Vec::new();
    while index < fields.len() {
        let status = std::str::from_utf8(fields[index]).map_err(checkpoint_error)?;
        index += 1;
        let previous_relative_path = if status.starts_with('R') || status.starts_with('C') {
            let previous = field_path(fields.get(index))?;
            index += 1;
            Some(previous)
        } else {
            None
        };
        let relative_path = field_path(fields.get(index))?;
        index += 1;
        let (additions, deletions) = stats.get(&relative_path).copied().unwrap_or((None, None));
        result.push(ChatChangedFileRead {
            relative_path,
            previous_relative_path,
            status: match status.chars().next() {
                Some('A') => "added",
                Some('M') => "modified",
                Some('D') => "deleted",
                Some('R') | Some('C') => "renamed",
                Some('T') => "type_changed",
                _ => "unknown",
            }
            .to_string(),
            additions,
            deletions,
            binary: additions.is_none() || deletions.is_none(),
            provider_reported: false,
            git_observed: true,
        });
    }
    Ok(result)
}

fn repository_fingerprint(root: &Path) -> ChatResult<String> {
    let mut hasher = Sha256::new();
    for arguments in [
        vec!["symbolic-ref", "-q", "HEAD"],
        vec!["rev-parse", "-q", "--verify", "HEAD^{commit}"],
        vec!["ls-files", "--stage", "-z"],
        vec!["status", "--porcelain=v2", "-z", "--untracked-files=all"],
    ] {
        let output = git_output(root, &arguments, None, Some(&[0, 1]))?;
        hasher.update(&output.stdout);
        hasher.update([0]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn verify_git_root(root: &Path) -> ChatResult<()> {
    let toplevel = git_text(
        root,
        &["rev-parse", "--path-format=absolute", "--show-toplevel"],
        None,
    )?;
    let canonical = fs::canonicalize(toplevel).map_err(checkpoint_error)?;
    if canonical != root {
        return Err(ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "Chat checkpoints require the bound folder to be the Git worktree root",
            true,
        ));
    }
    Ok(())
}

fn git_commit_tree(
    root: &Path,
    tree_oid: &str,
    parent_oid: Option<&str>,
    message: &str,
) -> ChatResult<String> {
    let mut arguments = vec!["commit-tree", tree_oid];
    if let Some(parent) = parent_oid {
        arguments.extend(["-p", parent]);
    }
    git_text_with_input(root, &arguments, format!("{message}\n").as_bytes())
}

fn git_text(root: &Path, arguments: &[&str], index: Option<&Path>) -> ChatResult<String> {
    let output = git_output(root, arguments, index, None)?;
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_string())
        .map_err(checkpoint_error)
}

fn git_text_with_input(root: &Path, arguments: &[&str], input: &[u8]) -> ChatResult<String> {
    let output = git_output_with_input(root, arguments, None, input)?;
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_string())
        .map_err(checkpoint_error)
}

fn optional_git_text(root: &Path, arguments: &[&str]) -> ChatResult<Option<String>> {
    let output = git_output(root, arguments, None, Some(&[0, 1]))?;
    if output.status.success() {
        Ok(Some(
            String::from_utf8(output.stdout)
                .map_err(checkpoint_error)?
                .trim()
                .to_string(),
        ))
    } else {
        Ok(None)
    }
}

fn git_output(
    root: &Path,
    arguments: &[&str],
    index: Option<&Path>,
    accepted_codes: Option<&[i32]>,
) -> ChatResult<Output> {
    git_output_with_input_codes(root, arguments, index, None, accepted_codes)
}

fn git_output_with_input(
    root: &Path,
    arguments: &[&str],
    index: Option<&Path>,
    input: &[u8],
) -> ChatResult<Output> {
    git_output_with_input_codes(root, arguments, index, Some(input), None)
}

fn git_output_with_input_codes(
    root: &Path,
    arguments: &[&str],
    index: Option<&Path>,
    input: Option<&[u8]>,
    accepted_codes: Option<&[i32]>,
) -> ChatResult<Output> {
    let mut command = Command::new("git");
    command
        .args(["-C"])
        .arg(root)
        .args(arguments)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .env("LC_ALL", "C")
        .env("GIT_AUTHOR_NAME", "Ganbaru AI")
        .env("GIT_AUTHOR_EMAIL", "local@ganbaru.invalid")
        .env("GIT_COMMITTER_NAME", "Ganbaru AI")
        .env("GIT_COMMITTER_EMAIL", "local@ganbaru.invalid")
        .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(index) = index {
        command.env("GIT_INDEX_FILE", index);
    }
    let mut child = command.spawn().map_err(checkpoint_error)?;
    if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
        stdin.write_all(input).map_err(checkpoint_error)?;
    }
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| checkpoint_error("Git stdout unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| checkpoint_error("Git stderr unavailable"))?;
    let stdout_reader = bounded_reader(stdout);
    let stderr_reader = bounded_reader(stderr);
    let status = child.wait().map_err(checkpoint_error)?;
    let stdout = stdout_reader
        .join()
        .map_err(checkpoint_error)?
        .map_err(checkpoint_error)?;
    let stderr = stderr_reader
        .join()
        .map_err(checkpoint_error)?
        .map_err(checkpoint_error)?;
    if stdout.len() > MAX_GIT_OUTPUT_BYTES || stderr.len() > MAX_GIT_OUTPUT_BYTES {
        return Err(checkpoint_error("Git output exceeds the safety limit"));
    }
    let code = status.code().unwrap_or(-1);
    if !status.success() && !accepted_codes.is_some_and(|codes| codes.contains(&code)) {
        return Err(checkpoint_error("Git checkpoint command failed"));
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

fn bounded_reader<R: std::io::Read + Send + 'static>(
    reader: R,
) -> std::thread::JoinHandle<std::io::Result<Vec<u8>>> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        reader
            .take((MAX_GIT_OUTPUT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

async fn latest_checkpoint_oid(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<Option<String>> {
    sqlx::query_scalar(
        "SELECT git_object_id FROM chat_checkpoints
         WHERE thread_id = ? AND status = 'available' AND invalidated_at IS NULL
         ORDER BY turn_count DESC LIMIT 1",
    )
    .bind(thread_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)
}

struct TemporaryIndex {
    path: PathBuf,
}

impl TemporaryIndex {
    fn new() -> ChatResult<Self> {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ganbaru-chat-index-{}-{sequence}",
            std::process::id()
        ));
        let file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(checkpoint_error)?;
        drop(file);
        fs::remove_file(&path).map_err(checkpoint_error)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryIndex {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        let lock_path = self.path.with_extension("lock");
        let _ = fs::remove_file(lock_path);
    }
}

fn checkpoint_id(
    thread_id: &ChatThreadId,
    turn_id: Option<&ChatTurnId>,
    turn_count: u64,
    now: &UtcTimestamp,
) -> ChatResult<ChatCheckpointId> {
    let mut hasher = Sha256::new();
    hasher.update(thread_id.as_str());
    hasher.update([0]);
    hasher.update(turn_id.map(ChatTurnId::as_str).unwrap_or("initial"));
    hasher.update([0]);
    hasher.update(turn_count.to_le_bytes());
    hasher.update(now.as_str());
    ChatCheckpointId::new(format!("checkpoint:{:x}", hasher.finalize())).map_err(checkpoint_error)
}

fn checkpoint_ref(thread_id: &ChatThreadId, checkpoint_id: &ChatCheckpointId) -> String {
    format!(
        "{REF_PREFIX}{}/{}",
        short_hash(thread_id.as_str()),
        short_hash(checkpoint_id.as_str())
    )
}

fn short_hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))[..32].to_string()
}

fn valid_hidden_ref(reference: &str) -> bool {
    reference.starts_with(REF_PREFIX)
        && !reference[REF_PREFIX.len()..].is_empty()
        && !reference.chars().any(char::is_whitespace)
        && !reference.contains("..")
        && !reference.contains("@{")
        && !reference.ends_with('/')
}

fn validate_diff_path(path: &str) -> ChatResult<()> {
    if path.is_empty()
        || path.len() > 4_096
        || Path::new(path).is_absolute()
        || path.contains('\u{005c}')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.chars().any(char::is_control)
    {
        return Err(ChatError::validation(
            "relativePath",
            "Diff path must be a normalized workspace-relative path",
        ));
    }
    Ok(())
}

fn field_path(field: Option<&&[u8]>) -> ChatResult<String> {
    let path = field.ok_or_else(|| checkpoint_error("Git diff metadata is incomplete"))?;
    let path = std::str::from_utf8(path).map_err(checkpoint_error)?;
    validate_diff_path(path)?;
    Ok(path.to_string())
}

fn parse_stat(value: Option<&str>) -> Option<u64> {
    value.and_then(|value| value.parse().ok())
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn required_string(row: &sqlx::sqlite::SqliteRow, column: &str) -> ChatResult<String> {
    row.try_get::<Option<String>, _>(column)
        .map_err(persistence_error)?
        .ok_or_else(|| checkpoint_error("Stored Chat checkpoint is incomplete"))
}

fn u64_column(row: &sqlx::sqlite::SqliteRow, column: &str) -> ChatResult<u64> {
    u64::try_from(row.try_get::<i64, _>(column).map_err(persistence_error)?)
        .map_err(checkpoint_error)
}

fn i64_value(value: u64) -> ChatResult<i64> {
    i64::try_from(value).map_err(|_| ChatError::validation("number", "Value is too large"))
}

fn parse_id(value: String) -> ChatResult<ChatCheckpointId> {
    ChatCheckpointId::new(value).map_err(checkpoint_error)
}

fn parse_thread_id(value: String) -> ChatResult<ChatThreadId> {
    ChatThreadId::new(value).map_err(checkpoint_error)
}

fn checkpoint_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        "Git checkpoint operation failed safely",
        true,
    )
}

fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat checkpoint persistence failed",
        true,
    )
}

fn serialization_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat checkpoint metadata could not be encoded",
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::models::{ChatWorkspaceId, RepositoryKind};
    use crate::chat::workspace::AuthorizedWorkspace;
    use std::collections::BTreeSet;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestRepository(PathBuf);

    impl TestRepository {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock should be valid")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ganbaru-chat-checkpoint-test-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("test repository should be created");
            command(&path, &["init", "-q"]);
            command(&path, &["config", "user.name", "Ganbaru Test"]);
            command(&path, &["config", "user.email", "test@ganbaru.invalid"]);
            Self(fs::canonicalize(path).expect("test repository path should canonicalize"))
        }

        fn write(&self, relative: &str, bytes: &[u8]) {
            let path = self.0.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("test parent should be created");
            }
            fs::write(path, bytes).expect("test file should be written");
        }

        fn commit_all(&self) {
            command(&self.0, &["add", "-A", "--", "."]);
            command(&self.0, &["commit", "-q", "-m", "fixture"]);
        }

        fn authorized(&self, identity: &str) -> AuthorizedWorkspace {
            AuthorizedWorkspace {
                workspace_id: ChatWorkspaceId::new("workspace:test")
                    .expect("workspace ID should be valid"),
                canonical_path: self.0.clone(),
                repository_kind: RepositoryKind::Git,
                repository_identity: Some(identity.to_string()),
            }
        }
    }

    impl Drop for TestRepository {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn command(root: &Path, arguments: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(arguments)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .expect("Git fixture command should start");
        assert!(
            output.status.success(),
            "Git fixture command failed: {arguments:?}"
        );
        String::from_utf8(output.stdout)
            .expect("Git fixture output should be UTF-8")
            .trim()
            .to_string()
    }

    fn capture(
        repository: &TestRepository,
        suffix: &str,
        previous: Option<&str>,
    ) -> CapturedCheckpoint {
        let thread = ChatThreadId::new("thread:test").expect("thread ID should be valid");
        let checkpoint = ChatCheckpointId::new(format!("checkpoint:{suffix}"))
            .expect("checkpoint ID should be valid");
        capture_git(&repository.0, &thread, &checkpoint, previous)
            .expect("checkpoint capture should succeed")
    }

    fn stored(captured: &CapturedCheckpoint, identity: &str, turn_count: u64) -> StoredCheckpoint {
        StoredCheckpoint {
            id: captured.id.clone(),
            thread_id: ChatThreadId::new("thread:test").expect("thread ID should be valid"),
            turn_count,
            repository_identity: identity.to_string(),
            hidden_ref_name: captured.hidden_ref_name.clone(),
            git_object_id: captured.git_object_id.clone(),
            index_tree_oid: captured.index_tree_oid.clone(),
            worktree_tree_oid: captured.worktree_tree_oid.clone(),
            head_oid: captured.head_oid.clone(),
            head_ref: captured.head_ref.clone(),
        }
    }

    #[test]
    fn checkpoint_capture_preserves_head_index_staging_and_worktree() {
        let repository = TestRepository::new();
        repository.write("tracked.txt", b"base\n");
        repository.commit_all();
        repository.write("tracked.txt", b"staged\n");
        command(&repository.0, &["add", "tracked.txt"]);
        repository.write("unstaged.txt", b"untracked\n");
        let before = repository_fingerprint(&repository.0).expect("fingerprint should succeed");
        let head = command(&repository.0, &["rev-parse", "HEAD"]);
        let captured = capture(&repository, "preserve", None);
        let after = repository_fingerprint(&repository.0).expect("fingerprint should succeed");

        assert_eq!(before, after);
        assert_eq!(head, command(&repository.0, &["rev-parse", "HEAD"]));
        assert_eq!(
            captured.git_object_id,
            command(&repository.0, &["rev-parse", &captured.hidden_ref_name])
        );
        assert_eq!(
            command(&repository.0, &["diff", "--cached", "--name-only"]),
            "tracked.txt"
        );
        assert!(repository.0.join("unstaged.txt").exists());
    }

    #[cfg(unix)]
    #[test]
    fn diff_covers_staged_unstaged_untracked_deleted_renamed_mode_binary_and_whitespace() {
        use std::os::unix::fs::PermissionsExt;

        let repository = TestRepository::new();
        for name in [
            "staged.txt",
            "unstaged.txt",
            "deleted.txt",
            "renamed.txt",
            "mode.sh",
            "whitespace.txt",
        ] {
            repository.write(name, format!("{name} base\n").as_bytes());
        }
        repository.write("binary.bin", &[0, 1, 2, 3]);
        repository.commit_all();
        let pre = capture(&repository, "pre", None);

        repository.write("staged.txt", b"staged change\n");
        command(&repository.0, &["add", "staged.txt"]);
        repository.write("unstaged.txt", b"unstaged change\n");
        repository.write("untracked.txt", b"new\n");
        fs::remove_file(repository.0.join("deleted.txt")).expect("file should be deleted");
        command(&repository.0, &["mv", "renamed.txt", "moved.txt"]);
        let mut permissions = fs::metadata(repository.0.join("mode.sh"))
            .expect("mode fixture should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(repository.0.join("mode.sh"), permissions).expect("mode should change");
        repository.write("binary.bin", &[0, 9, 8, 7]);
        repository.write("whitespace.txt", b"whitespace.txt base   \n");

        let before = repository_fingerprint(&repository.0).expect("fingerprint should succeed");
        let post = capture(&repository, "post", Some(&pre.git_object_id));
        assert_eq!(
            before,
            repository_fingerprint(&repository.0).expect("fingerprint should succeed")
        );

        let files = diff_files(
            &repository.0,
            &stored(&pre, "identity", 0),
            &stored(&post, "identity", 1),
        )
        .expect("diff should succeed");
        let paths = files
            .iter()
            .map(|file| file.relative_path.as_str())
            .collect::<BTreeSet<_>>();
        for expected in [
            "binary.bin",
            "deleted.txt",
            "mode.sh",
            "moved.txt",
            "staged.txt",
            "unstaged.txt",
            "untracked.txt",
            "whitespace.txt",
        ] {
            assert!(paths.contains(expected), "missing {expected}");
        }
        let renamed = files
            .iter()
            .find(|file| file.relative_path == "moved.txt")
            .expect("rename should exist");
        assert_eq!(renamed.status, "renamed");
        assert_eq!(
            renamed.previous_relative_path.as_deref(),
            Some("renamed.txt")
        );
        assert!(
            files
                .iter()
                .find(|file| file.relative_path == "binary.bin")
                .expect("binary should exist")
                .binary
        );

        let normal = file_diff(
            &repository.0,
            &stored(&pre, "identity", 0),
            &stored(&post, "identity", 1),
            "whitespace.txt",
            false,
        )
        .expect("normal diff should succeed");
        let ignored = file_diff(
            &repository.0,
            &stored(&pre, "identity", 0),
            &stored(&post, "identity", 1),
            "whitespace.txt",
            true,
        )
        .expect("whitespace diff should succeed");
        assert!(normal
            .patch
            .as_deref()
            .is_some_and(|patch| !patch.is_empty()));
        assert_eq!(ignored.patch.as_deref(), Some(""));
    }

    #[test]
    fn restore_reinstates_worktree_and_real_index_without_moving_head() {
        let repository = TestRepository::new();
        repository.write("staged.txt", b"base\n");
        repository.write("worktree.txt", b"base\n");
        repository.commit_all();
        repository.write("staged.txt", b"checkpoint staged\n");
        command(&repository.0, &["add", "staged.txt"]);
        repository.write("worktree.txt", b"checkpoint worktree\n");
        repository.write("checkpoint-only.txt", b"checkpoint\n");
        let target = capture(&repository, "restore-target", None);

        repository.write("staged.txt", b"later staged\n");
        command(&repository.0, &["add", "staged.txt"]);
        repository.write("worktree.txt", b"later worktree\n");
        fs::remove_file(repository.0.join("checkpoint-only.txt")).expect("file should be removed");
        repository.write("later-only.txt", b"later\n");
        let current = current_git_snapshot(&repository.0).expect("current snapshot should succeed");
        let head = current.head_oid.clone();

        restore_git_snapshot(&repository.0, &current, &stored(&target, "identity", 0))
            .expect("restore should succeed");
        let restored =
            current_git_snapshot(&repository.0).expect("restored snapshot should succeed");
        assert_eq!(restored.worktree_tree_oid, target.worktree_tree_oid);
        assert_eq!(restored.index_tree_oid, target.index_tree_oid);
        assert_eq!(restored.head_oid, head);
        assert_eq!(
            fs::read(repository.0.join("worktree.txt")).expect("file should read"),
            b"checkpoint worktree\n"
        );
        assert!(repository.0.join("checkpoint-only.txt").exists());
        assert!(!repository.0.join("later-only.txt").exists());
    }

    #[test]
    fn ref_validation_rejects_identity_changes_missing_refs_and_wrong_cleanup_oids() {
        let repository = TestRepository::new();
        repository.write("tracked.txt", b"base\n");
        repository.commit_all();
        let captured = capture(&repository, "validation", None);
        let checkpoint = stored(&captured, "identity-a", 0);

        let identity_error = verify_checkpoint(&repository.authorized("identity-b"), &checkpoint)
            .expect_err("identity mismatch should fail");
        assert_eq!(identity_error.code, ChatErrorCode::ConfigurationInvalid);
        let wrong_oid = command(&repository.0, &["rev-parse", "HEAD"]);
        assert!(delete_exact_ref(&repository.0, &captured.hidden_ref_name, &wrong_oid).is_err());
        verify_checkpoint(&repository.authorized("identity-a"), &checkpoint)
            .expect("wrong expected OID must retain ref");
        delete_exact_ref(
            &repository.0,
            &captured.hidden_ref_name,
            &captured.git_object_id,
        )
        .expect("exact cleanup should succeed");
        let missing = verify_checkpoint(&repository.authorized("identity-a"), &checkpoint)
            .expect_err("missing ref should fail");
        assert_eq!(missing.code, ChatErrorCode::NotFound);
    }

    #[test]
    fn stale_restore_snapshot_aborts_without_changing_files() {
        let repository = TestRepository::new();
        repository.write("tracked.txt", b"base\n");
        repository.commit_all();
        let target = capture(&repository, "stale-target", None);
        repository.write("tracked.txt", b"preview state\n");
        let preview = current_git_snapshot(&repository.0).expect("preview should succeed");
        repository.write("tracked.txt", b"changed after preview\n");

        let error = restore_git_snapshot(&repository.0, &preview, &stored(&target, "identity", 0))
            .expect_err("stale preview should abort");
        assert_eq!(error.code, ChatErrorCode::StaleRevision);
        assert_eq!(
            fs::read(repository.0.join("tracked.txt")).expect("file should read"),
            b"changed after preview\n"
        );
    }
}
