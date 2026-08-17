//! Closed-world organizational host tools for provider assignments.

use super::internal_mcp::{
    generate_opaque_handle, InternalMcpChannelSource, InternalMcpFolderSource, InternalMcpRunScope,
};
use super::models::{
    ChatError, ChatErrorCode, ChatFolderCapability, ChatResult, ChatRuntimeApprovalPolicy,
};
use super::workspace::WorkingFolderAuthorizationOperation;
use rmcp::model::{Tool, ToolAnnotations};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use tauri::Manager;
use tokio::sync::Mutex;

const MAX_TOOL_CALLS: u32 = 20;
const MAX_SCOPE_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_RESPONSE_BYTES: usize = 64 * 1024;
const DEFAULT_PAGE_SIZE: u32 = 20;
const MAX_PAGE_SIZE: u32 = 50;
const MAX_QUERY_BYTES: usize = 500;
const MAX_QUERY_TERMS: usize = 20;
const MAX_CURSORS: usize = 256;
const MAX_WORKSPACE_CONTENT_BYTES: usize = 1024 * 1024;
const MAX_WORKSPACE_PATCH_EDITS: usize = 256;
const GENERIC_DENIAL: &str = "The requested organizational context is unavailable";

#[derive(Default)]
pub(crate) struct InternalMcpToolRuntime {
    cursors: Mutex<HashMap<String, OpaqueCursor>>,
}

#[derive(Clone)]
enum OpaqueCursor {
    Channel {
        source_handle: String,
        query_hash: String,
        before_created_at: String,
        before_item_id: String,
    },
    Workspace {
        root_handle: String,
        query_hash: String,
        provider_cursor: String,
    },
}

pub(crate) struct HostToolContext<'a> {
    pub app: &'a tauri::AppHandle,
    pub pool: &'a SqlitePool,
    pub thread_id: &'a super::models::ChatThreadId,
    pub scope: &'a InternalMcpRunScope,
    pub runtime: &'a InternalMcpToolRuntime,
}

pub(crate) fn definitions(scope: &InternalMcpRunScope) -> Vec<Tool> {
    let mut tools = vec![
        tool(
            "chat_list_referenced_channels",
            "List the channel sources explicitly authorized for this assignment",
            json!({}),
            true,
        ),
        tool(
            "chat_search_channel_messages",
            "Search one explicitly referenced channel within its frozen authorization cutoff",
            json!({
                "sourceHandle": { "type": "string", "minLength": 1, "maxLength": 1024 },
                "query": { "type": "string", "minLength": 1, "maxLength": MAX_QUERY_BYTES },
                "limit": { "type": "integer", "minimum": 1, "maximum": MAX_PAGE_SIZE },
                "cursor": { "type": "string", "minLength": 1, "maxLength": 1024 }
            }),
            true,
        ),
        tool(
            "chat_read_channel_messages",
            "Read one explicitly referenced channel within its frozen authorization cutoff",
            json!({
                "sourceHandle": { "type": "string", "minLength": 1, "maxLength": 1024 },
                "limit": { "type": "integer", "minimum": 1, "maximum": MAX_PAGE_SIZE },
                "cursor": { "type": "string", "minLength": 1, "maxLength": 1024 }
            }),
            true,
        ),
        tool(
            "chat_list_authorized_roots",
            "List the project folders explicitly authorized for this assignment",
            json!({}),
            true,
        ),
        tool(
            "chat_search_workspace_paths",
            "Search paths in one authorized project folder",
            json!({
                "rootHandle": { "type": "string", "minLength": 1, "maxLength": 1024 },
                "query": { "type": "string", "maxLength": MAX_QUERY_BYTES },
                "limit": { "type": "integer", "minimum": 1, "maximum": MAX_PAGE_SIZE },
                "cursor": { "type": "string", "minLength": 1, "maxLength": 1024 }
            }),
            true,
        ),
        tool(
            "chat_read_workspace_file",
            "Read a bounded UTF-8 file from one authorized project folder",
            json!({
                "rootHandle": { "type": "string", "minLength": 1, "maxLength": 1024 },
                "relativePath": { "type": "string", "minLength": 1, "maxLength": 4096 }
            }),
            true,
        ),
    ];
    if scope
        .folder_sources
        .iter()
        .any(|source| source.capability.rank() >= ChatFolderCapability::Edit.rank())
    {
        tools.extend([
            tool(
                "chat_write_workspace_file",
                "Replace an authorized UTF-8 file when its expected revision still matches",
                json!({
                    "rootHandle": { "type": "string", "minLength": 1, "maxLength": 1024 },
                    "relativePath": { "type": "string", "minLength": 1, "maxLength": 4096 },
                    "contents": { "type": "string", "maxLength": MAX_WORKSPACE_CONTENT_BYTES },
                    "expectedRevision": { "type": "string", "minLength": 16, "maxLength": 128 }
                }),
                false,
            ),
            tool(
                "chat_create_workspace_file",
                "Create a new bounded UTF-8 file in one authorized project folder",
                json!({
                    "rootHandle": { "type": "string", "minLength": 1, "maxLength": 1024 },
                    "relativePath": { "type": "string", "minLength": 1, "maxLength": 4096 },
                    "contents": { "type": "string", "maxLength": MAX_WORKSPACE_CONTENT_BYTES }
                }),
                false,
            ),
            tool(
                "chat_patch_workspace_file",
                "Apply bounded byte-range edits when the authorized file revision still matches",
                json!({
                    "rootHandle": { "type": "string", "minLength": 1, "maxLength": 1024 },
                    "relativePath": { "type": "string", "minLength": 1, "maxLength": 4096 },
                    "expectedRevision": { "type": "string", "minLength": 16, "maxLength": 128 },
                    "edits": {
                        "type": "array",
                        "minItems": 1,
                        "maxItems": MAX_WORKSPACE_PATCH_EDITS,
                        "items": {
                            "type": "object",
                            "properties": {
                                "startByte": { "type": "integer", "minimum": 0 },
                                "endByte": { "type": "integer", "minimum": 0 },
                                "replacement": { "type": "string", "maxLength": MAX_WORKSPACE_CONTENT_BYTES }
                            },
                            "required": ["startByte", "endByte", "replacement"],
                            "additionalProperties": false
                        }
                    }
                }),
                false,
            ),
            destructive_tool(
                "chat_delete_workspace_file",
                "Delete an authorized file when its expected revision still matches",
                json!({
                    "rootHandle": { "type": "string", "minLength": 1, "maxLength": 1024 },
                    "relativePath": { "type": "string", "minLength": 1, "maxLength": 4096 },
                    "expectedRevision": { "type": "string", "minLength": 16, "maxLength": 128 }
                }),
            ),
        ]);
    }
    tools
}

pub(crate) async fn call(
    context: HostToolContext<'_>,
    name: &str,
    arguments: &Map<String, Value>,
) -> ChatResult<Value> {
    let request_hash = request_hash(name, arguments)?;
    let response_reservation = host_tool_is_mutating(name).then_some(MAX_RESPONSE_BYTES);
    let invocation_id = reserve_audit(
        context.pool,
        context.scope,
        name,
        &request_hash,
        response_reservation.unwrap_or(0),
    )
    .await
    .map_err(|_| generic_denial())?;
    let result = call_authorized(&context, name, arguments).await;
    match result {
        Ok(value) => {
            let response_bytes = serde_json::to_vec(&value)
                .map_err(|_| internal_error("encode host-tool response"))?
                .len();
            if response_bytes > MAX_RESPONSE_BYTES {
                complete_audit_denied(
                    context.pool,
                    context.scope,
                    &invocation_id,
                    "budget_exhausted",
                    host_tool_is_mutating(name),
                )
                .await?;
                return Err(generic_denial());
            }
            let truncated = value
                .get("truncated")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let returned_revisions = returned_message_revisions(&value)?;
            let queried_channel_sources = if name == "chat_list_referenced_channels" {
                context.scope.channel_sources.iter().collect::<Vec<_>>()
            } else {
                value
                    .get("sourceHandle")
                    .and_then(Value::as_str)
                    .and_then(|handle| {
                        context
                            .scope
                            .channel_sources
                            .iter()
                            .find(|source| source.source_handle == handle)
                    })
                    .into_iter()
                    .collect::<Vec<_>>()
            };
            complete_audit_allowed(
                context.pool,
                context.scope,
                &invocation_id,
                response_bytes,
                truncated,
                &returned_revisions,
                &queried_channel_sources,
            )
            .await?;
            Ok(value)
        }
        Err(error) => {
            let mutation_outcome_unknown = host_tool_is_mutating(name);
            complete_audit_denied(
                context.pool,
                context.scope,
                &invocation_id,
                if mutation_outcome_unknown {
                    "mutation_outcome_unknown"
                } else {
                    denial_category(&error)
                },
                mutation_outcome_unknown,
            )
            .await?;
            Err(generic_denial())
        }
    }
}

