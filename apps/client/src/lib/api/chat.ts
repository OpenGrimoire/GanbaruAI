import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";
import type {
  ChatBehaviorPreferences,
  ChatDiagnosticPreferences,
  ChatDiagnosticsRead,
  ChatStopAllResult,
  ChatRebuildResult,
  ChatDraftRead,
  ChatAttachmentRead,
  ChatInteractionStateRead,
  ChatUserInputDraftRead,
  ChatPromptCatalogEntry,
  ChatQueuedFollowupRead,
  ChatWorkspacePathPage,
  ChatWorkspaceDirectoryRead,
  ChatWorkspaceFilePreview,
  ChatTerminalCloseResult,
  ChatTerminalContextRead,
  ChatTerminalRead,
  ChatTerminalSnapshotRead,
  ChatCheckpointDiffRead,
  ChatCheckpointFileDiffRead,
  ChatRestorePreviewRead,
  ChatRestoreResultRead,
  ChatPanelPreferences,
  ChatProjectShellRead,
  ChatSettingsRead,
  ChatThreadId,
  ChatThreadShellRead,
  ChatTimelinePageRead,
  ChatWorkspaceId,
  ChatWorkspaceRead,
  CreateChatWorkspaceRequest,
  CredentialReferenceId,
  ModelId,
  ProviderInstanceConfig,
  ProviderInstanceId,
  ProviderInstanceRead,
  ProviderModelCatalog,
  ProviderProbeResult,
  ProviderSetupTestRead,
  RemoveProviderResult,
  RememberedComposerSelection,
  SaveChatDraftRequest,
  SaveQueuedFollowupRequest,
  SendChatTurnCommand,
  SendChatTurnResult,
  SteerChatTurnCommand,
  ResolveChatApprovalCommand,
  ResolveChatUserInputCommand,
  VersionedJson,
} from "$lib/chat/contracts";
import {
  parseChatProjectShells,
  parseChatDiagnosticPreferences,
  parseChatDiagnosticsRead,
  parseChatStopAllResult,
  parseChatRebuildResult,
  parseChatDraftRead,
  parseChatAttachmentRead,
  parseChatInteractionState,
  parseChatUserInputDraft,
  parseChatPromptCatalog,
  parseChatQueuedFollowup,
  parseChatWorkspacePathPage,
  parseChatWorkspaceDirectory,
  parseChatWorkspaceFilePreview,
  parseChatTerminalCloseResult,
  parseChatTerminalContext,
  parseChatTerminal,
  parseChatTerminalSnapshot,
  parseChatTerminals,
  parseChatCheckpointDiff,
  parseChatCheckpointFileDiff,
  parseChatRestorePreview,
  parseChatRestoreResult,
  parseTurnDispatchReceipt,
  parseChatError,
  parseChatSettingsRead,
  parseChatThreadShell,
  parseChatThreadShells,
  parseChatTimelinePage,
  parseChatVaultConfig,
  parseChatWorkspaceRead,
  parseChatWorkspaceReads,
  parseProviderInstanceRead,
  parseProviderModelCatalog,
  parseProviderProbeResult,
  parseProviderSetupTestRead,
  parseRemoveProviderResult,
} from "$lib/chat/validation";

export async function listChatWorkspaces(): Promise<ChatWorkspaceRead[]> {
  return parseChatWorkspaceReads(await invoke<unknown>("chat_list_workspaces", { dbUrl: await ensureDbUrl() }));
}

export async function createChatWorkspace(
  request: CreateChatWorkspaceRequest,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_create_workspace", { dbUrl: await ensureDbUrl(), request }));
}

export async function renameChatWorkspace(
  workspaceId: ChatWorkspaceId,
  displayName: string,
  expectedRevision: number,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_rename_workspace", {
    dbUrl: await ensureDbUrl(),
    workspaceId,
    displayName,
    expectedRevision,
  }));
}

export async function bindChatWorkspace(
  workspaceId: ChatWorkspaceId,
  title: string,
): Promise<ChatWorkspaceRead | null> {
  const value = await invoke<unknown>("chat_bind_workspace", { dbUrl: await ensureDbUrl(), workspaceId, title });
  return value === null ? null : parseChatWorkspaceRead(value);
}

export async function rebindChatWorkspace(
  workspaceId: ChatWorkspaceId,
  title: string,
): Promise<ChatWorkspaceRead | null> {
  const value = await invoke<unknown>("chat_rebind_workspace", { dbUrl: await ensureDbUrl(), workspaceId, title });
  return value === null ? null : parseChatWorkspaceRead(value);
}

