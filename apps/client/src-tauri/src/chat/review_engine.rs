//! Immutable, provider-neutral Chat review snapshots and guarded Git actions.

use super::checkpoints::{read_stored_checkpoint, verify_checkpoint, StoredCheckpoint};
use super::events::{CanonicalEvent, ChangedFileSummary};
use super::git_service;
use super::models::{
    ChatCheckpointId, ChatError, ChatErrorCode, ChatResult, ChatThreadId, ChatTurnId,
    ProjectWorkingFolderId,
};
use super::repository::workspaces;
use super::workspace::{
    authorize_workspace, AuthorizedWorkingFolder, WorkingFolderAuthorizationOperation,
};
use crate::db_path;
use crate::projects::working_folders::read_active_working_folder_scope;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fs;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_SNAPSHOTS: usize = 24;
const MAX_COMPLETED_OPERATIONS: usize = 128;
const SNAPSHOT_LIFETIME: Duration = Duration::from_secs(30 * 60);
const DEFAULT_PATCH_PAGE_BYTES: usize = 1024 * 1024;
const MIN_PATCH_PAGE_BYTES: usize = 64 * 1024;
const MAX_PATCH_PAGE_BYTES: usize = 4 * 1024 * 1024;
const MAX_CONTEXT_LINES: u32 = 100;
const MAX_FILE_SELECTION: usize = 100;
const MAX_COMMENT_SELECTION_BYTES: usize = 1024 * 1024;
const MAX_PATCHES_PER_RESPONSE: usize = 64;
const MAX_PATCH_HUNKS_PER_RESPONSE: usize = 2_048;
const PATCH_RESPONSE_ENVELOPE_BYTES: usize = 4 * 1024;
const MAX_PATCH_SPOOL_BYTES: usize = 512 * 1024 * 1024;
const MAX_REVIEW_OBJECT_BYTES: u64 = 512 * 1024 * 1024;
static REVIEW_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewWorkingTreeMode {
    Staged,
    Unstaged,
    All,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewCheckpointRange {
    Turn,
    Thread,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewBranchComparison {
    MergeBase,
    Direct,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ReviewDiffSource {
    WorkingTree {
        mode: ReviewWorkingTreeMode,
    },
    Checkpoint {
        range: ReviewCheckpointRange,
        turn_id: Option<ChatTurnId>,
    },
    Commit {
        revision: String,
    },
    Branch {
        base_ref: Option<String>,
        head_ref: String,
        comparison: ReviewBranchComparison,
    },
    ChangeRequest {
        provider: String,
        repository_slug: String,
        number: u64,
    },
    ProviderTurn {
        turn_id: ChatTurnId,
    },
}

impl ReviewDiffSource {
    pub(crate) fn kind_wire(&self) -> &'static str {
        match self {
            Self::WorkingTree { .. } => "working_tree",
            Self::Checkpoint { .. } => "checkpoint",
            Self::Commit { .. } => "commit",
            Self::Branch { .. } => "branch",
            Self::ChangeRequest { .. } => "change_request",
            Self::ProviderTurn { .. } => "provider_turn",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenChatReviewRequest {
    pub thread_id: Option<ChatThreadId>,
    pub working_folder_id: ProjectWorkingFolderId,
    pub execution_environment_id: Option<String>,
    pub source: ReviewDiffSource,
    #[serde(default)]
    pub ignore_whitespace: bool,
    #[serde(default = "default_context_lines")]
    pub context_lines: u32,
    pub preferred_relative_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadChatReviewPatchesRequest {
    pub thread_id: Option<ChatThreadId>,
    pub working_folder_id: ProjectWorkingFolderId,
    pub execution_environment_id: Option<String>,
    pub snapshot_id: String,
    pub review_revision: String,
    pub file_ids: Vec<String>,
    pub continuation_cursor: Option<String>,
    pub byte_limit: Option<u64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAction {
    Stage,
    Unstage,
    Discard,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChatReviewActionRequest {
    pub thread_id: Option<ChatThreadId>,
    pub snapshot_id: String,
    pub working_folder_id: ProjectWorkingFolderId,
    pub execution_environment_id: Option<String>,
    pub expected_review_revision: String,
    pub file_id: Option<String>,
    #[serde(default)]
    pub hunk_ids: Vec<String>,
    pub operation: ReviewAction,
    #[serde(default)]
    pub confirmed: bool,
    pub client_operation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFileFlagsRead {
    pub binary: bool,
    pub submodule: bool,
    pub conflict: bool,
    pub mode_only: bool,
    pub pure_rename: bool,
    pub untracked: bool,
    pub symlink: bool,
    pub provider_reported: bool,
    pub git_observed: bool,
    pub read_only: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFileCapabilitiesRead {
    pub stage: bool,
    pub unstage: bool,
    pub discard: bool,
    pub comment: bool,
    pub open_editor: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFileCapabilityReasonsRead {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unstage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_editor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFileRead {
    pub file_id: String,
    pub relative_path: String,
    pub previous_relative_path: Option<String>,
    pub status: String,
    pub additions: Option<u64>,
    pub deletions: Option<u64>,
    pub flags: ReviewFileFlagsRead,
    pub capabilities: ReviewFileCapabilitiesRead,
    pub capability_reasons: ReviewFileCapabilityReasonsRead,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewTotalsRead {
    pub files: u64,
    pub additions: u64,
    pub deletions: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewFreshness {
    Current,
    Outdated,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatReviewOpenRead {
    pub snapshot_id: String,
    pub review_revision: String,
    pub source: ReviewDiffSource,
    pub source_label: String,
    pub files: Vec<ReviewFileRead>,
    pub totals: ReviewTotalsRead,
    pub preferred_patch: Option<ReviewFilePatchRead>,
    pub freshness: ReviewFreshness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewPatchState {
    Complete,
    Partial,
    Binary,
    OversizedHunk,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewHunkRead {
    pub hunk_id: String,
    pub old_start: u64,
    pub old_count: u64,
    pub new_start: u64,
    pub new_count: u64,
    pub state: ReviewPatchState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFilePatchRead {
    pub file_id: String,
    pub patch: Option<String>,
    pub hunks: Vec<ReviewHunkRead>,
    pub continuation_cursor: Option<String>,
    pub state: ReviewPatchState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatReviewPatchesRead {
    pub patches: Vec<ReviewFilePatchRead>,
    pub continuation_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatReviewActionResultRead {
    pub snapshot: ChatReviewOpenRead,
}

#[derive(Default)]
pub struct ChatReviewRegistry {
    snapshots: Mutex<SnapshotStore>,
}

#[derive(Default)]
struct SnapshotStore {
    values: HashMap<String, Arc<ReviewSnapshot>>,
    order: VecDeque<String>,
    completed: HashMap<String, CompletedReviewOperation>,
    completed_order: VecDeque<String>,
}

#[derive(Clone)]
struct CompletedReviewOperation {
    database_identity: String,
    thread_id: Option<ChatThreadId>,
    working_folder_id: ProjectWorkingFolderId,
    environment_id: String,
    request_fingerprint: String,
    result: ChatReviewActionResultRead,
}

struct ReviewSnapshot {
    database_identity: String,
    snapshot_id: String,
    review_revision: String,
    thread_id: Option<ChatThreadId>,
    working_folder_id: ProjectWorkingFolderId,
    environment_id: String,
    root: PathBuf,
    source: ReviewDiffSource,
    source_label: String,
    before_oid: Option<String>,
    after_oid: Option<String>,
    context_lines: u32,
    ignore_whitespace: bool,
    files: Vec<ReviewFileInternal>,
    provider_patches: HashMap<String, String>,
    patch_cache: tokio::sync::Mutex<HashMap<String, Arc<PatchIndex>>>,
    object_store: Option<Arc<ReviewObjectStore>>,
    created_at: Instant,
}

#[derive(Clone)]
struct ReviewFileInternal {
    read: ReviewFileRead,
}

#[derive(Clone)]
struct SnapshotMaterial {
    before_oid: String,
    after_oid: String,
    label: String,
    status: git_service::GitStatusRead,
    object_store: Option<Arc<ReviewObjectStore>>,
}

#[derive(Clone)]
struct ParsedHunk {
    read: ReviewHunkRead,
    text: String,
}

struct ParsedPatch {
    preamble: String,
    hunks: Vec<ParsedHunk>,
}

enum PatchIndex {
    Memory(ParsedPatch),
    Spool(SpoolPatchIndex),
}

#[derive(Clone)]
struct SpoolPatchIndex {
    file: Arc<ReviewTemporaryFile>,
    byte_size: u64,
    preamble_end: u64,
    hunks: Vec<IndexedHunk>,
}

#[derive(Clone)]
struct IndexedHunk {
    read: ReviewHunkRead,
    start: u64,
    end: u64,
}

struct ReviewObjectStore {
    root: PathBuf,
    objects: PathBuf,
    index: PathBuf,
    alternate_objects: PathBuf,
}

struct ReviewTemporaryFile {
    root: PathBuf,
    path: PathBuf,
}

#[derive(Default)]
struct RawFileMetadata {
    old_mode: String,
    new_mode: String,
}

type ReviewNumstat = BTreeMap<String, (Option<u64>, Option<u64>)>;

pub(crate) struct ResolvedReviewSelection {
    pub relative_path: String,
    pub previous_relative_path: Option<String>,
    pub content_revision: String,
    pub selected_text: String,
    pub source: ReviewDiffSource,
}

pub(crate) struct ResolveReviewSelectionRequest<'a> {
    pub thread_id: &'a ChatThreadId,
    pub snapshot_id: &'a str,
    pub review_revision: &'a str,
    pub file_id: &'a str,
    pub side: &'a str,
    pub start_line: u64,
    pub start_column: u64,
    pub end_line: u64,
    pub end_column: u64,
}

#[tauri::command]
pub async fn chat_open_review(
    app: tauri::AppHandle,
    state: tauri::State<'_, ChatReviewRegistry>,
    db_url: String,
    request: OpenChatReviewRequest,
) -> ChatResult<ChatReviewOpenRead> {
    let mut request = request;
    validate_open_request(&request)?;
    request.context_lines = request.context_lines.max(3);
    let pool = chat_pool(app.clone(), db_url).await?;
    let database_identity = database_identity(&pool).await?;
    open_review(&app, &state, &pool, &database_identity, request).await
}

#[tauri::command]
pub async fn chat_read_review_patches(
    app: tauri::AppHandle,
    state: tauri::State<'_, ChatReviewRegistry>,
    db_url: String,
    request: ReadChatReviewPatchesRequest,
) -> ChatResult<ChatReviewPatchesRead> {
    validate_patch_request(&request)?;
    let pool = chat_pool(app.clone(), db_url).await?;
    let database_identity = database_identity(&pool).await?;
    let snapshot = state.snapshot(
        &request.snapshot_id,
        Some(&request.thread_id),
        &database_identity,
    )?;
    require_request_ownership(
        &pool,
        &snapshot,
        &request.working_folder_id,
        request.execution_environment_id.as_deref(),
    )
    .await?;
    require_snapshot_revision(&snapshot, &request.review_revision)?;
    authorize_snapshot(
        &app,
        &pool,
        &snapshot,
        WorkingFolderAuthorizationOperation::Diff,
    )
    .await?;
    let maximum = request
        .byte_limit
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(DEFAULT_PATCH_PAGE_BYTES)
        .clamp(MIN_PATCH_PAGE_BYTES, MAX_PATCH_PAGE_BYTES);
    let requested = request.file_ids.clone();
    if let Some(cursor) = request.continuation_cursor.as_deref() {
        if !requested
            .iter()
            .any(|file_id| parse_cursor(cursor, file_id).is_some())
        {
            return Err(ChatError::validation(
                "continuationCursor",
                "Review continuation cursor is invalid for the requested files",
            ));
        }
    }
    let mut patches = Vec::new();
    let mut remaining = maximum.saturating_sub(PATCH_RESPONSE_ENVELOPE_BYTES);
    let start_index = request
        .continuation_cursor
        .as_deref()
        .and_then(|cursor| {
            requested
                .iter()
                .position(|file_id| parse_cursor(cursor, file_id).is_some())
        })
        .unwrap_or(0);
    let mut response_cursor = None;
    for (index, file_id) in requested.iter().enumerate().skip(start_index) {
        if patches.len() >= MAX_PATCHES_PER_RESPONSE
            || (remaining < MIN_PATCH_PAGE_BYTES && !patches.is_empty())
        {
            response_cursor = Some(format!("{file_id}/0"));
            break;
        }
        let start_hunk = if index == start_index {
            request
                .continuation_cursor
                .as_deref()
                .and_then(|cursor| parse_cursor(cursor, file_id))
                .unwrap_or(0)
        } else {
            0
        };
        let mut patch = read_patch_page(&snapshot, file_id, start_hunk, remaining).await?;
        let mut encoded_bytes = serde_json::to_vec(&patch)
            .map_err(|_| corrupt_data())?
            .len();
        if encoded_bytes > remaining && !patches.is_empty() {
            response_cursor = Some(format!("{file_id}/{start_hunk}"));
            break;
        }
        if encoded_bytes > remaining {
            patch = unavailable_patch(file_id);
            encoded_bytes = serde_json::to_vec(&patch)
                .map_err(|_| corrupt_data())?
                .len();
        }
        remaining = remaining.saturating_sub(encoded_bytes);
        response_cursor = patch.continuation_cursor.clone();
        patches.push(patch);
        if response_cursor.is_some() {
            break;
        }
        if index + 1 < requested.len() && remaining < MIN_PATCH_PAGE_BYTES {
            response_cursor = Some(format!("{}/0", requested[index + 1]));
            break;
        }
    }
    Ok(ChatReviewPatchesRead {
        patches,
        continuation_cursor: response_cursor,
    })
}

#[tauri::command]
pub async fn chat_apply_review_action(
    app: tauri::AppHandle,
    state: tauri::State<'_, ChatReviewRegistry>,
    mutations: tauri::State<'_, super::workspace_mutation::ChatWorkspaceMutationRegistry>,
    observers: tauri::State<'_, super::workspace_observer::ChatWorkspaceObserverRegistry>,
    db_url: String,
    request: ApplyChatReviewActionRequest,
) -> ChatResult<ChatReviewActionResultRead> {
    validate_action_request(&request)?;
    let request_fingerprint = action_request_fingerprint(&request)?;
    let pool = chat_pool(app.clone(), db_url).await?;
    let database_identity = database_identity(&pool).await?;
    let requested_environment = resolve_requested_environment_id(
        &pool,
        &request.working_folder_id,
        request.execution_environment_id.as_deref(),
    )
    .await?;
    if let Some(result) = state.completed(
        &request.client_operation_id,
        &request.thread_id,
        &request.working_folder_id,
        &requested_environment,
        &request_fingerprint,
        &database_identity,
    )? {
        authorize_completed_action(&app, &pool, &request, &result).await?;
        return Ok(result);
    }
    let snapshot = state.snapshot(
        &request.snapshot_id,
        Some(&request.thread_id),
        &database_identity,
    )?;
    require_request_ownership(
        &pool,
        &snapshot,
        &request.working_folder_id,
        request.execution_environment_id.as_deref(),
    )
    .await?;
    require_snapshot_revision(&snapshot, &request.expected_review_revision)?;
    authorize_snapshot(
        &app,
        &pool,
        &snapshot,
        WorkingFolderAuthorizationOperation::Git,
    )
    .await?;
    let _guard = mutations.try_mutation(&snapshot.root)?;
    if let Some(result) = state.completed(
        &request.client_operation_id,
        &request.thread_id,
        &request.working_folder_id,
        &requested_environment,
        &request_fingerprint,
        &database_identity,
    )? {
        return Ok(result);
    }
    let mode = match &snapshot.source {
        ReviewDiffSource::WorkingTree { mode } => *mode,
        _ => return Err(ChatError::unsupported("This review source is read-only")),
    };
    let current = working_tree_material(&snapshot.root, mode).await?;
    let current_revision = review_revision(
        snapshot.thread_id.as_ref(),
        &snapshot.environment_id,
        &snapshot.source,
        &current.before_oid,
        &current.after_oid,
        snapshot.ignore_whitespace,
        snapshot.context_lines,
    )?;
    if current_revision != snapshot.review_revision {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The reviewed changes have changed. Refresh Review before applying this action.",
            true,
        ));
    }
    apply_action(&snapshot, &request, mode).await?;
    let affected_paths = action_paths(&snapshot, request.file_id.as_deref());
    observers.invalidate_paths(
        &snapshot.working_folder_id,
        request.execution_environment_id.as_deref(),
        affected_paths,
        true,
    );
    let refreshed_result = open_review(
        &app,
        &state,
        &pool,
        &database_identity,
        OpenChatReviewRequest {
            thread_id: request.thread_id.clone(),
            working_folder_id: snapshot.working_folder_id.clone(),
            execution_environment_id: Some(snapshot.environment_id.clone()),
            source: snapshot.source.clone(),
            ignore_whitespace: snapshot.ignore_whitespace,
            context_lines: snapshot.context_lines,
            preferred_relative_path: request.file_id.as_ref().and_then(|file_id| {
                snapshot
                    .files
                    .iter()
                    .find(|file| &file.read.file_id == file_id)
                    .map(|file| file.read.relative_path.clone())
            }),
        },
    )
    .await;
    let mut refreshed = match refreshed_result {
        Ok(refreshed) => refreshed,
        Err(_) => stale_snapshot_read(&snapshot),
    };
    if refreshed.review_revision == snapshot.review_revision {
        refreshed.freshness = ReviewFreshness::Outdated;
    }
    let result = ChatReviewActionResultRead {
        snapshot: refreshed,
    };
    state.complete(
        request.client_operation_id,
        CompletedReviewOperation {
            database_identity,
            thread_id: snapshot.thread_id.clone(),
            working_folder_id: snapshot.working_folder_id.clone(),
            environment_id: snapshot.environment_id.clone(),
            request_fingerprint,
            result: result.clone(),
        },
    )?;
    Ok(result)
}

async fn open_review(
    app: &tauri::AppHandle,
    state: &ChatReviewRegistry,
    pool: &SqlitePool,
    database_identity: &str,
    request: OpenChatReviewRequest,
) -> ChatResult<ChatReviewOpenRead> {
    let (working_folder_id, environment_id, authorized) = authorize_review_request(
        app,
        pool,
        &request,
        WorkingFolderAuthorizationOperation::Diff,
    )
    .await?;
    let (before_oid, after_oid, label, files, provider_patches, object_store) =
        match &request.source {
            ReviewDiffSource::WorkingTree { mode } => {
                let material = working_tree_material(&authorized.canonical_path, *mode).await?;
                let mut files = git_files(
                    &authorized.canonical_path,
                    &material.before_oid,
                    &material.after_oid,
                    &request.source,
                    request.ignore_whitespace,
                    material.object_store.as_deref(),
                )
                .await?;
                enrich_working_tree_files(&mut files, &material.status, &request.source);
                (
                    Some(material.before_oid),
                    Some(material.after_oid),
                    material.label,
                    files,
                    HashMap::new(),
                    material.object_store,
                )
            }
            ReviewDiffSource::Checkpoint { range, turn_id } => {
                let thread_id = request.thread_id.as_ref().ok_or_else(thread_required)?;
                let (pre, post) = checkpoint_pair(pool, thread_id, *range, turn_id.as_ref())
                    .await?
                    .ok_or_else(|| {
                        ChatError::new(
                            ChatErrorCode::NotFound,
                            "A settled checkpoint pair is not available for this review",
                            true,
                        )
                    })?;
                let pre = read_stored_checkpoint(pool, &pre).await?;
                let post = read_stored_checkpoint(pool, &post).await?;
                verify_checkpoint_pair(&authorized, thread_id, &pre, &post).await?;
                let files = git_files(
                    &authorized.canonical_path,
                    &pre.git_object_id,
                    &post.git_object_id,
                    &request.source,
                    request.ignore_whitespace,
                    None,
                )
                .await?;
                (
                    Some(pre.git_object_id),
                    Some(post.git_object_id),
                    match range {
                        ReviewCheckpointRange::Turn => "Agent turn",
                        ReviewCheckpointRange::Thread => "Entire chat",
                    }
                    .to_string(),
                    files,
                    HashMap::new(),
                    None,
                )
            }
            ReviewDiffSource::Commit { revision } => {
                validate_reference(revision, "revision")?;
                let commit = git_text(
                    &authorized.canonical_path,
                    &["rev-parse", "--verify", &format!("{revision}^{{commit}}")],
                    None,
                )
                .await?;
                let parents = git_text(
                    &authorized.canonical_path,
                    &["rev-list", "--parents", "-n", "1", &commit],
                    None,
                )
                .await?;
                let parent = parents
                    .split_whitespace()
                    .nth(1)
                    .map(ToOwned::to_owned)
                    .unwrap_or(empty_tree(&authorized.canonical_path).await?);
                let files = git_files(
                    &authorized.canonical_path,
                    &parent,
                    &commit,
                    &request.source,
                    request.ignore_whitespace,
                    None,
                )
                .await?;
                (
                    Some(parent),
                    Some(commit),
                    format!("Commit {revision}"),
                    files,
                    HashMap::new(),
                    None,
                )
            }
            ReviewDiffSource::Branch {
                base_ref,
                head_ref,
                comparison,
            } => {
                let base_ref = base_ref.as_deref().unwrap_or("HEAD");
                validate_reference(base_ref, "baseRef")?;
                validate_reference(head_ref, "headRef")?;
                let base = git_text(
                    &authorized.canonical_path,
                    &["rev-parse", "--verify", &format!("{base_ref}^{{commit}}")],
                    None,
                )
                .await?;
                let head = git_text(
                    &authorized.canonical_path,
                    &["rev-parse", "--verify", &format!("{head_ref}^{{commit}}")],
                    None,
                )
                .await?;
                let before = if *comparison == ReviewBranchComparison::MergeBase {
                    git_text(
                        &authorized.canonical_path,
                        &["merge-base", &base, &head],
                        None,
                    )
                    .await?
                } else {
                    base
                };
                let files = git_files(
                    &authorized.canonical_path,
                    &before,
                    &head,
                    &request.source,
                    request.ignore_whitespace,
                    None,
                )
                .await?;
                (
                    Some(before),
                    Some(head),
                    format!("{base_ref} to {head_ref}"),
                    files,
                    HashMap::new(),
                    None,
                )
            }
            ReviewDiffSource::ProviderTurn { turn_id } => {
                let thread_id = request.thread_id.as_ref().ok_or_else(thread_required)?;
                let (changed, patch) = provider_turn_patch(pool, thread_id, turn_id).await?;
                let mut provider_patches = HashMap::new();
                let mut files = Vec::new();
                for mut summary in changed {
                    if validate_path(&summary.relative_path).is_err() {
                        continue;
                    }
                    if summary
                        .previous_relative_path
                        .as_deref()
                        .is_some_and(|previous| validate_path(previous).is_err())
                    {
                        summary.previous_relative_path = None;
                    }
                    if let Some(file_patch) = extract_provider_file_patch(
                        &patch,
                        &summary.relative_path,
                        summary.previous_relative_path.as_deref(),
                    ) {
                        provider_patches.insert(summary.relative_path.clone(), file_patch);
                    }
                    files.push(file_from_summary(String::new(), summary, &request.source));
                }
                if files.is_empty() {
                    return Err(ChatError::new(
                        ChatErrorCode::NotFound,
                        "The provider did not report usable workspace-relative paths for this turn",
                        true,
                    ));
                }
                (
                    None,
                    None,
                    "Provider-reported turn".to_string(),
                    files,
                    provider_patches,
                    None,
                )
            }
            ReviewDiffSource::ChangeRequest { .. } => {
                return Err(ChatError::unsupported(
                    "Hosted change-request review is unavailable for this source-control adapter",
                ))
            }
        };
    let revision = if let (Some(before), Some(after)) = (&before_oid, &after_oid) {
        review_revision(
            request.thread_id.as_ref(),
            &environment_id,
            &request.source,
            before,
            after,
            request.ignore_whitespace,
            request.context_lines,
        )?
    } else {
        let mut hasher = Sha256::new();
        hasher.update(
            request
                .thread_id
                .as_ref()
                .map(ChatThreadId::as_str)
                .unwrap_or("workspace-draft"),
        );
        hasher.update([0]);
        hasher.update(environment_id.as_bytes());
        hasher.update([0]);
        hasher.update(serde_json::to_vec(&request.source).map_err(|_| corrupt_data())?);
        let mut patches = provider_patches.iter().collect::<Vec<_>>();
        patches.sort_by(|left, right| left.0.cmp(right.0));
        for (path, patch) in patches {
            hasher.update(path);
            hasher.update([0]);
            hasher.update(patch);
        }
        format!("{:x}", hasher.finalize())
    };
    let mut files = files
        .into_iter()
        .map(|mut file| {
            file.read.file_id = file_id(
                &revision,
                &file.read.relative_path,
                file.read.previous_relative_path.as_deref(),
            );
            ReviewFileInternal { read: file.read }
        })
        .collect::<Vec<_>>();
    reconcile_review_comment_applicability(
        pool,
        request.thread_id.as_ref(),
        &request.source,
        &revision,
    )
    .await?;
    let provider_patches = if provider_patches.is_empty() {
        provider_patches
    } else {
        files
            .iter()
            .filter_map(|file| {
                provider_patches
                    .get(&file.read.relative_path)
                    .cloned()
                    .map(|patch| (file.read.file_id.clone(), patch))
            })
            .collect()
    };
    if matches!(&request.source, ReviewDiffSource::ProviderTurn { .. }) {
        for file in &mut files {
            if !provider_patches.contains_key(&file.read.file_id) {
                file.read.capabilities.comment = false;
                file.read.capability_reasons.comment =
                    Some("The provider did not include patch content for this file".to_string());
            }
        }
    }
    let snapshot_id = snapshot_id(
        database_identity,
        request.thread_id.as_ref(),
        &environment_id,
        &revision,
    );
    let snapshot = Arc::new(ReviewSnapshot {
        database_identity: database_identity.to_string(),
        snapshot_id,
        review_revision: revision,
        thread_id: request.thread_id,
        working_folder_id,
        environment_id,
        root: authorized.canonical_path,
        source: request.source,
        source_label: label,
        before_oid,
        after_oid,
        context_lines: request.context_lines,
        ignore_whitespace: request.ignore_whitespace,
        files,
        provider_patches,
        patch_cache: tokio::sync::Mutex::new(HashMap::new()),
        object_store,
        created_at: Instant::now(),
    });
    state.insert(snapshot.clone())?;
    snapshot_read(&snapshot, request.preferred_relative_path.as_deref()).await
}

async fn reconcile_review_comment_applicability(
    pool: &SqlitePool,
    thread_id: Option<&ChatThreadId>,
    source: &ReviewDiffSource,
    review_revision: &str,
) -> ChatResult<()> {
    let Some(thread_id) = thread_id else {
        return Ok(());
    };
    let source_data = serde_json::to_string(source).map_err(|_| corrupt_data())?;
    sqlx::query(
        "UPDATE chat_review_comments
         SET applicability = CASE
               WHEN review_revision = ? THEN 'current'
               ELSE 'outdated'
             END
         WHERE thread_id = ? AND source_data = ? AND state IN ('open', 'resolved')",
    )
    .bind(review_revision)
    .bind(thread_id.as_str())
    .bind(source_data)
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

async fn snapshot_read(
    snapshot: &Arc<ReviewSnapshot>,
    preferred_path: Option<&str>,
) -> ChatResult<ChatReviewOpenRead> {
    let preferred = preferred_path
        .and_then(|path| {
            snapshot
                .files
                .iter()
                .find(|file| file.read.relative_path == path)
        })
        .or_else(|| snapshot.files.first());
    let preferred_patch = match preferred {
        Some(file) => {
            Some(read_patch_page(snapshot, &file.read.file_id, 0, DEFAULT_PATCH_PAGE_BYTES).await?)
        }
        None => None,
    };
    Ok(ChatReviewOpenRead {
        snapshot_id: snapshot.snapshot_id.clone(),
        review_revision: snapshot.review_revision.clone(),
        source: snapshot.source.clone(),
        source_label: snapshot.source_label.clone(),
        files: snapshot
            .files
            .iter()
            .map(|file| file.read.clone())
            .collect(),
        totals: ReviewTotalsRead {
            files: u64::try_from(snapshot.files.len()).unwrap_or(u64::MAX),
            additions: snapshot
                .files
                .iter()
                .filter_map(|file| file.read.additions)
                .sum(),
            deletions: snapshot
                .files
                .iter()
                .filter_map(|file| file.read.deletions)
                .sum(),
        },
        preferred_patch,
        freshness: ReviewFreshness::Current,
    })
}

fn stale_snapshot_read(snapshot: &ReviewSnapshot) -> ChatReviewOpenRead {
    ChatReviewOpenRead {
        snapshot_id: snapshot.snapshot_id.clone(),
        review_revision: snapshot.review_revision.clone(),
        source: snapshot.source.clone(),
        source_label: snapshot.source_label.clone(),
        files: snapshot
            .files
            .iter()
            .map(|file| file.read.clone())
            .collect(),
        totals: ReviewTotalsRead {
            files: u64::try_from(snapshot.files.len()).unwrap_or(u64::MAX),
            additions: snapshot
                .files
                .iter()
                .filter_map(|file| file.read.additions)
                .sum(),
            deletions: snapshot
                .files
                .iter()
                .filter_map(|file| file.read.deletions)
                .sum(),
        },
        preferred_patch: None,
        freshness: ReviewFreshness::Outdated,
    }
}

async fn working_tree_material(
    root: &Path,
    mode: ReviewWorkingTreeMode,
) -> ChatResult<SnapshotMaterial> {
    let before_status = git_service::status(root).await?;
    let has_conflicts = before_status.files.iter().any(|file| file.conflicted);
    if has_conflicts && mode != ReviewWorkingTreeMode::All {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Staged and unstaged review scopes are unavailable while the Git index has conflicts. Open all changes or resolve the conflicts.",
            true,
        ));
    }
    let head = optional_head(root).await?;
    if mode == ReviewWorkingTreeMode::Staged {
        let index_tree = git_text(root, &["write-tree"], None).await?;
        return Ok(SnapshotMaterial {
            before_oid: match head {
                Some(head) => head,
                None => empty_tree(root).await?,
            },
            after_oid: index_tree,
            label: "Staged changes".to_string(),
            status: before_status,
            object_store: None,
        });
    }
    let store = Arc::new(ReviewObjectStore::new(root).await?);
    let empty_tree_oid = if let Some(head) = head.as_deref() {
        review_storage_text(root, &["read-tree", head], Some(&store)).await?;
        None
    } else {
        review_storage_text(root, &["read-tree", "--empty"], Some(&store)).await?;
        Some(review_storage_text(root, &["write-tree"], Some(&store)).await?)
    };
    let index_tree = git_text(root, &["write-tree"], None).await?;
    review_storage_text(root, &["add", "-A", "--", "."], Some(&store)).await?;
    let worktree_tree = review_storage_text(root, &["write-tree"], Some(&store)).await?;
    let store_for_size = store.clone();
    tauri::async_runtime::spawn_blocking(move || store_for_size.verify_size())
        .await
        .map_err(|_| review_error("Review object size worker stopped"))??;
    let head = head
        .or(empty_tree_oid)
        .ok_or_else(|| review_error("Git HEAD is unavailable"))?;
    Ok(match mode {
        ReviewWorkingTreeMode::Unstaged => SnapshotMaterial {
            before_oid: index_tree,
            after_oid: worktree_tree,
            label: "Unstaged changes".to_string(),
            status: before_status,
            object_store: Some(store),
        },
        ReviewWorkingTreeMode::All => SnapshotMaterial {
            before_oid: head,
            after_oid: worktree_tree,
            label: "All uncommitted changes".to_string(),
            status: before_status,
            object_store: Some(store),
        },
        ReviewWorkingTreeMode::Staged => unreachable!("staged review returned above"),
    })
}

async fn git_files(
    root: &Path,
    before: &str,
    after: &str,
    source: &ReviewDiffSource,
    ignore_whitespace: bool,
    object_store: Option<&ReviewObjectStore>,
) -> ChatResult<Vec<ReviewFileInternal>> {
    let mut name_arguments = deterministic_diff_arguments(ignore_whitespace, None);
    name_arguments.extend(["--name-status", "-z", before, after]);
    let names = review_output(root, &name_arguments, None, object_store).await?;
    let mut stat_arguments = deterministic_diff_arguments(ignore_whitespace, None);
    stat_arguments.extend(["--numstat", "-z", before, after]);
    let stats = review_output(root, &stat_arguments, None, object_store).await?;
    let stats = parse_numstat(&stats)?;
    let mut raw_arguments = deterministic_diff_arguments(ignore_whitespace, None);
    raw_arguments.extend(["--raw", "--no-abbrev", "-z", before, after]);
    let raw = review_output(root, &raw_arguments, None, object_store).await?;
    let raw = parse_raw_metadata(&raw);
    let mut files = parse_name_status(&names, &stats, source)?;
    for file in &mut files {
        let Some(metadata) = raw.get(&file.read.relative_path) else {
            continue;
        };
        file.read.flags.submodule = metadata.old_mode == "160000" || metadata.new_mode == "160000";
        file.read.flags.symlink = metadata.old_mode == "120000" || metadata.new_mode == "120000";
        file.read.flags.mode_only = metadata.old_mode != metadata.new_mode
            && file.read.additions == Some(0)
            && file.read.deletions == Some(0)
            && !file.read.flags.pure_rename;
        if file.read.flags.submodule
            || file.read.flags.mode_only
            || file.read.flags.pure_rename
            || file.read.flags.symlink
        {
            file.read.capabilities.comment = false;
            file.read.capability_reasons.comment =
                Some("This change does not expose stable text lines for comments".to_string());
        }
        if file.read.flags.submodule || file.read.flags.symlink {
            file.read.capabilities.open_editor = false;
            file.read.capability_reasons.open_editor =
                Some("This path is not an editable regular workspace file".to_string());
        }
    }
    Ok(files)
}

fn parse_raw_metadata(bytes: &[u8]) -> HashMap<String, RawFileMetadata> {
    let mut fields = bytes
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty());
    let mut result = HashMap::new();
    while let Some(field) = fields.next() {
        let Ok(header) = std::str::from_utf8(field) else {
            continue;
        };
        if !header.starts_with(':') {
            continue;
        }
        let parts = header[1..].split_whitespace().collect::<Vec<_>>();
        if parts.len() < 5 {
            continue;
        }
        let rename = parts[4].starts_with('R') || parts[4].starts_with('C');
        let final_path = if rename {
            let _ = fields.next();
            fields.next()
        } else {
            fields.next()
        };
        let Some(final_path) = final_path else { break };
        let Ok(final_path) = std::str::from_utf8(final_path) else {
            continue;
        };
        result.insert(
            final_path.to_string(),
            RawFileMetadata {
                old_mode: parts[0].to_string(),
                new_mode: parts[1].to_string(),
            },
        );
    }
    result
}

fn enrich_working_tree_files(
    files: &mut Vec<ReviewFileInternal>,
    status: &git_service::GitStatusRead,
    source: &ReviewDiffSource,
) {
    for git_file in &status.files {
        let matching = files.iter().position(|file| {
            file.read.relative_path == git_file.relative_path
                || git_file
                    .original_relative_path
                    .as_deref()
                    .is_some_and(|original| {
                        file.read.previous_relative_path.as_deref() == Some(original)
                    })
        });
        if matching.is_none() && git_file.conflicted {
            let (flags, capabilities, capability_reasons) =
                file_contract(source, "conflicted", None, None, false);
            files.push(ReviewFileInternal {
                read: ReviewFileRead {
                    file_id: String::new(),
                    relative_path: git_file.relative_path.clone(),
                    previous_relative_path: git_file.original_relative_path.clone(),
                    status: "modified".to_string(),
                    additions: None,
                    deletions: None,
                    flags,
                    capabilities,
                    capability_reasons,
                },
            });
        }
        let index = match matching {
            Some(index) => index,
            None if git_file.conflicted => files.len().saturating_sub(1),
            None => continue,
        };
        let Some(file) = files.get_mut(index) else {
            continue;
        };
        file.read.flags.untracked = git_file.untracked;
        file.read.flags.conflict = git_file.conflicted;
        if git_file.conflicted {
            file.read.capabilities.stage = false;
            file.read.capabilities.unstage = false;
            file.read.capabilities.discard = false;
            let reason = "Resolve this conflict in the editor before changing Git state";
            file.read.capability_reasons.stage = Some(reason.to_string());
            file.read.capability_reasons.unstage = Some(reason.to_string());
            file.read.capability_reasons.discard = Some(reason.to_string());
        }
    }
}

fn deterministic_diff_arguments(
    ignore_whitespace: bool,
    context_argument: Option<&str>,
) -> Vec<&str> {
    let mut arguments = vec![
        "diff",
        "--binary",
        "--full-index",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--find-renames",
        "--diff-algorithm=histogram",
    ];
    if ignore_whitespace {
        arguments.push("--ignore-all-space");
    }
    if let Some(context_argument) = context_argument {
        arguments.push(context_argument);
    }
    arguments
}

async fn read_patch_page(
    snapshot: &Arc<ReviewSnapshot>,
    file_id: &str,
    start_hunk: usize,
    maximum: usize,
) -> ChatResult<ReviewFilePatchRead> {
    let file = snapshot
        .files
        .iter()
        .find(|file| file.read.file_id == file_id)
        .ok_or_else(|| {
            ChatError::new(ChatErrorCode::NotFound, "Review file was not found", true)
        })?;
    if file.read.flags.binary {
        return Ok(ReviewFilePatchRead {
            file_id: file_id.to_string(),
            patch: None,
            hunks: Vec::new(),
            continuation_cursor: None,
            state: ReviewPatchState::Binary,
        });
    }
    if matches!(&snapshot.source, ReviewDiffSource::ProviderTurn { .. })
        && !snapshot.provider_patches.contains_key(file_id)
    {
        return Ok(ReviewFilePatchRead {
            file_id: file_id.to_string(),
            patch: None,
            hunks: Vec::new(),
            continuation_cursor: None,
            state: ReviewPatchState::Unavailable,
        });
    }
    let index = load_patch_index(snapshot, file).await?;
    read_indexed_patch_page(index, file_id, start_hunk, maximum).await
}

async fn load_patch_index(
    snapshot: &Arc<ReviewSnapshot>,
    file: &ReviewFileInternal,
) -> ChatResult<Arc<PatchIndex>> {
    if let Some(index) = snapshot
        .patch_cache
        .lock()
        .await
        .get(&file.read.file_id)
        .cloned()
    {
        return Ok(index);
    }
    let index = if let Some(patch) = snapshot.provider_patches.get(&file.read.file_id) {
        Arc::new(PatchIndex::Memory(parse_patch(patch, &file.read.file_id)?))
    } else {
        let before = snapshot.before_oid.as_deref().ok_or_else(|| {
            ChatError::new(ChatErrorCode::NotFound, "Review patch is unavailable", true)
        })?;
        let after = snapshot.after_oid.as_deref().ok_or_else(|| {
            ChatError::new(ChatErrorCode::NotFound, "Review patch is unavailable", true)
        })?;
        let context = format!("--unified={}", snapshot.context_lines);
        let mut arguments =
            deterministic_diff_arguments(snapshot.ignore_whitespace, Some(&context));
        arguments.extend([before, after, "--", &file.read.relative_path]);
        if let Some(previous) = file.read.previous_relative_path.as_deref() {
            arguments.push(previous);
        }
        let spool = Arc::new(ReviewTemporaryFile::new("patch")?);
        git_service::review_output_to_file_in_storage(
            &snapshot.root,
            &arguments,
            spool.path(),
            MAX_PATCH_SPOOL_BYTES,
            snapshot
                .object_store
                .as_ref()
                .map(|store| store.objects.as_path()),
            snapshot
                .object_store
                .as_ref()
                .map(|store| store.alternate_objects.as_path()),
        )
        .await?;
        let spool_for_index = spool.clone();
        let file_id = file.read.file_id.clone();
        let indexed = tauri::async_runtime::spawn_blocking(move || {
            index_spooled_patch(spool_for_index, &file_id)
        })
        .await
        .map_err(|_| review_error("Review patch index worker stopped"))??;
        Arc::new(PatchIndex::Spool(indexed))
    };
    let mut cache = snapshot.patch_cache.lock().await;
    Ok(cache
        .entry(file.read.file_id.clone())
        .or_insert_with(|| index.clone())
        .clone())
}

async fn read_indexed_patch_page(
    index: Arc<PatchIndex>,
    file_id: &str,
    start_hunk: usize,
    maximum: usize,
) -> ChatResult<ReviewFilePatchRead> {
    match index.as_ref() {
        PatchIndex::Memory(parsed) => read_memory_patch_page(parsed, file_id, start_hunk, maximum),
        PatchIndex::Spool(spool) => {
            let file_id = file_id.to_string();
            let spool = spool.clone();
            tauri::async_runtime::spawn_blocking(move || {
                read_spooled_patch_page(&spool, &file_id, start_hunk, maximum)
            })
            .await
            .map_err(|_| review_error("Review patch page worker stopped"))?
        }
    }
}

fn read_memory_patch_page(
    parsed: &ParsedPatch,
    file_id: &str,
    start_hunk: usize,
    maximum: usize,
) -> ChatResult<ReviewFilePatchRead> {
    if parsed.hunks.is_empty() {
        let complete = parsed.preamble.len() <= maximum;
        return Ok(ReviewFilePatchRead {
            file_id: file_id.to_string(),
            patch: complete.then(|| parsed.preamble.clone()),
            hunks: Vec::new(),
            continuation_cursor: None,
            state: if complete {
                ReviewPatchState::Complete
            } else {
                ReviewPatchState::Unavailable
            },
        });
    }
    if parsed.preamble.len() > maximum {
        return Ok(unavailable_patch(file_id));
    }
    let mut page = parsed.preamble.clone();
    let mut reads = Vec::new();
    let mut index = start_hunk.min(parsed.hunks.len());
    while let Some(hunk) = parsed.hunks.get(index) {
        let metadata_bytes = hunk.read.hunk_id.len().saturating_add(256);
        if page
            .len()
            .saturating_add(hunk.text.len())
            .saturating_add(metadata_bytes)
            > maximum
            || reads.len() >= MAX_PATCH_HUNKS_PER_RESPONSE
        {
            if reads.is_empty() {
                let mut read = hunk.read.clone();
                read.state = ReviewPatchState::OversizedHunk;
                reads.push(read);
                index += 1;
            }
            break;
        }
        page.push_str(&hunk.text);
        reads.push(hunk.read.clone());
        index += 1;
    }
    let continuation_cursor = (index < parsed.hunks.len()).then(|| format!("{file_id}/{index}"));
    let oversized = reads
        .iter()
        .any(|hunk| hunk.state == ReviewPatchState::OversizedHunk);
    Ok(ReviewFilePatchRead {
        file_id: file_id.to_string(),
        patch: (!page.is_empty()
            && reads
                .iter()
                .any(|hunk| hunk.state == ReviewPatchState::Complete))
        .then_some(page),
        hunks: reads,
        continuation_cursor: continuation_cursor.clone(),
        state: if oversized {
            ReviewPatchState::OversizedHunk
        } else if continuation_cursor.is_some() || start_hunk > 0 {
            ReviewPatchState::Partial
        } else {
            ReviewPatchState::Complete
        },
    })
}

fn unavailable_patch(file_id: &str) -> ReviewFilePatchRead {
    ReviewFilePatchRead {
        file_id: file_id.to_string(),
        patch: None,
        hunks: Vec::new(),
        continuation_cursor: None,
        state: ReviewPatchState::Unavailable,
    }
}

fn index_spooled_patch(
    file: Arc<ReviewTemporaryFile>,
    file_id: &str,
) -> ChatResult<SpoolPatchIndex> {
    let input = fs::File::open(file.path()).map_err(|_| review_error("Open review patch spool"))?;
    let byte_size = input
        .metadata()
        .map_err(|_| review_error("Read review patch spool metadata"))?
        .len();
    let mut reader = BufReader::new(input);
    let mut position = 0_u64;
    let mut preamble_end = byte_size;
    let mut hunks = Vec::new();
    let mut active: Option<SpoolHunkBuilder> = None;
    let mut line = Vec::new();
    loop {
        line.clear();
        let read = reader
            .read_until(b'\n', &mut line)
            .map_err(|_| review_error("Read review patch spool"))?;
        if read == 0 {
            break;
        }
        let line_start = position;
        position = position
            .checked_add(u64::try_from(read).map_err(|_| corrupt_data())?)
            .ok_or_else(corrupt_data)?;
        let text = std::str::from_utf8(&line)
            .map_err(|_| review_error("Git diff output is not valid UTF-8"))?;
        if text.starts_with("@@ ") {
            if let Some(builder) = active.take() {
                hunks.push(builder.finish(line_start));
            } else {
                preamble_end = line_start;
            }
            active = Some(SpoolHunkBuilder::new(file_id, text, line_start)?);
        } else if let Some(builder) = active.as_mut() {
            builder.push_line(text);
        }
    }
    if let Some(builder) = active {
        hunks.push(builder.finish(byte_size));
    }
    Ok(SpoolPatchIndex {
        file,
        byte_size,
        preamble_end,
        hunks,
    })
}

struct SpoolHunkBuilder {
    start: u64,
    old_start: u64,
    old_count: u64,
    new_start: u64,
    new_count: u64,
    old_line: u64,
    new_line: u64,
    hasher: Sha256,
}

impl SpoolHunkBuilder {
    fn new(file_id: &str, header: &str, start: u64) -> ChatResult<Self> {
        let (old_start, old_count, new_start, new_count) = parse_hunk_header(header.trim_end())?;
        let mut hasher = Sha256::new();
        hasher.update(file_id);
        hasher.update([0]);
        Ok(Self {
            start,
            old_start,
            old_count,
            new_start,
            new_count,
            old_line: old_start,
            new_line: new_start,
            hasher,
        })
    }

    fn push_line(&mut self, line: &str) {
        let line = line.strip_suffix('\n').unwrap_or(line);
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.starts_with(' ') {
            self.old_line = self.old_line.saturating_add(1);
            self.new_line = self.new_line.saturating_add(1);
        } else if let Some(content) = line.strip_prefix('-') {
            self.hasher.update([b'-']);
            self.hasher.update(self.old_line.to_le_bytes());
            self.hasher.update(content);
            self.hasher.update([0]);
            self.old_line = self.old_line.saturating_add(1);
        } else if let Some(content) = line.strip_prefix('+') {
            self.hasher.update([b'+']);
            self.hasher.update(self.new_line.to_le_bytes());
            self.hasher.update(content);
            self.hasher.update([0]);
            self.new_line = self.new_line.saturating_add(1);
        }
    }

    fn finish(self, end: u64) -> IndexedHunk {
        IndexedHunk {
            read: ReviewHunkRead {
                hunk_id: format!("review-hunk:{:x}", self.hasher.finalize()),
                old_start: self.old_start,
                old_count: self.old_count,
                new_start: self.new_start,
                new_count: self.new_count,
                state: ReviewPatchState::Complete,
            },
            start: self.start,
            end,
        }
    }
}

fn read_spooled_patch_page(
    spool: &SpoolPatchIndex,
    file_id: &str,
    start_hunk: usize,
    maximum: usize,
) -> ChatResult<ReviewFilePatchRead> {
    if spool.hunks.is_empty() {
        let complete = usize::try_from(spool.byte_size)
            .ok()
            .is_some_and(|size| size <= maximum);
        return Ok(ReviewFilePatchRead {
            file_id: file_id.to_string(),
            patch: complete
                .then(|| read_spool_range(spool.file.path(), 0, spool.byte_size))
                .transpose()?,
            hunks: Vec::new(),
            continuation_cursor: None,
            state: if complete {
                ReviewPatchState::Complete
            } else {
                ReviewPatchState::Unavailable
            },
        });
    }
    let preamble_size = usize::try_from(spool.preamble_end).unwrap_or(usize::MAX);
    if preamble_size > maximum {
        return Ok(unavailable_patch(file_id));
    }
    let mut page = read_spool_range(spool.file.path(), 0, spool.preamble_end)?;
    let mut reads = Vec::new();
    let mut index = start_hunk.min(spool.hunks.len());
    while let Some(hunk) = spool.hunks.get(index) {
        let hunk_size = usize::try_from(hunk.end.saturating_sub(hunk.start)).unwrap_or(usize::MAX);
        let metadata_bytes = hunk.read.hunk_id.len().saturating_add(256);
        if page
            .len()
            .saturating_add(hunk_size)
            .saturating_add(metadata_bytes)
            > maximum
            || reads.len() >= MAX_PATCH_HUNKS_PER_RESPONSE
        {
            if reads.is_empty() {
                let mut read = hunk.read.clone();
                read.state = ReviewPatchState::OversizedHunk;
                reads.push(read);
                index += 1;
            }
            break;
        }
        page.push_str(&read_spool_range(spool.file.path(), hunk.start, hunk.end)?);
        reads.push(hunk.read.clone());
        index += 1;
    }
    let continuation_cursor = (index < spool.hunks.len()).then(|| format!("{file_id}/{index}"));
    let oversized = reads
        .iter()
        .any(|hunk| hunk.state == ReviewPatchState::OversizedHunk);
    Ok(ReviewFilePatchRead {
        file_id: file_id.to_string(),
        patch: (!page.is_empty()
            && reads
                .iter()
                .any(|hunk| hunk.state == ReviewPatchState::Complete))
        .then_some(page),
        hunks: reads,
        continuation_cursor: continuation_cursor.clone(),
        state: if oversized {
            ReviewPatchState::OversizedHunk
        } else if continuation_cursor.is_some() || start_hunk > 0 {
            ReviewPatchState::Partial
        } else {
            ReviewPatchState::Complete
        },
    })
}

fn read_spool_range(path: &Path, start: u64, end: u64) -> ChatResult<String> {
    let length = usize::try_from(end.saturating_sub(start)).map_err(|_| corrupt_data())?;
    if length > MAX_PATCH_PAGE_BYTES {
        return Err(review_error(
            "Review patch page exceeds the supported limit",
        ));
    }
    let mut file = fs::File::open(path).map_err(|_| review_error("Open review patch spool"))?;
    file.seek(SeekFrom::Start(start))
        .map_err(|_| review_error("Seek review patch spool"))?;
    let mut bytes = vec![0_u8; length];
    file.read_exact(&mut bytes)
        .map_err(|_| review_error("Read review patch spool"))?;
    String::from_utf8(bytes).map_err(|_| review_error("Git diff output is not valid UTF-8"))
}

impl ReviewObjectStore {
    async fn new(repository_root: &Path) -> ChatResult<Self> {
        let alternate_objects = PathBuf::from(
            git_text(
                repository_root,
                &[
                    "rev-parse",
                    "--path-format=absolute",
                    "--git-path",
                    "objects",
                ],
                None,
            )
            .await?,
        );
        if !alternate_objects.is_absolute() || !alternate_objects.is_dir() {
            return Err(review_error("Git object directory is unavailable"));
        }
        let root = create_review_temp_directory("objects")?;
        let objects = root.join("objects");
        if fs::create_dir(&objects).is_err() {
            let _ = fs::remove_dir_all(&root);
            return Err(review_error("Create review object directory"));
        }
        Ok(Self {
            index: root.join("index"),
            root,
            objects,
            alternate_objects,
        })
    }

    fn verify_size(&self) -> ChatResult<()> {
        let mut total = 0_u64;
        let mut pending = vec![self.objects.clone()];
        while let Some(directory) = pending.pop() {
            let entries = fs::read_dir(directory)
                .map_err(|_| review_error("Read review object directory"))?;
            for entry in entries {
                let entry = entry.map_err(|_| review_error("Read review object entry"))?;
                let metadata = entry
                    .metadata()
                    .map_err(|_| review_error("Read review object metadata"))?;
                if metadata.is_dir() {
                    pending.push(entry.path());
                } else if metadata.is_file() {
                    total = total.saturating_add(metadata.len());
                    if total > MAX_REVIEW_OBJECT_BYTES {
                        return Err(ChatError::new(
                            ChatErrorCode::Protocol,
                            "Review snapshot exceeds the supported temporary storage limit",
                            true,
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

impl Drop for ReviewObjectStore {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl ReviewTemporaryFile {
    fn new(kind: &str) -> ChatResult<Self> {
        let root = create_review_temp_directory(kind)?;
        Ok(Self {
            path: root.join("content"),
            root,
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ReviewTemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn create_review_temp_directory(kind: &str) -> ChatResult<PathBuf> {
    for _ in 0..32 {
        let sequence = REVIEW_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ganbaru-chat-review-{kind}-{}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(review_error("Create review temporary directory")),
        }
    }
    Err(review_error("Allocate review temporary directory"))
}

async fn optional_head(root: &Path) -> ChatResult<Option<String>> {
    match git_text(
        root,
        &["rev-parse", "-q", "--verify", "HEAD^{commit}"],
        None,
    )
    .await
    {
        Ok(head) => Ok(Some(head)),
        Err(error) if error.code == ChatErrorCode::Conflict => Ok(None),
        Err(error) => Err(error),
    }
}

async fn review_storage_text(
    root: &Path,
    arguments: &[&str],
    store: Option<&ReviewObjectStore>,
) -> ChatResult<String> {
    String::from_utf8(
        git_service::review_output_in_storage(
            root,
            arguments,
            None,
            store.map(|store| store.index.as_path()),
            store.map(|store| store.objects.as_path()),
            store.map(|store| store.alternate_objects.as_path()),
        )
        .await?,
    )
    .map(|value| value.trim().to_string())
    .map_err(|_| review_error("Git output is not valid UTF-8"))
}

async fn review_output(
    root: &Path,
    arguments: &[&str],
    input: Option<&[u8]>,
    store: Option<&ReviewObjectStore>,
) -> ChatResult<Vec<u8>> {
    git_service::review_output_in_storage(
        root,
        arguments,
        input,
        None,
        store.map(|store| store.objects.as_path()),
        store.map(|store| store.alternate_objects.as_path()),
    )
    .await
}

async fn apply_action(
    snapshot: &Arc<ReviewSnapshot>,
    request: &ApplyChatReviewActionRequest,
    mode: ReviewWorkingTreeMode,
) -> ChatResult<()> {
    if request.operation == ReviewAction::Discard && !request.confirmed {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Discarding reviewed changes requires explicit confirmation",
            true,
        ));
    }
    if snapshot.files.is_empty() {
        return Err(ChatError::new(
            ChatErrorCode::NotFound,
            "The review snapshot has no changes to apply",
            true,
        ));
    }
    let file = request
        .file_id
        .as_deref()
        .map(|file_id| {
            snapshot
                .files
                .iter()
                .find(|file| file.read.file_id == file_id)
                .ok_or_else(|| {
                    ChatError::new(ChatErrorCode::NotFound, "Review file was not found", true)
                })
        })
        .transpose()?;
    if file.is_none()
        && snapshot.files.iter().any(|file| match request.operation {
            ReviewAction::Stage => !file.read.capabilities.stage,
            ReviewAction::Unstage => !file.read.capabilities.unstage,
            ReviewAction::Discard => !file.read.capabilities.discard,
        })
    {
        return Err(ChatError::unsupported(
            "One or more reviewed files do not support this whole-scope action",
        ));
    }
    require_action_allowed(mode, request.operation, file, &request.hunk_ids)?;
    if request.operation == ReviewAction::Stage && request.hunk_ids.is_empty() {
        let owned_paths = action_paths(snapshot, request.file_id.as_deref());
        if owned_paths.is_empty() {
            return Err(ChatError::new(
                ChatErrorCode::NotFound,
                "The review snapshot has no paths to stage",
                true,
            ));
        }
        stage_owned_paths(&snapshot.root, &owned_paths).await?;
        return Ok(());
    }
    if request.hunk_ids.is_empty() {
        if let Some(file) = file {
            return apply_exact_file_action(snapshot, file, request.operation, mode).await;
        }
        return apply_exact_scope_action(snapshot, request.operation, mode).await;
    }
    let patch = action_patch(snapshot, file, &request.hunk_ids).await?;
    let mut check = vec!["apply", "--check", "--recount", "--whitespace=nowarn"];
    let mut apply = vec!["apply", "--recount", "--whitespace=nowarn"];
    match request.operation {
        ReviewAction::Stage => {
            check.push("--cached");
            apply.push("--cached");
        }
        ReviewAction::Unstage => {
            check.extend(["--cached", "--reverse"]);
            apply.extend(["--cached", "--reverse"]);
        }
        ReviewAction::Discard => {
            check.push("--reverse");
            apply.push("--reverse");
        }
    }
    git_service::review_output(&snapshot.root, &check, Some(patch.as_bytes())).await?;
    git_service::review_output(&snapshot.root, &apply, Some(patch.as_bytes())).await?;
    Ok(())
}

async fn apply_exact_file_action(
    snapshot: &ReviewSnapshot,
    file: &ReviewFileInternal,
    operation: ReviewAction,
    mode: ReviewWorkingTreeMode,
) -> ChatResult<()> {
    let before = snapshot
        .before_oid
        .as_deref()
        .ok_or_else(|| ChatError::unsupported("This review source cannot be changed"))?;
    let current = file.read.relative_path.as_str();
    match operation {
        ReviewAction::Stage => {
            let paths = action_paths(snapshot, Some(&file.read.file_id));
            if paths.is_empty() {
                return Err(ChatError::new(
                    ChatErrorCode::NotFound,
                    "The review file has no path to stage",
                    true,
                ));
            }
            stage_owned_paths(&snapshot.root, &paths).await?;
        }
        ReviewAction::Unstage => {
            if mode != ReviewWorkingTreeMode::Staged {
                return Err(ChatError::unsupported(
                    "This file is not in the staged review scope",
                ));
            }
            if file.read.status == "added" {
                git_service::review_output(
                    &snapshot.root,
                    &["rm", "--cached", "--ignore-unmatch", "--", current],
                    None,
                )
                .await?;
            } else if let Some(previous) = file.read.previous_relative_path.as_deref() {
                git_service::review_output(
                    &snapshot.root,
                    &["restore", "--staged", "--source", before, "--", previous],
                    None,
                )
                .await?;
                git_service::review_output(
                    &snapshot.root,
                    &["rm", "--cached", "--ignore-unmatch", "--", current],
                    None,
                )
                .await?;
            } else {
                git_service::review_output(
                    &snapshot.root,
                    &["restore", "--staged", "--source", before, "--", current],
                    None,
                )
                .await?;
            }
        }
        ReviewAction::Discard => {
            if mode != ReviewWorkingTreeMode::Unstaged {
                return Err(ChatError::unsupported(
                    "Open Unstaged changes before discarding this file",
                ));
            }
            if let Some(previous) = file.read.previous_relative_path.as_deref() {
                git_service::review_output(
                    &snapshot.root,
                    &["restore", "--worktree", "--source", before, "--", previous],
                    None,
                )
                .await?;
                git_service::review_output(&snapshot.root, &["clean", "-f", "--", current], None)
                    .await?;
            } else if file.read.flags.untracked {
                git_service::review_output(&snapshot.root, &["clean", "-f", "--", current], None)
                    .await?;
            } else {
                git_service::review_output(
                    &snapshot.root,
                    &["restore", "--worktree", "--source", before, "--", current],
                    None,
                )
                .await?;
            }
        }
    }
    Ok(())
}

async fn apply_exact_scope_action(
    snapshot: &ReviewSnapshot,
    operation: ReviewAction,
    mode: ReviewWorkingTreeMode,
) -> ChatResult<()> {
    let before = snapshot
        .before_oid
        .as_deref()
        .ok_or_else(|| ChatError::unsupported("This review source cannot be changed"))?;
    match operation {
        ReviewAction::Stage => unreachable!("whole-scope staging returns before this helper"),
        ReviewAction::Unstage if mode == ReviewWorkingTreeMode::Staged => {
            restore_owned_paths(&snapshot.root, before, true, &action_paths(snapshot, None)).await
        }
        ReviewAction::Discard if mode == ReviewWorkingTreeMode::Unstaged => {
            discard_owned_files(snapshot, before).await
        }
        _ => Err(ChatError::unsupported(
            "This action is unavailable for the selected review scope",
        )),
    }
}

async fn discard_owned_files(snapshot: &ReviewSnapshot, before: &str) -> ChatResult<()> {
    let mut restore_paths = Vec::new();
    let mut clean_paths = Vec::new();
    for file in &snapshot.files {
        if let Some(previous) = file.read.previous_relative_path.as_ref() {
            restore_paths.push(previous.clone());
            clean_paths.push(file.read.relative_path.clone());
            continue;
        }
        if file.read.flags.untracked {
            clean_paths.push(file.read.relative_path.clone());
            continue;
        }
        restore_paths.push(file.read.relative_path.clone());
    }
    restore_paths.sort();
    restore_paths.dedup();
    clean_paths.sort();
    clean_paths.dedup();
    restore_owned_paths(&snapshot.root, before, false, &restore_paths).await?;
    for chunk in bounded_path_chunks(&clean_paths) {
        let mut arguments = vec!["clean", "-f", "--"];
        arguments.extend(chunk.iter().map(String::as_str));
        git_service::review_output(&snapshot.root, &arguments, None).await?;
    }
    Ok(())
}

async fn restore_owned_paths(
    root: &Path,
    source: &str,
    staged: bool,
    paths: &[String],
) -> ChatResult<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let pathspec = pathspec_bytes(paths)?;
    let mut arguments = vec!["restore"];
    if staged {
        arguments.push("--staged");
    } else {
        arguments.push("--worktree");
    }
    arguments.extend([
        "--source",
        source,
        "--pathspec-from-file=-",
        "--pathspec-file-nul",
    ]);
    git_service::review_output(root, &arguments, Some(&pathspec)).await?;
    Ok(())
}

async fn stage_owned_paths(root: &Path, paths: &[String]) -> ChatResult<()> {
    let pathspec = pathspec_bytes(paths)?;
    git_service::review_output(
        root,
        &["add", "-A", "--pathspec-from-file=-", "--pathspec-file-nul"],
        Some(&pathspec),
    )
    .await?;
    Ok(())
}

fn pathspec_bytes(paths: &[String]) -> ChatResult<Vec<u8>> {
    let mut pathspec = Vec::new();
    for path in paths {
        validate_path(path)?;
        pathspec.extend_from_slice(path.as_bytes());
        pathspec.push(0);
    }
    Ok(pathspec)
}

fn bounded_path_chunks(paths: &[String]) -> Vec<&[String]> {
    const MAX_PATHS_PER_COMMAND: usize = 64;
    const MAX_PATH_BYTES_PER_COMMAND: usize = 16 * 1024;
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < paths.len() {
        let mut end = start;
        let mut bytes = 0_usize;
        while end < paths.len() && end - start < MAX_PATHS_PER_COMMAND {
            let next = paths[end].len().saturating_add(1);
            if end > start && bytes.saturating_add(next) > MAX_PATH_BYTES_PER_COMMAND {
                break;
            }
            bytes = bytes.saturating_add(next);
            end += 1;
        }
        chunks.push(&paths[start..end]);
        start = end;
    }
    chunks
}

fn action_paths(snapshot: &ReviewSnapshot, file_id: Option<&str>) -> Vec<String> {
    let mut paths = snapshot
        .files
        .iter()
        .filter(|file| file_id.is_none_or(|selected| file.read.file_id == selected))
        .flat_map(|file| {
            std::iter::once(file.read.relative_path.clone())
                .chain(file.read.previous_relative_path.clone())
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

async fn action_patch(
    snapshot: &Arc<ReviewSnapshot>,
    file: Option<&ReviewFileInternal>,
    selected_hunks: &[String],
) -> ChatResult<String> {
    let before = snapshot
        .before_oid
        .as_deref()
        .ok_or_else(|| ChatError::unsupported("Provider-reported patches cannot be applied"))?;
    let after = snapshot
        .after_oid
        .as_deref()
        .ok_or_else(|| ChatError::unsupported("Provider-reported patches cannot be applied"))?;
    let context = format!("--unified={}", snapshot.context_lines.max(3));
    let mut arguments = deterministic_diff_arguments(
        !selected_hunks.is_empty() && snapshot.ignore_whitespace,
        Some(&context),
    );
    arguments.extend([before, after]);
    if let Some(file) = file {
        arguments.extend(["--", &file.read.relative_path]);
        if let Some(previous) = file.read.previous_relative_path.as_deref() {
            arguments.push(previous);
        }
    }
    let patch = String::from_utf8(
        review_output(
            &snapshot.root,
            &arguments,
            None,
            snapshot.object_store.as_deref(),
        )
        .await?,
    )
    .map_err(|_| review_error("Git diff output is not valid UTF-8"))?;
    if selected_hunks.is_empty() {
        return Ok(patch);
    }
    let file_id = file
        .map(|file| file.read.file_id.as_str())
        .ok_or_else(|| ChatError::validation("fileId", "Hunk actions require one file"))?;
    let parsed = parse_patch(&patch, file_id)?;
    let selected = selected_hunks
        .iter()
        .map(String::as_str)
        .collect::<std::collections::HashSet<_>>();
    let mut result = parsed.preamble;
    let mut found = 0;
    for hunk in parsed.hunks {
        if selected.contains(hunk.read.hunk_id.as_str()) {
            result.push_str(&hunk.text);
            found += 1;
        }
    }
    if found != selected.len() {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "One or more reviewed hunks are no longer available",
            true,
        ));
    }
    Ok(result)
}

fn require_action_allowed(
    mode: ReviewWorkingTreeMode,
    operation: ReviewAction,
    file: Option<&ReviewFileInternal>,
    hunk_ids: &[String],
) -> ChatResult<()> {
    let scope_allowed = match (mode, operation) {
        (ReviewWorkingTreeMode::Staged, ReviewAction::Unstage) => true,
        (ReviewWorkingTreeMode::Unstaged, ReviewAction::Stage | ReviewAction::Discard) => true,
        (ReviewWorkingTreeMode::All, ReviewAction::Stage) if hunk_ids.is_empty() => true,
        _ => false,
    };
    let file_allowed = file.map(|file| match operation {
        ReviewAction::Stage => file.read.capabilities.stage,
        ReviewAction::Unstage => file.read.capabilities.unstage,
        ReviewAction::Discard => file.read.capabilities.discard,
    });
    if !scope_allowed || file_allowed == Some(false) {
        return Err(ChatError::unsupported(
            "This action is unavailable for the selected review scope",
        ));
    }
    if !hunk_ids.is_empty()
        && file.is_some_and(|file| {
            file.read.flags.binary
                || file.read.flags.submodule
                || file.read.flags.conflict
                || file.read.flags.mode_only
                || file.read.flags.pure_rename
                || file.read.flags.untracked
                || file.read.flags.symlink
        })
    {
        return Err(ChatError::unsupported(
            "This file only supports whole-file review actions",
        ));
    }
    Ok(())
}

async fn checkpoint_pair(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    range: ReviewCheckpointRange,
    turn_id: Option<&ChatTurnId>,
) -> ChatResult<Option<(ChatCheckpointId, ChatCheckpointId)>> {
    let row = match range {
        ReviewCheckpointRange::Turn => match turn_id {
            Some(turn_id) => {
                sqlx::query(
                    "SELECT pre_checkpoint_id, post_checkpoint_id FROM chat_turns
                 WHERE id = ? AND thread_id = ? AND invalidated_at IS NULL",
                )
                .bind(turn_id.as_str())
                .bind(thread_id.as_str())
                .fetch_optional(pool)
                .await
            }
            None => {
                sqlx::query(
                    "SELECT pre_checkpoint_id, post_checkpoint_id FROM chat_turns
                 WHERE thread_id = ? AND invalidated_at IS NULL
                 ORDER BY ordinal DESC LIMIT 1",
                )
                .bind(thread_id.as_str())
                .fetch_optional(pool)
                .await
            }
        },
        ReviewCheckpointRange::Thread => {
            sqlx::query(
                "SELECT
               (SELECT pre_checkpoint_id FROM chat_turns
                WHERE thread_id = ? AND invalidated_at IS NULL
                ORDER BY ordinal ASC LIMIT 1) AS pre_checkpoint_id,
               (SELECT post_checkpoint_id FROM chat_turns
                WHERE thread_id = ? AND invalidated_at IS NULL
                ORDER BY ordinal DESC LIMIT 1) AS post_checkpoint_id",
            )
            .bind(thread_id.as_str())
            .bind(thread_id.as_str())
            .fetch_optional(pool)
            .await
        }
    }
    .map_err(persistence_error)?;
    let Some(row) = row else { return Ok(None) };
    let pre: Option<String> = row
        .try_get("pre_checkpoint_id")
        .map_err(persistence_error)?;
    let post: Option<String> = row
        .try_get("post_checkpoint_id")
        .map_err(persistence_error)?;
    match (pre, post) {
        (Some(pre), Some(post)) => Ok(Some((
            ChatCheckpointId::new(pre).map_err(|_| corrupt_data())?,
            ChatCheckpointId::new(post).map_err(|_| corrupt_data())?,
        ))),
        _ => Ok(None),
    }
}

async fn verify_checkpoint_pair(
    authorized: &AuthorizedWorkingFolder,
    thread_id: &ChatThreadId,
    pre: &StoredCheckpoint,
    post: &StoredCheckpoint,
) -> ChatResult<()> {
    if &pre.thread_id != thread_id
        || &post.thread_id != thread_id
        || pre.repository_identity != post.repository_identity
    {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Checkpoint pair belongs to another Chat thread",
            false,
        ));
    }
    let authorized = authorized.clone();
    let pre = pre.clone();
    let post = post.clone();
    tauri::async_runtime::spawn_blocking(move || {
        verify_checkpoint(&authorized, &pre)?;
        verify_checkpoint(&authorized, &post)
    })
    .await
    .map_err(|_| review_error("Git checkpoint verification worker stopped"))?
}

async fn provider_turn_patch(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    turn_id: &ChatTurnId,
) -> ChatResult<(Vec<ChangedFileSummary>, String)> {
    let payload: String = sqlx::query_scalar(
        "SELECT payload_data FROM chat_events
         WHERE thread_id = ? AND turn_id = ? AND event_type = 'diff_updated'
           AND json_extract(payload_data, '$.payload.providerDiff') IS NOT NULL
           AND instr(json_extract(payload_data, '$.payload.providerDiff'), 'diff --git ') > 0
         ORDER BY sequence DESC LIMIT 1",
    )
    .bind(thread_id.as_str())
    .bind(turn_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "The provider did not report a patch for this turn",
            true,
        )
    })?;
    match serde_json::from_str::<CanonicalEvent>(&payload).map_err(|_| corrupt_data())? {
        CanonicalEvent::DiffUpdated(event) => event
            .provider_diff
            .filter(|patch| !patch.is_empty())
            .map(|patch| (event.files, patch))
            .ok_or_else(|| {
                ChatError::new(
                    ChatErrorCode::NotFound,
                    "The provider did not report patch content for this turn",
                    true,
                )
            }),
        _ => Err(corrupt_data()),
    }
}

fn parse_name_status(
    bytes: &[u8],
    stats: &ReviewNumstat,
    source: &ReviewDiffSource,
) -> ChatResult<Vec<ReviewFileInternal>> {
    let mut fields = bytes
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty());
    let mut files = Vec::new();
    while let Some(status_field) = fields.next() {
        let status = std::str::from_utf8(status_field).map_err(|_| corrupt_data())?;
        let previous = if status.starts_with('R') || status.starts_with('C') {
            Some(path_field(fields.next())?)
        } else {
            None
        };
        let path = path_field(fields.next())?;
        let (additions, deletions) = stats.get(&path).copied().unwrap_or((None, None));
        let binary = additions.is_none() || deletions.is_none();
        let normalized = match status.chars().next() {
            Some('A') => "added",
            Some('M') => "modified",
            Some('D') => "deleted",
            Some('R') | Some('C') => "renamed",
            Some('T') => "type_changed",
            Some('U') => "modified",
            _ => "unknown",
        };
        let (flags, capabilities, capability_reasons) =
            file_contract(source, normalized, additions, deletions, binary);
        files.push(ReviewFileInternal {
            read: ReviewFileRead {
                file_id: String::new(),
                relative_path: path,
                previous_relative_path: previous,
                status: normalized.to_string(),
                additions,
                deletions,
                flags,
                capabilities,
                capability_reasons,
            },
        });
    }
    Ok(files)
}

fn parse_numstat(bytes: &[u8]) -> ChatResult<ReviewNumstat> {
    let mut values = BTreeMap::new();
    let mut fields = bytes.split(|byte| *byte == 0).peekable();
    while let Some(field) = fields.next() {
        if field.is_empty() {
            continue;
        }
        let text = std::str::from_utf8(field).map_err(|_| corrupt_data())?;
        let mut parts = text.splitn(3, '\t');
        let additions = parts.next().and_then(|value| value.parse().ok());
        let deletions = parts.next().and_then(|value| value.parse().ok());
        let path = parts.next().unwrap_or_default();
        let final_path = if path.is_empty() {
            let _ = fields.next();
            fields
                .next()
                .and_then(|value| std::str::from_utf8(value).ok())
                .unwrap_or_default()
        } else {
            path
        };
        if !final_path.is_empty() {
            validate_path(final_path)?;
            values.insert(final_path.to_string(), (additions, deletions));
        }
    }
    Ok(values)
}

fn parse_patch(patch: &str, file_id: &str) -> ChatResult<ParsedPatch> {
    let mut starts = patch
        .match_indices("\n@@ ")
        .map(|(index, _)| index + 1)
        .collect::<Vec<_>>();
    if patch.starts_with("@@ ") {
        starts.insert(0, 0);
    }
    let preamble_end = starts.first().copied().unwrap_or(patch.len());
    let mut hunks = Vec::new();
    for (index, start) in starts.iter().copied().enumerate() {
        let end = starts.get(index + 1).copied().unwrap_or(patch.len());
        let text = patch[start..end].to_string();
        let header = text.lines().next().ok_or_else(corrupt_data)?;
        let (old_start, old_count, new_start, new_count) = parse_hunk_header(header)?;
        hunks.push(ParsedHunk {
            read: ReviewHunkRead {
                hunk_id: hunk_id(file_id, &text, old_start, new_start),
                old_start,
                old_count,
                new_start,
                new_count,
                state: ReviewPatchState::Complete,
            },
            text,
        });
    }
    Ok(ParsedPatch {
        preamble: patch[..preamble_end].to_string(),
        hunks,
    })
}

fn parse_hunk_header(header: &str) -> ChatResult<(u64, u64, u64, u64)> {
    let body = header
        .strip_prefix("@@ -")
        .and_then(|value| value.split_once(" @@").map(|(range, _)| range))
        .ok_or_else(corrupt_data)?;
    let (old, new) = body.split_once(" +").ok_or_else(corrupt_data)?;
    let old = parse_hunk_range(old)?;
    let new = parse_hunk_range(new)?;
    Ok((old.0, old.1, new.0, new.1))
}

fn parse_hunk_range(value: &str) -> ChatResult<(u64, u64)> {
    let (start, lines) = value
        .split_once(',')
        .map_or((value, "1"), |(start, lines)| (start, lines));
    Ok((
        start.parse().map_err(|_| corrupt_data())?,
        lines.parse().map_err(|_| corrupt_data())?,
    ))
}

fn file_contract(
    source: &ReviewDiffSource,
    status: &str,
    additions: Option<u64>,
    deletions: Option<u64>,
    binary: bool,
) -> (
    ReviewFileFlagsRead,
    ReviewFileCapabilitiesRead,
    ReviewFileCapabilityReasonsRead,
) {
    let read_only = !matches!(source, ReviewDiffSource::WorkingTree { .. });
    let conflict = status == "conflicted";
    let pure_rename = status == "renamed" && additions == Some(0) && deletions == Some(0);
    let mode_only = status == "type_changed";
    let untracked = matches!(
        source,
        ReviewDiffSource::WorkingTree {
            mode: ReviewWorkingTreeMode::Unstaged
        }
    ) && status == "added";
    let mut capabilities = ReviewFileCapabilitiesRead {
        stage: false,
        unstage: false,
        discard: false,
        comment: !binary,
        open_editor: status != "deleted",
    };
    let mut reasons = ReviewFileCapabilityReasonsRead::default();
    if conflict {
        reasons.stage = Some("Resolve this conflict in the editor before staging".to_string());
        reasons.unstage = Some("Conflicted entries cannot be changed from Review".to_string());
        reasons.discard = Some("Conflicted entries cannot be discarded from Review".to_string());
    } else {
        match source {
            ReviewDiffSource::WorkingTree {
                mode: ReviewWorkingTreeMode::Staged,
            } => {
                capabilities.unstage = true;
                reasons.stage = Some("Already staged".to_string());
                reasons.discard = Some("Unstage this change before discarding it".to_string());
            }
            ReviewDiffSource::WorkingTree {
                mode: ReviewWorkingTreeMode::Unstaged,
            } => {
                capabilities.stage = true;
                capabilities.discard = true;
                reasons.unstage = Some("Not staged".to_string());
            }
            ReviewDiffSource::WorkingTree {
                mode: ReviewWorkingTreeMode::All,
            } => {
                capabilities.stage = true;
                reasons.unstage = Some("Open Staged changes to unstage".to_string());
                reasons.discard = Some("Open Unstaged changes to discard".to_string());
            }
            _ => {
                let reason = "This review source is read-only".to_string();
                reasons.stage = Some(reason.clone());
                reasons.unstage = Some(reason.clone());
                reasons.discard = Some(reason);
            }
        }
    }
    if binary {
        capabilities.comment = false;
        reasons.comment = Some("Binary files do not support line comments".to_string());
    }
    if !capabilities.open_editor {
        reasons.open_editor = Some("Deleted files cannot be opened in the editor".to_string());
    }
    (
        ReviewFileFlagsRead {
            binary,
            submodule: false,
            conflict,
            mode_only,
            pure_rename,
            untracked,
            symlink: false,
            provider_reported: read_only && matches!(source, ReviewDiffSource::ProviderTurn { .. }),
            git_observed: !matches!(source, ReviewDiffSource::ProviderTurn { .. }),
            read_only,
        },
        capabilities,
        reasons,
    )
}

fn file_from_summary(
    file_id: String,
    summary: ChangedFileSummary,
    source: &ReviewDiffSource,
) -> ReviewFileInternal {
    let normalized_status = normalize_file_status(&summary.status);
    let (mut flags, capabilities, capability_reasons) = file_contract(
        source,
        normalized_status,
        summary.additions,
        summary.deletions,
        summary.binary,
    );
    flags.provider_reported = true;
    flags.git_observed = false;
    flags.conflict = summary.status == "conflicted";
    ReviewFileInternal {
        read: ReviewFileRead {
            file_id,
            relative_path: summary.relative_path,
            previous_relative_path: summary.previous_relative_path,
            status: normalized_status.to_string(),
            additions: summary.additions,
            deletions: summary.deletions,
            flags,
            capabilities,
            capability_reasons,
        },
    }
}

fn normalize_file_status(status: &str) -> &str {
    match status {
        "added" | "modified" | "deleted" | "renamed" | "type_changed" => status,
        _ => "unknown",
    }
}

fn extract_provider_file_patch(
    patch: &str,
    relative_path: &str,
    previous_relative_path: Option<&str>,
) -> Option<String> {
    let expected = format!(
        "diff --git a/{} b/{relative_path}",
        previous_relative_path.unwrap_or(relative_path)
    );
    let start = patch.match_indices("diff --git ").find_map(|(index, _)| {
        patch[index..]
            .lines()
            .next()
            .is_some_and(|header| header == expected)
            .then_some(index)
    })?;
    let remainder = &patch[start..];
    let end = remainder[1..]
        .find("\ndiff --git ")
        .map(|index| index + 2)
        .unwrap_or(remainder.len());
    Some(remainder[..end].to_string())
}

async fn require_request_ownership(
    pool: &SqlitePool,
    snapshot: &ReviewSnapshot,
    working_folder_id: &ProjectWorkingFolderId,
    execution_environment_id: Option<&str>,
) -> ChatResult<()> {
    let requested_environment =
        resolve_requested_environment_id(pool, working_folder_id, execution_environment_id).await?;
    if &snapshot.working_folder_id != working_folder_id
        || requested_environment != snapshot.environment_id
    {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Review snapshot belongs to another execution environment",
            false,
        ));
    }
    Ok(())
}

async fn resolve_requested_environment_id(
    pool: &SqlitePool,
    working_folder_id: &ProjectWorkingFolderId,
    execution_environment_id: Option<&str>,
) -> ChatResult<String> {
    match execution_environment_id {
        Some(environment) => Ok(environment.to_string()),
        None => sqlx::query_scalar(
            "SELECT id FROM chat_execution_environments
             WHERE working_folder_id = ? AND kind = 'current_folder'
               AND lifecycle_state = 'available' AND archived_at IS NULL",
        )
        .bind(working_folder_id.as_str())
        .fetch_optional(pool)
        .await
        .map_err(persistence_error)?
        .ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::ConfigurationInvalid,
                "The current-folder execution environment is unavailable",
                true,
            )
        }),
    }
}

async fn authorize_review_request(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    request: &OpenChatReviewRequest,
    operation: WorkingFolderAuthorizationOperation,
) -> ChatResult<(ProjectWorkingFolderId, String, AuthorizedWorkingFolder)> {
    if source_requires_thread(&request.source) {
        let thread_id = request.thread_id.as_ref().ok_or_else(thread_required)?;
        let (working_folder_id, environment_id, authorized, _) =
            super::execution_environment::authorize_thread_environment(
                app, pool, thread_id, operation,
            )
            .await?;
        let requested_environment = resolve_requested_environment_id(
            pool,
            &request.working_folder_id,
            request.execution_environment_id.as_deref(),
        )
        .await?;
        if working_folder_id != request.working_folder_id || requested_environment != environment_id
        {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Review source belongs to another workspace or execution environment",
                false,
            ));
        }
        return Ok((working_folder_id, environment_id, authorized));
    }
    let environment_id = resolve_requested_environment_id(
        pool,
        &request.working_folder_id,
        request.execution_environment_id.as_deref(),
    )
    .await?;
    if let Some(thread_id) = request.thread_id.as_ref() {
        let row = sqlx::query(
            "SELECT working_folder_id, execution_environment_id FROM chat_threads
             WHERE id = ? AND state != 'closed'",
        )
        .bind(thread_id.as_str())
        .fetch_optional(pool)
        .await
        .map_err(persistence_error)?
        .ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::NotFound,
                "Active Chat thread was not found",
                true,
            )
        })?;
        let thread_working_folder: String = row
            .try_get("working_folder_id")
            .map_err(persistence_error)?;
        let thread_environment: Option<String> = row
            .try_get("execution_environment_id")
            .map_err(persistence_error)?;
        if thread_working_folder != request.working_folder_id.as_str()
            || thread_environment.as_deref() != Some(environment_id.as_str())
        {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Chat thread belongs to another workspace or execution environment",
                false,
            ));
        }
    }
    let workspace = workspaces::read_workspace(pool, &request.working_folder_id).await?;
    let scope = read_active_working_folder_scope(app).map_err(|_| {
        ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "Working folder bindings are unavailable",
            true,
        )
    })?;
    let authorized = authorize_workspace(&workspace, &scope, operation)?;
    let authorized = super::execution_environment::resolve_environment_workspace(
        app,
        pool,
        authorized,
        Some(&environment_id),
    )
    .await?;
    Ok((
        request.working_folder_id.clone(),
        environment_id,
        authorized,
    ))
}

fn source_requires_thread(source: &ReviewDiffSource) -> bool {
    matches!(
        source,
        ReviewDiffSource::Checkpoint { .. } | ReviewDiffSource::ProviderTurn { .. }
    )
}

fn thread_required() -> ChatError {
    ChatError::validation(
        "threadId",
        "This review source requires an existing Chat thread",
    )
}

async fn authorize_snapshot(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    snapshot: &ReviewSnapshot,
    operation: WorkingFolderAuthorizationOperation,
) -> ChatResult<()> {
    let request = OpenChatReviewRequest {
        thread_id: snapshot.thread_id.clone(),
        working_folder_id: snapshot.working_folder_id.clone(),
        execution_environment_id: Some(snapshot.environment_id.clone()),
        source: snapshot.source.clone(),
        ignore_whitespace: snapshot.ignore_whitespace,
        context_lines: snapshot.context_lines,
        preferred_relative_path: None,
    };
    let (working_folder_id, environment_id, authorized) =
        authorize_review_request(app, pool, &request, operation).await?;
    if working_folder_id != snapshot.working_folder_id
        || environment_id != snapshot.environment_id
        || authorized.canonical_path != snapshot.root
    {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Review snapshot belongs to another execution environment",
            false,
        ));
    }
    Ok(())
}

async fn authorize_completed_action(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    request: &ApplyChatReviewActionRequest,
    result: &ChatReviewActionResultRead,
) -> ChatResult<()> {
    let completed_request = OpenChatReviewRequest {
        thread_id: request.thread_id.clone(),
        working_folder_id: request.working_folder_id.clone(),
        execution_environment_id: request.execution_environment_id.clone(),
        source: result.snapshot.source.clone(),
        ignore_whitespace: false,
        context_lines: default_context_lines(),
        preferred_relative_path: None,
    };
    authorize_review_request(
        app,
        pool,
        &completed_request,
        WorkingFolderAuthorizationOperation::Git,
    )
    .await
    .map(|_| ())
}

impl ChatReviewRegistry {
    pub(crate) async fn resolve_selection(
        &self,
        app: &tauri::AppHandle,
        pool: &SqlitePool,
        database_identity: &str,
        request: ResolveReviewSelectionRequest<'_>,
    ) -> ChatResult<ResolvedReviewSelection> {
        if !matches!(request.side, "old" | "new") {
            return Err(ChatError::validation(
                "selectionSide",
                "Snapshot review comments require an old or new side",
            ));
        }
        let thread_scope = Some(request.thread_id.clone());
        let snapshot =
            self.snapshot(request.snapshot_id, Some(&thread_scope), database_identity)?;
        require_snapshot_revision(&snapshot, request.review_revision)?;
        authorize_snapshot(
            app,
            pool,
            &snapshot,
            WorkingFolderAuthorizationOperation::FileRead,
        )
        .await?;
        let file = snapshot
            .files
            .iter()
            .find(|file| file.read.file_id == request.file_id)
            .ok_or_else(|| {
                ChatError::new(ChatErrorCode::NotFound, "Review file was not found", true)
            })?;
        if !file.read.capabilities.comment {
            return Err(ChatError::unsupported(
                file.read
                    .capability_reasons
                    .comment
                    .as_deref()
                    .unwrap_or("This file does not support line comments"),
            ));
        }
        let selected_text = if let Some(patch) = snapshot.provider_patches.get(request.file_id) {
            select_provider_patch_lines(
                patch,
                request.side,
                request.start_line,
                request.start_column,
                request.end_line,
                request.end_column,
            )?
        } else {
            let (oid, path) = if request.side == "old" {
                (
                    snapshot.before_oid.as_deref(),
                    file.read
                        .previous_relative_path
                        .as_deref()
                        .unwrap_or(&file.read.relative_path),
                )
            } else {
                (
                    snapshot.after_oid.as_deref(),
                    file.read.relative_path.as_str(),
                )
            };
            let oid = oid.ok_or_else(|| {
                ChatError::new(
                    ChatErrorCode::NotFound,
                    "The selected review side is unavailable",
                    true,
                )
            })?;
            let object = format!("{oid}:{path}");
            let contents = String::from_utf8(
                review_output(
                    &snapshot.root,
                    &["cat-file", "blob", &object],
                    None,
                    snapshot.object_store.as_deref(),
                )
                .await?,
            )
            .map_err(|_| ChatError::unsupported("The selected file is not UTF-8 text"))?;
            select_text_range(
                &contents,
                request.start_line,
                request.start_column,
                request.end_line,
                request.end_column,
            )?
        };
        Ok(ResolvedReviewSelection {
            relative_path: file.read.relative_path.clone(),
            previous_relative_path: file.read.previous_relative_path.clone(),
            content_revision: snapshot.review_revision.clone(),
            selected_text,
            source: snapshot.source.clone(),
        })
    }

    fn insert(&self, snapshot: Arc<ReviewSnapshot>) -> ChatResult<()> {
        let mut store = self.snapshots.lock().map_err(|_| registry_error())?;
        purge_snapshots(&mut store);
        store.order.push_back(snapshot.snapshot_id.clone());
        store.values.insert(snapshot.snapshot_id.clone(), snapshot);
        while store.order.len() > MAX_SNAPSHOTS {
            if let Some(id) = store.order.pop_front() {
                store.values.remove(&id);
            }
        }
        Ok(())
    }

    fn snapshot(
        &self,
        id: &str,
        thread_id: Option<&Option<ChatThreadId>>,
        database_identity: &str,
    ) -> ChatResult<Arc<ReviewSnapshot>> {
        let mut store = self.snapshots.lock().map_err(|_| registry_error())?;
        purge_snapshots(&mut store);
        let snapshot = store.values.get(id).cloned().ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::NotFound,
                "Review snapshot expired. Open Review again.",
                true,
            )
        })?;
        if thread_id.is_some_and(|thread_id| &snapshot.thread_id != thread_id) {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Review snapshot belongs to another Chat thread",
                false,
            ));
        }
        if snapshot.database_identity != database_identity {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Review snapshot belongs to another Ganbaru database",
                false,
            ));
        }
        Ok(snapshot)
    }

    fn completed(
        &self,
        operation_id: &str,
        thread_id: &Option<ChatThreadId>,
        working_folder_id: &ProjectWorkingFolderId,
        environment_id: &str,
        request_fingerprint: &str,
        database_identity: &str,
    ) -> ChatResult<Option<ChatReviewActionResultRead>> {
        let store = self.snapshots.lock().map_err(|_| registry_error())?;
        let storage_key = completed_operation_key(database_identity, operation_id);
        match store.completed.get(&storage_key) {
            Some(operation)
                if &operation.thread_id == thread_id
                    && operation.database_identity == database_identity
                    && &operation.working_folder_id == working_folder_id
                    && operation.environment_id == environment_id
                    && operation.request_fingerprint == request_fingerprint =>
            {
                Ok(Some(operation.result.clone()))
            }
            Some(_) => Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Review operation ID was already used for another review action",
                false,
            )),
            None => Ok(None),
        }
    }

    fn complete(&self, id: String, operation: CompletedReviewOperation) -> ChatResult<()> {
        let mut store = self.snapshots.lock().map_err(|_| registry_error())?;
        let storage_key = completed_operation_key(&operation.database_identity, &id);
        store.completed_order.push_back(storage_key.clone());
        store.completed.insert(storage_key, operation);
        while store.completed_order.len() > MAX_COMPLETED_OPERATIONS {
            if let Some(id) = store.completed_order.pop_front() {
                store.completed.remove(&id);
            }
        }
        Ok(())
    }
}

