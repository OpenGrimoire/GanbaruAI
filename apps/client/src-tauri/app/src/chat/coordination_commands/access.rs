//! Teammate-centric organizational access commands.

use super::common::{
    new_id, parse_access_profile_builtin_key, parse_folder_capability, parse_history_boundary,
    parse_runtime_approval_policy, wire_folder_capability, wire_history_boundary,
    wire_runtime_approval_policy,
};
use super::*;
use ganbaru_chat::chat::coordination::access::history_boundary_is_expansion;
use ganbaru_chat::chat::coordination::access_profiles::profile_revision_is_expansion;
use sqlx::{Row, SqlitePool};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
struct ResolvedChannelAccess {
    channel_id: ChatChannelId,
    project_id: String,
    group_id: String,
    conversation_id: ChatConversationId,
    channel_name: String,
    access_profile_id: ChatAccessProfileId,
    access_profile_revision: u64,
    profile_default_channel_capabilities: ChatChannelCapabilities,
    profile_default_history_boundary: ChatHistoryBoundary,
    profile_maximum_folder_capability: ChatFolderCapability,
    capabilities: ChatChannelCapabilities,
    history_boundary: ChatHistoryBoundary,
    runtime_approval_override: Option<ChatRuntimeApprovalPolicy>,
    scratch_runtime_approval_override: Option<ChatRuntimeApprovalPolicy>,
    folder_grants: Vec<ChatFolderGrantInput>,
}

struct ValidatedTeammateConfiguration {
    display_name: String,
    role: String,
    instructions: String,
    avatar_schema_version: u32,
    avatar_data: String,
    expected_revision: u64,
}

fn validate_teammate_configuration(
    app: &tauri::AppHandle,
    teammate_id: &ChatParticipantId,
    teammate_profile: Option<&UpdateChatTeammateProfileCommand>,
    policy: Option<&ChatTeammatePolicyInput>,
) -> ChatResult<Option<ValidatedTeammateConfiguration>> {
    match (teammate_profile, policy) {
        (None, None) => Ok(None),
        (Some(profile), Some(policy)) => {
            if &profile.teammate_id != teammate_id {
                return Err(ChatError::validation(
                    "teammateProfile.teammateId",
                    "The teammate profile does not match the access draft",
                ));
            }
            let display_name = validate_display_name(&profile.display_name)?;
            let role = validate_teammate_role(&profile.role)?;
            validate_profile_text(&profile.instructions, 65_536, "instructions")?;
            validate_policy(app, policy)?;
            Ok(Some(ValidatedTeammateConfiguration {
                display_name,
                role,
                instructions: profile.instructions.trim().to_string(),
                avatar_schema_version: profile.avatar.schema_version,
                avatar_data: json_object(&profile.avatar, "avatar")?,
                expected_revision: profile.expected_revision,
            }))
        }
        _ => Err(ChatError::validation(
            "teammateProfile",
            "Teammate profile and policy changes must be saved together",
        )),
    }
}

#[tauri::command]
pub async fn chat_list_access_profiles(
    app: tauri::AppHandle,
    db_url: String,
    archived: bool,
) -> ChatResult<Vec<ChatAccessProfileRead>> {
    let pool = chat_pool(app, db_url).await?;
    let rows = sqlx::query(
        "SELECT id FROM chat_access_profiles
         WHERE (archived_at IS NOT NULL) = ?
         ORDER BY builtin_key IS NULL, display_name COLLATE NOCASE, id",
    )
    .bind(archived)
    .fetch_all(&pool)
    .await
    .map_err(persistence_error)?;
    let mut profiles = Vec::with_capacity(rows.len());
    for row in rows {
        let id =
            ChatAccessProfileId::new(row.try_get::<String, _>("id").map_err(persistence_error)?)
                .map_err(identifier_error)?;
        profiles.push(read_access_profile(&pool, &id).await?);
    }
    Ok(profiles)
}

#[tauri::command]
pub async fn chat_create_access_profile(
    app: tauri::AppHandle,
    db_url: String,
    request: CreateChatAccessProfileCommand,
) -> ChatResult<ChatAccessProfileRead> {
    let display_name = validate_access_profile_name(&request.display_name)?;
    validate_profile_revision_input(&request.revision)?;
    let pool = chat_pool(app, db_url).await?;
    let now = now_timestamp()?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_access_profiles
            (id, display_name, latest_revision, revision, created_at, updated_at)
         VALUES (?, ?, 1, 1, ?, ?)",
    )
    .bind(request.access_profile_id.as_str())
    .bind(display_name)
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(map_access_profile_write_error)?;
    insert_access_profile_revision(
        &mut transaction,
        &request.access_profile_id,
        1,
        &request.revision,
        &now,
    )
    .await?;
    transaction.commit().await.map_err(persistence_error)?;
    read_access_profile(&pool, &request.access_profile_id).await
}

#[tauri::command]
pub async fn chat_duplicate_access_profile(
    app: tauri::AppHandle,
    db_url: String,
    request: DuplicateChatAccessProfileCommand,
) -> ChatResult<ChatAccessProfileRead> {
    let pool = chat_pool(app.clone(), db_url.clone()).await?;
    let source = read_access_profile(&pool, &request.source_access_profile_id).await?;
    chat_create_access_profile(
        app,
        db_url,
        CreateChatAccessProfileCommand {
            access_profile_id: request.access_profile_id,
            display_name: request.display_name,
            revision: ChatAccessProfileRevisionInput {
                default_channel_capabilities: source.latest_revision.default_channel_capabilities,
                default_history_boundary: source.latest_revision.default_history_boundary,
                maximum_folder_capability: source.latest_revision.maximum_folder_capability,
            },
        },
    )
    .await
}

#[tauri::command]
pub async fn chat_preview_access_profile_revision(
    app: tauri::AppHandle,
    db_url: String,
    request: PreviewChatAccessProfileRevisionCommand,
) -> ChatResult<ChatAccessProfileImpactPreview> {
    validate_profile_revision_input(&request.revision)?;
    let pool = chat_pool(app, db_url).await?;
    let current = read_access_profile(&pool, &request.access_profile_id).await?;
    if current.revision != request.expected_revision {
        return Err(stale_access_profile());
    }
    let is_expansion = profile_revision_is_expansion(
        current.latest_revision.default_channel_capabilities,
        &current.latest_revision.default_history_boundary,
        current.latest_revision.maximum_folder_capability,
        request.revision.default_channel_capabilities,
        &request.revision.default_history_boundary,
        request.revision.maximum_folder_capability,
    );
    let is_reduction = profile_revision_is_reduction(&current.latest_revision, &request.revision);
    let membership_rows = sqlx::query(
        "SELECT DISTINCT membership.teammate_id, channel.id AS channel_id
         FROM chat_ai_channel_memberships membership
         JOIN chat_channels channel ON channel.conversation_id = membership.conversation_id
         WHERE membership.access_profile_id = ?
         ORDER BY membership.teammate_id, channel.id",
    )
    .bind(request.access_profile_id.as_str())
    .fetch_all(&pool)
    .await
    .map_err(persistence_error)?;
    let affected_teammate_ids = membership_rows
        .iter()
        .map(|row| {
            ChatParticipantId::new(
                row.try_get::<String, _>("teammate_id")
                    .map_err(persistence_error)?,
            )
            .map_err(identifier_error)
        })
        .collect::<ChatResult<BTreeSet<_>>>()?
        .into_iter()
        .collect::<Vec<_>>();
    let affected_channel_ids = membership_rows
        .iter()
        .map(|row| {
            ChatChannelId::new(
                row.try_get::<String, _>("channel_id")
                    .map_err(persistence_error)?,
            )
            .map_err(identifier_error)
        })
        .collect::<ChatResult<BTreeSet<_>>>()?
        .into_iter()
        .collect::<Vec<_>>();
    let active_authorization_count: i64 = sqlx::query_scalar(
        "SELECT count(*)
         FROM chat_assignment_authorization_revisions authorization
         JOIN chat_access_profile_revisions revision
           ON revision.id = authorization.access_profile_revision_id
         WHERE revision.access_profile_id = ?
           AND authorization.decision_state = 'allowed'
           AND authorization.revoked_at IS NULL",
    )
    .bind(request.access_profile_id.as_str())
    .fetch_one(&pool)
    .await
    .map_err(persistence_error)?;
    let mut issues = if current.builtin_key.is_some() {
        vec![issue(
            ChatAccessIssueCode::ProfileCeilingExceeded,
            "accessProfileId",
            "Built-in access profiles are immutable",
        )]
    } else {
        Vec::new()
    };
    if current.builtin_key.is_none() && is_expansion {
        let mut transaction = pool.begin().await.map_err(persistence_error)?;
        let preview_now = now_timestamp()?;
        insert_access_profile_revision(
            &mut transaction,
            &request.access_profile_id,
            current.latest_revision.revision + 1,
            &request.revision,
            &preview_now,
        )
        .await?;
        if is_reduction {
            apply_profile_reduction(
                &mut transaction,
                &request.access_profile_id,
                &request.revision,
                &preview_now,
            )
            .await?;
        }
        issues.extend(
            retained_profile_reference_issues_in_transaction(
                &mut transaction,
                &request.access_profile_id,
            )
            .await?,
        );
        transaction.rollback().await.map_err(persistence_error)?;
    }
    Ok(ChatAccessProfileImpactPreview {
        access_profile_id: request.access_profile_id,
        current_revision: current.latest_revision,
        is_expansion,
        is_reduction,
        affected_teammate_ids,
        affected_channel_ids,
        active_authorization_count: u64_value(active_authorization_count)?,
        issues,
    })
}

#[tauri::command]
pub async fn chat_publish_access_profile_revision(
    app: tauri::AppHandle,
    db_url: String,
    request: PublishChatAccessProfileRevisionCommand,
) -> ChatResult<ChatAccessProfileRead> {
    validate_profile_revision_input(&request.revision)?;
    let pool = chat_pool(app.clone(), db_url).await?;
    let current = read_access_profile(&pool, &request.access_profile_id).await?;
    if current.builtin_key.is_some() {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Built-in access profiles cannot be edited",
            true,
        ));
    }
    let is_expansion = profile_revision_is_expansion(
        current.latest_revision.default_channel_capabilities,
        &current.latest_revision.default_history_boundary,
        current.latest_revision.maximum_folder_capability,
        request.revision.default_channel_capabilities,
        &request.revision.default_history_boundary,
        request.revision.maximum_folder_capability,
    );
    let is_reduction = profile_revision_is_reduction(&current.latest_revision, &request.revision);
    let next_revision = current.latest_revision.revision + 1;
    let now = now_timestamp()?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    let locked = sqlx::query(
        "UPDATE chat_access_profiles SET updated_at = updated_at
         WHERE id = ? AND revision = ? AND builtin_key IS NULL",
    )
    .bind(request.access_profile_id.as_str())
    .bind(i64_value(request.expected_revision)?)
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    if locked.rows_affected() != 1 {
        return Err(stale_access_profile());
    }
    insert_access_profile_revision(
        &mut transaction,
        &request.access_profile_id,
        next_revision,
        &request.revision,
        &now,
    )
    .await?;
    if is_reduction {
        apply_profile_reduction(
            &mut transaction,
            &request.access_profile_id,
            &request.revision,
            &now,
        )
        .await?;
    }
    if is_expansion {
        let retained_issues = retained_profile_reference_issues_in_transaction(
            &mut transaction,
            &request.access_profile_id,
        )
        .await?;
        if let Some(issue) = retained_issues.first() {
            return Err(ChatError::validation(
                &issue.field_path,
                issue.message.clone(),
            ));
        }
    }
    if is_reduction {
        revoke_profile_authorizations(
            &mut transaction,
            &request.access_profile_id,
            &now,
            "Access profile authority was reduced",
        )
        .await?;
    }
    if is_expansion {
        // Existing authorization revisions keep their narrower immutable snapshots.
    }
    transaction.commit().await.map_err(persistence_error)?;
    if is_reduction {
        super::super::revocation::drain_access_revocation_jobs(&app, &pool).await?;
    }
    read_access_profile(&pool, &request.access_profile_id).await
}

