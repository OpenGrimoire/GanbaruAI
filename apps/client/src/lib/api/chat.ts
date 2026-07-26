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
  ProjectWorkingFolderPathPage,
  ProjectWorkingFolderDirectoryRead,
  ProjectWorkingFolderFilePreview,
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
  ProjectWorkingFolderId,
  CredentialReferenceId,
  ModelId,
  ProviderInstanceConfig,
  ProviderInstanceId,
  ProviderInstanceRead,
  ProviderModelCatalog,
  ProviderProbeResult,
  ProviderRefreshResult,
  ProviderFileRead,
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
  parseProjectWorkingFolderPathPage,
  parseProjectWorkingFolderDirectory,
  parseProjectWorkingFolderFilePreview,
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
  parseProviderInstanceRead,
  parseProviderModelCatalog,
  parseProviderProbeResult,
  parseProviderRefreshResult,
  parseProviderFileRead,
  parseProviderFiles,
  parseProviderSetupTestRead,
  parseRemoveProviderResult,
} from "$lib/chat/validation";

export async function listProjectWorkingFolderDirectory(
  workingFolderId: ProjectWorkingFolderId,
  relativePath: string,
  includeIgnored = false,
): Promise<ProjectWorkingFolderDirectoryRead> {
  return parseProjectWorkingFolderDirectory(await invoke<unknown>("project_list_working_folder_directory", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    relativePath,
    includeIgnored,
  }));
}

export async function previewProjectWorkingFolderFile(
  workingFolderId: ProjectWorkingFolderId,
  relativePath: string,
): Promise<ProjectWorkingFolderFilePreview> {
  return parseProjectWorkingFolderFilePreview(await invoke<unknown>("project_preview_working_folder_file", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    relativePath,
  }));
}

export async function openProjectWorkingFolderFile(
  workingFolderId: ProjectWorkingFolderId,
  relativePath: string,
): Promise<void> {
  await invoke("project_open_working_folder_file", { dbUrl: await ensureDbUrl(), workingFolderId, relativePath });
}

export async function listChatTerminals(
  threadId: ChatThreadId,
  workingFolderId: ProjectWorkingFolderId,
): Promise<ChatTerminalRead[]> {
  return parseChatTerminals(await invoke<unknown>("chat_list_terminals", {
    dbUrl: await ensureDbUrl(),
    threadId,
    workingFolderId,
  }));
}