async fn call_authorized(
    context: &HostToolContext<'_>,
    name: &str,
    arguments: &Map<String, Value>,
) -> ChatResult<Value> {
    verify_scope(context.pool, context.thread_id, context.scope).await?;
    match name {
        "chat_list_referenced_channels" => list_referenced_channels(context).await,
        "chat_search_channel_messages" => channel_messages(context, arguments, true).await,
        "chat_read_channel_messages" => channel_messages(context, arguments, false).await,
        "chat_list_authorized_roots" => list_authorized_roots(context).await,
        "chat_search_workspace_paths" => search_workspace_paths(context, arguments).await,
        "chat_read_workspace_file" => read_workspace_file(context, arguments).await,
        "chat_write_workspace_file" => write_workspace_file(context, arguments, false).await,
        "chat_create_workspace_file" => write_workspace_file(context, arguments, true).await,
        "chat_patch_workspace_file" => patch_workspace_file(context, arguments).await,
        "chat_delete_workspace_file" => delete_workspace_file(context, arguments).await,
        _ => Err(generic_denial()),
    }
}

pub(crate) async fn verify_scope(
    pool: &SqlitePool,
    thread_id: &super::models::ChatThreadId,
    scope: &InternalMcpRunScope,
) -> ChatResult<()> {
    let valid: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1
            FROM chat_assignment_authorization_revisions authorization
            JOIN chat_work_assignments assignment
              ON assignment.id = authorization.assignment_id
            JOIN chat_ai_channel_memberships channel_access
              ON channel_access.conversation_id = authorization.destination_conversation_id
             AND channel_access.teammate_id = assignment.teammate_id
            JOIN chat_conversation_memberships membership
              ON membership.conversation_id = channel_access.conversation_id
             AND membership.participant_id = channel_access.teammate_id
             AND membership.removed_at IS NULL
            JOIN chat_access_profiles profile
              ON profile.id = channel_access.access_profile_id
            JOIN chat_access_profile_revisions profile_revision
              ON profile_revision.access_profile_id = profile.id
             AND profile_revision.revision = profile.latest_revision
            JOIN chat_agent_runs run
              ON run.id = ?
             AND run.assignment_id = authorization.assignment_id
             AND run.provider_thread_id = ?
             AND run.provider_turn_id = ?
             AND run.authorization_revision_id = authorization.id
             AND run.authorization_scope_digest = authorization.scope_digest
             AND run.state IN ('starting', 'working', 'waiting')
            WHERE authorization.id = ?
              AND authorization.assignment_id = ?
              AND authorization.destination_conversation_id = ?
              AND authorization.scope_digest = ?
              AND authorization.decision_state = 'allowed'
              AND authorization.revoked_at IS NULL
              AND CASE
                    WHEN channel_access.participate_inherits_profile = 1
                      THEN profile_revision.default_participate
                    ELSE channel_access.participate AND profile_revision.default_participate
                  END = 1
        )",
    )
    .bind(scope.run_id.as_str())
    .bind(thread_id.as_str())
    .bind(scope.provider_turn_id.as_str())
    .bind(scope.authorization_revision_id.as_str())
    .bind(scope.assignment_id.as_str())
    .bind(scope.destination_conversation_id.as_str())
    .bind(&scope.authorization_scope_digest)
    .fetch_one(pool)
    .await
    .map_err(persistence_error)?;
    if !valid {
        return Err(generic_denial());
    }
    if let Some(scratch_generation_id) = scope.scratch_generation_id.as_deref() {
        super::scratch::require_reusable_generation(
            pool,
            scratch_generation_id,
            scope.authorization_revision_id.as_str(),
        )
        .await
        .map_err(|_| generic_denial())?;
    }
    Ok(())
}

pub(crate) async fn verify_publication_scope(
    pool: &SqlitePool,
    thread_id: &super::models::ChatThreadId,
    scope: &InternalMcpRunScope,
) -> ChatResult<()> {
    verify_scope(pool, thread_id, scope).await?;
    for source in &scope.channel_sources {
        verify_channel_source(pool, scope, source).await?;
    }
    for source in &scope.folder_sources {
        verify_folder_source(pool, scope, source).await?;
    }
    Ok(())
}