async fn apply_profile_reduction(
    transaction: &mut Transaction<'_, Sqlite>,
    access_profile_id: &ChatAccessProfileId,
    revision: &ChatAccessProfileRevisionInput,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let history_boundary = wire_history_boundary(&revision.default_history_boundary);
    sqlx::query(
        "UPDATE chat_ai_channel_memberships
         SET history_from_ordinal = (
               SELECT coalesce(max(item.ordinal), 0) + 1
               FROM chat_conversation_items item
               WHERE item.conversation_id = chat_ai_channel_memberships.conversation_id
                 AND item.reply_thread_id IS NULL
             ),
             history_boundary = 'from_grant',
             updated_at = ?
         WHERE access_profile_id = ?
           AND history_boundary_inherits_profile = 1
           AND ? = 'from_grant'
           AND (history_boundary != 'from_grant' OR history_from_ordinal IS NULL)",
    )
    .bind(now.as_str())
    .bind(access_profile_id.as_str())
    .bind(history_boundary)
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_ai_teammate_access_state
         SET access_revision = access_revision + 1, updated_at = ?
         WHERE teammate_id IN (
           SELECT teammate_id FROM chat_ai_channel_memberships
           WHERE access_profile_id = ?
         )",
    )
    .bind(now.as_str())
    .bind(access_profile_id.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

#[tauri::command]
pub async fn chat_archive_access_profile(
    app: tauri::AppHandle,
    db_url: String,
    request: ArchiveChatAccessProfileCommand,
) -> ChatResult<ChatAccessProfileRead> {
    let pool = chat_pool(app, db_url).await?;
    let current = read_access_profile(&pool, &request.access_profile_id).await?;
    if current.builtin_key.is_some() {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Built-in access profiles cannot be archived",
            true,
        ));
    }
    if request.archived {
        let membership_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM chat_ai_channel_memberships
             WHERE access_profile_id = ?",
        )
        .bind(request.access_profile_id.as_str())
        .fetch_one(&pool)
        .await
        .map_err(persistence_error)?;
        if membership_count > 0 {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Reassign channel access before archiving this profile",
                true,
            ));
        }
    }
    let now = now_timestamp()?;
    let updated = sqlx::query(
        "UPDATE chat_access_profiles
         SET archived_at = ?, revision = revision + 1, updated_at = ?
         WHERE id = ? AND revision = ? AND builtin_key IS NULL",
    )
    .bind(request.archived.then(|| now.as_str()))
    .bind(now.as_str())
    .bind(request.access_profile_id.as_str())
    .bind(i64_value(request.expected_revision)?)
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    if updated.rows_affected() != 1 {
        return Err(stale_access_profile());
    }
    read_access_profile(&pool, &request.access_profile_id).await
}

#[tauri::command]
pub async fn chat_read_teammate_access(
    app: tauri::AppHandle,
    db_url: String,
    teammate_id: ChatParticipantId,
) -> ChatResult<ChatTeammateAccessRead> {
    read_teammate_access(&chat_pool(app, db_url).await?, &teammate_id).await
}

#[tauri::command]
pub async fn chat_read_channel_roster(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
) -> ChatResult<ChatChannelRosterRead> {
    let pool = chat_pool(app, db_url).await?;
    let channel = super::super::channel_commands::read_channel(&pool, &channel_id).await?;
    let audience_revision: i64 = sqlx::query_scalar(
        "SELECT revision FROM chat_conversation_audience_state WHERE conversation_id = ?",
    )
    .bind(channel.conversation_id.as_str())
    .fetch_one(&pool)
    .await
    .map_err(persistence_error)?;
    Ok(ChatChannelRosterRead {
        channel_id,
        conversation_id: channel.conversation_id.clone(),
        audience_revision: u64_value(audience_revision)?,
        memberships: read_memberships_for_conversation(&pool, &channel.conversation_id, false)
            .await?,
    })
}

#[tauri::command]
pub async fn chat_list_assignment_targets(
    app: tauri::AppHandle,
    db_url: String,
    request: ListChatAssignmentTargetsCommand,
) -> ChatResult<Vec<ChatAssignmentTargetRead>> {
    let pool = chat_pool(app.clone(), db_url.clone()).await?;
    let channel = super::super::channel_commands::read_channel(&pool, &request.channel_id).await?;
    let access = read_teammate_access(&pool, &request.teammate_id).await?;
    let channel_access = access
        .channels
        .iter()
        .find(|candidate| candidate.conversation_id == channel.conversation_id)
        .ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::NotFound,
                "The teammate is not a member of this channel",
                true,
            )
        })?;
    if !channel_access.capabilities.participate {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "The teammate cannot participate in this channel",
            true,
        ));
    }
    let folder_reads =
        super::super::workspace_commands::projects_list_working_folders_cached(app.clone(), db_url)
            .await?;
    let bindings = folder_reads
        .into_iter()
        .map(|read| (read.working_folder.id.clone(), read.binding_status))
        .collect::<BTreeMap<_, _>>();
    let mut targets = Vec::new();
    for grant in &channel_access.folder_grants {
        let rows = sqlx::query(
            "SELECT environment.id, environment.kind, environment.display_name,
                    environment.lifecycle_state, worktree.cleanup_state
             FROM chat_execution_environments environment
             LEFT JOIN chat_worktrees worktree
               ON worktree.execution_environment_id = environment.id
             WHERE environment.working_folder_id = ?
               AND environment.archived_at IS NULL
             ORDER BY environment.kind = 'current_folder' DESC,
                      environment.created_at, environment.id",
        )
        .bind(grant.working_folder_id.as_str())
        .fetch_all(&pool)
        .await
        .map_err(persistence_error)?;
        let binding_state = match bindings.get(&grant.working_folder_id) {
            Some(ganbaru_chat::chat::workspace::WorkingFolderBindingStatus::Available) => {
                ChatAssignmentTargetBindingState::Ready
            }
            Some(ganbaru_chat::chat::workspace::WorkingFolderBindingStatus::Unbound) => {
                ChatAssignmentTargetBindingState::Locate
            }
            Some(ganbaru_chat::chat::workspace::WorkingFolderBindingStatus::Missing) => {
                ChatAssignmentTargetBindingState::Missing
            }
            Some(ganbaru_chat::chat::workspace::WorkingFolderBindingStatus::RepositoryMismatch) => {
                ChatAssignmentTargetBindingState::Relink
            }
            None => ChatAssignmentTargetBindingState::UnavailableOnThisDevice,
        };
        for row in rows {
            let environment_id = ChatExecutionEnvironmentId::new(
                row.try_get::<String, _>("id").map_err(persistence_error)?,
            )
            .map_err(identifier_error)?;
            let environment_kind: String = row.try_get("kind").map_err(persistence_error)?;
            let lifecycle_state = parse_assignment_target_lifecycle(
                &row.try_get::<String, _>("lifecycle_state")
                    .map_err(persistence_error)?,
            )?;
            let is_busy: i64 = sqlx::query_scalar(
                "SELECT EXISTS(
                    SELECT 1 FROM chat_agent_runs
                    WHERE execution_environment_id = ?
                      AND state IN ('queued', 'starting', 'working', 'waiting')
                 )",
            )
            .bind(environment_id.as_str())
            .fetch_one(&pool)
            .await
            .map_err(persistence_error)?;
            let is_worktree = environment_kind == "worktree";
            let eligible = grant.capability != ChatFolderCapability::None
                && binding_state == ChatAssignmentTargetBindingState::Ready
                && lifecycle_state == ChatAssignmentTargetLifecycleState::Available;
            targets.push(ChatAssignmentTargetRead {
                execution_target: Some(ChatExecutionTarget::WorkingFolder {
                    working_folder_id: grant.working_folder_id.clone(),
                    execution_environment_id: environment_id,
                }),
                kind: if is_worktree {
                    ChatAssignmentTargetKind::Worktree
                } else {
                    ChatAssignmentTargetKind::CurrentFolder
                },
                display_name: row.try_get("display_name").map_err(persistence_error)?,
                folder_capability: Some(grant.capability),
                effective_runtime_approval: grant
                    .runtime_approval_override
                    .or(channel_access.runtime_approval_override)
                    .unwrap_or(access.teammate_default_runtime_approval),
                is_default: grant.is_default && !is_worktree,
                binding_state,
                lifecycle_state,
                is_busy: is_busy != 0,
                is_dirty: row
                    .try_get::<Option<String>, _>("cleanup_state")
                    .map_err(persistence_error)?
                    .and_then(|state| (state == "dirty").then_some(true)),
                eligible,
                unavailable_reason: None,
            });
        }
    }
    let scratch = read_scratch_assignment_target(
        &app,
        &pool,
        &request.teammate_id,
        request.reply_thread_id.as_ref(),
        channel_access.scratch_runtime_approval_override,
        channel_access.runtime_approval_override,
        access.teammate_default_runtime_approval,
    )
    .await?;
    targets.push(scratch);
    targets.sort_by(|left, right| {
        right
            .is_default
            .cmp(&left.is_default)
            .then_with(|| {
                assignment_target_kind_rank(left.kind).cmp(&assignment_target_kind_rank(right.kind))
            })
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    Ok(targets)
}

async fn read_scratch_assignment_target(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    teammate_id: &ChatParticipantId,
    reply_thread_id: Option<&ChatReplyThreadId>,
    scratch_override: Option<ChatRuntimeApprovalPolicy>,
    channel_override: Option<ChatRuntimeApprovalPolicy>,
    teammate_default: ChatRuntimeApprovalPolicy,
) -> ChatResult<ChatAssignmentTargetRead> {
    let row = match reply_thread_id {
        Some(reply_thread_id) => sqlx::query(
            "SELECT scope.id AS scratch_scope_id, generation.id AS scratch_generation_id,
                    environment.id AS execution_environment_id,
                    environment.lifecycle_state
             FROM chat_scratch_scopes scope
             JOIN chat_scratch_generations generation
               ON generation.scratch_scope_id = scope.id
              AND generation.lifecycle_state = 'active'
             JOIN chat_execution_environments environment
               ON environment.scratch_generation_id = generation.id
              AND environment.archived_at IS NULL
             WHERE scope.reply_thread_id = ? AND scope.teammate_id = ?
               AND scope.removed_at IS NULL",
        )
        .bind(reply_thread_id.as_str())
        .bind(teammate_id.as_str())
        .fetch_optional(pool)
        .await
        .map_err(persistence_error)?,
        None => None,
    };
    let (execution_target, lifecycle_state, is_busy, binding_state) = match row {
        Some(row) => {
            let environment_id = ChatExecutionEnvironmentId::new(
                row.try_get::<String, _>("execution_environment_id")
                    .map_err(persistence_error)?,
            )
            .map_err(identifier_error)?;
            let is_busy: i64 = sqlx::query_scalar(
                "SELECT EXISTS(
                    SELECT 1 FROM chat_agent_runs
                    WHERE execution_environment_id = ?
                      AND state IN ('queued', 'starting', 'working', 'waiting')
                 )",
            )
            .bind(environment_id.as_str())
            .fetch_one(pool)
            .await
            .map_err(persistence_error)?;
            let generation_id = ChatScratchGenerationId::new(
                row.try_get::<String, _>("scratch_generation_id")
                    .map_err(persistence_error)?,
            )
            .map_err(identifier_error)?;
            let binding_state =
                if super::super::scratch::resolve_managed_scratch_path_for_inspection(
                    app,
                    pool,
                    generation_id.as_str(),
                    environment_id.as_str(),
                )
                .await
                .is_ok()
                {
                    ChatAssignmentTargetBindingState::Ready
                } else {
                    ChatAssignmentTargetBindingState::UnavailableOnThisDevice
                };
            (
                Some(ChatExecutionTarget::Scratch {
                    scratch_scope_id: ChatScratchScopeId::new(
                        row.try_get::<String, _>("scratch_scope_id")
                            .map_err(persistence_error)?,
                    )
                    .map_err(identifier_error)?,
                    scratch_generation_id: generation_id,
                    execution_environment_id: environment_id,
                }),
                parse_assignment_target_lifecycle(
                    &row.try_get::<String, _>("lifecycle_state")
                        .map_err(persistence_error)?,
                )?,
                is_busy != 0,
                binding_state,
            )
        }
        None => (
            None,
            ChatAssignmentTargetLifecycleState::Available,
            false,
            ChatAssignmentTargetBindingState::Ready,
        ),
    };
    Ok(ChatAssignmentTargetRead {
        execution_target,
        kind: ChatAssignmentTargetKind::Scratch,
        display_name: "Private scratch".to_string(),
        folder_capability: None,
        effective_runtime_approval: scratch_override
            .or(channel_override)
            .unwrap_or(teammate_default),
        is_default: false,
        binding_state,
        eligible: lifecycle_state == ChatAssignmentTargetLifecycleState::Available
            && binding_state == ChatAssignmentTargetBindingState::Ready,
        lifecycle_state,
        is_busy,
        is_dirty: None,
        unavailable_reason: (binding_state
            == ChatAssignmentTargetBindingState::UnavailableOnThisDevice)
            .then(|| "Private scratch is unavailable on this device".to_string()),
    })
}