fn select_text_range(
    contents: &str,
    start_line: u64,
    start_column: u64,
    end_line: u64,
    end_column: u64,
) -> ChatResult<String> {
    let start = usize::try_from(start_line.saturating_sub(1)).map_err(|_| corrupt_data())?;
    let end = usize::try_from(end_line.saturating_sub(1)).map_err(|_| corrupt_data())?;
    if end < start {
        return Err(ChatError::validation(
            "range",
            "Review selection is outside the immutable file revision",
        ));
    }
    let full_lines = start_column == 1 && end_column == 1;
    let mut selection = String::new();
    let mut found_end = false;
    for (index, line) in contents.split('\n').enumerate() {
        if index < start {
            continue;
        }
        if index > end {
            break;
        }
        let from = if full_lines || index != start {
            0
        } else {
            column_byte_index(line, start_column)?
        };
        let to = if full_lines || index != end {
            line.len()
        } else {
            column_byte_index(line, end_column)?
        };
        if to < from {
            return Err(ChatError::validation("range", "Review range is invalid"));
        }
        if index > start {
            push_selection_text(&mut selection, "\n")?;
        }
        push_selection_text(&mut selection, &line[from..to])?;
        if index == end {
            found_end = true;
            break;
        }
    }
    if !found_end {
        return Err(ChatError::validation(
            "range",
            "Review selection is outside the immutable file revision",
        ));
    }
    Ok(selection)
}