export async function removeChatWorkspaceBinding(
  workspaceId: ChatWorkspaceId,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_remove_workspace_binding", { dbUrl: await ensureDbUrl(), workspaceId }));
}

export async function archiveChatWorkspace(
  workspaceId: ChatWorkspaceId,
  expectedRevision: number,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_archive_workspace", { dbUrl: await ensureDbUrl(), workspaceId, expectedRevision }));
}

export async function restoreChatWorkspace(
  workspaceId: ChatWorkspaceId,
  expectedRevision: number,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_restore_workspace", { dbUrl: await ensureDbUrl(), workspaceId, expectedRevision }));
}

export async function openChatWorkspaceFolder(workspaceId: ChatWorkspaceId): Promise<void> {
  await invoke("chat_open_workspace_folder", { dbUrl: await ensureDbUrl(), workspaceId });
}

export async function listChatWorkspaceDirectory(
  workspaceId: ChatWorkspaceId,
  relativePath: string,
  includeIgnored = false,
): Promise<ChatWorkspaceDirectoryRead> {
  return parseChatWorkspaceDirectory(await invoke<unknown>("chat_list_workspace_directory", {
    dbUrl: await ensureDbUrl(),
    workspaceId,
    relativePath,
    includeIgnored,
  }));
}

export async function previewChatWorkspaceFile(
  workspaceId: ChatWorkspaceId,
  relativePath: string,
): Promise<ChatWorkspaceFilePreview> {
  return parseChatWorkspaceFilePreview(await invoke<unknown>("chat_preview_workspace_file", {
    dbUrl: await ensureDbUrl(),
    workspaceId,
    relativePath,
  }));
}

export async function openChatWorkspaceFile(
  workspaceId: ChatWorkspaceId,
  relativePath: string,
): Promise<void> {
  await invoke("chat_open_workspace_file", { dbUrl: await ensureDbUrl(), workspaceId, relativePath });
}

export async function listChatTerminals(
  threadId: ChatThreadId,
  workspaceId: ChatWorkspaceId,
): Promise<ChatTerminalRead[]> {
  return parseChatTerminals(await invoke<unknown>("chat_list_terminals", {
    dbUrl: await ensureDbUrl(),
    threadId,
    workspaceId,
  }));
}

export async function createChatTerminal(request: {
  terminalId: string;
  threadId: ChatThreadId;
  workspaceId: ChatWorkspaceId;
  columns: number;
  rows: number;
}): Promise<ChatTerminalSnapshotRead> {
  return parseChatTerminalSnapshot(await invoke<unknown>("chat_terminal_create", {
    dbUrl: await ensureDbUrl(),
    request,
  }));
}

export async function readChatTerminalSnapshot(
  terminalId: string,
  threadId: ChatThreadId,
  workspaceId: ChatWorkspaceId,
): Promise<ChatTerminalSnapshotRead> {
  return parseChatTerminalSnapshot(await invoke<unknown>("chat_terminal_snapshot", {
    dbUrl: await ensureDbUrl(),
    terminalId,
    threadId,
    workspaceId,
  }));
}

export async function writeChatTerminal(
  terminalId: string,
  threadId: ChatThreadId,
  workspaceId: ChatWorkspaceId,
  text: string,
): Promise<void> {
  await invoke("chat_terminal_input", {
    dbUrl: await ensureDbUrl(),
    request: { terminalId, threadId, workspaceId, text },
  });
}

export async function resizeChatTerminal(
  terminalId: string,
  threadId: ChatThreadId,
  workspaceId: ChatWorkspaceId,
  columns: number,
  rows: number,
): Promise<void> {
  await invoke("chat_terminal_resize", {
    dbUrl: await ensureDbUrl(),
    request: { terminalId, threadId, workspaceId, columns, rows },
  });
}

export async function closeChatTerminal(
  terminalId: string,
  threadId: ChatThreadId,
  workspaceId: ChatWorkspaceId,
  confirmed: boolean,
): Promise<ChatTerminalCloseResult> {
  return parseChatTerminalCloseResult(await invoke<unknown>("chat_terminal_close", {
    dbUrl: await ensureDbUrl(),
    terminalId,
    threadId,
    workspaceId,
    confirmed,
  }));
}