fn parse_assignment_target_lifecycle(
    value: &str,
) -> ChatResult<ChatAssignmentTargetLifecycleState> {
    match value {
        "creating" => Ok(ChatAssignmentTargetLifecycleState::Creating),
        "available" => Ok(ChatAssignmentTargetLifecycleState::Available),
        "missing" => Ok(ChatAssignmentTargetLifecycleState::Missing),
        "cleanup_pending" => Ok(ChatAssignmentTargetLifecycleState::CleanupPending),
        "cleanup_failed" => Ok(ChatAssignmentTargetLifecycleState::CleanupFailed),
        "removed" => Ok(ChatAssignmentTargetLifecycleState::Removed),
        _ => Err(ChatError::new(
            ChatErrorCode::Persistence,
            "Stored assignment target lifecycle is invalid",
            false,
        )),
    }
}

const fn assignment_target_kind_rank(kind: ChatAssignmentTargetKind) -> u8 {
    match kind {
        ChatAssignmentTargetKind::CurrentFolder => 0,
        ChatAssignmentTargetKind::Worktree => 1,
        ChatAssignmentTargetKind::Scratch => 2,
    }
}

#[tauri::command]
pub async fn chat_preview_channel_membership_removal(
    app: tauri::AppHandle,
    db_url: String,
    request: PreviewChatChannelMembershipRemovalCommand,
) -> ChatResult<ChatChannelMembershipRemovalPreview> {
    let pool = chat_pool(app, db_url).await?;
    let channel = super::super::channel_commands::read_channel(&pool, &request.channel_id).await?;
    let mut proposed_access = read_teammate_access(&pool, &request.teammate_id).await?;
    if proposed_access.access_revision != request.expected_access_revision {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The teammate access changed before the removal preview",
            true,
        ));
    }
    if !proposed_access
        .channels
        .iter()
        .any(|member| member.channel_id == request.channel_id)
    {
        return Err(ChatError::new(
            ChatErrorCode::NotFound,
            "The teammate is not a member of this channel",
            true,
        ));
    }
    let active_assignment_count: i64 = sqlx::query_scalar(
        "SELECT count(*)
         FROM chat_work_assignments assignment
         JOIN chat_reply_threads thread ON thread.id = assignment.reply_thread_id
         WHERE assignment.teammate_id = ? AND thread.conversation_id = ?
           AND assignment.state IN (
             'queued', 'working', 'waiting_for_answer',
             'waiting_for_approval', 'ready_for_review'
           )",
    )
    .bind(request.teammate_id.as_str())
    .bind(channel.conversation_id.as_str())
    .fetch_one(&pool)
    .await
    .map_err(persistence_error)?;
    let active_authorization_count: i64 = sqlx::query_scalar(
        "SELECT count(*)
         FROM chat_assignment_authorization_revisions authorization
         JOIN chat_work_assignments assignment ON assignment.id = authorization.assignment_id
         WHERE assignment.teammate_id = ?
           AND authorization.destination_conversation_id = ?
           AND authorization.decision_state = 'allowed'
           AND authorization.revoked_at IS NULL",
    )
    .bind(request.teammate_id.as_str())
    .bind(channel.conversation_id.as_str())
    .fetch_one(&pool)
    .await
    .map_err(persistence_error)?;
    proposed_access
        .channels
        .retain(|member| member.channel_id != request.channel_id);
    proposed_access.access_revision += 1;
    Ok(ChatChannelMembershipRemovalPreview {
        teammate_id: request.teammate_id,
        channel_id: request.channel_id,
        active_assignment_count: u64_value(active_assignment_count)?,
        active_authorization_count: u64_value(active_authorization_count)?,
        will_revoke_active_work: active_assignment_count > 0 || active_authorization_count > 0,
        proposed_access,
    })
}

#[tauri::command]
pub async fn chat_preview_teammate_access(
    app: tauri::AppHandle,
    db_url: String,
    request: PreviewChatTeammateAccessCommand,
) -> ChatResult<ChatTeammateAccessPreview> {
    let validated_configuration = validate_teammate_configuration(
        &app,
        &request.teammate_id,
        request.teammate_profile.as_ref(),
        request.policy.as_ref(),
    )?;
    let pool = chat_pool(app, db_url).await?;
    if let Some(configuration) = validated_configuration {
        let current = read_teammate(&pool, &request.teammate_id).await?;
        if current.participant.revision != configuration.expected_revision {
            return Err(ChatError::new(
                ChatErrorCode::StaleRevision,
                "The teammate changed before the draft could be previewed",
                true,
            ));
        }
    }
    preview_teammate_access(
        &pool,
        &request.teammate_id,
        request.expected_access_revision,
        request.teammate_default_runtime_approval,
        &request.channels,
    )
    .await
}