fn column_byte_index(line: &str, column: u64) -> ChatResult<usize> {
    let target = usize::try_from(column.saturating_sub(1)).map_err(|_| corrupt_data())?;
    line.char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(line.len()))
        .nth(target)
        .ok_or_else(|| {
            ChatError::validation(
                "range",
                "Review column is outside the immutable file revision",
            )
        })
}

fn select_provider_patch_lines(
    patch: &str,
    side: &str,
    start_line: u64,
    start_column: u64,
    end_line: u64,
    end_column: u64,
) -> ChatResult<String> {
    let mut old_line = 0_u64;
    let mut new_line = 0_u64;
    let mut selected = String::new();
    let mut previous_selected_line = None;
    let full_lines = start_column == 1 && end_column == 1;
    let mut in_hunk = false;
    for line in patch.lines() {
        if line.starts_with("@@ ") {
            let (old_start, _, new_start, _) = parse_hunk_header(line)?;
            old_line = old_start;
            new_line = new_start;
            in_hunk = true;
            continue;
        }
        if !in_hunk {
            continue;
        }
        let (line_number, text) = if let Some(text) = line.strip_prefix('-') {
            let number = old_line;
            old_line = old_line.saturating_add(1);
            if side == "old" {
                (Some(number), text)
            } else {
                (None, text)
            }
        } else if let Some(text) = line.strip_prefix('+') {
            let number = new_line;
            new_line = new_line.saturating_add(1);
            if side == "new" {
                (Some(number), text)
            } else {
                (None, text)
            }
        } else if let Some(text) = line.strip_prefix(' ') {
            let number = if side == "old" { old_line } else { new_line };
            old_line = old_line.saturating_add(1);
            new_line = new_line.saturating_add(1);
            (Some(number), text)
        } else {
            continue;
        };
        if let Some(number) =
            line_number.filter(|number| *number >= start_line && *number <= end_line)
        {
            if (previous_selected_line.is_none() && number != start_line)
                || previous_selected_line.is_some_and(|previous| number != previous + 1)
            {
                return Err(ChatError::validation(
                    "range",
                    "Review selection crosses content omitted from the provider patch",
                ));
            }
            let from = if full_lines || number != start_line {
                0
            } else {
                column_byte_index(text, start_column)?
            };
            let to = if full_lines || number != end_line {
                text.len()
            } else {
                column_byte_index(text, end_column)?
            };
            if to < from {
                return Err(ChatError::validation("range", "Review range is invalid"));
            }
            if previous_selected_line.is_some() {
                push_selection_text(&mut selected, "\n")?;
            }
            push_selection_text(&mut selected, &text[from..to])?;
            previous_selected_line = Some(number);
        }
    }
    if previous_selected_line != Some(end_line) {
        return Err(ChatError::validation(
            "range",
            "Review selection is outside the provider patch",
        ));
    }
    Ok(selected)
}