export async function importChatTerminalContext(request: {
  terminalId: string;
  threadId: ChatThreadId;
  workspaceId: ChatWorkspaceId;
  attachmentId: string;
  sourceKind: "selection" | "last_command_output";
  text: string;
  startOutputSequence: number | null;
  endOutputSequence: number | null;
  truncated: boolean;
}): Promise<ChatTerminalContextRead> {
  return parseChatTerminalContext(await invoke<unknown>("chat_terminal_import_context", {
    dbUrl: await ensureDbUrl(),
    request,
  }));
}

export async function readChatCheckpointDiff(
  threadId: ChatThreadId,
  scope: "current_turn" | "entire_thread",
  turnId: string | null = null,
): Promise<ChatCheckpointDiffRead> {
  return parseChatCheckpointDiff(await invoke<unknown>("chat_read_checkpoint_diff", {
    dbUrl: await ensureDbUrl(),
    threadId,
    scope,
    turnId,
  }));
}

export async function readChatCheckpointFileDiff(
  threadId: ChatThreadId,
  preCheckpointId: string,
  postCheckpointId: string,
  relativePath: string,
  ignoreWhitespace: boolean,
): Promise<ChatCheckpointFileDiffRead> {
  return parseChatCheckpointFileDiff(await invoke<unknown>("chat_read_checkpoint_file_diff", {
    dbUrl: await ensureDbUrl(),
    threadId,
    preCheckpointId,
    postCheckpointId,
    relativePath,
    ignoreWhitespace,
  }));
}

export async function previewChatCheckpointRestore(
  threadId: ChatThreadId,
  checkpointId: string,
): Promise<ChatRestorePreviewRead> {
  return parseChatRestorePreview(await invoke<unknown>("chat_preview_checkpoint_restore", {
    dbUrl: await ensureDbUrl(),
    request: { threadId, checkpointId },
  }));
}

export async function executeChatCheckpointRestore(request: {
  command: { clientCommandId: string; expectedThreadRevision: number | null };
  threadId: ChatThreadId;
  previewId: string;
  confirmed: boolean;
}): Promise<ChatRestoreResultRead> {
  return parseChatRestoreResult(await invoke<unknown>("chat_execute_checkpoint_restore", {
    dbUrl: await ensureDbUrl(),
    request,
  }));
}

export async function readChatSettings(): Promise<ChatSettingsRead> {
  return parseChatSettingsRead(await invoke<unknown>("chat_read_settings"));
}

export async function readChatDiagnostics(): Promise<ChatDiagnosticsRead> {
  return parseChatDiagnosticsRead(await invoke<unknown>("chat_read_diagnostics", { dbUrl: await ensureDbUrl() }));
}

export async function updateChatDiagnosticPreferences(preferences: ChatDiagnosticPreferences): Promise<ChatDiagnosticPreferences> {
  return parseChatDiagnosticPreferences(await invoke<unknown>("chat_update_diagnostic_preferences", { preferences }));
}

export async function deleteChatDiagnostics(): Promise<number> {
  return invoke<number>("chat_delete_diagnostics", { dbUrl: await ensureDbUrl() });
}

export async function retryChatCheckpointCleanup(): Promise<number> {
  return invoke<number>("chat_run_checkpoint_cleanup", { dbUrl: await ensureDbUrl() });
}

export async function exportRedactedChatDiagnostics(pickerTitle: string): Promise<boolean> {
  return invoke<boolean>("chat_export_redacted_diagnostics", { dbUrl: await ensureDbUrl(), pickerTitle });
}

export async function stopAllChatProcesses(confirmation: string): Promise<ChatStopAllResult> {
  return parseChatStopAllResult(await invoke<unknown>("chat_stop_all_processes", { request: { confirmation } }));
}

export async function rebuildChatProjections(confirmation: string): Promise<ChatRebuildResult> {
  return parseChatRebuildResult(await invoke<unknown>("chat_rebuild_projections", {
    dbUrl: await ensureDbUrl(), request: { confirmation },
  }));
}

export async function setLastSelectedChatThread(threadId: ChatThreadId | null): Promise<void> {
  await invoke("chat_set_last_selected_thread", { threadId });
}

export async function saveChatProvider(configuration: ProviderInstanceConfig): Promise<ProviderInstanceRead> {
  return parseProviderInstanceRead(await invoke<unknown>("chat_save_provider", {
    request: { configuration },
  }));
}