async fn verify_channel_source(
    pool: &SqlitePool,
    scope: &InternalMcpRunScope,
    source: &InternalMcpChannelSource,
) -> ChatResult<()> {
    let valid: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1
            FROM chat_assignment_authorized_channel_sources source
            JOIN chat_assignment_authorization_revisions authorization
              ON authorization.id = source.authorization_revision_id
             AND authorization.decision_state = 'allowed'
             AND authorization.revoked_at IS NULL
            JOIN chat_work_assignments assignment
              ON assignment.id = authorization.assignment_id
            JOIN chat_ai_channel_memberships channel_access
              ON channel_access.conversation_id = source.conversation_id
             AND channel_access.teammate_id = assignment.teammate_id
            JOIN chat_conversation_memberships membership
              ON membership.conversation_id = channel_access.conversation_id
             AND membership.participant_id = channel_access.teammate_id
             AND membership.removed_at IS NULL
            JOIN chat_access_profiles teammate_profile
              ON teammate_profile.id = channel_access.access_profile_id
            JOIN chat_access_profile_revisions teammate_profile_revision
              ON teammate_profile_revision.access_profile_id = teammate_profile.id
             AND teammate_profile_revision.revision = teammate_profile.latest_revision
            JOIN chat_conversation_memberships requester_membership
              ON requester_membership.conversation_id = source.conversation_id
             AND requester_membership.participant_id = authorization.requester_participant_id
             AND requester_membership.removed_at IS NULL
            JOIN chat_participants requester_participant
              ON requester_participant.id = requester_membership.participant_id
            JOIN chat_conversation_audience_state destination_audience
              ON destination_audience.conversation_id = authorization.destination_conversation_id
            JOIN chat_message_references reference
              ON reference.id = source.message_reference_id
            JOIN chat_communication_message_revisions reference_revision
              ON reference_revision.id = reference.message_revision_id
            JOIN chat_conversation_items reference_item
              ON reference_item.id = reference_revision.message_item_id
            LEFT JOIN chat_reply_threads reference_thread
              ON reference_thread.id = reference_item.reply_thread_id
            LEFT JOIN chat_conversation_items reference_root
              ON reference_root.id = reference_thread.root_item_id
            WHERE source.authorization_revision_id = ?
              AND source.source_handle = ?
              AND source.message_reference_id = ?
              AND source.conversation_id = ?
              AND source.lower_ordinal = ?
              AND source.high_ordinal = ?
              AND source.source_revision_cutoff_id = ?
              AND source.destination_audience_revision = ?
              AND destination_audience.revision >= source.destination_audience_revision
              AND reference_item.conversation_id = authorization.destination_conversation_id
              AND CASE
                    WHEN channel_access.read_history_inherits_profile = 1
                      THEN teammate_profile_revision.default_read_history
                    ELSE channel_access.read_history
                         AND teammate_profile_revision.default_read_history
                  END = 1
              AND (
                CASE
                  WHEN channel_access.history_boundary_inherits_profile = 1
                    THEN teammate_profile_revision.default_history_boundary
                  WHEN channel_access.history_boundary = 'from_grant'
                    OR teammate_profile_revision.default_history_boundary = 'from_grant'
                    THEN 'from_grant'
                  ELSE 'entire'
                END = 'entire'
                OR (
                  channel_access.history_from_ordinal IS NOT NULL
                  AND channel_access.history_from_ordinal <= source.lower_ordinal
                )
              )
              AND (
                requester_participant.participant_kind != 'ai_teammate'
                OR EXISTS (
                  SELECT 1
                  FROM chat_ai_channel_memberships requester_source_access
                  JOIN chat_access_profiles requester_profile
                    ON requester_profile.id = requester_source_access.access_profile_id
                  JOIN chat_access_profile_revisions requester_profile_revision
                    ON requester_profile_revision.access_profile_id = requester_profile.id
                   AND requester_profile_revision.revision = requester_profile.latest_revision
                  WHERE requester_source_access.conversation_id = source.conversation_id
                    AND requester_source_access.teammate_id = authorization.requester_participant_id
                    AND CASE
                          WHEN requester_source_access.read_history_inherits_profile = 1
                            THEN requester_profile_revision.default_read_history
                          ELSE requester_source_access.read_history
                               AND requester_profile_revision.default_read_history
                        END = 1
                    AND (
                      CASE
                        WHEN requester_source_access.history_boundary_inherits_profile = 1
                          THEN requester_profile_revision.default_history_boundary
                        WHEN requester_source_access.history_boundary = 'from_grant'
                          OR requester_profile_revision.default_history_boundary = 'from_grant'
                          THEN 'from_grant'
                        ELSE 'entire'
                      END = 'entire'
                      OR (
                        requester_source_access.history_from_ordinal IS NOT NULL
                        AND requester_source_access.history_from_ordinal <= source.lower_ordinal
                      )
                    )
                )
              )
              AND NOT EXISTS (
                SELECT 1
                FROM chat_conversation_memberships destination_member
                JOIN chat_participants destination_participant
                  ON destination_participant.id = destination_member.participant_id
                WHERE destination_member.conversation_id = authorization.destination_conversation_id
                  AND destination_member.removed_at IS NULL
                  AND (
                    destination_participant.participant_kind != 'ai_teammate'
                    OR EXISTS (
                      SELECT 1
                      FROM chat_ai_channel_memberships destination_ai_access
                      JOIN chat_access_profiles destination_profile
                        ON destination_profile.id = destination_ai_access.access_profile_id
                      JOIN chat_access_profile_revisions destination_profile_revision
                        ON destination_profile_revision.access_profile_id = destination_profile.id
                       AND destination_profile_revision.revision = destination_profile.latest_revision
                      WHERE destination_ai_access.conversation_id = authorization.destination_conversation_id
                        AND destination_ai_access.teammate_id = destination_member.participant_id
                        AND CASE
                              WHEN destination_ai_access.read_history_inherits_profile = 1
                                THEN destination_profile_revision.default_read_history
                              ELSE destination_ai_access.read_history
                                   AND destination_profile_revision.default_read_history
                            END = 1
                        AND (
                          CASE
                            WHEN destination_ai_access.history_boundary_inherits_profile = 1
                              THEN destination_profile_revision.default_history_boundary
                            WHEN destination_ai_access.history_boundary = 'from_grant'
                              OR destination_profile_revision.default_history_boundary = 'from_grant'
                              THEN 'from_grant'
                            ELSE 'entire'
                          END = 'entire'
                          OR (
                            destination_ai_access.history_from_ordinal IS NOT NULL
                            AND destination_ai_access.history_from_ordinal
                              <= coalesce(reference_root.ordinal, reference_item.ordinal)
                          )
                        )
                    )
                  )
                  AND (
                    NOT EXISTS (
                      SELECT 1 FROM chat_conversation_memberships source_member
                      WHERE source_member.conversation_id = source.conversation_id
                        AND source_member.participant_id = destination_member.participant_id
                        AND source_member.removed_at IS NULL
                    )
                    OR (
                      destination_participant.participant_kind = 'ai_teammate'
                      AND NOT EXISTS (
                        SELECT 1
                        FROM chat_ai_channel_memberships source_ai_access
                        JOIN chat_access_profiles source_profile
                          ON source_profile.id = source_ai_access.access_profile_id
                        JOIN chat_access_profile_revisions source_profile_revision
                          ON source_profile_revision.access_profile_id = source_profile.id
                         AND source_profile_revision.revision = source_profile.latest_revision
                        WHERE source_ai_access.conversation_id = source.conversation_id
                          AND source_ai_access.teammate_id = destination_member.participant_id
                          AND CASE
                                WHEN source_ai_access.read_history_inherits_profile = 1
                                  THEN source_profile_revision.default_read_history
                                ELSE source_ai_access.read_history
                                     AND source_profile_revision.default_read_history
                              END = 1
                          AND (
                            CASE
                              WHEN source_ai_access.history_boundary_inherits_profile = 1
                                THEN source_profile_revision.default_history_boundary
                              WHEN source_ai_access.history_boundary = 'from_grant'
                                OR source_profile_revision.default_history_boundary = 'from_grant'
                                THEN 'from_grant'
                              ELSE 'entire'
                            END = 'entire'
                            OR (
                              source_ai_access.history_from_ordinal IS NOT NULL
                              AND source_ai_access.history_from_ordinal <= source.lower_ordinal
                            )
                          )
                      )
                    )
                  )
              )
        )",
    )
    .bind(scope.authorization_revision_id.as_str())
    .bind(&source.source_handle)
    .bind(&source.message_reference_id)
    .bind(&source.conversation_id)
    .bind(i64::try_from(source.lower_ordinal).map_err(|_| generic_denial())?)
    .bind(i64::try_from(source.high_ordinal).map_err(|_| generic_denial())?)
    .bind(&source.source_revision_cutoff_id)
    .bind(i64::try_from(source.destination_audience_revision).map_err(|_| generic_denial())?)
    .fetch_one(pool)
    .await
    .map_err(persistence_error)?;
    if valid {
        Ok(())
    } else {
        Err(generic_denial())
    }
}