#[tauri::command]
pub async fn chat_replace_teammate_access(
    app: tauri::AppHandle,
    db_url: String,
    request: ReplaceChatTeammateAccessCommand,
) -> ChatResult<ChatTeammateAccessRead> {
    let validated_configuration = validate_teammate_configuration(
        &app,
        &request.teammate_id,
        request.teammate_profile.as_ref(),
        request.policy.as_ref(),
    )?;
    let pool = chat_pool(app.clone(), db_url).await?;
    let preview = preview_teammate_access(
        &pool,
        &request.teammate_id,
        request.expected_access_revision,
        request.teammate_default_runtime_approval,
        &request.channels,
    )
    .await?;
    if let Some(issue) = preview.issues.first() {
        return Err(ChatError::validation(
            &issue.field_path,
            issue.message.clone(),
        ));
    }
    let current = read_teammate_access(&pool, &request.teammate_id).await?;
    let resolved = resolve_channel_access(&pool, &request.teammate_id, &request.channels)
        .await?
        .0;
    let has_reduction = teammate_access_has_reduction(
        &current,
        request.teammate_default_runtime_approval,
        &resolved,
    );
    let now = now_timestamp()?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    let retained_issues =
        retained_reference_issues_in_transaction(&mut transaction, &resolved).await?;
    if let Some(issue) = retained_issues.first() {
        return Err(ChatError::validation(
            &issue.field_path,
            issue.message.clone(),
        ));
    }
    let updated = sqlx::query(
        "UPDATE chat_ai_teammate_access_state
         SET access_revision = access_revision + 1,
             runtime_approval_policy = ?, updated_at = ?
         WHERE teammate_id = ? AND access_revision = ?",
    )
    .bind(wire_runtime_approval_policy(
        request.teammate_default_runtime_approval,
    ))
    .bind(now.as_str())
    .bind(request.teammate_id.as_str())
    .bind(i64_value(request.expected_access_revision)?)
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    if updated.rows_affected() != 1 {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The teammate access changed before it could be saved",
            true,
        ));
    }
    require_expected_profile_revisions_in_transaction(&mut transaction, &request.channels).await?;

    if let Some(configuration) = validated_configuration {
        let updated = sqlx::query(
            "UPDATE chat_participants
             SET display_name = ?, avatar_schema_version = ?, avatar_data = ?,
                 revision = revision + 1, updated_at = ?
             WHERE id = ? AND participant_kind = 'ai_teammate' AND revision = ?",
        )
        .bind(configuration.display_name)
        .bind(i64::from(configuration.avatar_schema_version))
        .bind(configuration.avatar_data)
        .bind(now.as_str())
        .bind(request.teammate_id.as_str())
        .bind(i64_value(configuration.expected_revision)?)
        .execute(&mut *transaction)
        .await
        .map_err(map_teammate_write_error)?;
        if updated.rows_affected() != 1 {
            return Err(ChatError::new(
                ChatErrorCode::StaleRevision,
                "The teammate changed before the access draft could be saved",
                true,
            ));
        }
        sqlx::query(
            "UPDATE chat_ai_teammates SET role = ?, instructions = ?, updated_at = ?
             WHERE participant_id = ?",
        )
        .bind(configuration.role)
        .bind(configuration.instructions)
        .bind(now.as_str())
        .bind(request.teammate_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        let next_policy_revision: i64 = sqlx::query_scalar(
            "SELECT latest_policy_revision + 1
             FROM chat_ai_teammates WHERE participant_id = ?",
        )
        .bind(request.teammate_id.as_str())
        .fetch_one(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        insert_policy_revision(
            &mut transaction,
            &request.teammate_id,
            u64_value(next_policy_revision)?,
            request
                .policy
                .as_ref()
                .expect("validated teammate policy must be present"),
            &now,
        )
        .await?;
    }

    let proposed_conversations = resolved
        .iter()
        .map(|channel| channel.conversation_id.as_str())
        .collect::<BTreeSet<_>>();
    let existing_conversations = sqlx::query_scalar::<_, String>(
        "SELECT conversation_id FROM chat_ai_channel_memberships
         WHERE teammate_id = ?",
    )
    .bind(request.teammate_id.as_str())
    .fetch_all(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    for conversation_id in existing_conversations {
        if !proposed_conversations.contains(conversation_id.as_str()) {
            sqlx::query(
                "UPDATE chat_teammate_working_folder_grants
                 SET revoked_at = ?, is_default = 0, revision = revision + 1
                 WHERE conversation_id = ? AND teammate_id = ? AND revoked_at IS NULL",
            )
            .bind(now.as_str())
            .bind(&conversation_id)
            .bind(request.teammate_id.as_str())
            .execute(&mut *transaction)
            .await
            .map_err(persistence_error)?;
            sqlx::query(
                "DELETE FROM chat_ai_channel_memberships
                 WHERE conversation_id = ? AND teammate_id = ?",
            )
            .bind(&conversation_id)
            .bind(request.teammate_id.as_str())
            .execute(&mut *transaction)
            .await
            .map_err(persistence_error)?;
            sqlx::query(
                "UPDATE chat_conversation_memberships
                 SET removed_at = ?, revision = revision + 1, updated_at = ?
                 WHERE conversation_id = ? AND participant_id = ?",
            )
            .bind(now.as_str())
            .bind(now.as_str())
            .bind(&conversation_id)
            .bind(request.teammate_id.as_str())
            .execute(&mut *transaction)
            .await
            .map_err(persistence_error)?;
        }
    }

    for channel in &resolved {
        sqlx::query(
            "INSERT INTO chat_conversation_memberships
                (conversation_id, participant_id, membership_role, created_at, updated_at)
             VALUES (?, ?, 'member', ?, ?)
             ON CONFLICT(conversation_id, participant_id) DO UPDATE SET
                membership_role = 'member', removed_at = NULL,
                revision = chat_conversation_memberships.revision + 1,
                updated_at = excluded.updated_at",
        )
        .bind(channel.conversation_id.as_str())
        .bind(request.teammate_id.as_str())
        .bind(now.as_str())
        .bind(now.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        let history_from_ordinal = match channel.history_boundary {
            ChatHistoryBoundary::Entire => None,
            ChatHistoryBoundary::FromGrant { lower_ordinal } => {
                Some(i64_value(lower_ordinal.ok_or_else(|| {
                    ChatError::new(
                        ChatErrorCode::Persistence,
                        "Resolved history boundary is missing its lower ordinal",
                        false,
                    )
                })?)?)
            }
        };
        let read_history_inherits_profile = channel.capabilities.read_history
            == channel.profile_default_channel_capabilities.read_history;
        let participate_inherits_profile = channel.capabilities.participate
            == channel.profile_default_channel_capabilities.participate;
        let history_boundary_inherits_profile = wire_history_boundary(&channel.history_boundary)
            == wire_history_boundary(&channel.profile_default_history_boundary);
        sqlx::query(
            "INSERT INTO chat_ai_channel_memberships
                (conversation_id, teammate_id, access_profile_id,
                 read_history, read_history_inherits_profile,
                 participate, participate_inherits_profile,
                 history_boundary, history_boundary_inherits_profile,
                 history_from_ordinal, runtime_approval_policy,
                 scratch_runtime_approval_policy, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(conversation_id, teammate_id) DO UPDATE SET
                access_profile_id = excluded.access_profile_id,
                read_history = excluded.read_history,
                read_history_inherits_profile = excluded.read_history_inherits_profile,
                participate = excluded.participate,
                participate_inherits_profile = excluded.participate_inherits_profile,
                history_boundary = excluded.history_boundary,
                history_boundary_inherits_profile = excluded.history_boundary_inherits_profile,
                history_from_ordinal = excluded.history_from_ordinal,
                runtime_approval_policy = excluded.runtime_approval_policy,
                scratch_runtime_approval_policy = excluded.scratch_runtime_approval_policy,
                updated_at = excluded.updated_at",
        )
        .bind(channel.conversation_id.as_str())
        .bind(request.teammate_id.as_str())
        .bind(channel.access_profile_id.as_str())
        .bind(channel.capabilities.read_history)
        .bind(read_history_inherits_profile)
        .bind(channel.capabilities.participate)
        .bind(participate_inherits_profile)
        .bind(wire_history_boundary(&channel.history_boundary))
        .bind(history_boundary_inherits_profile)
        .bind(history_from_ordinal)
        .bind(
            channel
                .runtime_approval_override
                .map(wire_runtime_approval_policy),
        )
        .bind(
            channel
                .scratch_runtime_approval_override
                .map(wire_runtime_approval_policy),
        )
        .bind(now.as_str())
        .bind(now.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;

        sqlx::query(
            "UPDATE chat_teammate_working_folder_grants
             SET revoked_at = ?, is_default = 0, revision = revision + 1
             WHERE conversation_id = ? AND teammate_id = ? AND revoked_at IS NULL",
        )
        .bind(now.as_str())
        .bind(channel.conversation_id.as_str())
        .bind(request.teammate_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        for grant in &channel.folder_grants {
            sqlx::query(
                "INSERT INTO chat_teammate_working_folder_grants
                    (conversation_id, teammate_id, project_id, working_folder_id,
                     capability, capability_inherits_profile,
                     is_default, runtime_approval_policy,
                     revision, created_at, revoked_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?, NULL)
                 ON CONFLICT(conversation_id, teammate_id, working_folder_id) DO UPDATE SET
                    project_id = excluded.project_id,
                    capability = excluded.capability,
                    capability_inherits_profile = excluded.capability_inherits_profile,
                    is_default = excluded.is_default,
                    runtime_approval_policy = excluded.runtime_approval_policy,
                    revision = chat_teammate_working_folder_grants.revision + 1,
                    revoked_at = NULL",
            )
            .bind(channel.conversation_id.as_str())
            .bind(request.teammate_id.as_str())
            .bind(&channel.project_id)
            .bind(grant.working_folder_id.as_str())
            .bind(wire_folder_capability(grant.capability))
            .bind(grant.capability == channel.profile_maximum_folder_capability)
            .bind(grant.is_default)
            .bind(
                grant
                    .runtime_approval_override
                    .map(wire_runtime_approval_policy),
            )
            .bind(now.as_str())
            .execute(&mut *transaction)
            .await
            .map_err(persistence_error)?;
        }
    }

    if has_reduction {
        revoke_teammate_authorizations(
            &mut transaction,
            &request.teammate_id,
            &now,
            "Teammate access was reduced",
        )
        .await?;
    }
    transaction.commit().await.map_err(persistence_error)?;
    if has_reduction {
        super::super::revocation::drain_access_revocation_jobs(&app, &pool).await?;
    }
    read_teammate_access(&pool, &request.teammate_id).await
}

async fn read_access_profile(
    pool: &SqlitePool,
    access_profile_id: &ChatAccessProfileId,
) -> ChatResult<ChatAccessProfileRead> {
    let row = sqlx::query(
        "SELECT profile.builtin_key, profile.display_name, profile.latest_revision,
                profile.revision AS profile_revision, profile.archived_at,
                revision.id AS access_profile_revision_id,
                revision.default_read_history, revision.default_participate,
                revision.default_history_boundary, revision.maximum_folder_capability,
                revision.created_at
         FROM chat_access_profiles profile
         JOIN chat_access_profile_revisions revision
           ON revision.access_profile_id = profile.id
          AND revision.revision = profile.latest_revision
         WHERE profile.id = ?",
    )
    .bind(access_profile_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Access profile was not found",
            true,
        )
    })?;
    let latest_revision = u64_value(row.try_get("latest_revision").map_err(persistence_error)?)?;
    Ok(ChatAccessProfileRead {
        id: access_profile_id.clone(),
        builtin_key: parse_access_profile_builtin_key(
            row.try_get::<Option<String>, _>("builtin_key")
                .map_err(persistence_error)?
                .as_deref(),
        )?,
        display_name: row.try_get("display_name").map_err(persistence_error)?,
        latest_revision: ChatAccessProfileRevision {
            id: ChatAccessProfileRevisionId::new(
                row.try_get::<String, _>("access_profile_revision_id")
                    .map_err(persistence_error)?,
            )
            .map_err(identifier_error)?,
            access_profile_id: access_profile_id.clone(),
            revision: latest_revision,
            default_channel_capabilities: ChatChannelCapabilities {
                read_history: row
                    .try_get::<i64, _>("default_read_history")
                    .map_err(persistence_error)?
                    != 0,
                participate: row
                    .try_get::<i64, _>("default_participate")
                    .map_err(persistence_error)?
                    != 0,
            },
            default_history_boundary: parse_history_boundary(
                &row.try_get::<String, _>("default_history_boundary")
                    .map_err(persistence_error)?,
                None,
            )?,
            maximum_folder_capability: parse_folder_capability(
                &row.try_get::<String, _>("maximum_folder_capability")
                    .map_err(persistence_error)?,
            )?,
            created_at: timestamp(row.try_get("created_at").map_err(persistence_error)?)?,
        },
        revision: u64_value(row.try_get("profile_revision").map_err(persistence_error)?)?,
        archived_at: optional_timestamp(row.try_get("archived_at").map_err(persistence_error)?)?,
    })
}

async fn read_teammate_access(
    pool: &SqlitePool,
    teammate_id: &ChatParticipantId,
) -> ChatResult<ChatTeammateAccessRead> {
    let state = sqlx::query(
        "SELECT access.access_revision, access.runtime_approval_policy
         FROM chat_ai_teammate_access_state access
         JOIN chat_participants participant ON participant.id = access.teammate_id
         WHERE access.teammate_id = ? AND participant.archived_at IS NULL",
    )
    .bind(teammate_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| ChatError::new(ChatErrorCode::NotFound, "AI teammate was not found", true))?;
    let rows = sqlx::query(
        "SELECT channel.id AS channel_id, channel.project_id, project.group_id,
                channel.conversation_id, channel.name AS channel_name,
                ai.access_profile_id, profile.latest_revision AS access_profile_revision,
                profile_revision.default_read_history AS profile_read_history,
                profile_revision.default_participate AS profile_participate,
                profile_revision.default_history_boundary AS profile_history_boundary,
                profile_revision.maximum_folder_capability AS profile_folder_capability,
                ai.read_history, ai.read_history_inherits_profile,
                ai.participate, ai.participate_inherits_profile,
                ai.history_boundary, ai.history_boundary_inherits_profile,
                ai.history_from_ordinal, ai.runtime_approval_policy,
                ai.scratch_runtime_approval_policy,
                membership.revision AS membership_revision, membership.removed_at
         FROM chat_ai_channel_memberships ai
         JOIN chat_conversation_memberships membership
           ON membership.conversation_id = ai.conversation_id
          AND membership.participant_id = ai.teammate_id
         JOIN chat_channels channel ON channel.conversation_id = ai.conversation_id
         JOIN projects project ON project.id = channel.project_id
         JOIN chat_access_profiles profile ON profile.id = ai.access_profile_id
         JOIN chat_access_profile_revisions profile_revision
           ON profile_revision.access_profile_id = profile.id
          AND profile_revision.revision = profile.latest_revision
         WHERE ai.teammate_id = ?
         ORDER BY project.group_id, channel.project_id, channel.name COLLATE NOCASE, channel.id",
    )
    .bind(teammate_id.as_str())
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    let mut channels = Vec::with_capacity(rows.len());
    for row in rows {
        let conversation_id = ChatConversationId::new(
            row.try_get::<String, _>("conversation_id")
                .map_err(persistence_error)?,
        )
        .map_err(identifier_error)?;
        let grant_rows = sqlx::query(
            "SELECT grant_row.working_folder_id, folder.display_name,
                    grant_row.capability, grant_row.capability_inherits_profile,
                    grant_row.is_default,
                    grant_row.runtime_approval_policy, grant_row.revision,
                    grant_row.revoked_at
             FROM chat_teammate_working_folder_grants grant_row
             JOIN project_working_folders folder ON folder.id = grant_row.working_folder_id
             WHERE grant_row.conversation_id = ? AND grant_row.teammate_id = ?
               AND grant_row.revoked_at IS NULL
             ORDER BY grant_row.is_default DESC, folder.sort_order, folder.display_name, folder.id",
        )
        .bind(conversation_id.as_str())
        .bind(teammate_id.as_str())
        .fetch_all(pool)
        .await
        .map_err(persistence_error)?;
        let profile_folder_capability = parse_folder_capability(
            &row.try_get::<String, _>("profile_folder_capability")
                .map_err(persistence_error)?,
        )?;
        let mut folder_grants = Vec::with_capacity(grant_rows.len());
        for grant in grant_rows {
            folder_grants.push(ChatFolderGrantRead {
                working_folder_id: ProjectWorkingFolderId::new(
                    grant
                        .try_get::<String, _>("working_folder_id")
                        .map_err(persistence_error)?,
                )
                .map_err(identifier_error)?,
                display_name: grant.try_get("display_name").map_err(persistence_error)?,
                capability: if grant
                    .try_get::<i64, _>("capability_inherits_profile")
                    .map_err(persistence_error)?
                    != 0
                {
                    profile_folder_capability
                } else {
                    parse_folder_capability(
                        &grant
                            .try_get::<String, _>("capability")
                            .map_err(persistence_error)?,
                    )?
                    .intersect(profile_folder_capability)
                },
                is_default: grant
                    .try_get::<i64, _>("is_default")
                    .map_err(persistence_error)?
                    != 0,
                runtime_approval_override: grant
                    .try_get::<Option<String>, _>("runtime_approval_policy")
                    .map_err(persistence_error)?
                    .as_deref()
                    .map(parse_runtime_approval_policy)
                    .transpose()?,
                revision: u64_value(grant.try_get("revision").map_err(persistence_error)?)?,
                revoked_at: optional_timestamp(
                    grant.try_get("revoked_at").map_err(persistence_error)?,
                )?,
            });
        }
        let profile_history_boundary = parse_history_boundary(
            &row.try_get::<String, _>("profile_history_boundary")
                .map_err(persistence_error)?,
            None,
        )?;
        let stored_history_boundary = parse_history_boundary(
            &row.try_get::<String, _>("history_boundary")
                .map_err(persistence_error)?,
            row.try_get("history_from_ordinal")
                .map_err(persistence_error)?,
        )?;
        let history_inherits_profile = row
            .try_get::<i64, _>("history_boundary_inherits_profile")
            .map_err(persistence_error)?
            != 0;
        let effective_history_boundary = if history_inherits_profile {
            match profile_history_boundary.clone() {
                ChatHistoryBoundary::Entire => ChatHistoryBoundary::Entire,
                ChatHistoryBoundary::FromGrant { .. } => ChatHistoryBoundary::FromGrant {
                    lower_ordinal: match stored_history_boundary {
                        ChatHistoryBoundary::FromGrant { lower_ordinal } => lower_ordinal,
                        ChatHistoryBoundary::Entire => None,
                    },
                },
            }
        } else if history_boundary_is_expansion(&profile_history_boundary, &stored_history_boundary)
        {
            let lower_ordinal: i64 = sqlx::query_scalar(
                "SELECT coalesce(max(ordinal), 0) + 1
                 FROM chat_conversation_items
                 WHERE conversation_id = ? AND reply_thread_id IS NULL",
            )
            .bind(conversation_id.as_str())
            .fetch_one(pool)
            .await
            .map_err(persistence_error)?;
            ChatHistoryBoundary::FromGrant {
                lower_ordinal: Some(u64_value(lower_ordinal)?),
            }
        } else {
            stored_history_boundary
        };
        channels.push(ChatTeammateChannelAccessRead {
            channel_id: ChatChannelId::new(
                row.try_get::<String, _>("channel_id")
                    .map_err(persistence_error)?,
            )
            .map_err(identifier_error)?,
            project_id: row.try_get("project_id").map_err(persistence_error)?,
            group_id: row.try_get("group_id").map_err(persistence_error)?,
            conversation_id,
            channel_name: row.try_get("channel_name").map_err(persistence_error)?,
            access_profile_id: ChatAccessProfileId::new(
                row.try_get::<String, _>("access_profile_id")
                    .map_err(persistence_error)?,
            )
            .map_err(identifier_error)?,
            access_profile_revision: u64_value(
                row.try_get("access_profile_revision")
                    .map_err(persistence_error)?,
            )?,
            capabilities: ChatChannelCapabilities {
                read_history: if row
                    .try_get::<i64, _>("read_history_inherits_profile")
                    .map_err(persistence_error)?
                    != 0
                {
                    row.try_get::<i64, _>("profile_read_history")
                        .map_err(persistence_error)?
                        != 0
                } else {
                    row.try_get::<i64, _>("read_history")
                        .map_err(persistence_error)?
                        != 0
                        && row
                            .try_get::<i64, _>("profile_read_history")
                            .map_err(persistence_error)?
                            != 0
                },
                participate: if row
                    .try_get::<i64, _>("participate_inherits_profile")
                    .map_err(persistence_error)?
                    != 0
                {
                    row.try_get::<i64, _>("profile_participate")
                        .map_err(persistence_error)?
                        != 0
                } else {
                    row.try_get::<i64, _>("participate")
                        .map_err(persistence_error)?
                        != 0
                        && row
                            .try_get::<i64, _>("profile_participate")
                            .map_err(persistence_error)?
                            != 0
                },
            },
            history_boundary: effective_history_boundary,
            runtime_approval_override: row
                .try_get::<Option<String>, _>("runtime_approval_policy")
                .map_err(persistence_error)?
                .as_deref()
                .map(parse_runtime_approval_policy)
                .transpose()?,
            scratch_runtime_approval_override: row
                .try_get::<Option<String>, _>("scratch_runtime_approval_policy")
                .map_err(persistence_error)?
                .as_deref()
                .map(parse_runtime_approval_policy)
                .transpose()?,
            folder_grants,
            membership_revision: u64_value(
                row.try_get("membership_revision")
                    .map_err(persistence_error)?,
            )?,
            removed_at: optional_timestamp(row.try_get("removed_at").map_err(persistence_error)?)?,
        });
    }
    Ok(ChatTeammateAccessRead {
        teammate_id: teammate_id.clone(),
        access_revision: u64_value(
            state
                .try_get("access_revision")
                .map_err(persistence_error)?,
        )?,
        teammate_default_runtime_approval: parse_runtime_approval_policy(
            &state
                .try_get::<String, _>("runtime_approval_policy")
                .map_err(persistence_error)?,
        )?,
        channels,
    })
}

async fn preview_teammate_access(
    pool: &SqlitePool,
    teammate_id: &ChatParticipantId,
    expected_access_revision: u64,
    teammate_default_runtime_approval: ChatRuntimeApprovalPolicy,
    channels: &[ChatTeammateChannelAccessInput],
) -> ChatResult<ChatTeammateAccessPreview> {
    let current = read_teammate_access(pool, teammate_id).await?;
    if current.access_revision != expected_access_revision {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The teammate access changed before the preview",
            true,
        ));
    }
    let (resolved, mut issues) = resolve_channel_access(pool, teammate_id, channels).await?;
    issues.extend(retained_reference_issues(pool, &resolved).await?);
    let current_ids = current
        .channels
        .iter()
        .map(|channel| channel.channel_id.clone())
        .collect::<BTreeSet<_>>();
    let proposed_ids = resolved
        .iter()
        .map(|channel| channel.channel_id.clone())
        .collect::<BTreeSet<_>>();
    let added_channel_ids = proposed_ids.difference(&current_ids).cloned().collect();
    let removed_channel_ids = current_ids.difference(&proposed_ids).cloned().collect();
    let is_expansion =
        teammate_access_is_expansion(&current, teammate_default_runtime_approval, &resolved);
    let proposed = if issues.is_empty() {
        Some(proposed_access_read(
            teammate_id,
            expected_access_revision + 1,
            teammate_default_runtime_approval,
            &resolved,
        ))
    } else {
        None
    };
    Ok(ChatTeammateAccessPreview {
        proposed,
        is_expansion,
        added_channel_ids,
        removed_channel_ids,
        issues,
    })
}

async fn resolve_channel_access(
    pool: &SqlitePool,
    teammate_id: &ChatParticipantId,
    channels: &[ChatTeammateChannelAccessInput],
) -> ChatResult<(Vec<ResolvedChannelAccess>, Vec<ChatAccessValidationIssue>)> {
    let mut resolved = Vec::with_capacity(channels.len());
    let mut issues = Vec::new();
    let mut channel_ids = BTreeSet::new();
    for (channel_index, input) in channels.iter().enumerate() {
        let channel_path = format!("channels[{channel_index}]");
        if !channel_ids.insert(input.channel_id.as_str()) {
            issues.push(issue(
                ChatAccessIssueCode::DuplicateChannel,
                &format!("{channel_path}.channelId"),
                "Each channel may be selected only once",
            ));
            continue;
        }
        let channel_row = sqlx::query(
            "SELECT channel.project_id, project.group_id, channel.conversation_id,
                    channel.name, channel.archived_at
             FROM chat_channels channel
             JOIN projects project ON project.id = channel.project_id
             WHERE channel.id = ?",
        )
        .bind(input.channel_id.as_str())
        .fetch_optional(pool)
        .await
        .map_err(persistence_error)?;
        let Some(channel_row) = channel_row else {
            issues.push(issue(
                ChatAccessIssueCode::ArchivedChannel,
                &format!("{channel_path}.channelId"),
                "The selected channel is unavailable",
            ));
            continue;
        };
        if channel_row
            .try_get::<Option<String>, _>("archived_at")
            .map_err(persistence_error)?
            .is_some()
        {
            issues.push(issue(
                ChatAccessIssueCode::ArchivedChannel,
                &format!("{channel_path}.channelId"),
                "Archived channels cannot receive teammate access",
            ));
            continue;
        }
        let profile = match read_access_profile(pool, &input.access_profile_id).await {
            Ok(profile) if profile.archived_at.is_none() => profile,
            _ => {
                issues.push(issue(
                    ChatAccessIssueCode::ProfileNotFound,
                    &format!("{channel_path}.accessProfileId"),
                    "The selected access profile is unavailable",
                ));
                continue;
            }
        };
        if profile.latest_revision.revision != input.access_profile_revision {
            return Err(ChatError::new(
                ChatErrorCode::StaleRevision,
                "An access profile changed before teammate access could be previewed",
                true,
            ));
        }
        if input.capabilities.read_history
            && !profile
                .latest_revision
                .default_channel_capabilities
                .read_history
            || input.capabilities.participate
                && !profile
                    .latest_revision
                    .default_channel_capabilities
                    .participate
            || history_boundary_is_expansion(
                &profile.latest_revision.default_history_boundary,
                &input.history_boundary,
            )
        {
            issues.push(issue(
                ChatAccessIssueCode::ProfileCeilingExceeded,
                &format!("{channel_path}.capabilities"),
                "Channel access cannot exceed the linked profile",
            ));
        }
        let project_id: String = channel_row
            .try_get("project_id")
            .map_err(persistence_error)?;
        let mut folder_ids = BTreeSet::new();
        let mut defaults = 0_u32;
        for (grant_index, grant) in input.folder_grants.iter().enumerate() {
            let grant_path = format!("{channel_path}.folderGrants[{grant_index}]");
            if !folder_ids.insert(grant.working_folder_id.as_str()) {
                issues.push(issue(
                    ChatAccessIssueCode::DuplicateFolder,
                    &format!("{grant_path}.workingFolderId"),
                    "Each folder may be granted only once",
                ));
                continue;
            }
            defaults += u32::from(grant.is_default);
            if grant.capability.rank() > profile.latest_revision.maximum_folder_capability.rank() {
                issues.push(issue(
                    ChatAccessIssueCode::ProfileCeilingExceeded,
                    &format!("{grant_path}.capability"),
                    "Folder authority cannot exceed the linked profile",
                ));
            }
            let folder = sqlx::query(
                "SELECT project_id, archived_at FROM project_working_folders WHERE id = ?",
            )
            .bind(grant.working_folder_id.as_str())
            .fetch_optional(pool)
            .await
            .map_err(persistence_error)?;
            match folder {
                None => issues.push(issue(
                    ChatAccessIssueCode::ArchivedFolder,
                    &format!("{grant_path}.workingFolderId"),
                    "The selected folder is unavailable",
                )),
                Some(row)
                    if row
                        .try_get::<Option<String>, _>("archived_at")
                        .map_err(persistence_error)?
                        .is_some() =>
                {
                    issues.push(issue(
                        ChatAccessIssueCode::ArchivedFolder,
                        &format!("{grant_path}.workingFolderId"),
                        "Archived folders cannot be granted",
                    ));
                }
                Some(row)
                    if row
                        .try_get::<String, _>("project_id")
                        .map_err(persistence_error)?
                        != project_id =>
                {
                    issues.push(issue(
                        ChatAccessIssueCode::CrossProjectFolder,
                        &format!("{grant_path}.workingFolderId"),
                        "Folders must belong to the channel project",
                    ));
                }
                Some(_) => {}
            }
        }
        if defaults > 1 {
            issues.push(issue(
                ChatAccessIssueCode::MultipleDefaultFolders,
                &format!("{channel_path}.folderGrants"),
                "Choose at most one default execution folder",
            ));
        }
        let conversation_id = ChatConversationId::new(
            channel_row
                .try_get::<String, _>("conversation_id")
                .map_err(persistence_error)?,
        )
        .map_err(identifier_error)?;
        let history_boundary = match input.history_boundary {
            ChatHistoryBoundary::Entire => ChatHistoryBoundary::Entire,
            ChatHistoryBoundary::FromGrant { .. } => {
                let current_lower = sqlx::query_scalar::<_, Option<i64>>(
                    "SELECT history_from_ordinal FROM chat_ai_channel_memberships
                     WHERE conversation_id = ? AND teammate_id = ?
                       AND history_boundary = 'from_grant'",
                )
                .bind(conversation_id.as_str())
                .bind(teammate_id.as_str())
                .fetch_optional(pool)
                .await
                .map_err(persistence_error)?
                .flatten();
                let lower = match current_lower {
                    Some(value) => u64_value(value)?,
                    None => {
                        let value: i64 = sqlx::query_scalar(
                            "SELECT COALESCE(max(ordinal), 0) + 1
                             FROM chat_conversation_items
                             WHERE conversation_id = ? AND reply_thread_id IS NULL",
                        )
                        .bind(conversation_id.as_str())
                        .fetch_one(pool)
                        .await
                        .map_err(persistence_error)?;
                        u64_value(value)?
                    }
                };
                ChatHistoryBoundary::FromGrant {
                    lower_ordinal: Some(lower),
                }
            }
        };
        resolved.push(ResolvedChannelAccess {
            channel_id: input.channel_id.clone(),
            project_id,
            group_id: channel_row.try_get("group_id").map_err(persistence_error)?,
            conversation_id,
            channel_name: channel_row.try_get("name").map_err(persistence_error)?,
            access_profile_id: input.access_profile_id.clone(),
            access_profile_revision: profile.latest_revision.revision,
            profile_maximum_folder_capability: profile.latest_revision.maximum_folder_capability,
            profile_default_channel_capabilities: profile
                .latest_revision
                .default_channel_capabilities,
            profile_default_history_boundary: profile.latest_revision.default_history_boundary,
            capabilities: input.capabilities,
            history_boundary,
            runtime_approval_override: input.runtime_approval_override,
            scratch_runtime_approval_override: input.scratch_runtime_approval_override,
            folder_grants: input.folder_grants.clone(),
        });
    }
    Ok((resolved, issues))
}

async fn require_expected_profile_revisions_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    channels: &[ChatTeammateChannelAccessInput],
) -> ChatResult<()> {
    let expected = channels
        .iter()
        .map(|channel| {
            (
                channel.access_profile_id.as_str(),
                channel.access_profile_revision,
            )
        })
        .collect::<BTreeSet<_>>();
    for (access_profile_id, expected_revision) in expected {
        let latest_revision = sqlx::query_scalar::<_, i64>(
            "SELECT latest_revision FROM chat_access_profiles
             WHERE id = ? AND archived_at IS NULL",
        )
        .bind(access_profile_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(persistence_error)?;
        let Some(latest_revision) = latest_revision else {
            return Err(ChatError::new(
                ChatErrorCode::StaleRevision,
                "An access profile became unavailable before teammate access could be saved",
                true,
            ));
        };
        if u64_value(latest_revision)? != expected_revision {
            return Err(ChatError::new(
                ChatErrorCode::StaleRevision,
                "An access profile changed before teammate access could be saved",
                true,
            ));
        }
    }
    Ok(())
}

async fn retained_reference_issues(
    pool: &SqlitePool,
    proposed: &[ResolvedChannelAccess],
) -> ChatResult<Vec<ChatAccessValidationIssue>> {
    let source_access = proposed_source_access(proposed);
    let mut issues = Vec::new();
    for (index, destination) in proposed.iter().enumerate() {
        if !destination.capabilities.read_history {
            continue;
        }
        let references = sqlx::query(
            "SELECT DISTINCT target.source_conversation_id,
                    target.source_lower_ordinal,
                    COALESCE(root_item.ordinal, item.ordinal) AS destination_root_ordinal
             FROM chat_channel_reference_targets target
             JOIN chat_message_references reference
               ON reference.id = target.reference_id
             JOIN chat_communication_message_revisions revision
               ON revision.id = reference.message_revision_id
             JOIN chat_communication_messages message
               ON message.item_id = revision.message_item_id
             JOIN chat_conversation_items item
               ON item.id = message.item_id
             LEFT JOIN chat_reply_threads reply_thread
               ON reply_thread.id = item.reply_thread_id
             LEFT JOIN chat_conversation_items root_item
               ON root_item.id = reply_thread.root_item_id
             WHERE target.destination_conversation_id = ?
               AND message.deleted_at IS NULL
               AND message.current_revision_id = revision.id",
        )
        .bind(destination.conversation_id.as_str())
        .fetch_all(pool)
        .await
        .map_err(persistence_error)?;
        for reference in references {
            let source_conversation_id: String = reference
                .try_get("source_conversation_id")
                .map_err(persistence_error)?;
            let source_lower_ordinal = u64_value(
                reference
                    .try_get("source_lower_ordinal")
                    .map_err(persistence_error)?,
            )?;
            let destination_root_ordinal = u64_value(
                reference
                    .try_get("destination_root_ordinal")
                    .map_err(persistence_error)?,
            )?;
            if !retained_reference_is_visible(
                &destination.history_boundary,
                destination_root_ordinal,
            ) {
                continue;
            }
            if source_conversation_id != destination.conversation_id.as_str()
                && !can_read_retained_source(
                    source_access.get(source_conversation_id.as_str()),
                    source_lower_ordinal,
                )
            {
                issues.push(issue(
                    ChatAccessIssueCode::RetainedReferenceDisclosure,
                    &format!("channels[{index}].historyBoundary"),
                    "Use From access grant because earlier channel references are not readable by this teammate",
                ));
                break;
            }
        }
    }
    Ok(issues)
}

async fn retained_reference_issues_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    proposed: &[ResolvedChannelAccess],
) -> ChatResult<Vec<ChatAccessValidationIssue>> {
    let source_access = proposed_source_access(proposed);
    let mut issues = Vec::new();
    for (index, destination) in proposed.iter().enumerate() {
        if !destination.capabilities.read_history {
            continue;
        }
        let references = sqlx::query(
            "SELECT DISTINCT target.source_conversation_id,
                    target.source_lower_ordinal,
                    COALESCE(root_item.ordinal, item.ordinal) AS destination_root_ordinal
             FROM chat_channel_reference_targets target
             JOIN chat_message_references reference
               ON reference.id = target.reference_id
             JOIN chat_communication_message_revisions revision
               ON revision.id = reference.message_revision_id
             JOIN chat_communication_messages message
               ON message.item_id = revision.message_item_id
             JOIN chat_conversation_items item
               ON item.id = message.item_id
             LEFT JOIN chat_reply_threads reply_thread
               ON reply_thread.id = item.reply_thread_id
             LEFT JOIN chat_conversation_items root_item
               ON root_item.id = reply_thread.root_item_id
             WHERE target.destination_conversation_id = ?
               AND message.deleted_at IS NULL
               AND message.current_revision_id = revision.id",
        )
        .bind(destination.conversation_id.as_str())
        .fetch_all(&mut **transaction)
        .await
        .map_err(persistence_error)?;
        for reference in references {
            let source_conversation_id: String = reference
                .try_get("source_conversation_id")
                .map_err(persistence_error)?;
            let source_lower_ordinal = u64_value(
                reference
                    .try_get("source_lower_ordinal")
                    .map_err(persistence_error)?,
            )?;
            let destination_root_ordinal = u64_value(
                reference
                    .try_get("destination_root_ordinal")
                    .map_err(persistence_error)?,
            )?;
            if !retained_reference_is_visible(
                &destination.history_boundary,
                destination_root_ordinal,
            ) {
                continue;
            }
            if source_conversation_id != destination.conversation_id.as_str()
                && !can_read_retained_source(
                    source_access.get(source_conversation_id.as_str()),
                    source_lower_ordinal,
                )
            {
                issues.push(issue(
                    ChatAccessIssueCode::RetainedReferenceDisclosure,
                    &format!("channels[{index}].historyBoundary"),
                    "Use From access grant because earlier channel references are not readable by this teammate",
                ));
                break;
            }
        }
    }
    Ok(issues)
}

async fn retained_profile_reference_issues_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    access_profile_id: &ChatAccessProfileId,
) -> ChatResult<Vec<ChatAccessValidationIssue>> {
    let teammate_ids = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT ai.teammate_id
         FROM chat_ai_channel_memberships ai
         JOIN chat_conversation_memberships membership
           ON membership.conversation_id = ai.conversation_id
          AND membership.participant_id = ai.teammate_id
         WHERE ai.access_profile_id = ? AND membership.removed_at IS NULL
         ORDER BY ai.teammate_id",
    )
    .bind(access_profile_id.as_str())
    .fetch_all(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    for teammate_id in teammate_ids {
        let resolved =
            read_effective_disclosure_access_in_transaction(transaction, teammate_id.as_str())
                .await?;
        if !retained_reference_issues_in_transaction(transaction, &resolved)
            .await?
            .is_empty()
        {
            return Ok(vec![issue(
                ChatAccessIssueCode::RetainedReferenceDisclosure,
                "revision.defaultHistoryBoundary",
                "This profile expansion would expose retained channel references to a teammate without source access",
            )]);
        }
    }
    Ok(Vec::new())
}