export async function setChatProviderEnabled(instanceId: ProviderInstanceId, enabled: boolean): Promise<ProviderInstanceRead> {
  return parseProviderInstanceRead(await invoke<unknown>("chat_set_provider_enabled", { instanceId, enabled }));
}

export async function removeChatProvider(instanceId: ProviderInstanceId): Promise<RemoveProviderResult> {
  return parseRemoveProviderResult(await invoke<unknown>("chat_remove_provider", { instanceId }));
}

export async function testChatProvider(configuration: ProviderInstanceConfig): Promise<ProviderSetupTestRead> {
  return parseProviderSetupTestRead(await invoke<unknown>("chat_test_provider", {
    request: { configuration },
  }));
}

export async function probeChatProvider(instanceId: ProviderInstanceId): Promise<ProviderProbeResult> {
  return parseProviderProbeResult(await invoke<unknown>("chat_probe_provider", { instanceId }));
}

export async function refreshChatProviderModels(instanceId: ProviderInstanceId): Promise<ProviderModelCatalog> {
  return parseProviderModelCatalog(await invoke<unknown>("chat_refresh_provider_models", { instanceId }));
}

export async function updateChatProviderModels(
  instanceId: ProviderInstanceId,
  visibleModelIds: ModelId[],
  favoriteModelIds: ModelId[],
): Promise<ProviderInstanceRead> {
  return parseProviderInstanceRead(await invoke<unknown>("chat_update_provider_models", {
    instanceId,
    visibleModelIds,
    favoriteModelIds,
  }));
}

export async function updateChatBehavior(behavior: ChatBehaviorPreferences): Promise<ChatSettingsRead["configuration"]> {
  return parseChatVaultConfig(await invoke<unknown>("chat_update_behavior", { behavior }));
}

export async function updateChatPanels(panels: ChatPanelPreferences): Promise<ChatSettingsRead["configuration"]> {
  return parseChatVaultConfig(await invoke<unknown>("chat_update_panels", { panels }));
}

export async function setChatWorkspaceProviderPreference(
  workspaceId: ChatWorkspaceId,
  instanceId: ProviderInstanceId | null,
): Promise<ChatSettingsRead["configuration"]> {
  return parseChatVaultConfig(await invoke<unknown>("chat_set_workspace_provider_preference", { workspaceId, instanceId }));
}

export async function rememberChatComposerSelection(selection: RememberedComposerSelection): Promise<ChatSettingsRead["configuration"]> {
  return parseChatVaultConfig(await invoke<unknown>("chat_remember_composer_selection", { selection }));
}

export async function replaceChatCredential(referenceId: CredentialReferenceId, secret: string): Promise<void> {
  await invoke("chat_replace_credential", { referenceId, secret });
}

export async function removeChatCredential(referenceId: CredentialReferenceId): Promise<boolean> {
  return invoke<boolean>("chat_remove_credential", { referenceId });
}

export async function pickChatProviderExecutable(title: string): Promise<string | null> {
  return invoke<string | null>("chat_pick_provider_executable", { title });
}

export async function pickChatProviderHome(title: string): Promise<string | null> {
  return invoke<string | null>("chat_pick_provider_home", { title });
}

export async function listChatProjectShells(): Promise<ChatProjectShellRead[]> {
  return parseChatProjectShells(await invoke<unknown>("chat_list_project_shells", { dbUrl: await ensureDbUrl() }));
}

export async function listChatThreads(
  workspaceId: ChatWorkspaceId | null,
  archived: boolean,
): Promise<ChatThreadShellRead[]> {
  return parseChatThreadShells(await invoke<unknown>("chat_list_threads", {
    dbUrl: await ensureDbUrl(),
    workspaceId,
    archived,
  }));
}

export async function searchChatThreadTitles(query: string, archived: boolean | null, limit = 100): Promise<ChatThreadShellRead[]> {
  return parseChatThreadShells(await invoke<unknown>("chat_search_thread_titles", {
    dbUrl: await ensureDbUrl(),
    query,
    archived,
    limit,
  }));
}

export async function readChatTimelinePage(
  threadId: ChatThreadId,
  cursor: string | null = null,
  limit = 100,
): Promise<ChatTimelinePageRead> {
  return parseChatTimelinePage(await invoke<unknown>("chat_read_timeline_page", {
    dbUrl: await ensureDbUrl(),
    threadId,
    cursor,
    limit,
  }));
}