async fn verify_folder_source(
    pool: &SqlitePool,
    scope: &InternalMcpRunScope,
    source: &InternalMcpFolderSource,
) -> ChatResult<()> {
    let capability = wire_folder_capability(source.capability);
    let valid: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1
            FROM chat_assignment_authorized_folder_sources source
            JOIN chat_assignment_authorization_revisions authorization
              ON authorization.id = source.authorization_revision_id
             AND authorization.decision_state = 'allowed'
             AND authorization.revoked_at IS NULL
            JOIN chat_work_assignments assignment
              ON assignment.id = authorization.assignment_id
            JOIN chat_conversation_memberships membership
              ON membership.conversation_id = authorization.destination_conversation_id
             AND membership.participant_id = assignment.teammate_id
             AND membership.removed_at IS NULL
            JOIN chat_ai_channel_memberships channel_access
              ON channel_access.conversation_id = membership.conversation_id
             AND channel_access.teammate_id = membership.participant_id
            JOIN chat_ai_teammate_access_state access_state
              ON access_state.teammate_id = membership.participant_id
            JOIN chat_access_profiles profile
              ON profile.id = channel_access.access_profile_id
            JOIN chat_access_profile_revisions profile_revision
              ON profile_revision.access_profile_id = profile.id
             AND profile_revision.revision = profile.latest_revision
            JOIN chat_teammate_working_folder_grants live_grant
              ON live_grant.conversation_id = authorization.destination_conversation_id
             AND live_grant.teammate_id = assignment.teammate_id
             AND live_grant.working_folder_id = source.working_folder_id
             AND live_grant.revoked_at IS NULL
            WHERE source.authorization_revision_id = ?
              AND source.root_handle = ?
              AND source.working_folder_id = ?
              AND source.capability = ?
              AND source.is_execution_target = ?
              AND (
                coalesce(live_grant.runtime_approval_policy,
                         channel_access.runtime_approval_policy,
                         access_state.runtime_approval_policy) = ?
                OR CASE coalesce(live_grant.runtime_approval_policy,
                                 channel_access.runtime_approval_policy,
                                 access_state.runtime_approval_policy)
                     WHEN 'ask' THEN 0 WHEN 'auto_approve' THEN 1
                     WHEN 'unattended' THEN 2 ELSE -1
                   END >= CASE ?
                     WHEN 'ask' THEN 0 WHEN 'auto_approve' THEN 1
                     WHEN 'unattended' THEN 2 ELSE 3
                   END
              )
              AND CASE source.capability
                    WHEN 'read' THEN 1 WHEN 'edit' THEN 2
                    WHEN 'execute' THEN 3 WHEN 'publish' THEN 4 ELSE 5
                  END <= CASE
                    WHEN live_grant.capability_inherits_profile = 1 THEN
                      CASE profile_revision.maximum_folder_capability
                        WHEN 'none' THEN 0 WHEN 'read' THEN 1 WHEN 'edit' THEN 2
                        WHEN 'execute' THEN 3 WHEN 'publish' THEN 4 ELSE 0
                      END
                    ELSE min(
                      CASE live_grant.capability
                        WHEN 'none' THEN 0 WHEN 'read' THEN 1 WHEN 'edit' THEN 2
                        WHEN 'execute' THEN 3 WHEN 'publish' THEN 4 ELSE 0
                      END,
                      CASE profile_revision.maximum_folder_capability
                        WHEN 'none' THEN 0 WHEN 'read' THEN 1 WHEN 'edit' THEN 2
                        WHEN 'execute' THEN 3 WHEN 'publish' THEN 4 ELSE 0
                      END
                    )
                  END
        )",
    )
    .bind(scope.authorization_revision_id.as_str())
    .bind(&source.root_handle)
    .bind(source.working_folder_id.as_str())
    .bind(capability)
    .bind(source.is_execution_target)
    .bind(wire_runtime_approval_policy(source.runtime_approval_policy))
    .bind(wire_runtime_approval_policy(source.runtime_approval_policy))
    .fetch_one(pool)
    .await
    .map_err(persistence_error)?;
    if valid {
        Ok(())
    } else {
        Err(generic_denial())
    }
}

async fn list_referenced_channels(context: &HostToolContext<'_>) -> ChatResult<Value> {
    let mut channels = Vec::with_capacity(context.scope.channel_sources.len());
    for source in &context.scope.channel_sources {
        verify_channel_source(context.pool, context.scope, source).await?;
        channels.push(json!({
            "sourceHandle": source.source_handle,
            "label": source.label_snapshot,
            "lowerOrdinal": source.lower_ordinal,
            "highOrdinal": source.high_ordinal,
        }));
    }
    Ok(json!({ "channels": channels, "truncated": false }))
}

async fn list_authorized_roots(context: &HostToolContext<'_>) -> ChatResult<Value> {
    let mut roots = Vec::with_capacity(context.scope.folder_sources.len());
    for source in &context.scope.folder_sources {
        verify_folder_source(context.pool, context.scope, source).await?;
        roots.push(json!({
            "rootHandle": source.root_handle,
            "capability": wire_folder_capability(source.capability),
            "isExecutionTarget": source.is_execution_target,
        }));
    }
    Ok(json!({ "roots": roots, "truncated": false }))
}

async fn channel_messages(
    context: &HostToolContext<'_>,
    arguments: &Map<String, Value>,
    searching: bool,
) -> ChatResult<Value> {
    let source_handle = required_string(arguments, "sourceHandle", 1024)?;
    let source = context
        .scope
        .channel_sources
        .iter()
        .find(|source| source.source_handle == source_handle)
        .ok_or_else(generic_denial)?;
    verify_channel_source(context.pool, context.scope, source).await?;
    let terms = if searching {
        normalized_query_terms(required_string(arguments, "query", MAX_QUERY_BYTES)?)?
    } else {
        Vec::new()
    };
    let query_hash = query_terms_hash(&terms);
    let cursor = optional_string(arguments, "cursor", 1024)?;
    let cursor = match cursor {
        Some(cursor) => Some(
            context
                .runtime
                .channel_cursor(cursor, source_handle, &query_hash)
                .await?,
        ),
        None => None,
    };
    let limit = optional_limit(arguments)?.unwrap_or(DEFAULT_PAGE_SIZE);
    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT item.id AS item_id, item.reply_thread_id, item.created_at, \
                revision.id AS revision_id, revision.revision, revision.normalized_markdown, \
                message.author_participant_id AS author_id, \
                message.author_label_snapshot AS author_name \
         FROM chat_conversation_items item \
         JOIN chat_communication_messages message ON message.item_id = item.id \
         JOIN chat_assignment_authorization_revisions authorization ON authorization.id = ",
    );
    query.push_bind(context.scope.authorization_revision_id.as_str());
    query.push(
        " JOIN chat_assignment_authorized_channel_sources source_grant \
            ON source_grant.authorization_revision_id = authorization.id \
           AND source_grant.source_handle = ",
    );
    query.push_bind(source_handle);
    query.push(
        " JOIN chat_communication_message_revision_ordinals cutoff_ordinal \
            ON cutoff_ordinal.message_revision_id = source_grant.source_revision_cutoff_id \
         JOIN chat_communication_message_revisions revision \
            ON revision.message_item_id = message.item_id \
           AND revision.revision = ( \
             SELECT max(candidate.revision) \
             FROM chat_communication_message_revisions candidate \
             JOIN chat_communication_message_revision_ordinals candidate_ordinal \
               ON candidate_ordinal.message_revision_id = candidate.id \
             WHERE candidate.message_item_id = message.item_id \
               AND candidate_ordinal.ordinal <= cutoff_ordinal.ordinal \
           ) \
         LEFT JOIN chat_reply_threads reply_thread ON reply_thread.id = item.reply_thread_id \
         LEFT JOIN chat_conversation_items root_item ON root_item.id = reply_thread.root_item_id \
         WHERE item.conversation_id = ",
    );
    query.push_bind(&source.conversation_id);
    query.push(" AND message.deleted_at IS NULL AND coalesce(root_item.ordinal, item.ordinal) >= ");
    query.push_bind(i64::try_from(source.lower_ordinal).map_err(|_| generic_denial())?);
    query.push(" AND coalesce(root_item.ordinal, item.ordinal) <= ");
    query.push_bind(i64::try_from(source.high_ordinal).map_err(|_| generic_denial())?);
    if let Some(cursor) = cursor {
        query.push(" AND (item.created_at < ");
        query.push_bind(cursor.before_created_at);
        query.push(" OR (item.created_at = ");
        query.push_bind(cursor.before_created_at_again);
        query.push(" AND item.id < ");
        query.push_bind(cursor.before_item_id);
        query.push("))");
    }
    for term in &terms {
        query.push(" AND lower(revision.normalized_markdown) LIKE ");
        query.push_bind(format!("%{}%", escape_like(term)));
        query.push(" ESCAPE '\\'");
    }
    query.push(" ORDER BY item.created_at DESC, item.id DESC LIMIT ");
    query.push_bind(i64::from(limit.saturating_add(1)));
    let rows = query
        .build()
        .fetch_all(context.pool)
        .await
        .map_err(persistence_error)?;
    let has_more_rows = rows.len() > limit as usize;
    let mut messages = Vec::new();
    let mut last_position = None;
    let mut response_truncated = false;
    for row in rows.into_iter().take(limit as usize) {
        let text: String = row
            .try_get("normalized_markdown")
            .map_err(persistence_error)?;
        let content_hash = sha256_hex(text.as_bytes());
        let text = truncate_utf8(&text, 16 * 1024);
        let created_at: String = row.try_get("created_at").map_err(persistence_error)?;
        let item_id: String = row.try_get("item_id").map_err(persistence_error)?;
        let candidate = json!({
            "messageId": item_id,
            "revisionId": row.try_get::<String, _>("revision_id").map_err(persistence_error)?,
            "revision": u64::try_from(row.try_get::<i64, _>("revision").map_err(persistence_error)?)
                .map_err(|_| generic_denial())?,
            "author": {
                "id": row.try_get::<String, _>("author_id").map_err(persistence_error)?,
                "displayName": row.try_get::<String, _>("author_name").map_err(persistence_error)?,
            },
            "createdAt": created_at,
            "thread": {
                "kind": if row.try_get::<Option<String>, _>("reply_thread_id").map_err(persistence_error)?.is_some() {
                    "reply"
                } else {
                    "root"
                },
                "replyThreadId": row.try_get::<Option<String>, _>("reply_thread_id").map_err(persistence_error)?,
            },
            "normalizedMarkdown": text.value,
            "contentHash": content_hash,
            "truncated": text.truncated,
        });
        let mut proposed = messages.clone();
        proposed.push(candidate.clone());
        let proposed_size = serde_json::to_vec(&json!({ "messages": proposed }))
            .map_err(|_| internal_error("encode channel messages"))?
            .len();
        if proposed_size > MAX_RESPONSE_BYTES.saturating_sub(4096) {
            response_truncated = true;
            break;
        }
        last_position = Some((created_at, item_id));
        messages.push(candidate);
    }
    let next_cursor = if has_more_rows || response_truncated {
        match last_position {
            Some((before_created_at, before_item_id)) => Some(
                context
                    .runtime
                    .store_cursor(OpaqueCursor::Channel {
                        source_handle: source_handle.to_string(),
                        query_hash,
                        before_created_at,
                        before_item_id,
                    })
                    .await?,
            ),
            None => None,
        }
    } else {
        None
    };
    Ok(json!({
        "sourceHandle": source_handle,
        "sourceLabel": source.label_snapshot,
        "messages": messages,
        "nextCursor": next_cursor,
        "truncated": has_more_rows || response_truncated,
    }))
}