fn push_selection_text(selection: &mut String, value: &str) -> ChatResult<()> {
    if selection.len().saturating_add(value.len()) > MAX_COMMENT_SELECTION_BYTES {
        return Err(ChatError::validation(
            "range",
            "Review selection exceeds the supported limit",
        ));
    }
    selection.push_str(value);
    Ok(())
}

fn purge_snapshots(store: &mut SnapshotStore) {
    while let Some(id) = store.order.front() {
        let expired = store
            .values
            .get(id)
            .is_none_or(|snapshot| snapshot.created_at.elapsed() > SNAPSHOT_LIFETIME);
        if !expired {
            break;
        }
        if let Some(id) = store.order.pop_front() {
            store.values.remove(&id);
        }
    }
}

fn review_revision(
    thread_id: Option<&ChatThreadId>,
    environment_id: &str,
    source: &ReviewDiffSource,
    before: &str,
    after: &str,
    _ignore_whitespace: bool,
    _context_lines: u32,
) -> ChatResult<String> {
    let mut hasher = Sha256::new();
    hasher.update(
        thread_id
            .map(ChatThreadId::as_str)
            .unwrap_or("workspace-draft"),
    );
    hasher.update([0]);
    hasher.update(environment_id);
    hasher.update([0]);
    hasher.update(serde_json::to_vec(source).map_err(|_| corrupt_data())?);
    hasher.update([0]);
    hasher.update(before);
    hasher.update([0]);
    hasher.update(after);
    Ok(format!("{:x}", hasher.finalize()))
}