export async function openChatExternalUrl(url: string): Promise<void> {
  await invoke("chat_open_external_url", { url });
}

export async function readChatDraft(draftId: string): Promise<ChatDraftRead | null> {
  const value = await invoke<unknown>("chat_read_draft", { dbUrl: await ensureDbUrl(), draftId });
  return value === null ? null : parseChatDraftRead(value);
}

export async function saveChatDraft(draft: SaveChatDraftRequest): Promise<ChatDraftRead> {
  return parseChatDraftRead(await invoke<unknown>("chat_save_draft", {
    dbUrl: await ensureDbUrl(),
    draft,
  }));
}

export async function deleteChatDraft(draftId: string): Promise<boolean> {
  return invoke<boolean>("chat_delete_draft", { dbUrl: await ensureDbUrl(), draftId });
}

export async function importChatImage(
  workspaceId: ChatWorkspaceId,
  attachmentId: string,
  displayName: string,
  bytes: number[],
): Promise<ChatAttachmentRead> {
  return parseChatAttachmentRead(await invoke<unknown>("chat_import_image", {
    dbUrl: await ensureDbUrl(),
    request: { workspaceId, attachmentId, displayName, bytes },
  }));
}

export async function importChatTextSnippet(
  workspaceId: ChatWorkspaceId,
  attachmentId: string,
  displayName: string,
  text: string,
): Promise<ChatAttachmentRead> {
  return parseChatAttachmentRead(await invoke<unknown>("chat_import_text_snippet", {
    dbUrl: await ensureDbUrl(),
    request: { workspaceId, attachmentId, displayName, text },
  }));
}

export async function pickChatImages(
  workspaceId: ChatWorkspaceId,
  attachmentIds: string[],
  title: string,
): Promise<ChatAttachmentRead[]> {
  const value = await invoke<unknown>("chat_pick_images", {
    dbUrl: await ensureDbUrl(),
    request: { workspaceId, attachmentIds, title },
  });
  if (!Array.isArray(value)) throw new Error("Chat image picker response must be an array");
  return value.map((entry, index) => parseChatAttachmentRead(entry, `Chat image picker response[${index}]`));
}

export async function chatAttachmentDataUrl(attachmentId: string): Promise<string> {
  return invoke<string>("chat_attachment_data_url", { dbUrl: await ensureDbUrl(), attachmentId });
}

export async function readChatAttachments(
  workspaceId: ChatWorkspaceId,
  attachmentIds: string[],
): Promise<ChatAttachmentRead[]> {
  const value = await invoke<unknown>("chat_read_attachments", {
    dbUrl: await ensureDbUrl(), workspaceId, attachmentIds,
  });
  if (!Array.isArray(value)) throw new Error("Chat attachments response must be an array");
  return value.map((entry, index) => parseChatAttachmentRead(entry, `Chat attachments response[${index}]`));
}

export async function searchChatWorkspacePaths(
  workspaceId: ChatWorkspaceId,
  query: string,
  includeIgnored: boolean,
  cursor: string | null = null,
  limit = 50,
): Promise<ChatWorkspacePathPage> {
  return parseChatWorkspacePathPage(await invoke<unknown>("chat_search_workspace_paths", {
    dbUrl: await ensureDbUrl(), workspaceId, query, includeIgnored, cursor, limit,
  }));
}

export async function validateChatWorkspaceMentions(
  workspaceId: ChatWorkspaceId,
  relativePaths: string[],
): Promise<void> {
  await invoke("chat_validate_workspace_mentions", {
    dbUrl: await ensureDbUrl(), workspaceId, relativePaths,
  });
}

export async function listChatPromptCatalog(providerInstanceId: ProviderInstanceId): Promise<ChatPromptCatalogEntry[]> {
  return parseChatPromptCatalog(await invoke<unknown>("chat_list_prompt_catalog", { providerInstanceId }));
}

export async function readChatInteractionState(threadId: ChatThreadId): Promise<ChatInteractionStateRead> {
  return parseChatInteractionState(await invoke<unknown>("chat_read_interaction_state", {
    dbUrl: await ensureDbUrl(), threadId,
  }));
}

export async function setChatFullAccessTrust(
  providerInstanceId: ProviderInstanceId,
  workspaceId: ChatWorkspaceId,
  trusted: boolean,
): Promise<boolean> {
  return invoke<boolean>("chat_set_full_access_trust", { providerInstanceId, workspaceId, trusted });
}