struct ChannelCursorRead {
    before_created_at: String,
    before_created_at_again: String,
    before_item_id: String,
}

struct TruncatedText<'a> {
    value: &'a str,
    truncated: bool,
}

fn truncate_utf8(value: &str, maximum_bytes: usize) -> TruncatedText<'_> {
    if value.len() <= maximum_bytes {
        return TruncatedText {
            value,
            truncated: false,
        };
    }
    let mut end = maximum_bytes.min(value.len());
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    TruncatedText {
        value: &value[..end],
        truncated: true,
    }
}

async fn search_workspace_paths(
    context: &HostToolContext<'_>,
    arguments: &Map<String, Value>,
) -> ChatResult<Value> {
    let root_handle = required_string(arguments, "rootHandle", 1024)?;
    let source = folder_source(context.scope, root_handle, ChatFolderCapability::Read)?;
    verify_folder_source(context.pool, context.scope, source).await?;
    let authorized = authorize_folder(context, source, false).await?;
    let query = required_string(arguments, "query", MAX_QUERY_BYTES)?;
    let query_hash = sha256_hex(query.trim().to_lowercase().as_bytes());
    let provider_cursor = match optional_string(arguments, "cursor", 1024)? {
        Some(cursor) => Some(
            context
                .runtime
                .workspace_cursor(cursor, root_handle, &query_hash)
                .await?,
        ),
        None => None,
    };
    let limit = optional_limit(arguments)?.unwrap_or(DEFAULT_PAGE_SIZE);
    let root = authorized.canonical_path;
    let query_owned = query.to_string();
    let page = tauri::async_runtime::spawn_blocking(move || {
        super::composer::workspace_mentions::search_workspace_paths(
            &root,
            &query_owned,
            false,
            provider_cursor.as_deref(),
            limit,
        )
    })
    .await
    .map_err(|_| internal_error("search authorized workspace"))??;
    let next_cursor = match page.next_cursor {
        Some(provider_cursor) => Some(
            context
                .runtime
                .store_cursor(OpaqueCursor::Workspace {
                    root_handle: root_handle.to_string(),
                    query_hash,
                    provider_cursor,
                })
                .await?,
        ),
        None => None,
    };
    let truncated = next_cursor.is_some();
    Ok(json!({
        "rootHandle": root_handle,
        "entries": page.entries,
        "nextCursor": next_cursor,
        "truncated": truncated,
    }))
}

async fn read_workspace_file(
    context: &HostToolContext<'_>,
    arguments: &Map<String, Value>,
) -> ChatResult<Value> {
    let root_handle = required_string(arguments, "rootHandle", 1024)?;
    let source = folder_source(context.scope, root_handle, ChatFolderCapability::Read)?;
    verify_folder_source(context.pool, context.scope, source).await?;
    let authorized = authorize_folder(context, source, false).await?;
    let relative_path = required_string(arguments, "relativePath", 4096)?.to_string();
    let preview = tauri::async_runtime::spawn_blocking(move || {
        super::workspace_files::preview_workspace_file(&authorized, &relative_path)
    })
    .await
    .map_err(|_| internal_error("read authorized workspace file"))??;
    if preview.binary || preview.oversized || preview.text.is_none() {
        return Err(generic_denial());
    }
    let text = preview.text.as_deref().ok_or_else(generic_denial)?;
    let text = truncate_utf8(text, 48 * 1024);
    Ok(json!({
        "rootHandle": root_handle,
        "relativePath": preview.relative_path,
        "normalizedText": text.value,
        "byteSize": preview.byte_size,
        "lineCount": preview.line_count,
        "contentRevision": preview.content_revision,
        "truncated": text.truncated,
    }))
}

async fn write_workspace_file(
    context: &HostToolContext<'_>,
    arguments: &Map<String, Value>,
    create: bool,
) -> ChatResult<Value> {
    let root_handle = required_string(arguments, "rootHandle", 1024)?;
    let source = folder_source(context.scope, root_handle, ChatFolderCapability::Edit)?;
    require_resolved_mutation_approval(source.runtime_approval_policy)?;
    verify_folder_source(context.pool, context.scope, source).await?;
    let authorized = authorize_folder(context, source, true).await?;
    let relative_path = required_string(arguments, "relativePath", 4096)?.to_string();
    let invalidation_path = relative_path.clone();
    let contents =
        bounded_text_argument(arguments, "contents", MAX_WORKSPACE_CONTENT_BYTES)?.to_string();
    let expected_revision = if create {
        None
    } else {
        Some(required_string(arguments, "expectedRevision", 128)?.to_string())
    };
    let preview = tauri::async_runtime::spawn_blocking(move || match expected_revision {
        Some(expected_revision) => super::workspace_files::save_workspace_file(
            &authorized,
            &relative_path,
            &contents,
            &expected_revision,
        ),
        None => super::workspace_files::recreate_workspace_file(
            &authorized,
            &relative_path,
            &contents,
            true,
        ),
    })
    .await
    .map_err(|_| internal_error("write authorized workspace file"))??;
    context
        .app
        .state::<super::workspace_observer::ChatWorkspaceObserverRegistry>()
        .invalidate_paths(
            &source.working_folder_id,
            source
                .is_execution_target
                .then_some(context.scope.execution_environment_id.as_deref())
                .flatten(),
            vec![invalidation_path],
            false,
        );
    Ok(json!({
        "rootHandle": root_handle,
        "relativePath": preview.relative_path,
        "byteSize": preview.byte_size,
        "lineCount": preview.line_count,
        "contentRevision": preview.content_revision,
        "created": create,
        "truncated": false,
    }))
}

struct WorkspacePatchEdit {
    start_byte: usize,
    end_byte: usize,
    replacement: String,
}