fn snapshot_id(
    database_identity: &str,
    thread_id: Option<&ChatThreadId>,
    environment_id: &str,
    revision: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(database_identity);
    hasher.update([0]);
    hasher.update(
        thread_id
            .map(ChatThreadId::as_str)
            .unwrap_or("workspace-draft"),
    );
    hasher.update([0]);
    hasher.update(environment_id);
    hasher.update([0]);
    hasher.update(revision);
    hasher.update([0]);
    hasher.update(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .to_le_bytes(),
    );
    format!("review:{:x}", hasher.finalize())
}

fn completed_operation_key(database_identity: &str, operation_id: &str) -> String {
    format!(
        "{:x}",
        Sha256::digest([database_identity.as_bytes(), &[0], operation_id.as_bytes()].concat())
    )
}

fn file_id(revision: &str, path: &str, previous: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(revision);
    hasher.update([0]);
    hasher.update(path);
    hasher.update([0]);
    hasher.update(previous.unwrap_or_default());
    format!("review-file:{:x}", hasher.finalize())
}

fn hunk_id(file_id: &str, text: &str, old_start: u64, new_start: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(file_id);
    hasher.update([0]);
    let mut old_line = old_start;
    let mut new_line = new_start;
    for line in text.lines().skip(1) {
        if let Some(content) = line.strip_prefix(' ') {
            old_line = old_line.saturating_add(1);
            new_line = new_line.saturating_add(1);
            let _ = content;
        } else if let Some(content) = line.strip_prefix('-') {
            hasher.update([b'-']);
            hasher.update(old_line.to_le_bytes());
            hasher.update(content);
            hasher.update([0]);
            old_line = old_line.saturating_add(1);
        } else if let Some(content) = line.strip_prefix('+') {
            hasher.update([b'+']);
            hasher.update(new_line.to_le_bytes());
            hasher.update(content);
            hasher.update([0]);
            new_line = new_line.saturating_add(1);
        }
    }
    format!("review-hunk:{:x}", hasher.finalize())
}