export async function hasChatFullAccessTrust(
  providerInstanceId: ProviderInstanceId,
  workspaceId: ChatWorkspaceId,
): Promise<boolean> {
  return invoke<boolean>("chat_has_full_access_trust", { providerInstanceId, workspaceId });
}

export async function saveChatQueuedFollowup(request: SaveQueuedFollowupRequest): Promise<ChatQueuedFollowupRead> {
  return parseChatQueuedFollowup(await invoke<unknown>("chat_save_queued_followup", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function cancelChatQueuedFollowup(threadId: ChatThreadId): Promise<boolean> {
  return invoke<boolean>("chat_cancel_queued_followup", { dbUrl: await ensureDbUrl(), threadId });
}

export async function markChatQueuedFollowupDispatched(threadId: ChatThreadId, queuedFollowupId: string): Promise<boolean> {
  return invoke<boolean>("chat_mark_queued_followup_dispatched", {
    dbUrl: await ensureDbUrl(), threadId, queuedFollowupId,
  });
}

export async function readChatUserInputDraft(requestId: string): Promise<ChatUserInputDraftRead | null> {
  const value = await invoke<unknown>("chat_read_user_input_draft", {
    dbUrl: await ensureDbUrl(), requestId,
  });
  return value === null ? null : parseChatUserInputDraft(value);
}

export async function saveChatUserInputDraft(requestId: string, answers: VersionedJson): Promise<ChatUserInputDraftRead> {
  return parseChatUserInputDraft(await invoke<unknown>("chat_save_user_input_draft", {
    dbUrl: await ensureDbUrl(), requestId, answers,
  }));
}

export async function stopChatSession(threadId: ChatThreadId, force: boolean): Promise<void> {
  await invoke("chat_stop_session", {
    dbUrl: await ensureDbUrl(), threadId, force, clientCommandId: crypto.randomUUID(),
  });
}

export async function sendChatTurn(request: SendChatTurnCommand): Promise<SendChatTurnResult> {
  const value = await invoke<unknown>("chat_send_turn", { dbUrl: await ensureDbUrl(), request });
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error("Chat send response must be an object");
  const record = value as Record<string, unknown>;
  return {
    thread: parseChatThreadShell(record.thread),
    dispatch: record.dispatch === null ? null : parseTurnDispatchReceipt(record.dispatch, "Chat send response.dispatch"),
    launchError: record.launchError === null ? null : parseChatError(record.launchError),
  };
}

export async function steerChatTurn(request: SteerChatTurnCommand): Promise<void> {
  await invoke("chat_steer_turn", { dbUrl: await ensureDbUrl(), request });
}

export async function resolveChatApproval(request: ResolveChatApprovalCommand): Promise<void> {
  await invoke("chat_resolve_approval", { dbUrl: await ensureDbUrl(), request });
}

export async function resolveChatUserInput(request: ResolveChatUserInputCommand): Promise<void> {
  await invoke("chat_resolve_user_input", { dbUrl: await ensureDbUrl(), request });
}

async function threadMutation(command: string, args: Record<string, unknown>): Promise<ChatThreadShellRead> {
  return parseChatThreadShell(await invoke<unknown>(command, { dbUrl: await ensureDbUrl(), ...args }));
}

export function renameChatThread(threadId: ChatThreadId, title: string, expectedRevision: number): Promise<ChatThreadShellRead> {
  return threadMutation("chat_rename_thread", { threadId, title, expectedRevision });
}

export function setChatThreadRead(threadId: ChatThreadId, read: boolean, expectedRevision: number): Promise<ChatThreadShellRead> {
  return threadMutation("chat_set_thread_read", { threadId, read, expectedRevision });
}

export function archiveChatThread(threadId: ChatThreadId, expectedRevision: number): Promise<ChatThreadShellRead> {
  return threadMutation("chat_archive_thread", { threadId, expectedRevision });
}

export function restoreChatThread(threadId: ChatThreadId, expectedRevision: number): Promise<ChatThreadShellRead> {
  return threadMutation("chat_restore_thread", { threadId, expectedRevision });
}

export async function deleteChatThreadPermanently(
  threadId: ChatThreadId,
  expectedRevision: number,
  confirmedTitle: string,
): Promise<void> {
  await invoke("chat_delete_thread_permanently", {
    dbUrl: await ensureDbUrl(),
    threadId,
    expectedRevision,
    confirmedTitle,
  });
}