async fn patch_workspace_file(
    context: &HostToolContext<'_>,
    arguments: &Map<String, Value>,
) -> ChatResult<Value> {
    let root_handle = required_string(arguments, "rootHandle", 1024)?;
    let source = folder_source(context.scope, root_handle, ChatFolderCapability::Edit)?;
    require_resolved_mutation_approval(source.runtime_approval_policy)?;
    verify_folder_source(context.pool, context.scope, source).await?;
    let authorized = authorize_folder(context, source, true).await?;
    let relative_path = required_string(arguments, "relativePath", 4096)?.to_string();
    let expected_revision = required_string(arguments, "expectedRevision", 128)?.to_string();
    let preview_authorization = authorized.clone();
    let preview_path = relative_path.clone();
    let preview = tauri::async_runtime::spawn_blocking(move || {
        super::workspace_files::preview_workspace_file(&preview_authorization, &preview_path)
    })
    .await
    .map_err(|_| internal_error("read authorized workspace file for patching"))??;
    let current_revision = preview
        .content_revision
        .as_deref()
        .ok_or_else(generic_denial)?;
    if current_revision != expected_revision {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "The workspace file changed before the patch was applied",
            true,
        ));
    }
    let current = preview.text.ok_or_else(generic_denial)?;
    let edits = workspace_patch_edits(arguments, &current)?;
    let mut patched = current;
    for edit in edits.iter().rev() {
        patched.replace_range(edit.start_byte..edit.end_byte, &edit.replacement);
    }
    let invalidation_path = relative_path.clone();
    let saved = tauri::async_runtime::spawn_blocking(move || {
        super::workspace_files::save_workspace_file(
            &authorized,
            &relative_path,
            &patched,
            &expected_revision,
        )
    })
    .await
    .map_err(|_| internal_error("patch authorized workspace file"))??;
    invalidate_workspace_path(context, source, invalidation_path);
    Ok(json!({
        "rootHandle": root_handle,
        "relativePath": saved.relative_path,
        "byteSize": saved.byte_size,
        "lineCount": saved.line_count,
        "contentRevision": saved.content_revision,
        "patched": true,
        "truncated": false,
    }))
}

async fn delete_workspace_file(
    context: &HostToolContext<'_>,
    arguments: &Map<String, Value>,
) -> ChatResult<Value> {
    let root_handle = required_string(arguments, "rootHandle", 1024)?;
    let source = folder_source(context.scope, root_handle, ChatFolderCapability::Edit)?;
    require_resolved_mutation_approval(source.runtime_approval_policy)?;
    verify_folder_source(context.pool, context.scope, source).await?;
    let authorized = authorize_folder(context, source, true).await?;
    let relative_path = required_string(arguments, "relativePath", 4096)?.to_string();
    let expected_revision = required_string(arguments, "expectedRevision", 128)?.to_string();
    let invalidation_path = relative_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        super::workspace_files::delete_workspace_file(
            &authorized,
            &relative_path,
            &expected_revision,
        )
    })
    .await
    .map_err(|_| internal_error("delete authorized workspace file"))??;
    invalidate_workspace_path(context, source, invalidation_path.clone());
    Ok(json!({
        "rootHandle": root_handle,
        "relativePath": invalidation_path,
        "deleted": true,
        "truncated": false,
    }))
}

fn workspace_patch_edits(
    arguments: &Map<String, Value>,
    current: &str,
) -> ChatResult<Vec<WorkspacePatchEdit>> {
    let values = arguments
        .get("edits")
        .and_then(Value::as_array)
        .filter(|values| !values.is_empty() && values.len() <= MAX_WORKSPACE_PATCH_EDITS)
        .ok_or_else(generic_denial)?;
    let mut edits = Vec::with_capacity(values.len());
    let mut next_minimum = 0_usize;
    let mut previous_start = None;
    let mut resulting_bytes = current.len();
    for value in values {
        let edit = value.as_object().ok_or_else(generic_denial)?;
        if edit.len() != 3 {
            return Err(generic_denial());
        }
        let start_byte = bounded_usize(edit, "startByte")?;
        let end_byte = bounded_usize(edit, "endByte")?;
        let replacement = bounded_text_argument(edit, "replacement", MAX_WORKSPACE_CONTENT_BYTES)?;
        if start_byte > end_byte
            || end_byte > current.len()
            || start_byte < next_minimum
            || previous_start == Some(start_byte)
            || !current.is_char_boundary(start_byte)
            || !current.is_char_boundary(end_byte)
        {
            return Err(generic_denial());
        }
        resulting_bytes = resulting_bytes
            .checked_sub(end_byte - start_byte)
            .and_then(|value| value.checked_add(replacement.len()))
            .filter(|value| *value <= MAX_WORKSPACE_CONTENT_BYTES)
            .ok_or_else(generic_denial)?;
        edits.push(WorkspacePatchEdit {
            start_byte,
            end_byte,
            replacement: replacement.to_string(),
        });
        next_minimum = end_byte;
        previous_start = Some(start_byte);
    }
    Ok(edits)
}

fn bounded_usize(arguments: &Map<String, Value>, name: &str) -> ChatResult<usize> {
    arguments
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(generic_denial)
}

fn require_resolved_mutation_approval(policy: ChatRuntimeApprovalPolicy) -> ChatResult<()> {
    if matches!(
        policy,
        ChatRuntimeApprovalPolicy::AutoApprove | ChatRuntimeApprovalPolicy::Unattended
    ) {
        Ok(())
    } else {
        Err(ChatError::new(
            ChatErrorCode::Permission,
            "This host-tool mutation requires a resolved runtime approval",
            true,
        ))
    }
}

fn invalidate_workspace_path(
    context: &HostToolContext<'_>,
    source: &InternalMcpFolderSource,
    relative_path: String,
) {
    context
        .app
        .state::<super::workspace_observer::ChatWorkspaceObserverRegistry>()
        .invalidate_paths(
            &source.working_folder_id,
            source
                .is_execution_target
                .then_some(context.scope.execution_environment_id.as_deref())
                .flatten(),
            vec![relative_path],
            false,
        );
}

fn folder_source<'a>(
    scope: &'a InternalMcpRunScope,
    root_handle: &str,
    minimum: ChatFolderCapability,
) -> ChatResult<&'a InternalMcpFolderSource> {
    scope
        .folder_sources
        .iter()
        .find(|source| {
            source.root_handle == root_handle && source.capability.rank() >= minimum.rank()
        })
        .ok_or_else(generic_denial)
}

async fn authorize_folder(
    context: &HostToolContext<'_>,
    source: &InternalMcpFolderSource,
    write: bool,
) -> ChatResult<super::workspace::AuthorizedWorkingFolder> {
    let operation = if write {
        WorkingFolderAuthorizationOperation::FileWrite
    } else {
        WorkingFolderAuthorizationOperation::FileRead
    };
    let authorized = super::workspace_commands::authorize_working_folder(
        context.app,
        context.pool,
        &source.working_folder_id,
        operation,
    )
    .await?;
    let environment_id = source
        .is_execution_target
        .then_some(context.scope.execution_environment_id.as_deref())
        .flatten();
    super::execution_environment::resolve_environment_workspace(
        context.app,
        context.pool,
        authorized,
        environment_id,
    )
    .await
}

impl InternalMcpToolRuntime {
    pub(crate) async fn reset_cursors(&self) {
        self.cursors.lock().await.clear();
    }

    async fn store_cursor(&self, cursor: OpaqueCursor) -> ChatResult<String> {
        let token = generate_opaque_handle("cursor")?;
        let mut cursors = self.cursors.lock().await;
        if cursors.len() >= MAX_CURSORS {
            cursors.clear();
        }
        cursors.insert(token.clone(), cursor);
        Ok(token)
    }

    async fn channel_cursor(
        &self,
        token: &str,
        source_handle: &str,
        query_hash: &str,
    ) -> ChatResult<ChannelCursorRead> {
        match self.cursors.lock().await.get(token).cloned() {
            Some(OpaqueCursor::Channel {
                source_handle: stored_source,
                query_hash: stored_query,
                before_created_at,
                before_item_id,
            }) if stored_source == source_handle && stored_query == query_hash => {
                Ok(ChannelCursorRead {
                    before_created_at_again: before_created_at.clone(),
                    before_created_at,
                    before_item_id,
                })
            }
            _ => Err(generic_denial()),
        }
    }