fn action_request_fingerprint(request: &ApplyChatReviewActionRequest) -> ChatResult<String> {
    serde_json::to_vec(request)
        .map(|value| format!("{:x}", Sha256::digest(value)))
        .map_err(|_| corrupt_data())
}

async fn empty_tree(root: &Path) -> ChatResult<String> {
    git_text(root, &["mktree"], Some(&[])).await
}

async fn git_text(root: &Path, arguments: &[&str], input: Option<&[u8]>) -> ChatResult<String> {
    String::from_utf8(git_service::review_output(root, arguments, input).await?)
        .map(|value| value.trim().to_string())
        .map_err(|_| review_error("Git output is not valid UTF-8"))
}

fn validate_open_request(request: &OpenChatReviewRequest) -> ChatResult<()> {
    if source_requires_thread(&request.source) && request.thread_id.is_none() {
        return Err(thread_required());
    }
    if request.context_lines > MAX_CONTEXT_LINES {
        return Err(ChatError::validation(
            "contextLines",
            "Review context is too large",
        ));
    }
    if let Some(path) = request.preferred_relative_path.as_deref() {
        validate_path(path)?;
    }
    match &request.source {
        ReviewDiffSource::Commit { revision } => validate_reference(revision, "revision")?,
        ReviewDiffSource::Branch {
            base_ref, head_ref, ..
        } => {
            if let Some(base_ref) = base_ref {
                validate_reference(base_ref, "baseRef")?;
            }
            validate_reference(head_ref, "headRef")?;
        }
        ReviewDiffSource::ChangeRequest {
            provider,
            repository_slug,
            number,
        } => {
            if !matches!(
                provider.as_str(),
                "github" | "gitlab" | "azure_devops" | "bitbucket"
            ) {
                return Err(ChatError::validation(
                    "provider",
                    "Hosted source-control provider is invalid",
                ));
            }
            if repository_slug.is_empty()
                || repository_slug.len() > 2_048
                || repository_slug.contains('\0')
                || repository_slug.chars().any(char::is_control)
                || *number == 0
            {
                return Err(ChatError::validation(
                    "repositorySlug",
                    "Hosted repository reference is invalid",
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_patch_request(request: &ReadChatReviewPatchesRequest) -> ChatResult<()> {
    validate_opaque_id(&request.snapshot_id, "snapshotId")?;
    validate_revision(&request.review_revision)?;
    if request.file_ids.is_empty() || request.file_ids.len() > MAX_FILE_SELECTION {
        return Err(ChatError::validation(
            "fileIds",
            "Review patch requests require between one and 100 files",
        ));
    }
    for file_id in &request.file_ids {
        validate_opaque_id(file_id, "fileId")?;
    }
    if let Some(cursor) = request.continuation_cursor.as_deref() {
        validate_opaque_id(cursor, "continuationCursor")?;
    }
    Ok(())
}

fn validate_action_request(request: &ApplyChatReviewActionRequest) -> ChatResult<()> {
    validate_opaque_id(&request.snapshot_id, "snapshotId")?;
    validate_revision(&request.expected_review_revision)?;
    validate_opaque_id(&request.client_operation_id, "clientOperationId")?;
    if let Some(file_id) = request.file_id.as_deref() {
        validate_opaque_id(file_id, "fileId")?;
    }
    if request.hunk_ids.len() > 1_000 || (!request.hunk_ids.is_empty() && request.file_id.is_none())
    {
        return Err(ChatError::validation(
            "hunkIds",
            "Review hunk selection is invalid",
        ));
    }
    for hunk_id in &request.hunk_ids {
        validate_opaque_id(hunk_id, "hunkId")?;
    }
    Ok(())
}

fn validate_revision(value: &str) -> ChatResult<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ChatError::validation(
            "reviewRevision",
            "Review revision is invalid",
        ));
    }
    Ok(())
}

fn validate_opaque_id(value: &str, field: &str) -> ChatResult<()> {
    if value.is_empty()
        || value.len() > 1_024
        || value.chars().any(char::is_control)
        || value.chars().any(char::is_whitespace)
    {
        return Err(ChatError::validation(field, "Review identifier is invalid"));
    }
    Ok(())
}

fn validate_reference(value: &str, field: &str) -> ChatResult<()> {
    if value.is_empty()
        || value.len() > 1_024
        || value.starts_with('-')
        || value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
    {
        return Err(ChatError::validation(field, "Git reference is invalid"));
    }
    Ok(())
}

fn validate_path(path: &str) -> ChatResult<()> {
    if path.is_empty()
        || path.len() > 4_096
        || path.starts_with('/')
        || path.contains('\0')
        || path.contains('\\')
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
        || !Path::new(path)
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(ChatError::validation(
            "relativePath",
            "Review path is invalid",
        ));
    }
    Ok(())
}

fn path_field(field: Option<&[u8]>) -> ChatResult<String> {
    let value = field.ok_or_else(corrupt_data)?;
    let path = std::str::from_utf8(value).map_err(|_| corrupt_data())?;
    validate_path(path)?;
    Ok(path.to_string())
}

fn parse_cursor(value: &str, file_id: &str) -> Option<usize> {
    let (cursor_file, hunk) = value.rsplit_once('/')?;
    (cursor_file == file_id)
        .then(|| hunk.parse().ok())
        .flatten()
}

fn require_snapshot_revision(snapshot: &ReviewSnapshot, revision: &str) -> ChatResult<()> {
    if snapshot.review_revision != revision {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "Review snapshot revision does not match",
            true,
        ));
    }
    Ok(())
}