export async function createChatTerminal(request: {
  terminalId: string;
  threadId: ChatThreadId;
  workingFolderId: ProjectWorkingFolderId;
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
  workingFolderId: ProjectWorkingFolderId,
): Promise<ChatTerminalSnapshotRead> {
  return parseChatTerminalSnapshot(await invoke<unknown>("chat_terminal_snapshot", {
    dbUrl: await ensureDbUrl(),
    terminalId,
    threadId,
    workingFolderId,
  }));
}

export async function writeChatTerminal(
  terminalId: string,
  threadId: ChatThreadId,
  workingFolderId: ProjectWorkingFolderId,
  text: string,
): Promise<void> {
  await invoke("chat_terminal_input", {
    dbUrl: await ensureDbUrl(),
    request: { terminalId, threadId, workingFolderId, text },
  });
}

export async function resizeChatTerminal(
  terminalId: string,
  threadId: ChatThreadId,
  workingFolderId: ProjectWorkingFolderId,
  columns: number,
  rows: number,
): Promise<void> {
  await invoke("chat_terminal_resize", {
    dbUrl: await ensureDbUrl(),
    request: { terminalId, threadId, workingFolderId, columns, rows },
  });
}

export async function closeChatTerminal(
  terminalId: string,
  threadId: ChatThreadId,
  workingFolderId: ProjectWorkingFolderId,
  confirmed: boolean,
): Promise<ChatTerminalCloseResult> {
  return parseChatTerminalCloseResult(await invoke<unknown>("chat_terminal_close", {
    dbUrl: await ensureDbUrl(),
    terminalId,
    threadId,
    workingFolderId,
    confirmed,
  }));
}

export async function importChatTerminalContext(request: {
  terminalId: string;
  threadId: ChatThreadId;
  workingFolderId: ProjectWorkingFolderId;
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

export async function refreshAllChatProviders(): Promise<ProviderRefreshResult> {
  return parseProviderRefreshResult(await invoke<unknown>("chat_refresh_all_providers"));
}

export async function readChatProviderFiles(instanceId: ProviderInstanceId): Promise<ProviderFileRead[]> {
  return parseProviderFiles(await invoke<unknown>("chat_read_provider_files", { instanceId }));
}

export async function saveChatProviderFile(request: {
  instanceId: ProviderInstanceId;
  fileId: string;
  contents: string;
  expectedRevision: string;
}): Promise<ProviderFileRead> {
  return parseProviderFileRead(await invoke<unknown>("chat_save_provider_file", { request }));
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

export async function setChatWorkingFolderProviderPreference(
  workingFolderId: ProjectWorkingFolderId,
  instanceId: ProviderInstanceId | null,
): Promise<ChatSettingsRead["configuration"]> {
  return parseChatVaultConfig(await invoke<unknown>("chat_set_working_folder_provider_preference", { workingFolderId, instanceId }));
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
  workingFolderId: ProjectWorkingFolderId | null,
  archived: boolean,
): Promise<ChatThreadShellRead[]> {
  return parseChatThreadShells(await invoke<unknown>("chat_list_threads", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
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
  workingFolderId: ProjectWorkingFolderId,
  attachmentId: string,
  displayName: string,
  bytes: number[],
): Promise<ChatAttachmentRead> {
  return parseChatAttachmentRead(await invoke<unknown>("chat_import_image", {
    dbUrl: await ensureDbUrl(),
    request: { workingFolderId, attachmentId, displayName, bytes },
  }));
}

export async function importChatTextSnippet(
  workingFolderId: ProjectWorkingFolderId,
  attachmentId: string,
  displayName: string,
  text: string,
): Promise<ChatAttachmentRead> {
  return parseChatAttachmentRead(await invoke<unknown>("chat_import_text_snippet", {
    dbUrl: await ensureDbUrl(),
    request: { workingFolderId, attachmentId, displayName, text },
  }));
}

export async function pickChatImages(
  workingFolderId: ProjectWorkingFolderId,
  attachmentIds: string[],
  title: string,
): Promise<ChatAttachmentRead[]> {
  const value = await invoke<unknown>("chat_pick_images", {
    dbUrl: await ensureDbUrl(),
    request: { workingFolderId, attachmentIds, title },
  });
  if (!Array.isArray(value)) throw new Error("Chat image picker response must be an array");
  return value.map((entry, index) => parseChatAttachmentRead(entry, `Chat image picker response[${index}]`));
}

export async function chatAttachmentDataUrl(attachmentId: string): Promise<string> {
  return invoke<string>("chat_attachment_data_url", { dbUrl: await ensureDbUrl(), attachmentId });
}

export async function readChatAttachments(
  workingFolderId: ProjectWorkingFolderId,
  attachmentIds: string[],
): Promise<ChatAttachmentRead[]> {
  const value = await invoke<unknown>("chat_read_attachments", {
    dbUrl: await ensureDbUrl(), workingFolderId, attachmentIds,
  });
  if (!Array.isArray(value)) throw new Error("Chat attachments response must be an array");
  return value.map((entry, index) => parseChatAttachmentRead(entry, `Chat attachments response[${index}]`));
}

export async function searchChatWorkingFolderPaths(
  workingFolderId: ProjectWorkingFolderId,
  query: string,
  includeIgnored: boolean,
  cursor: string | null = null,
  limit = 50,
): Promise<ProjectWorkingFolderPathPage> {
  return parseProjectWorkingFolderPathPage(await invoke<unknown>("chat_search_working_folder_paths", {
    dbUrl: await ensureDbUrl(), workingFolderId, query, includeIgnored, cursor, limit,
  }));
}

export async function validateChatWorkingFolderMentions(
  workingFolderId: ProjectWorkingFolderId,
  relativePaths: string[],
): Promise<void> {
  await invoke("chat_validate_working_folder_mentions", {
    dbUrl: await ensureDbUrl(), workingFolderId, relativePaths,
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
  workingFolderId: ProjectWorkingFolderId,
  trusted: boolean,
): Promise<boolean> {
  return invoke<boolean>("chat_set_full_access_trust", { providerInstanceId, workingFolderId, trusted });
}

export async function hasChatFullAccessTrust(
  providerInstanceId: ProviderInstanceId,
  workingFolderId: ProjectWorkingFolderId,
): Promise<boolean> {
  return invoke<boolean>("chat_has_full_access_trust", { providerInstanceId, workingFolderId });
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