    async fn workspace_cursor(
        &self,
        token: &str,
        root_handle: &str,
        query_hash: &str,
    ) -> ChatResult<String> {
        match self.cursors.lock().await.get(token).cloned() {
            Some(OpaqueCursor::Workspace {
                root_handle: stored_root,
                query_hash: stored_query,
                provider_cursor,
            }) if stored_root == root_handle && stored_query == query_hash => Ok(provider_cursor),
            _ => Err(generic_denial()),
        }
    }
}

fn tool(name: &'static str, description: &'static str, properties: Value, read_only: bool) -> Tool {
    configured_tool(name, description, properties, read_only, false)
}

fn destructive_tool(name: &'static str, description: &'static str, properties: Value) -> Tool {
    configured_tool(name, description, properties, false, true)
}

fn configured_tool(
    name: &'static str,
    description: &'static str,
    properties: Value,
    read_only: bool,
    destructive: bool,
) -> Tool {
    let properties = properties.as_object().cloned().unwrap_or_default();
    let required = properties
        .keys()
        .filter(|key| !matches!(key.as_str(), "cursor" | "limit"))
        .cloned()
        .map(Value::String)
        .collect::<Vec<_>>();
    let schema = json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
    .as_object()
    .cloned()
    .unwrap_or_default();
    Tool::new(name, description, schema).with_annotations(
        ToolAnnotations::new()
            .read_only(read_only)
            .destructive(destructive)
            .open_world(false),
    )
}

fn required_string<'a>(
    arguments: &'a Map<String, Value>,
    name: &str,
    maximum_bytes: usize,
) -> ChatResult<&'a str> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| value.len() <= maximum_bytes)
        .filter(|value| !value.contains('\0') && !value.chars().any(char::is_control))
        .ok_or_else(generic_denial)
}

fn optional_string<'a>(
    arguments: &'a Map<String, Value>,
    name: &str,
    maximum_bytes: usize,
) -> ChatResult<Option<&'a str>> {
    match arguments.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => required_string(arguments, name, maximum_bytes).map(Some),
    }
}

fn bounded_text_argument<'a>(
    arguments: &'a Map<String, Value>,
    name: &str,
    maximum_bytes: usize,
) -> ChatResult<&'a str> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| value.len() <= maximum_bytes && !value.contains('\0'))
        .ok_or_else(generic_denial)
}

fn optional_limit(arguments: &Map<String, Value>) -> ChatResult<Option<u32>> {
    match arguments.get("limit") {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| (1..=MAX_PAGE_SIZE).contains(value))
            .map(Some)
            .ok_or_else(generic_denial),
    }
}

fn normalized_query_terms(query: &str) -> ChatResult<Vec<String>> {
    if query.is_empty() || query.len() > MAX_QUERY_BYTES {
        return Err(generic_denial());
    }
    let mut terms = Vec::new();
    for term in query.split_whitespace().map(str::to_lowercase) {
        if term.is_empty() || terms.contains(&term) {
            continue;
        }
        if terms.len() >= MAX_QUERY_TERMS {
            return Err(generic_denial());
        }
        terms.push(term);
    }
    if terms.is_empty() {
        Err(generic_denial())
    } else {
        Ok(terms)
    }
}

fn query_terms_hash(terms: &[String]) -> String {
    sha256_hex(terms.join("\0").as_bytes())
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn request_hash(name: &str, arguments: &Map<String, Value>) -> ChatResult<String> {
    let encoded = serde_json::to_vec(&(name, arguments))
        .map_err(|_| internal_error("encode host-tool request"))?;
    Ok(sha256_hex(&encoded))
}

fn sha256_hex(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn wire_folder_capability(capability: ChatFolderCapability) -> &'static str {
    match capability {
        ChatFolderCapability::None => "none",
        ChatFolderCapability::Read => "read",
        ChatFolderCapability::Edit => "edit",
        ChatFolderCapability::Execute => "execute",
        ChatFolderCapability::Publish => "publish",
    }
}

fn wire_runtime_approval_policy(policy: ChatRuntimeApprovalPolicy) -> &'static str {
    match policy {
        ChatRuntimeApprovalPolicy::Ask => "ask",
        ChatRuntimeApprovalPolicy::AutoApprove => "auto_approve",
        ChatRuntimeApprovalPolicy::Unattended => "unattended",
        ChatRuntimeApprovalPolicy::ProviderCustom => "provider_custom",
    }
}

async fn reserve_audit(
    pool: &SqlitePool,
    scope: &InternalMcpRunScope,
    tool_name: &str,
    request_hash: &str,
    response_reservation: usize,
) -> ChatResult<String> {
    let id = generate_opaque_handle("host-tool")?;
    let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let inserted = sqlx::query(
        "INSERT INTO chat_host_tool_invocations
            (id, authorization_revision_id, tool_name, request_hash,
             decision_state, denial_category, response_bytes, truncated, created_at)
         SELECT ?, ?, ?, ?, 'denied', 'pending', ?, 0, ?
         WHERE EXISTS (
             SELECT 1 FROM chat_assignment_authorization_revisions authorization
             WHERE authorization.id = ?
               AND authorization.decision_state = 'allowed'
               AND authorization.revoked_at IS NULL
         )
           AND (
             SELECT count(*) FROM chat_host_tool_invocations invocation
             WHERE invocation.authorization_revision_id = ?
           ) < ?
           AND ? + coalesce((
             SELECT sum(invocation.response_bytes)
             FROM chat_host_tool_invocations invocation
             WHERE invocation.authorization_revision_id = ?
           ), 0) <= ?",
    )
    .bind(&id)
    .bind(scope.authorization_revision_id.as_str())
    .bind(tool_name)
    .bind(request_hash)
    .bind(i64::try_from(response_reservation).map_err(|_| generic_denial())?)
    .bind(created_at)
    .bind(scope.authorization_revision_id.as_str())
    .bind(scope.authorization_revision_id.as_str())
    .bind(i64::from(MAX_TOOL_CALLS))
    .bind(i64::try_from(response_reservation).map_err(|_| generic_denial())?)
    .bind(scope.authorization_revision_id.as_str())
    .bind(i64::try_from(MAX_SCOPE_RESPONSE_BYTES).map_err(|_| generic_denial())?)
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    if inserted.rows_affected() == 1 {
        Ok(id)
    } else {
        Err(generic_denial())
    }
}