fn default_context_lines() -> u32 {
    3
}

async fn chat_pool(app: tauri::AppHandle, db_url: String) -> ChatResult<SqlitePool> {
    db_path::connect_sqlite(app, db_url)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Persistence, "open Chat database", true))
}

async fn database_identity(pool: &SqlitePool) -> ChatResult<String> {
    let path: String =
        sqlx::query_scalar("SELECT file FROM pragma_database_list WHERE name = 'main' LIMIT 1")
            .fetch_one(pool)
            .await
            .map_err(persistence_error)?;
    if path.is_empty() {
        return Err(persistence_error("Chat database path is unavailable"));
    }
    fs::canonicalize(path)
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(persistence_error)
}

fn review_error(_message: &str) -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        "Git review operation failed safely",
        true,
    )
}

fn registry_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        "Chat review registry is unavailable",
        true,
    )
}

fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat review persistence failed",
        true,
    )
}

fn corrupt_data() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Stored Chat review data is invalid",
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    struct TestRepository(PathBuf);

    impl TestRepository {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock should follow the Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ganbaru-chat-review-test-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("test repository should be created");
            let repository =
                Self(fs::canonicalize(path).expect("test repository path should canonicalize"));
            repository.command(&["init", "-q"]);
            repository.command(&["config", "user.name", "Ganbaru test"]);
            repository.command(&["config", "user.email", "test@ganbaru.invalid"]);
            repository.command(&["config", "commit.gpgsign", "false"]);
            repository
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn write(&self, relative_path: &str, contents: impl AsRef<[u8]>) {
            let path = self.0.join(relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("test file parent should be created");
            }
            fs::write(path, contents).expect("test file should be written");
        }

        fn command(&self, arguments: &[&str]) -> String {
            let output = Command::new("git")
                .arg("-C")
                .arg(&self.0)
                .args(arguments)
                .env("GIT_TERMINAL_PROMPT", "0")
                .output()
                .expect("Git fixture command should start");
            assert!(
                output.status.success(),
                "Git fixture command failed: {arguments:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout)
                .expect("Git fixture output should be UTF-8")
                .trim()
                .to_string()
        }

        fn commit_all(&self) {
            self.command(&["add", "-A", "--", "."]);
            self.command(&["commit", "-q", "-m", "test: add review fixture"]);
        }
    }

    impl Drop for TestRepository {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn working_source(mode: ReviewWorkingTreeMode) -> ReviewDiffSource {
        ReviewDiffSource::WorkingTree { mode }
    }

    fn test_file(
        source: &ReviewDiffSource,
        file_id: &str,
        relative_path: &str,
        status: &str,
        additions: Option<u64>,
        deletions: Option<u64>,
        binary: bool,
    ) -> ReviewFileInternal {
        let (flags, capabilities, capability_reasons) =
            file_contract(source, status, additions, deletions, binary);
        ReviewFileInternal {
            read: ReviewFileRead {
                file_id: file_id.to_string(),
                relative_path: relative_path.to_string(),
                previous_relative_path: None,
                status: status.to_string(),
                additions,
                deletions,
                flags,
                capabilities,
                capability_reasons,
            },
        }
    }

    fn provider_snapshot(patch: &str) -> (Arc<ReviewSnapshot>, String) {
        let source = ReviewDiffSource::ProviderTurn {
            turn_id: ChatTurnId::new("turn:review-test").expect("turn ID should be valid"),
        };
        let revision = "a".repeat(64);
        let file_id = file_id(&revision, "src/lib.rs", None);
        let file = test_file(
            &source,
            &file_id,
            "src/lib.rs",
            "modified",
            Some(2),
            Some(2),
            false,
        );
        let snapshot = Arc::new(ReviewSnapshot {
            database_identity: "database:review-test".to_string(),
            snapshot_id: "review:test".to_string(),
            review_revision: revision,
            thread_id: Some(
                ChatThreadId::new("thread:review-test").expect("thread ID should be valid"),
            ),
            working_folder_id: ProjectWorkingFolderId::new("workspace:review-test")
                .expect("working folder ID should be valid"),
            environment_id: "environment:review-test".to_string(),
            root: PathBuf::new(),
            source,
            source_label: "Provider turn".to_string(),
            before_oid: None,
            after_oid: None,
            context_lines: 3,
            ignore_whitespace: false,
            files: vec![file],
            provider_patches: HashMap::from([(file_id.clone(), patch.to_string())]),
            patch_cache: tokio::sync::Mutex::new(HashMap::new()),
            object_store: None,
            created_at: Instant::now(),
        });
        (snapshot, file_id)
    }

    async fn working_snapshot(
        repository: &TestRepository,
        mode: ReviewWorkingTreeMode,
    ) -> Arc<ReviewSnapshot> {
        let source = working_source(mode);
        let material = working_tree_material(repository.path(), mode)
            .await
            .expect("working snapshot should capture");
        let mut files = git_files(
            repository.path(),
            &material.before_oid,
            &material.after_oid,
            &source,
            false,
            material.object_store.as_deref(),
        )
        .await
        .expect("working snapshot files should read");
        enrich_working_tree_files(&mut files, &material.status, &source);
        let revision = review_revision(
            None,
            "environment:review-test",
            &source,
            &material.before_oid,
            &material.after_oid,
            false,
            3,
        )
        .expect("working snapshot revision should build");
        for file in &mut files {
            file.read.file_id = file_id(
                &revision,
                &file.read.relative_path,
                file.read.previous_relative_path.as_deref(),
            );
        }
        Arc::new(ReviewSnapshot {
            database_identity: "database:review-test".to_string(),
            snapshot_id: format!("review:{mode:?}"),
            review_revision: revision,
            thread_id: None,
            working_folder_id: ProjectWorkingFolderId::new("workspace:review-test")
                .expect("working folder ID should be valid"),
            environment_id: "environment:review-test".to_string(),
            root: repository.path().to_path_buf(),
            source,
            source_label: material.label,
            before_oid: Some(material.before_oid),
            after_oid: Some(material.after_oid),
            context_lines: 3,
            ignore_whitespace: false,
            files,
            provider_patches: HashMap::new(),
            patch_cache: tokio::sync::Mutex::new(HashMap::new()),
            object_store: material.object_store,
            created_at: Instant::now(),
        })
    }

    #[test]
    fn review_revision_is_stable_across_rendering_options_and_scoped_to_source() {
        let thread = ChatThreadId::new("thread:revision-test").expect("thread ID should be valid");
        let source = working_source(ReviewWorkingTreeMode::All);
        let baseline = review_revision(
            Some(&thread),
            "environment:one",
            &source,
            "before",
            "after",
            false,
            3,
        )
        .expect("review revision should be created");
        let different_rendering = review_revision(
            Some(&thread),
            "environment:one",
            &source,
            "before",
            "after",
            true,
            MAX_CONTEXT_LINES,
        )
        .expect("review revision should be created");

        assert_eq!(baseline, different_rendering);
        assert_eq!(baseline.len(), 64);
        assert_ne!(
            baseline,
            review_revision(
                Some(&thread),
                "environment:one",
                &working_source(ReviewWorkingTreeMode::Staged),
                "before",
                "after",
                false,
                3,
            )
            .expect("source-scoped revision should be created")
        );
        assert_ne!(
            baseline,
            review_revision(
                Some(&thread),
                "environment:two",
                &source,
                "before",
                "after",
                false,
                3,
            )
            .expect("environment-scoped revision should be created")
        );
        assert_eq!(
            file_id(&baseline, "src/lib.rs", None),
            file_id(&different_rendering, "src/lib.rs", None)
        );
        assert_ne!(
            file_id(&baseline, "src/lib.rs", None),
            file_id(&baseline, "src/lib.rs", Some("src/old.rs"))
        );
        assert_ne!(
            snapshot_id("database:one", Some(&thread), "environment:one", &baseline),
            snapshot_id("database:two", Some(&thread), "environment:one", &baseline)
        );
        assert_ne!(
            completed_operation_key("database:one", "operation:shared"),
            completed_operation_key("database:two", "operation:shared")
        );
    }

    #[test]
    fn patch_parser_preserves_complete_hunks_and_stable_change_ids() {
        let with_context = concat!(
            "diff --git a/src/lib.rs b/src/lib.rs\n",
            "--- a/src/lib.rs\n",
            "+++ b/src/lib.rs\n",
            "@@ -10,3 +10,3 @@ fn value()\n",
            " keep\n",
            "-old\n",
            "+new\n",
            " tail\n",
            "@@ -30 +30 @@\n",
            "-before\n",
            "+after\n"
        );
        let without_context = "@@ -11 +11 @@\n-old\n+new\n";
        let parsed =
            parse_patch(with_context, "file:stable").expect("complete Git patch should parse");
        let compact =
            parse_patch(without_context, "file:stable").expect("compact Git patch should parse");

        assert_eq!(parsed.hunks.len(), 2);
        assert_eq!(parsed.hunks[0].read.old_start, 10);
        assert_eq!(parsed.hunks[0].read.old_count, 3);
        assert_eq!(parsed.hunks[0].read.new_start, 10);
        assert_eq!(parsed.hunks[0].read.new_count, 3);
        assert_eq!(parsed.hunks[1].read.old_count, 1);
        assert_eq!(parsed.hunks[1].read.new_count, 1);
        assert_eq!(parsed.hunks[0].read.hunk_id, compact.hunks[0].read.hunk_id);
        assert_ne!(
            parsed.hunks[0].read.hunk_id,
            hunk_id("file:stable", "@@ -11 +11 @@\n-old\n+other\n", 11, 11)
        );
        assert!(parsed.preamble.starts_with("diff --git "));
        assert!(parsed.hunks.iter().all(|hunk| {
            hunk.text.starts_with("@@ ") && hunk.read.state == ReviewPatchState::Complete
        }));
    }

    #[tokio::test]
    async fn provider_turn_prefers_a_unified_patch_over_later_raw_file_contents() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("review fixture database should open");
        sqlx::query(
            "CREATE TABLE chat_events (
               thread_id TEXT NOT NULL,
               turn_id TEXT,
               sequence INTEGER NOT NULL,
               event_type TEXT NOT NULL,
               payload_data TEXT NOT NULL
             )",
        )
        .execute(&pool)
        .await
        .expect("review fixture table should be created");
        let thread_id = ChatThreadId::new("thread:provider-patch").expect("thread ID is valid");
        let turn_id = ChatTurnId::new("turn:provider-patch").expect("turn ID is valid");
        let summary = ChangedFileSummary {
            relative_path: "hello.py".to_string(),
            previous_relative_path: None,
            additions: Some(1),
            deletions: Some(0),
            binary: false,
            status: "added".to_string(),
        };
        let unified = CanonicalEvent::DiffUpdated(crate::chat::events::DiffUpdatedEvent {
            source: "provider".to_string(),
            files: vec![summary],
            provider_diff: Some(
                "diff --git a/hello.py b/hello.py\nnew file mode 100644\n@@ -0,0 +1 @@\n+print('hello')\n"
                    .to_string(),
            ),
        });
        let raw = CanonicalEvent::DiffUpdated(crate::chat::events::DiffUpdatedEvent {
            source: "provider_file_change".to_string(),
            files: vec![ChangedFileSummary {
                relative_path: "/workspace/hello.py".to_string(),
                previous_relative_path: None,
                additions: Some(0),
                deletions: Some(0),
                binary: false,
                status: "modified".to_string(),
            }],
            provider_diff: Some("print('hello')\n".to_string()),
        });
        for (sequence, event) in [(1_i64, unified), (2_i64, raw)] {
            sqlx::query(
                "INSERT INTO chat_events
                   (thread_id, turn_id, sequence, event_type, payload_data)
                 VALUES (?, ?, ?, 'diff_updated', ?)",
            )
            .bind(thread_id.as_str())
            .bind(turn_id.as_str())
            .bind(sequence)
            .bind(serde_json::to_string(&event).expect("event should serialize"))
            .execute(&pool)
            .await
            .expect("review fixture event should insert");
        }

        let (files, patch) = provider_turn_patch(&pool, &thread_id, &turn_id)
            .await
            .expect("unified provider patch should load");

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].relative_path, "hello.py");
        assert!(patch.starts_with("diff --git a/hello.py b/hello.py"));
    }

    #[tokio::test]
    async fn patch_pages_stop_only_between_hunks_and_resume_with_the_cursor() {
        let patch = concat!(
            "diff --git a/src/lib.rs b/src/lib.rs\n",
            "--- a/src/lib.rs\n",
            "+++ b/src/lib.rs\n",
            "@@ -1 +1 @@\n",
            "-first old\n",
            "+first new\n",
            "@@ -10 +10 @@\n",
            "-second old\n",
            "+second new\n"
        );
        let (snapshot, file_id) = provider_snapshot(patch);
        let parsed = parse_patch(patch, &file_id).expect("fixture patch should parse");
        let first_page_limit = parsed.preamble.len()
            + parsed.hunks[0].text.len()
            + parsed.hunks[0].read.hunk_id.len()
            + 256;

        let first = read_patch_page(&snapshot, &file_id, 0, first_page_limit)
            .await
            .expect("first patch page should read");
        assert_eq!(first.state, ReviewPatchState::Partial);
        assert_eq!(first.hunks.len(), 1);
        assert_eq!(first.hunks[0].hunk_id, parsed.hunks[0].read.hunk_id);
        let expected_first_patch = format!("{}{}", parsed.preamble, parsed.hunks[0].text);
        assert_eq!(first.patch.as_deref(), Some(expected_first_patch.as_str()));
        let expected_cursor = format!("{file_id}/1");
        assert_eq!(
            first.continuation_cursor.as_deref(),
            Some(expected_cursor.as_str())
        );

        let second = read_patch_page(&snapshot, &file_id, 1, patch.len() + 4_096)
            .await
            .expect("second patch page should read");
        assert_eq!(second.state, ReviewPatchState::Partial);
        assert_eq!(second.hunks.len(), 1);
        assert_eq!(second.hunks[0].hunk_id, parsed.hunks[1].read.hunk_id);
        assert!(second.continuation_cursor.is_none());
        let expected_second_patch = format!("{}{}", parsed.preamble, parsed.hunks[1].text);
        assert_eq!(
            second.patch.as_deref(),
            Some(expected_second_patch.as_str())
        );
    }

    #[tokio::test]
    async fn oversized_hunk_advances_without_returning_partial_patch_text() {
        let patch = concat!(
            "diff --git a/src/lib.rs b/src/lib.rs\n",
            "--- a/src/lib.rs\n",
            "+++ b/src/lib.rs\n",
            "@@ -1 +1 @@\n",
            "-a very long first line\n",
            "+another very long first line\n",
            "@@ -10 +10 @@\n",
            "-small\n",
            "+tiny\n"
        );
        let (snapshot, file_id) = provider_snapshot(patch);
        let parsed = parse_patch(patch, &file_id).expect("fixture patch should parse");
        let too_small = parsed.preamble.len() + parsed.hunks[0].text.len() - 1;

        let oversized = read_patch_page(&snapshot, &file_id, 0, too_small)
            .await
            .expect("oversized patch page should be described");
        assert_eq!(oversized.state, ReviewPatchState::OversizedHunk);
        assert_eq!(oversized.hunks.len(), 1);
        assert_eq!(oversized.hunks[0].state, ReviewPatchState::OversizedHunk);
        assert!(oversized.patch.is_none());
        let expected_cursor = format!("{file_id}/1");
        assert_eq!(
            oversized.continuation_cursor.as_deref(),
            Some(expected_cursor.as_str())
        );

        let resumed = read_patch_page(&snapshot, &file_id, 1, patch.len() + 4_096)
            .await
            .expect("page after oversized hunk should read");
        assert_eq!(resumed.hunks.len(), 1);
        assert_eq!(resumed.hunks[0].hunk_id, parsed.hunks[1].read.hunk_id);
        assert!(resumed.patch.as_deref().is_some_and(|value| {
            value.contains("-small\n+tiny\n") && !value.contains("very long first line")
        }));
    }

    #[test]
    fn canonical_relative_paths_accept_unicode_and_reject_ambiguous_components() {
        for valid in [
            "src/lib.rs",
            "src/café file.rs",
            ".config/settings.json",
            "データ/結果.txt",
        ] {
            validate_path(valid).expect("canonical relative path should be accepted");
        }
        for invalid in [
            "",
            "/src/lib.rs",
            "src//lib.rs",
            "src/./lib.rs",
            "src/../lib.rs",
            "../src/lib.rs",
            "src\\lib.rs",
            "src/\0lib.rs",
            "src/\nlib.rs",
        ] {
            let error = validate_path(invalid).expect_err("unsafe path should be rejected");
            assert_eq!(error.code, ChatErrorCode::Validation);
            assert_eq!(error.field.as_deref(), Some("relativePath"));
        }
    }

    #[test]
    fn unicode_review_selection_uses_character_columns_and_bounded_lines() {
        let contents = "αβγ\ncafé\n終";
        assert_eq!(
            select_text_range(contents, 1, 2, 2, 5).expect("multiline Unicode range should select"),
            "βγ\ncafé"
        );
        assert_eq!(
            select_text_range(contents, 2, 1, 3, 1)
                .expect("whole-line Unicode range should select"),
            "café\n終"
        );
        let invalid = select_text_range(contents, 1, 5, 1, 6)
            .expect_err("column beyond Unicode content should be rejected");
        assert_eq!(invalid.code, ChatErrorCode::Validation);
        assert_eq!(invalid.field.as_deref(), Some("range"));
    }

    #[test]
    fn provider_selection_treats_marker_shaped_hunk_lines_as_content() {
        let patch = concat!(
            "diff --git a/src/lib.rs b/src/lib.rs\n",
            "--- a/src/lib.rs\n",
            "+++ b/src/lib.rs\n",
            "@@ -4 +4 @@\n",
            "---old marker\n",
            "+++new marker\n"
        );
        assert_eq!(
            select_provider_patch_lines(patch, "old", 4, 1, 4, 1)
                .expect("old marker-shaped content should select"),
            "--old marker"
        );
        assert_eq!(
            select_provider_patch_lines(patch, "new", 4, 1, 4, 1)
                .expect("new marker-shaped content should select"),
            "++new marker"
        );
    }

    #[test]
    fn patch_requests_require_an_explicit_bounded_file_selection() {
        let request = ReadChatReviewPatchesRequest {
            thread_id: None,
            working_folder_id: ProjectWorkingFolderId::new("workspace:review-test")
                .expect("working folder ID should be valid"),
            execution_environment_id: Some("environment:review-test".to_string()),
            snapshot_id: "review:test".to_string(),
            review_revision: "a".repeat(64),
            file_ids: Vec::new(),
            continuation_cursor: None,
            byte_limit: None,
        };
        let error = validate_patch_request(&request)
            .expect_err("implicit all-file patch reads should be rejected");
        assert_eq!(error.code, ChatErrorCode::Validation);
        assert_eq!(error.field.as_deref(), Some("fileIds"));
    }

    #[test]
    fn name_status_and_status_metadata_cover_rename_binary_untracked_and_conflict() {
        let source = working_source(ReviewWorkingTreeMode::Unstaged);
        let stats = BTreeMap::from([
            ("src/new.rs".to_string(), (Some(0), Some(0))),
            ("assets/image.bin".to_string(), (None, None)),
            ("src/untracked.rs".to_string(), (Some(1), Some(0))),
            ("src/conflict.rs".to_string(), (Some(2), Some(2))),
        ]);
        let names = b"R100\0src/old.rs\0src/new.rs\0M\0assets/image.bin\0A\0src/untracked.rs\0U\0src/conflict.rs\0";
        let mut files =
            parse_name_status(names, &stats, &source).expect("name status should parse");
        let status = git_service::GitStatusRead {
            branch: Some("main".to_string()),
            detached: false,
            upstream: None,
            ahead: 0,
            behind: 0,
            files: vec![git_service::GitChangedPathRead {
                relative_path: "src/conflict.rs".to_string(),
                original_relative_path: None,
                index_status: "U".to_string(),
                worktree_status: "U".to_string(),
                untracked: false,
                ignored: false,
                conflicted: true,
            }],
        };
        enrich_working_tree_files(&mut files, &status, &source);

        let renamed = files
            .iter()
            .find(|file| file.read.relative_path == "src/new.rs")
            .expect("rename should exist");
        assert_eq!(
            renamed.read.previous_relative_path.as_deref(),
            Some("src/old.rs")
        );
        assert_eq!(renamed.read.status, "renamed");
        assert!(renamed.read.flags.pure_rename);
        let binary = files
            .iter()
            .find(|file| file.read.relative_path == "assets/image.bin")
            .expect("binary change should exist");
        assert!(binary.read.flags.binary);
        assert!(!binary.read.capabilities.comment);
        let untracked = files
            .iter()
            .find(|file| file.read.relative_path == "src/untracked.rs")
            .expect("untracked change should exist");
        assert!(untracked.read.flags.untracked);
        let conflict = files
            .iter()
            .find(|file| file.read.relative_path == "src/conflict.rs")
            .expect("conflict should exist");
        assert!(conflict.read.flags.conflict);
        assert!(!conflict.read.capabilities.stage);
        assert!(!conflict.read.capabilities.unstage);
        assert!(!conflict.read.capabilities.discard);
    }

    #[test]
    fn conflict_status_is_added_even_when_head_and_worktree_contents_match() {
        let source = working_source(ReviewWorkingTreeMode::All);
        let mut files = Vec::new();
        let status = git_service::GitStatusRead {
            branch: Some("main".to_string()),
            detached: false,
            upstream: None,
            ahead: 0,
            behind: 0,
            files: vec![git_service::GitChangedPathRead {
                relative_path: "src/conflict.rs".to_string(),
                original_relative_path: None,
                index_status: "U".to_string(),
                worktree_status: "U".to_string(),
                untracked: false,
                ignored: false,
                conflicted: true,
            }],
        };

        enrich_working_tree_files(&mut files, &status, &source);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].read.relative_path, "src/conflict.rs");
        assert_eq!(files[0].read.status, "modified");
        assert!(files[0].read.flags.conflict);
        assert!(!files[0].read.capabilities.stage);
        assert!(!files[0].read.capabilities.unstage);
        assert!(!files[0].read.capabilities.discard);
    }

    #[tokio::test]
    async fn immutable_git_snapshot_observes_rename_binary_and_untracked_files() {
        let repository = TestRepository::new();
        repository.write(
            "src/original.rs",
            "fn stable_name() {\n    println!(\"stable\");\n}\n",
        );
        repository.write("assets/data.bin", [0_u8, 1, 2, 3]);
        repository.commit_all();
        fs::rename(
            repository.path().join("src/original.rs"),
            repository.path().join("src/renamed.rs"),
        )
        .expect("fixture file should rename");
        repository.write("assets/data.bin", [0_u8, 1, 9, 3]);
        repository.write("src/untracked.rs", "pub fn new_file() {}\n");

        let source = working_source(ReviewWorkingTreeMode::All);
        let material = working_tree_material(repository.path(), ReviewWorkingTreeMode::All)
            .await
            .expect("immutable worktree tree should be captured");
        let mut files = git_files(
            repository.path(),
            &material.before_oid,
            &material.after_oid,
            &source,
            false,
            material.object_store.as_deref(),
        )
        .await
        .expect("immutable Git diff should read");
        let status = git_service::status(repository.path())
            .await
            .expect("Git status should read");
        enrich_working_tree_files(&mut files, &status, &source);

        let renamed = files
            .iter()
            .find(|file| file.read.relative_path == "src/renamed.rs")
            .expect("renamed file should be present");
        assert_eq!(
            renamed.read.previous_relative_path.as_deref(),
            Some("src/original.rs")
        );
        assert!(renamed.read.flags.pure_rename);
        assert!(files.iter().any(|file| {
            file.read.relative_path == "assets/data.bin" && file.read.flags.binary
        }));
        assert!(files.iter().any(|file| {
            file.read.relative_path == "src/untracked.rs" && file.read.flags.untracked
        }));
    }

    #[tokio::test]
    async fn whole_scope_actions_do_not_materialize_large_patch_text() {
        let repository = TestRepository::new();
        repository.write("tracked.txt", "before\n");
        repository.commit_all();
        repository.write("tracked.txt", "after\n");
        repository.write("new.txt", "new\n");

        let unstaged = working_snapshot(&repository, ReviewWorkingTreeMode::Unstaged).await;
        let mut request = ApplyChatReviewActionRequest {
            thread_id: None,
            snapshot_id: unstaged.snapshot_id.clone(),
            working_folder_id: unstaged.working_folder_id.clone(),
            execution_environment_id: Some(unstaged.environment_id.clone()),
            expected_review_revision: unstaged.review_revision.clone(),
            file_id: None,
            hunk_ids: Vec::new(),
            operation: ReviewAction::Stage,
            confirmed: false,
            client_operation_id: "operation:stage-all".to_string(),
        };
        apply_action(&unstaged, &request, ReviewWorkingTreeMode::Unstaged)
            .await
            .expect("whole scope should stage without creating a patch payload");
        let staged_status = git_service::status(repository.path())
            .await
            .expect("staged status should read");
        assert!(staged_status
            .files
            .iter()
            .all(|file| file.index_status != "." && !file.untracked));

        let staged = working_snapshot(&repository, ReviewWorkingTreeMode::Staged).await;
        request.snapshot_id = staged.snapshot_id.clone();
        request.expected_review_revision = staged.review_revision.clone();
        request.operation = ReviewAction::Unstage;
        request.client_operation_id = "operation:unstage-all".to_string();
        apply_action(&staged, &request, ReviewWorkingTreeMode::Staged)
            .await
            .expect("whole scope should unstage without creating a patch payload");
        let unstaged_status = git_service::status(repository.path())
            .await
            .expect("unstaged status should read");
        assert!(unstaged_status
            .files
            .iter()
            .all(|file| file.index_status == "." || file.untracked));

        let discard = working_snapshot(&repository, ReviewWorkingTreeMode::Unstaged).await;
        request.snapshot_id = discard.snapshot_id.clone();
        request.expected_review_revision = discard.review_revision.clone();
        request.operation = ReviewAction::Discard;
        request.confirmed = true;
        request.client_operation_id = "operation:discard-all".to_string();
        apply_action(&discard, &request, ReviewWorkingTreeMode::Unstaged)
            .await
            .expect("whole scope should discard without creating a patch payload");
        assert!(git_service::status(repository.path())
            .await
            .expect("clean status should read")
            .files
            .is_empty());
    }

    #[tokio::test]
    async fn exact_discard_restores_unstaged_rename_with_untracked_destination() {
        let repository = TestRepository::new();
        repository.write("src/original.rs", "pub fn original() {}\n");
        repository.commit_all();
        repository.command(&["config", "status.renames", "false"]);
        fs::rename(
            repository.path().join("src/original.rs"),
            repository.path().join("src/renamed.rs"),
        )
        .expect("fixture file should rename");

        let snapshot = working_snapshot(&repository, ReviewWorkingTreeMode::Unstaged).await;
        let renamed = snapshot
            .files
            .iter()
            .find(|file| file.read.relative_path == "src/renamed.rs")
            .expect("synthetic review tree should detect the rename");
        assert_eq!(
            renamed.read.previous_relative_path.as_deref(),
            Some("src/original.rs")
        );
        assert!(renamed.read.flags.untracked);
        let request = ApplyChatReviewActionRequest {
            thread_id: None,
            snapshot_id: snapshot.snapshot_id.clone(),
            working_folder_id: snapshot.working_folder_id.clone(),
            execution_environment_id: Some(snapshot.environment_id.clone()),
            expected_review_revision: snapshot.review_revision.clone(),
            file_id: Some(renamed.read.file_id.clone()),
            hunk_ids: Vec::new(),
            operation: ReviewAction::Discard,
            confirmed: true,
            client_operation_id: "operation:discard-rename".to_string(),
        };

        apply_action(&snapshot, &request, ReviewWorkingTreeMode::Unstaged)
            .await
            .expect("exact rename discard should restore the original path");

        assert_eq!(
            fs::read_to_string(repository.path().join("src/original.rs"))
                .expect("original file should be restored"),
            "pub fn original() {}\n"
        );
        assert!(!repository.path().join("src/renamed.rs").exists());
        assert!(git_service::status(repository.path())
            .await
            .expect("discarded rename status should read")
            .files
            .is_empty());
    }

    #[tokio::test]
    async fn whole_scope_discard_restores_unstaged_rename_with_untracked_destination() {
        let repository = TestRepository::new();
        repository.write("src/original.rs", "pub fn original() {}\n");
        repository.write("src/modified.rs", "pub fn value() -> u8 { 1 }\n");
        repository.commit_all();
        repository.command(&["config", "status.renames", "false"]);
        fs::rename(
            repository.path().join("src/original.rs"),
            repository.path().join("src/renamed.rs"),
        )
        .expect("fixture file should rename");
        repository.write("src/modified.rs", "pub fn value() -> u8 { 2 }\n");

        let snapshot = working_snapshot(&repository, ReviewWorkingTreeMode::Unstaged).await;
        let renamed = snapshot
            .files
            .iter()
            .find(|file| file.read.relative_path == "src/renamed.rs")
            .expect("synthetic review tree should detect the rename");
        assert_eq!(
            renamed.read.previous_relative_path.as_deref(),
            Some("src/original.rs")
        );
        assert!(renamed.read.flags.untracked);
        let request = ApplyChatReviewActionRequest {
            thread_id: None,
            snapshot_id: snapshot.snapshot_id.clone(),
            working_folder_id: snapshot.working_folder_id.clone(),
            execution_environment_id: Some(snapshot.environment_id.clone()),
            expected_review_revision: snapshot.review_revision.clone(),
            file_id: None,
            hunk_ids: Vec::new(),
            operation: ReviewAction::Discard,
            confirmed: true,
            client_operation_id: "operation:discard-scope-with-rename".to_string(),
        };

        apply_action(&snapshot, &request, ReviewWorkingTreeMode::Unstaged)
            .await
            .expect("whole-scope discard should restore the original path");

        assert_eq!(
            fs::read_to_string(repository.path().join("src/original.rs"))
                .expect("original file should be restored"),
            "pub fn original() {}\n"
        );
        assert!(!repository.path().join("src/renamed.rs").exists());
        assert_eq!(
            fs::read_to_string(repository.path().join("src/modified.rs"))
                .expect("modified file should be restored"),
            "pub fn value() -> u8 { 1 }\n"
        );
        assert!(git_service::status(repository.path())
            .await
            .expect("discarded scope status should read")
            .files
            .is_empty());
    }

    #[test]
    fn action_validation_rejects_unsafe_scope_and_hunk_combinations() {
        let unstaged_source = working_source(ReviewWorkingTreeMode::Unstaged);
        let normal = test_file(
            &unstaged_source,
            "file:normal",
            "src/lib.rs",
            "modified",
            Some(1),
            Some(1),
            false,
        );
        require_action_allowed(
            ReviewWorkingTreeMode::Unstaged,
            ReviewAction::Stage,
            Some(&normal),
            &["hunk:one".to_string()],
        )
        .expect("unstaged text hunk should be stageable");
        let wrong_scope = require_action_allowed(
            ReviewWorkingTreeMode::Staged,
            ReviewAction::Discard,
            Some(&normal),
            &[],
        )
        .expect_err("staged changes should not discard directly");
        assert_eq!(wrong_scope.code, ChatErrorCode::CapabilityUnsupported);

        let mut rename = test_file(
            &unstaged_source,
            "file:rename",
            "src/new.rs",
            "renamed",
            Some(0),
            Some(0),
            false,
        );
        rename.read.previous_relative_path = Some("src/old.rs".to_string());
        let partial_rename = require_action_allowed(
            ReviewWorkingTreeMode::Unstaged,
            ReviewAction::Stage,
            Some(&rename),
            &["hunk:rename".to_string()],
        )
        .expect_err("pure rename should require a whole-file action");
        assert_eq!(partial_rename.code, ChatErrorCode::CapabilityUnsupported);

        let request = ApplyChatReviewActionRequest {
            thread_id: None,
            snapshot_id: "review:test".to_string(),
            working_folder_id: ProjectWorkingFolderId::new("workspace:review-test")
                .expect("working folder ID should be valid"),
            execution_environment_id: Some("environment:review-test".to_string()),
            expected_review_revision: "a".repeat(64),
            file_id: None,
            hunk_ids: vec!["hunk:without-file".to_string()],
            operation: ReviewAction::Stage,
            confirmed: false,
            client_operation_id: "operation:test".to_string(),
        };
        let invalid = validate_action_request(&request)
            .expect_err("hunk action without file should be rejected");
        assert_eq!(invalid.code, ChatErrorCode::Validation);
        assert_eq!(invalid.field.as_deref(), Some("hunkIds"));
    }

    #[tokio::test]
    async fn discard_requires_confirmation_before_any_workspace_access() {
        let source = working_source(ReviewWorkingTreeMode::Unstaged);
        let file = test_file(
            &source,
            "file:discard",
            "src/lib.rs",
            "modified",
            Some(1),
            Some(1),
            false,
        );
        let snapshot = Arc::new(ReviewSnapshot {
            database_identity: "database:review-test".to_string(),
            snapshot_id: "review:discard".to_string(),
            review_revision: "b".repeat(64),
            thread_id: None,
            working_folder_id: ProjectWorkingFolderId::new("workspace:review-test")
                .expect("working folder ID should be valid"),
            environment_id: "environment:review-test".to_string(),
            root: PathBuf::from("path-that-must-not-be-read"),
            source,
            source_label: "Unstaged changes".to_string(),
            before_oid: Some("before".to_string()),
            after_oid: Some("after".to_string()),
            context_lines: 3,
            ignore_whitespace: false,
            files: vec![file],
            provider_patches: HashMap::new(),
            patch_cache: tokio::sync::Mutex::new(HashMap::new()),
            object_store: None,
            created_at: Instant::now(),
        });
        let request = ApplyChatReviewActionRequest {
            thread_id: None,
            snapshot_id: snapshot.snapshot_id.clone(),
            working_folder_id: snapshot.working_folder_id.clone(),
            execution_environment_id: Some(snapshot.environment_id.clone()),
            expected_review_revision: snapshot.review_revision.clone(),
            file_id: None,
            hunk_ids: Vec::new(),
            operation: ReviewAction::Discard,
            confirmed: false,
            client_operation_id: "operation:discard".to_string(),
        };

        let error = apply_action(&snapshot, &request, ReviewWorkingTreeMode::Unstaged)
            .await
            .expect_err("unconfirmed discard should stop before workspace access");
        assert_eq!(error.code, ChatErrorCode::Permission);
    }
}