async fn read_effective_disclosure_access_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    teammate_id: &str,
) -> ChatResult<Vec<ResolvedChannelAccess>> {
    let rows = sqlx::query(
        "SELECT channel.id AS channel_id, channel.project_id, project.group_id,
                channel.conversation_id, channel.name AS channel_name,
                ai.access_profile_id, profile.latest_revision AS access_profile_revision,
                profile_revision.default_read_history AS profile_read_history,
                profile_revision.default_participate AS profile_participate,
                profile_revision.default_history_boundary AS profile_history_boundary,
                profile_revision.maximum_folder_capability AS profile_folder_capability,
                ai.read_history, ai.read_history_inherits_profile,
                ai.participate, ai.participate_inherits_profile,
                ai.history_boundary, ai.history_boundary_inherits_profile,
                ai.history_from_ordinal, ai.runtime_approval_policy,
                ai.scratch_runtime_approval_policy
         FROM chat_ai_channel_memberships ai
         JOIN chat_conversation_memberships membership
           ON membership.conversation_id = ai.conversation_id
          AND membership.participant_id = ai.teammate_id
         JOIN chat_channels channel ON channel.conversation_id = ai.conversation_id
         JOIN projects project ON project.id = channel.project_id
         JOIN chat_access_profiles profile ON profile.id = ai.access_profile_id
         JOIN chat_access_profile_revisions profile_revision
           ON profile_revision.access_profile_id = profile.id
          AND profile_revision.revision = profile.latest_revision
         WHERE ai.teammate_id = ? AND membership.removed_at IS NULL
         ORDER BY project.group_id, channel.project_id, channel.name COLLATE NOCASE, channel.id",
    )
    .bind(teammate_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let mut resolved = Vec::with_capacity(rows.len());
    for row in rows {
        let conversation_id = ChatConversationId::new(
            row.try_get::<String, _>("conversation_id")
                .map_err(persistence_error)?,
        )
        .map_err(identifier_error)?;
        let profile_history_boundary = parse_history_boundary(
            &row.try_get::<String, _>("profile_history_boundary")
                .map_err(persistence_error)?,
            None,
        )?;
        let stored_history_boundary = parse_history_boundary(
            &row.try_get::<String, _>("history_boundary")
                .map_err(persistence_error)?,
            row.try_get("history_from_ordinal")
                .map_err(persistence_error)?,
        )?;
        let history_boundary = if row
            .try_get::<i64, _>("history_boundary_inherits_profile")
            .map_err(persistence_error)?
            != 0
        {
            match profile_history_boundary {
                ChatHistoryBoundary::Entire => ChatHistoryBoundary::Entire,
                ChatHistoryBoundary::FromGrant { .. } => ChatHistoryBoundary::FromGrant {
                    lower_ordinal: match stored_history_boundary {
                        ChatHistoryBoundary::FromGrant { lower_ordinal } => lower_ordinal,
                        ChatHistoryBoundary::Entire => None,
                    },
                },
            }
        } else if history_boundary_is_expansion(&profile_history_boundary, &stored_history_boundary)
        {
            let lower_ordinal: i64 = sqlx::query_scalar(
                "SELECT coalesce(max(ordinal), 0) + 1
                 FROM chat_conversation_items
                 WHERE conversation_id = ? AND reply_thread_id IS NULL",
            )
            .bind(conversation_id.as_str())
            .fetch_one(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            ChatHistoryBoundary::FromGrant {
                lower_ordinal: Some(u64_value(lower_ordinal)?),
            }
        } else {
            stored_history_boundary
        };
        let profile_capabilities = ChatChannelCapabilities {
            read_history: row
                .try_get::<i64, _>("profile_read_history")
                .map_err(persistence_error)?
                != 0,
            participate: row
                .try_get::<i64, _>("profile_participate")
                .map_err(persistence_error)?
                != 0,
        };
        let capabilities = ChatChannelCapabilities {
            read_history: if row
                .try_get::<i64, _>("read_history_inherits_profile")
                .map_err(persistence_error)?
                != 0
            {
                profile_capabilities.read_history
            } else {
                row.try_get::<i64, _>("read_history")
                    .map_err(persistence_error)?
                    != 0
                    && profile_capabilities.read_history
            },
            participate: if row
                .try_get::<i64, _>("participate_inherits_profile")
                .map_err(persistence_error)?
                != 0
            {
                profile_capabilities.participate
            } else {
                row.try_get::<i64, _>("participate")
                    .map_err(persistence_error)?
                    != 0
                    && profile_capabilities.participate
            },
        };
        resolved.push(ResolvedChannelAccess {
            channel_id: ChatChannelId::new(
                row.try_get::<String, _>("channel_id")
                    .map_err(persistence_error)?,
            )
            .map_err(identifier_error)?,
            project_id: row.try_get("project_id").map_err(persistence_error)?,
            group_id: row.try_get("group_id").map_err(persistence_error)?,
            conversation_id,
            channel_name: row.try_get("channel_name").map_err(persistence_error)?,
            access_profile_id: ChatAccessProfileId::new(
                row.try_get::<String, _>("access_profile_id")
                    .map_err(persistence_error)?,
            )
            .map_err(identifier_error)?,
            access_profile_revision: u64_value(
                row.try_get("access_profile_revision")
                    .map_err(persistence_error)?,
            )?,
            profile_default_channel_capabilities: profile_capabilities,
            profile_default_history_boundary: profile_history_boundary,
            profile_maximum_folder_capability: parse_folder_capability(
                &row.try_get::<String, _>("profile_folder_capability")
                    .map_err(persistence_error)?,
            )?,
            capabilities,
            history_boundary,
            runtime_approval_override: row
                .try_get::<Option<String>, _>("runtime_approval_policy")
                .map_err(persistence_error)?
                .as_deref()
                .map(parse_runtime_approval_policy)
                .transpose()?,
            scratch_runtime_approval_override: row
                .try_get::<Option<String>, _>("scratch_runtime_approval_policy")
                .map_err(persistence_error)?
                .as_deref()
                .map(parse_runtime_approval_policy)
                .transpose()?,
            folder_grants: Vec::new(),
        });
    }
    Ok(resolved)
}

fn proposed_source_access(
    proposed: &[ResolvedChannelAccess],
) -> BTreeMap<&str, (&ChatChannelCapabilities, &ChatHistoryBoundary)> {
    proposed
        .iter()
        .map(|channel| {
            (
                channel.conversation_id.as_str(),
                (&channel.capabilities, &channel.history_boundary),
            )
        })
        .collect()
}

fn can_read_retained_source(
    source: Option<&(&ChatChannelCapabilities, &ChatHistoryBoundary)>,
    source_lower_ordinal: u64,
) -> bool {
    let Some((capabilities, boundary)) = source else {
        return false;
    };
    if !capabilities.read_history {
        return false;
    }
    match boundary {
        ChatHistoryBoundary::Entire => true,
        ChatHistoryBoundary::FromGrant { lower_ordinal } => {
            lower_ordinal.is_some_and(|lower_ordinal| lower_ordinal <= source_lower_ordinal)
        }
    }
}

fn retained_reference_is_visible(
    destination_boundary: &ChatHistoryBoundary,
    destination_root_ordinal: u64,
) -> bool {
    match destination_boundary {
        ChatHistoryBoundary::Entire => true,
        ChatHistoryBoundary::FromGrant { lower_ordinal } => {
            lower_ordinal.is_none_or(|lower_ordinal| lower_ordinal <= destination_root_ordinal)
        }
    }
}

fn proposed_access_read(
    teammate_id: &ChatParticipantId,
    access_revision: u64,
    teammate_default_runtime_approval: ChatRuntimeApprovalPolicy,
    resolved: &[ResolvedChannelAccess],
) -> ChatTeammateAccessRead {
    ChatTeammateAccessRead {
        teammate_id: teammate_id.clone(),
        access_revision,
        teammate_default_runtime_approval,
        channels: resolved
            .iter()
            .map(|channel| ChatTeammateChannelAccessRead {
                channel_id: channel.channel_id.clone(),
                project_id: channel.project_id.clone(),
                group_id: channel.group_id.clone(),
                conversation_id: channel.conversation_id.clone(),
                channel_name: channel.channel_name.clone(),
                access_profile_id: channel.access_profile_id.clone(),
                access_profile_revision: channel.access_profile_revision,
                capabilities: channel.capabilities,
                history_boundary: channel.history_boundary.clone(),
                runtime_approval_override: channel.runtime_approval_override,
                scratch_runtime_approval_override: channel.scratch_runtime_approval_override,
                folder_grants: channel
                    .folder_grants
                    .iter()
                    .map(|grant| ChatFolderGrantRead {
                        working_folder_id: grant.working_folder_id.clone(),
                        display_name: grant.working_folder_id.to_string(),
                        capability: grant.capability,
                        is_default: grant.is_default,
                        runtime_approval_override: grant.runtime_approval_override,
                        revision: 1,
                        revoked_at: None,
                    })
                    .collect(),
                membership_revision: 1,
                removed_at: None,
            })
            .collect(),
    }
}

fn teammate_access_is_expansion(
    current: &ChatTeammateAccessRead,
    proposed_runtime: ChatRuntimeApprovalPolicy,
    proposed: &[ResolvedChannelAccess],
) -> bool {
    if runtime_approval_is_expansion(current.teammate_default_runtime_approval, proposed_runtime) {
        return true;
    }
    let current_by_channel = current
        .channels
        .iter()
        .map(|channel| (channel.channel_id.as_str(), channel))
        .collect::<BTreeMap<_, _>>();
    proposed.iter().any(|channel| {
        let Some(current_channel) = current_by_channel.get(channel.channel_id.as_str()) else {
            return true;
        };
        (!current_channel.capabilities.read_history && channel.capabilities.read_history)
            || (!current_channel.capabilities.participate && channel.capabilities.participate)
            || history_boundary_is_expansion(
                &current_channel.history_boundary,
                &channel.history_boundary,
            )
            || runtime_approval_is_expansion(
                resolve_runtime_approval(
                    current_channel.runtime_approval_override,
                    current.teammate_default_runtime_approval,
                ),
                resolve_runtime_approval(channel.runtime_approval_override, proposed_runtime),
            )
            || runtime_approval_is_expansion(
                resolve_nested_runtime_approval(
                    current_channel.scratch_runtime_approval_override,
                    current_channel.runtime_approval_override,
                    current.teammate_default_runtime_approval,
                ),
                resolve_nested_runtime_approval(
                    channel.scratch_runtime_approval_override,
                    channel.runtime_approval_override,
                    proposed_runtime,
                ),
            )
            || channel.folder_grants.iter().any(|grant| {
                current_channel
                    .folder_grants
                    .iter()
                    .find(|current| current.working_folder_id == grant.working_folder_id)
                    .is_none_or(|current_grant| {
                        grant.capability.rank() > current_grant.capability.rank()
                            || runtime_approval_is_expansion(
                                resolve_nested_runtime_approval(
                                    current_grant.runtime_approval_override,
                                    current_channel.runtime_approval_override,
                                    current.teammate_default_runtime_approval,
                                ),
                                resolve_nested_runtime_approval(
                                    grant.runtime_approval_override,
                                    channel.runtime_approval_override,
                                    proposed_runtime,
                                ),
                            )
                    })
            })
    })
}

fn teammate_access_has_reduction(
    current: &ChatTeammateAccessRead,
    proposed_runtime: ChatRuntimeApprovalPolicy,
    proposed: &[ResolvedChannelAccess],
) -> bool {
    if runtime_approval_is_reduction(current.teammate_default_runtime_approval, proposed_runtime) {
        return true;
    }
    let proposed_by_channel = proposed
        .iter()
        .map(|channel| (channel.channel_id.as_str(), channel))
        .collect::<BTreeMap<_, _>>();
    current.channels.iter().any(|channel| {
        let Some(proposed_channel) = proposed_by_channel.get(channel.channel_id.as_str()) else {
            return true;
        };
        (channel.capabilities.read_history && !proposed_channel.capabilities.read_history)
            || (channel.capabilities.participate && !proposed_channel.capabilities.participate)
            || history_boundary_is_expansion(
                &proposed_channel.history_boundary,
                &channel.history_boundary,
            )
            || runtime_approval_is_reduction(
                resolve_runtime_approval(
                    channel.runtime_approval_override,
                    current.teammate_default_runtime_approval,
                ),
                resolve_runtime_approval(
                    proposed_channel.runtime_approval_override,
                    proposed_runtime,
                ),
            )
            || runtime_approval_is_reduction(
                resolve_nested_runtime_approval(
                    channel.scratch_runtime_approval_override,
                    channel.runtime_approval_override,
                    current.teammate_default_runtime_approval,
                ),
                resolve_nested_runtime_approval(
                    proposed_channel.scratch_runtime_approval_override,
                    proposed_channel.runtime_approval_override,
                    proposed_runtime,
                ),
            )
            || channel.folder_grants.iter().any(|grant| {
                proposed_channel
                    .folder_grants
                    .iter()
                    .find(|proposed| proposed.working_folder_id == grant.working_folder_id)
                    .is_none_or(|proposed_grant| {
                        proposed_grant.capability.rank() < grant.capability.rank()
                            || runtime_approval_is_reduction(
                                resolve_nested_runtime_approval(
                                    grant.runtime_approval_override,
                                    channel.runtime_approval_override,
                                    current.teammate_default_runtime_approval,
                                ),
                                resolve_nested_runtime_approval(
                                    proposed_grant.runtime_approval_override,
                                    proposed_channel.runtime_approval_override,
                                    proposed_runtime,
                                ),
                            )
                    })
            })
    })
}

fn resolve_runtime_approval(
    override_policy: Option<ChatRuntimeApprovalPolicy>,
    fallback: ChatRuntimeApprovalPolicy,
) -> ChatRuntimeApprovalPolicy {
    override_policy.unwrap_or(fallback)
}

fn resolve_nested_runtime_approval(
    resource_override: Option<ChatRuntimeApprovalPolicy>,
    channel_override: Option<ChatRuntimeApprovalPolicy>,
    teammate_default: ChatRuntimeApprovalPolicy,
) -> ChatRuntimeApprovalPolicy {
    resource_override
        .or(channel_override)
        .unwrap_or(teammate_default)
}

fn runtime_approval_is_expansion(
    current: ChatRuntimeApprovalPolicy,
    proposed: ChatRuntimeApprovalPolicy,
) -> bool {
    if current == proposed {
        return false;
    }
    match (
        runtime_approval_rank(current),
        runtime_approval_rank(proposed),
    ) {
        (Some(current), Some(proposed)) => proposed > current,
        _ => true,
    }
}

fn runtime_approval_is_reduction(
    current: ChatRuntimeApprovalPolicy,
    proposed: ChatRuntimeApprovalPolicy,
) -> bool {
    if current == proposed {
        return false;
    }
    match (
        runtime_approval_rank(current),
        runtime_approval_rank(proposed),
    ) {
        (Some(current), Some(proposed)) => proposed < current,
        _ => true,
    }
}

fn runtime_approval_rank(policy: ChatRuntimeApprovalPolicy) -> Option<u8> {
    match policy {
        ChatRuntimeApprovalPolicy::Ask => Some(0),
        ChatRuntimeApprovalPolicy::AutoApprove => Some(1),
        ChatRuntimeApprovalPolicy::Unattended => Some(2),
        ChatRuntimeApprovalPolicy::ProviderCustom => None,
    }
}

fn profile_revision_is_reduction(
    current: &ChatAccessProfileRevision,
    proposed: &ChatAccessProfileRevisionInput,
) -> bool {
    (current.default_channel_capabilities.read_history
        && !proposed.default_channel_capabilities.read_history)
        || (current.default_channel_capabilities.participate
            && !proposed.default_channel_capabilities.participate)
        || history_boundary_is_expansion(
            &proposed.default_history_boundary,
            &current.default_history_boundary,
        )
        || proposed.maximum_folder_capability.rank() < current.maximum_folder_capability.rank()
}

async fn insert_access_profile_revision(
    transaction: &mut Transaction<'_, Sqlite>,
    access_profile_id: &ChatAccessProfileId,
    revision: u64,
    input: &ChatAccessProfileRevisionInput,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    sqlx::query(
        "INSERT INTO chat_access_profile_revisions
            (id, access_profile_id, revision, default_read_history,
             default_participate, default_history_boundary,
             maximum_folder_capability, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new_id("access-profile-revision"))
    .bind(access_profile_id.as_str())
    .bind(i64_value(revision)?)
    .bind(input.default_channel_capabilities.read_history)
    .bind(input.default_channel_capabilities.participate)
    .bind(wire_history_boundary(&input.default_history_boundary))
    .bind(wire_folder_capability(input.maximum_folder_capability))
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

async fn revoke_profile_authorizations(
    transaction: &mut Transaction<'_, Sqlite>,
    access_profile_id: &ChatAccessProfileId,
    now: &UtcTimestamp,
    reason: &str,
) -> ChatResult<()> {
    let teammate_ids = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT teammate_id FROM chat_ai_channel_memberships
         WHERE access_profile_id = ?",
    )
    .bind(access_profile_id.as_str())
    .fetch_all(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    for teammate_id in teammate_ids {
        let teammate_id = ChatParticipantId::new(teammate_id).map_err(identifier_error)?;
        revoke_teammate_authorizations(transaction, &teammate_id, now, reason).await?;
    }
    Ok(())
}

async fn revoke_teammate_authorizations(
    transaction: &mut Transaction<'_, Sqlite>,
    teammate_id: &ChatParticipantId,
    now: &UtcTimestamp,
    reason: &str,
) -> ChatResult<()> {
    sqlx::query(
        "UPDATE chat_assignment_authorization_revisions
         SET decision_state = 'revoked', reason = ?, revoked_at = ?
         WHERE id IN (
             SELECT authorization.id
             FROM chat_assignment_authorization_revisions authorization
             JOIN chat_work_assignments assignment
               ON assignment.id = authorization.assignment_id
             WHERE assignment.teammate_id = ?
               AND authorization.decision_state = 'allowed'
               AND authorization.revoked_at IS NULL
         )",
    )
    .bind(reason)
    .bind(now.as_str())
    .bind(teammate_id.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_access_revocation_jobs
            (id, teammate_id, authorization_revision_id, reason,
             available_at, created_at, updated_at)
         SELECT 'access-revocation:' || lower(hex(randomblob(16))), ?, authorization.id, ?, ?, ?, ?
         FROM chat_assignment_authorization_revisions authorization
         JOIN chat_work_assignments assignment ON assignment.id = authorization.assignment_id
         WHERE assignment.teammate_id = ? AND authorization.revoked_at = ?
           AND NOT EXISTS (
               SELECT 1 FROM chat_access_revocation_jobs existing
               WHERE existing.authorization_revision_id = authorization.id
                 AND existing.state IN ('queued', 'claimed')
           )",
    )
    .bind(teammate_id.as_str())
    .bind(reason)
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(teammate_id.as_str())
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

fn validate_profile_revision_input(input: &ChatAccessProfileRevisionInput) -> ChatResult<()> {
    if matches!(
        input.default_history_boundary,
        ChatHistoryBoundary::FromGrant {
            lower_ordinal: Some(_)
        }
    ) {
        return Err(ChatError::validation(
            "revision.defaultHistoryBoundary.lowerOrdinal",
            "History lower ordinals are captured per channel",
        ));
    }
    Ok(())
}

fn validate_access_profile_name(value: &str) -> ChatResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 160 || value.chars().any(char::is_control) {
        return Err(ChatError::validation(
            "displayName",
            "Access profile name must contain 1 to 160 characters",
        ));
    }
    Ok(value.to_string())
}

fn issue(code: ChatAccessIssueCode, field_path: &str, message: &str) -> ChatAccessValidationIssue {
    ChatAccessValidationIssue {
        code,
        field_path: field_path.to_string(),
        message: message.to_string(),
    }
}

fn map_access_profile_write_error(error: sqlx::Error) -> ChatError {
    if error
        .to_string()
        .contains("idx_chat_access_profiles_custom_name")
    {
        ChatError::validation("displayName", "Access profile name already in use")
    } else {
        persistence_error(error)
    }
}

fn stale_access_profile() -> ChatError {
    ChatError::new(
        ChatErrorCode::StaleRevision,
        "The access profile changed before the update",
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_grant_exposes_only_references_at_or_after_its_root_boundary() {
        let boundary = ChatHistoryBoundary::FromGrant {
            lower_ordinal: Some(12),
        };
        assert!(!retained_reference_is_visible(&boundary, 11));
        assert!(retained_reference_is_visible(&boundary, 12));
        assert!(retained_reference_is_visible(&boundary, 13));
        assert!(retained_reference_is_visible(
            &ChatHistoryBoundary::Entire,
            1,
        ));
    }

    #[test]
    fn unresolved_from_grant_boundary_fails_closed_for_retained_references() {
        assert!(retained_reference_is_visible(
            &ChatHistoryBoundary::FromGrant {
                lower_ordinal: None,
            },
            1,
        ));
    }

    #[test]
    fn approval_changes_are_classified_by_effective_authority() {
        assert!(runtime_approval_is_expansion(
            ChatRuntimeApprovalPolicy::Ask,
            ChatRuntimeApprovalPolicy::AutoApprove,
        ));
        assert!(runtime_approval_is_expansion(
            ChatRuntimeApprovalPolicy::AutoApprove,
            ChatRuntimeApprovalPolicy::Unattended,
        ));
        assert!(runtime_approval_is_reduction(
            ChatRuntimeApprovalPolicy::Unattended,
            ChatRuntimeApprovalPolicy::Ask,
        ));
        assert!(!runtime_approval_is_reduction(
            ChatRuntimeApprovalPolicy::Ask,
            ChatRuntimeApprovalPolicy::AutoApprove,
        ));
    }

    #[test]
    fn provider_custom_requires_review_in_both_directions() {
        assert!(runtime_approval_is_expansion(
            ChatRuntimeApprovalPolicy::Ask,
            ChatRuntimeApprovalPolicy::ProviderCustom,
        ));
        assert!(runtime_approval_is_reduction(
            ChatRuntimeApprovalPolicy::Ask,
            ChatRuntimeApprovalPolicy::ProviderCustom,
        ));
        assert!(runtime_approval_is_expansion(
            ChatRuntimeApprovalPolicy::ProviderCustom,
            ChatRuntimeApprovalPolicy::Ask,
        ));
        assert!(runtime_approval_is_reduction(
            ChatRuntimeApprovalPolicy::ProviderCustom,
            ChatRuntimeApprovalPolicy::Ask,
        ));
    }

    #[test]
    fn resource_approval_precedes_channel_and_teammate_defaults() {
        assert_eq!(
            resolve_nested_runtime_approval(
                Some(ChatRuntimeApprovalPolicy::Ask),
                Some(ChatRuntimeApprovalPolicy::AutoApprove),
                ChatRuntimeApprovalPolicy::Unattended,
            ),
            ChatRuntimeApprovalPolicy::Ask,
        );
        assert_eq!(
            resolve_nested_runtime_approval(
                None,
                Some(ChatRuntimeApprovalPolicy::AutoApprove),
                ChatRuntimeApprovalPolicy::Unattended,
            ),
            ChatRuntimeApprovalPolicy::AutoApprove,
        );
    }
}