async fn complete_audit_allowed(
    pool: &SqlitePool,
    scope: &InternalMcpRunScope,
    invocation_id: &str,
    response_bytes: usize,
    truncated: bool,
    returned_revisions: &[(String, String)],
    queried_channel_sources: &[&InternalMcpChannelSource],
) -> ChatResult<()> {
    let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let response_bytes = i64::try_from(response_bytes).map_err(|_| generic_denial())?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    let completed = sqlx::query(
        "UPDATE chat_host_tool_invocations
         SET decision_state = 'allowed', denial_category = NULL,
             response_bytes = ?, truncated = ?
         WHERE id = ? AND authorization_revision_id = ?
           AND decision_state = 'denied' AND denial_category = 'pending'
           AND EXISTS (
             SELECT 1 FROM chat_assignment_authorization_revisions authorization
             WHERE authorization.id = ?
               AND authorization.decision_state = 'allowed'
               AND authorization.revoked_at IS NULL
           )
           AND ? + coalesce((
             SELECT sum(other.response_bytes)
             FROM chat_host_tool_invocations other
             WHERE other.authorization_revision_id = ? AND other.id != ?
           ), 0) <= ?",
    )
    .bind(response_bytes)
    .bind(truncated)
    .bind(invocation_id)
    .bind(scope.authorization_revision_id.as_str())
    .bind(scope.authorization_revision_id.as_str())
    .bind(response_bytes)
    .bind(scope.authorization_revision_id.as_str())
    .bind(invocation_id)
    .bind(i64::try_from(MAX_SCOPE_RESPONSE_BYTES).map_err(|_| generic_denial())?)
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    if completed.rows_affected() != 1 {
        sqlx::query(
            "UPDATE chat_host_tool_invocations
             SET denial_category = 'budget_exhausted'
             WHERE id = ? AND authorization_revision_id = ?
               AND decision_state = 'denied' AND denial_category = 'pending'",
        )
        .bind(invocation_id)
        .bind(scope.authorization_revision_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        transaction.commit().await.map_err(persistence_error)?;
        return Err(generic_denial());
    }
    for (ordinal, (revision_id, content_sha256)) in returned_revisions.iter().enumerate() {
        sqlx::query(
            "INSERT INTO chat_host_tool_returned_message_revisions
                (invocation_id, message_revision_id, content_sha256, ordinal)
             VALUES (?, ?, ?, ?)",
        )
        .bind(invocation_id)
        .bind(revision_id)
        .bind(content_sha256)
        .bind(i64::try_from(ordinal).map_err(|_| generic_denial())?)
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
    }
    if let Some(scratch_generation_id) = scope.scratch_generation_id.as_deref() {
        for source in queried_channel_sources {
            let recorded = sqlx::query(
                "INSERT INTO chat_scratch_generation_sources
                (scratch_generation_id, conversation_id, lower_ordinal,
                 high_ordinal, audience_revision, created_at)
             SELECT ?, ?, ?, ?, audience.revision, ?
             FROM chat_scratch_generations generation
             JOIN chat_conversation_audience_state audience
               ON audience.conversation_id = ?
             WHERE generation.id = ?
               AND generation.lifecycle_state = 'active'
               AND generation.removed_at IS NULL
             ON CONFLICT(scratch_generation_id, conversation_id) DO UPDATE SET
               lower_ordinal = min(lower_ordinal, excluded.lower_ordinal),
               high_ordinal = max(high_ordinal, excluded.high_ordinal),
               audience_revision = excluded.audience_revision",
            )
            .bind(scratch_generation_id)
            .bind(&source.conversation_id)
            .bind(i64::try_from(source.lower_ordinal).map_err(|_| generic_denial())?)
            .bind(i64::try_from(source.high_ordinal).map_err(|_| generic_denial())?)
            .bind(&created_at)
            .bind(scope.destination_conversation_id.as_str())
            .bind(scratch_generation_id)
            .execute(&mut *transaction)
            .await
            .map_err(persistence_error)?;
            if recorded.rows_affected() != 1 {
                return Err(generic_denial());
            }
        }
    }
    transaction.commit().await.map_err(persistence_error)
}

async fn complete_audit_denied(
    pool: &SqlitePool,
    scope: &InternalMcpRunScope,
    invocation_id: &str,
    denial_category: &str,
    preserve_response_reservation: bool,
) -> ChatResult<()> {
    let updated = sqlx::query(
        "UPDATE chat_host_tool_invocations
         SET denial_category = ?,
             response_bytes = CASE WHEN ? THEN response_bytes ELSE 0 END
         WHERE id = ? AND authorization_revision_id = ?
           AND decision_state = 'denied' AND denial_category = 'pending'",
    )
    .bind(denial_category)
    .bind(preserve_response_reservation)
    .bind(invocation_id)
    .bind(scope.authorization_revision_id.as_str())
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    if updated.rows_affected() == 1 {
        Ok(())
    } else {
        Err(generic_denial())
    }
}

fn host_tool_is_mutating(name: &str) -> bool {
    matches!(
        name,
        "chat_write_workspace_file"
            | "chat_create_workspace_file"
            | "chat_patch_workspace_file"
            | "chat_delete_workspace_file"
    )
}

fn returned_message_revisions(value: &Value) -> ChatResult<Vec<(String, String)>> {
    let Some(messages) = value.get("messages") else {
        return Ok(Vec::new());
    };
    messages
        .as_array()
        .ok_or_else(generic_denial)?
        .iter()
        .map(|message| {
            let revision_id = message
                .get("revisionId")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty() && value.len() <= 1024)
                .ok_or_else(generic_denial)?;
            let content_hash = message
                .get("contentHash")
                .and_then(Value::as_str)
                .filter(|value| value.len() == 64)
                .ok_or_else(generic_denial)?;
            Ok((revision_id.to_string(), content_hash.to_string()))
        })
        .collect()
}

fn denial_category(error: &ChatError) -> &'static str {
    match error.code {
        ChatErrorCode::Conflict | ChatErrorCode::StaleRevision => "stale_scope",
        ChatErrorCode::Validation => "invalid_request",
        ChatErrorCode::NotFound => "unavailable_resource",
        ChatErrorCode::Permission => "permission_denied",
        _ => "runtime_unavailable",
    }
}

fn generic_denial() -> ChatError {
    ChatError::new(ChatErrorCode::Permission, GENERIC_DENIAL, false)
}

fn internal_error(operation: &str) -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        format!("Could not {operation}"),
        true,
    )
}

fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Organizational host-tool authorization could not be verified",
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_search_terms_are_bounded_and_like_escaped() {
        let terms = normalized_query_terms("Release 100% _Ready_").expect("terms");
        assert_eq!(terms, vec!["release", "100%", "_ready_"]);
        assert_eq!(escape_like("100%_ready\\now"), "100\\%\\_ready\\\\now");
        assert!(normalized_query_terms("").is_err());
        let too_many_terms = (0..=MAX_QUERY_TERMS)
            .map(|index| format!("term{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        assert!(normalized_query_terms(&too_many_terms).is_err());
    }

    #[test]
    fn workspace_patch_edits_require_ordered_utf8_byte_ranges() {
        let arguments = json!({
            "edits": [
                { "startByte": 1, "endByte": 3, "replacement": "o" },
                { "startByte": 3, "endByte": 3, "replacement": "!" }
            ]
        });
        let edits = workspace_patch_edits(arguments.as_object().unwrap(), "aéz")
            .expect("valid UTF-8 patch");
        let mut patched = "aéz".to_string();
        for edit in edits.iter().rev() {
            patched.replace_range(edit.start_byte..edit.end_byte, &edit.replacement);
        }
        assert_eq!(patched, "ao!z");

        let split_codepoint = json!({
            "edits": [{ "startByte": 2, "endByte": 3, "replacement": "x" }]
        });
        assert!(workspace_patch_edits(split_codepoint.as_object().unwrap(), "aéz").is_err());
        let overlapping = json!({
            "edits": [
                { "startByte": 0, "endByte": 2, "replacement": "x" },
                { "startByte": 1, "endByte": 3, "replacement": "y" }
            ]
        });
        assert!(workspace_patch_edits(overlapping.as_object().unwrap(), "abcd").is_err());
    }

    #[test]
    fn workspace_mutations_require_a_resolved_noninteractive_approval() {
        assert!(require_resolved_mutation_approval(ChatRuntimeApprovalPolicy::AutoApprove).is_ok());
        assert!(require_resolved_mutation_approval(ChatRuntimeApprovalPolicy::Unattended).is_ok());

        let ask = require_resolved_mutation_approval(ChatRuntimeApprovalPolicy::Ask)
            .expect_err("ask must remain fail closed without a durable host-tool approval");
        assert_eq!(ask.code, ChatErrorCode::Permission);

        let provider_custom =
            require_resolved_mutation_approval(ChatRuntimeApprovalPolicy::ProviderCustom)
                .expect_err("provider custom cannot widen a brokered folder grant");
        assert_eq!(provider_custom.code, ChatErrorCode::Permission);
    }

    #[tokio::test]
    async fn activating_a_scope_discards_opaque_cursors() {
        let runtime = InternalMcpToolRuntime::default();
        let cursor = runtime
            .store_cursor(OpaqueCursor::Channel {
                source_handle: "source".to_string(),
                query_hash: "query".to_string(),
                before_created_at: "2026-08-17T00:00:00.000Z".to_string(),
                before_item_id: "item".to_string(),
            })
            .await
            .expect("cursor");
        runtime
            .channel_cursor(&cursor, "source", "query")
            .await
            .expect("active cursor");
        runtime.reset_cursors().await;
        assert!(runtime
            .channel_cursor(&cursor, "source", "query")
            .await
            .is_err());
    }
}
