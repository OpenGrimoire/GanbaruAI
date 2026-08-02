import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";
import type {
  ChatBehaviorPreferences,
  ChatChannelId,
  ChatChannelRead,
  ChatParticipantId,
  ChatReplyThreadId,
  ChatWorkAssignmentId,
  ChatAiTeammateRead,
  ChatTeammatePolicyRead,
  ChatConversationMembershipRead,
  ChatProjectPrimaryWorkingFolderRead,
  ChatChannelPageRead,
  ChatReplyThreadPageRead,
  ChatMessageSearchResultRead,
  CreateChatTeammateRequest,
  UpdateChatTeammateProfileRequest,
  PublishChatTeammatePolicyRequest,
  UpsertChatTeammateMembershipRequest,
  PostChatMessageRequest,
  PostChatMessageResult,
  ChatDiagnosticPreferences,
  ChatDiagnosticsRead,
  ChatStopAllResult,
  ChatRebuildResult,
  ChatDraftRead,
  ChatAttachmentRead,
  ChatInteractionStateRead,
  ChatUserInputDraftRead,
  ChatPromptCatalogEntry,
  McpStatusRead,
  ChatQueuedFollowupRead,
  ProjectWorkingFolderPathPage,
  ProjectWorkingFolderDirectoryRead,
  ProjectWorkingFolderFilePreview,
  ChatTerminalCloseResult,
  ChatTerminalContextRead,
  ChatTerminalRead,
  ChatTerminalLayoutRead,
  ChatTerminalPanelLayout,
  ChatTerminalSnapshotRead,
  ChatCheckpointDiffRead,
  ChatCheckpointFileDiffRead,
  ChatReviewCommentRead,
  ChatReviewSnapshotRead,
  ChatReviewPatchPageRead,
  OpenChatReviewRequest,
  ReadChatReviewPatchesRequest,
  ApplyChatReviewActionRequest,
  ApplyChatReviewActionResult,
  ChatExecutionEnvironmentRead,
  CreateChatWorktreeRequest,
  CreateChatChannelRequest,
  UpdateChatChannelDetailsRequest,
  CreateChatReviewCommentRequest,
  ChatRestorePreviewRead,
  ChatRestoreResultRead,
  ChatPanelPreferences,
  ChatProjectShellRead,
  ChatSettingsRead,
  ChatThreadId,
  ChatTurnId,
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
  GitStatusRead,
  GitRemoteRead,
  GitBranchRead,
  GitWorktreeRead,
  PreviewBounds,
  PreviewTabRead,
  HostedChangeRequestRead,
  CreateHostedChangeRequest,
  HostedSourceControlRead,
} from "$lib/chat/contracts";
import {
  parseChatProjectShells,
  parseChatChannel,
  parseChatChannels,
  parseChatAiTeammate,
  parseChatAiTeammates,
  parseChatTeammatePolicy,
  parseChatConversationMemberships,
  parseChatProjectPrimaryWorkingFolder,
  parseChatChannelPage,
  parseChatReplyThreadPage,
  parseChatMessageSearchResults,
  parsePostChatMessageResult,
  parseChatWorkAssignment,
  parseChatDiagnosticPreferences,
  parseChatDiagnosticsRead,
  parseChatStopAllResult,
  parseChatRebuildResult,
  parseChatDraftRead,
  parseChatAttachmentRead,
  parseChatInteractionState,
  parseChatUserInputDraft,
  parseChatPromptCatalog,
  parseMcpStatus,
  parseChatQueuedFollowup,
  parseProjectWorkingFolderPathPage,
  parseProjectWorkingFolderDirectory,
  parseProjectWorkingFolderFilePreview,
  parseChatTerminalCloseResult,
  parseChatTerminalContext,
  parseChatTerminal,
  parseChatTerminalLayout,
  parseChatTerminalSnapshot,
  parseChatTerminals,
  parseChatCheckpointDiff,
  parseChatCheckpointFileDiff,
  parseChatReviewComment,
  parseChatReviewComments,
  parseChatReviewSnapshot,
  parseChatReviewPatchPage,
  parseChatExecutionEnvironment,
  parseChatExecutionEnvironments,
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
  parseGitStatus,
  parseGitRemotes,
  parseGitBranches,
  parseGitWorktrees,
  parsePreviewTabRead,
  parsePreviewTabs,
  parseHostedSourceControls,
  parseHostedChangeRequest,
  parseHostedChangeRequests,
} from "$lib/chat/validation";

export async function listChatReviewComments(
  threadId: ChatThreadId,
  includeResolved: boolean,
): Promise<ChatReviewCommentRead[]> {
  return parseChatReviewComments(await invoke<unknown>("chat_list_review_comments", {
    dbUrl: await ensureDbUrl(), threadId, includeResolved,
  }));
}

export async function createChatReviewComment(
  request: CreateChatReviewCommentRequest,
): Promise<ChatReviewCommentRead> {
  return parseChatReviewComment(await invoke<unknown>("chat_create_review_comment", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function setChatReviewCommentResolved(
  threadId: ChatThreadId,
  commentId: string,
  resolved: boolean,
): Promise<ChatReviewCommentRead> {
  return parseChatReviewComment(await invoke<unknown>("chat_set_review_comment_resolved", {
    dbUrl: await ensureDbUrl(), threadId, commentId, resolved,
  }));
}

export async function attachChatReviewComment(
  threadId: ChatThreadId,
  commentId: string,
  attachmentId: string,
): Promise<ChatAttachmentRead> {
  return parseChatAttachmentRead(await invoke<unknown>("chat_attach_review_comment", {
    dbUrl: await ensureDbUrl(), request: { threadId, commentId, attachmentId },
  }));
}

export async function openChatReview(
  request: OpenChatReviewRequest,
): Promise<ChatReviewSnapshotRead> {
  return parseChatReviewSnapshot(await invoke<unknown>("chat_open_review", {
    dbUrl: await ensureDbUrl(),
    request,
  }));
}

export async function readChatReviewPatches(
  request: ReadChatReviewPatchesRequest,
): Promise<ChatReviewPatchPageRead> {
  return parseChatReviewPatchPage(await invoke<unknown>("chat_read_review_patches", {
    dbUrl: await ensureDbUrl(),
    request,
  }));
}

export async function applyChatReviewAction(
  request: ApplyChatReviewActionRequest,
): Promise<ApplyChatReviewActionResult> {
  const value = await invoke<unknown>("chat_apply_review_action", {
    dbUrl: await ensureDbUrl(),
    request,
  });
  if (typeof value !== "object" || value === null || !("snapshot" in value)) {
    throw new Error("Invalid Chat review action result");
  }
  return { snapshot: parseChatReviewSnapshot(value.snapshot) };
}

export async function listChatExecutionEnvironments(
  workingFolderId: ProjectWorkingFolderId,
): Promise<ChatExecutionEnvironmentRead[]> {
  return parseChatExecutionEnvironments(await invoke<unknown>("chat_list_execution_environments", {
    dbUrl: await ensureDbUrl(), workingFolderId,
  }));
}

export async function createChatWorktreeEnvironment(
  request: CreateChatWorktreeRequest,
): Promise<ChatExecutionEnvironmentRead> {
  return parseChatExecutionEnvironment(await invoke<unknown>("chat_create_worktree_environment", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function removeChatWorktreeEnvironment(
  workingFolderId: ProjectWorkingFolderId,
  environmentId: string,
  confirmed: boolean,
): Promise<void> {
  await invoke("chat_remove_worktree_environment", {
    dbUrl: await ensureDbUrl(), workingFolderId, environmentId, confirmed,
  });
}

export async function readChatThreadExecutionEnvironment(threadId: ChatThreadId): Promise<string | null> {
  const value = await invoke<unknown>("chat_read_thread_execution_environment", {
    dbUrl: await ensureDbUrl(), threadId,
  });
  if (value === null) return null;
  if (typeof value !== "string" || value.length === 0) {
    throw new Error("Chat execution environment response is invalid");
  }
  return value;
}

export async function discoverHostedSourceControl(
  workingFolderId: ProjectWorkingFolderId,
  executionEnvironmentId: string | null = null,
): Promise<HostedSourceControlRead[]> {
  return parseHostedSourceControls(await invoke<unknown>("chat_discover_source_control", {
    dbUrl: await ensureDbUrl(), workingFolderId, executionEnvironmentId,
  }));
}

export async function listHostedChangeRequests(
  workingFolderId: ProjectWorkingFolderId,
  executionEnvironmentId: string | null,
  providerKind: HostedSourceControlRead["kind"],
  repositorySlug: string,
  limit = 50,
): Promise<HostedChangeRequestRead[]> {
  return parseHostedChangeRequests(await invoke<unknown>("chat_list_hosted_change_requests", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    executionEnvironmentId,
    providerKind,
    repositorySlug,
    limit,
  }));
}

export async function createHostedChangeRequest(
  request: CreateHostedChangeRequest,
): Promise<HostedChangeRequestRead> {
  return parseHostedChangeRequest(await invoke<unknown>("chat_create_hosted_change_request", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function checkoutHostedChangeRequest(
  workingFolderId: ProjectWorkingFolderId,
  executionEnvironmentId: string | null,
  providerKind: HostedSourceControlRead["kind"],
  reference: string,
  remoteName: string | null,
): Promise<GitStatusRead> {
  return parseGitStatus(await invoke<unknown>("chat_checkout_hosted_change_request", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    executionEnvironmentId,
    providerKind,
    reference,
    remoteName,
  }));
}

export async function configureBitbucketCredential(
  repositorySlug: string,
  username: string,
  token: string,
): Promise<void> {
  await invoke("chat_configure_bitbucket_credential", {
    request: { repositorySlug, username, token },
  });
}

export async function removeBitbucketCredential(repositorySlug: string): Promise<boolean> {
  return invoke<boolean>("chat_remove_bitbucket_credential", { repositorySlug });
}

export async function readPreviewStatus(threadId: ChatThreadId): Promise<PreviewTabRead[]> {
  return parsePreviewTabs(await invoke<unknown>("chat_preview_status", {
    dbUrl: await ensureDbUrl(), threadId,
  }));
}

export interface DiscoveredPreviewServer {
  url: string;
  sourceLabel: string;
}

export async function discoverPreviewServers(threadId: ChatThreadId): Promise<DiscoveredPreviewServer[]> {
  const value = await invoke<unknown>("chat_preview_discover_servers", {
    dbUrl: await ensureDbUrl(), threadId,
  });
  if (!Array.isArray(value)) throw new Error("Invalid preview server list");
  return value.map((entry) => {
    if (typeof entry !== "object" || entry === null) throw new Error("Invalid preview server");
    const record = entry as Record<string, unknown>;
    if (typeof record.url !== "string" || typeof record.sourceLabel !== "string") {
      throw new Error("Invalid preview server");
    }
    return { url: record.url, sourceLabel: record.sourceLabel };
  });
}

export async function openPreview(request: {
  threadId: ChatThreadId;
  tabId: string;
  url: string;
  bounds: PreviewBounds;
  externalNavigationConfirmed: boolean;
}): Promise<PreviewTabRead> {
  return parsePreviewTabRead(await invoke<unknown>("chat_preview_open", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function navigatePreview(
  threadId: ChatThreadId,
  tabId: string,
  url: string,
  externalNavigationConfirmed: boolean,
): Promise<PreviewTabRead> {
  return parsePreviewTabRead(await invoke<unknown>("chat_preview_navigate", {
    dbUrl: await ensureDbUrl(), threadId, tabId, url, externalNavigationConfirmed,
  }));
}

export async function resizePreview(
  threadId: ChatThreadId,
  tabId: string,
  bounds: PreviewBounds,
): Promise<PreviewTabRead> {
  return parsePreviewTabRead(await invoke<unknown>("chat_preview_resize", {
    dbUrl: await ensureDbUrl(), threadId, tabId, bounds,
  }));
}

export async function setPreviewVisible(
  threadId: ChatThreadId,
  tabId: string,
  visible: boolean,
): Promise<PreviewTabRead> {
  return parsePreviewTabRead(await invoke<unknown>("chat_preview_set_visible", {
    dbUrl: await ensureDbUrl(), threadId, tabId, visible,
  }));
}

export async function previewBack(threadId: ChatThreadId, tabId: string): Promise<void> {
  await invoke("chat_preview_back", { threadId, tabId });
}

export async function previewForward(threadId: ChatThreadId, tabId: string): Promise<void> {
  await invoke("chat_preview_forward", { threadId, tabId });
}

export async function refreshPreview(threadId: ChatThreadId, tabId: string): Promise<void> {
  await invoke("chat_preview_refresh", { threadId, tabId });
}

export interface BrowserArtifactRead {
  id: string;
  displayName: string;
  mimeType: string;
  byteSize: number;
  resourceUri: string;
}

function parseBrowserArtifactRead(value: unknown): BrowserArtifactRead {
  if (typeof value !== "object" || value === null) throw new Error("Invalid browser artifact");
  const record = value as Record<string, unknown>;
  if (typeof record.id !== "string" || typeof record.displayName !== "string"
    || typeof record.mimeType !== "string" || typeof record.byteSize !== "number"
    || typeof record.resourceUri !== "string") {
    throw new Error("Invalid browser artifact");
  }
  return {
    id: record.id,
    displayName: record.displayName,
    mimeType: record.mimeType,
    byteSize: record.byteSize,
    resourceUri: record.resourceUri,
  };
}

export async function capturePreviewScreenshot(threadId: ChatThreadId, tabId: string): Promise<BrowserArtifactRead> {
  return parseBrowserArtifactRead(await invoke<unknown>("chat_preview_screenshot", {
    dbUrl: await ensureDbUrl(), threadId, tabId,
  }));
}

export async function startPreviewRecording(threadId: ChatThreadId, tabId: string): Promise<void> {
  await invoke("chat_preview_recording_start", { threadId, tabId, approved: true });
}

export async function stopPreviewRecording(threadId: ChatThreadId, tabId: string): Promise<BrowserArtifactRead> {
  return parseBrowserArtifactRead(await invoke<unknown>("chat_preview_recording_stop", {
    dbUrl: await ensureDbUrl(), threadId, tabId,
  }));
}

export async function closePreview(threadId: ChatThreadId, tabId: string): Promise<void> {
  await invoke("chat_preview_close", { dbUrl: await ensureDbUrl(), threadId, tabId });
}

export async function readGitStatus(workingFolderId: ProjectWorkingFolderId, executionEnvironmentId: string | null = null): Promise<GitStatusRead> {
  return parseGitStatus(await invoke<unknown>("chat_git_status", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    executionEnvironmentId,
  }));
}

export async function readGitDiff(
  workingFolderId: ProjectWorkingFolderId,
  staged: boolean,
  relativePath: string | null,
  executionEnvironmentId: string | null = null,
): Promise<string> {
  return await invoke<string>("chat_git_diff", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    staged,
    relativePath,
    executionEnvironmentId,
  });
}

export async function stageGitPaths(
  workingFolderId: ProjectWorkingFolderId,
  paths: string[],
  executionEnvironmentId: string | null = null,
): Promise<GitStatusRead> {
  return parseGitStatus(await invoke<unknown>("chat_git_stage", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    paths,
    executionEnvironmentId,
  }));
}

export async function unstageGitPaths(
  workingFolderId: ProjectWorkingFolderId,
  paths: string[],
  executionEnvironmentId: string | null = null,
): Promise<GitStatusRead> {
  return parseGitStatus(await invoke<unknown>("chat_git_unstage", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    paths,
    executionEnvironmentId,
  }));
}

export async function commitGitChanges(
  workingFolderId: ProjectWorkingFolderId,
  message: string,
  executionEnvironmentId: string | null = null,
): Promise<GitStatusRead> {
  return parseGitStatus(await invoke<unknown>("chat_git_commit", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    message,
    executionEnvironmentId,
  }));
}

export async function fetchGitRemote(
  workingFolderId: ProjectWorkingFolderId,
  remote: string | null = null,
  executionEnvironmentId: string | null = null,
): Promise<GitStatusRead> {
  return parseGitStatus(await invoke<unknown>("chat_git_fetch", {
    dbUrl: await ensureDbUrl(), workingFolderId, remote, executionEnvironmentId,
  }));
}

export async function pullGitBranch(
  workingFolderId: ProjectWorkingFolderId,
  remote: string | null = null,
  branch: string | null = null,
  executionEnvironmentId: string | null = null,
): Promise<GitStatusRead> {
  return parseGitStatus(await invoke<unknown>("chat_git_pull", {
    dbUrl: await ensureDbUrl(), workingFolderId, remote, branch, executionEnvironmentId,
  }));
}

export async function pushGitBranch(
  workingFolderId: ProjectWorkingFolderId,
  remote: string | null = null,
  branch: string | null = null,
  forceWithLease = false,
  destructiveConfirmed = false,
  executionEnvironmentId: string | null = null,
): Promise<GitStatusRead> {
  return parseGitStatus(await invoke<unknown>("chat_git_push", {
    dbUrl: await ensureDbUrl(),
    request: { workingFolderId, remote, branch, forceWithLease, destructiveConfirmed, executionEnvironmentId },
  }));
}

export async function listGitRemotes(workingFolderId: ProjectWorkingFolderId, executionEnvironmentId: string | null = null): Promise<GitRemoteRead[]> {
  return parseGitRemotes(await invoke<unknown>("chat_git_remotes", {
    dbUrl: await ensureDbUrl(), workingFolderId, executionEnvironmentId,
  }));
}

export async function listGitBranches(workingFolderId: ProjectWorkingFolderId, executionEnvironmentId: string | null = null): Promise<GitBranchRead[]> {
  return parseGitBranches(await invoke<unknown>("chat_git_branches", {
    dbUrl: await ensureDbUrl(), workingFolderId, executionEnvironmentId,
  }));
}

export async function listGitWorktrees(workingFolderId: ProjectWorkingFolderId, executionEnvironmentId: string | null = null): Promise<GitWorktreeRead[]> {
  return parseGitWorktrees(await invoke<unknown>("chat_git_worktrees", {
    dbUrl: await ensureDbUrl(), workingFolderId, executionEnvironmentId,
  }));
}

export async function initializeGitRepository(
  workingFolderId: ProjectWorkingFolderId,
  initialBranch: string | null,
  executionEnvironmentId: string | null = null,
): Promise<GitStatusRead> {
  return parseGitStatus(await invoke<unknown>("chat_git_initialize", {
    dbUrl: await ensureDbUrl(), workingFolderId, initialBranch, executionEnvironmentId,
  }));
}

export async function cloneGitRepository(
  workingFolderId: ProjectWorkingFolderId,
  remoteUrl: string,
  remoteName = "origin",
  executionEnvironmentId: string | null = null,
): Promise<GitStatusRead> {
  return parseGitStatus(await invoke<unknown>("chat_git_clone", {
    dbUrl: await ensureDbUrl(), workingFolderId, remoteUrl, remoteName, executionEnvironmentId,
  }));
}

export async function discardGitPaths(
  workingFolderId: ProjectWorkingFolderId,
  paths: string[],
  destructiveConfirmed: boolean,
  executionEnvironmentId: string | null = null,
): Promise<GitStatusRead> {
  return parseGitStatus(await invoke<unknown>("chat_git_discard", {
    dbUrl: await ensureDbUrl(), workingFolderId, paths, destructiveConfirmed, executionEnvironmentId,
  }));
}

export async function deleteGitBranch(
  workingFolderId: ProjectWorkingFolderId,
  branch: string,
  force: boolean,
  destructiveConfirmed: boolean,
  executionEnvironmentId: string | null = null,
): Promise<GitBranchRead[]> {
  return parseGitBranches(await invoke<unknown>("chat_git_delete_branch", {
    dbUrl: await ensureDbUrl(), workingFolderId, branch, force, destructiveConfirmed, executionEnvironmentId,
  }));
}

export async function readChatTerminalLayout(threadId: ChatThreadId): Promise<ChatTerminalLayoutRead> {
  return parseChatTerminalLayout(await invoke<unknown>("chat_read_terminal_layout", {
    dbUrl: await ensureDbUrl(), threadId,
  }));
}

export async function saveChatTerminalPanelLayout(
  threadId: ChatThreadId,
  panel: ChatTerminalPanelLayout,
): Promise<ChatTerminalLayoutRead> {
  return parseChatTerminalLayout(await invoke<unknown>("chat_save_terminal_panel_layout", {
    dbUrl: await ensureDbUrl(), threadId, panel,
  }));
}

export async function listProjectWorkingFolderDirectory(
  workingFolderId: ProjectWorkingFolderId,
  relativePath: string,
  includeIgnored = false,
  executionEnvironmentId: string | null = null,
): Promise<ProjectWorkingFolderDirectoryRead> {
  return parseProjectWorkingFolderDirectory(await invoke<unknown>("project_list_working_folder_directory", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    relativePath,
    includeIgnored,
    executionEnvironmentId,
  }));
}

export async function previewProjectWorkingFolderFile(
  workingFolderId: ProjectWorkingFolderId,
  relativePath: string,
  executionEnvironmentId: string | null = null,
): Promise<ProjectWorkingFolderFilePreview> {
  return parseProjectWorkingFolderFilePreview(await invoke<unknown>("project_preview_working_folder_file", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    relativePath,
    executionEnvironmentId,
  }));
}

export async function saveProjectWorkingFolderFile(request: {
  workingFolderId: ProjectWorkingFolderId;
  relativePath: string;
  contents: string;
  expectedRevision: string;
  executionEnvironmentId?: string | null;
}): Promise<ProjectWorkingFolderFilePreview> {
  return parseProjectWorkingFolderFilePreview(await invoke<unknown>("project_save_working_folder_file", {
    dbUrl: await ensureDbUrl(),
    request,
  }));
}

export async function saveProjectWorkingFolderFileCopy(request: {
  workingFolderId: ProjectWorkingFolderId;
  sourceRelativePath: string;
  targetRelativePath: string;
  contents: string;
  executionEnvironmentId?: string | null;
}): Promise<ProjectWorkingFolderFilePreview> {
  return parseProjectWorkingFolderFilePreview(await invoke<unknown>("project_save_working_folder_file_copy", {
    dbUrl: await ensureDbUrl(),
    request,
  }));
}

export async function recreateProjectWorkingFolderFile(request: {
  workingFolderId: ProjectWorkingFolderId;
  relativePath: string;
  contents: string;
  confirmed: boolean;
  executionEnvironmentId?: string | null;
}): Promise<ProjectWorkingFolderFilePreview> {
  return parseProjectWorkingFolderFilePreview(await invoke<unknown>("project_recreate_working_folder_file", {
    dbUrl: await ensureDbUrl(),
    request,
  }));
}

export async function openProjectWorkingFolderFile(
  workingFolderId: ProjectWorkingFolderId,
  relativePath: string,
  executionEnvironmentId: string | null = null,
): Promise<void> {
  await invoke("project_open_working_folder_file", { dbUrl: await ensureDbUrl(), workingFolderId, relativePath, executionEnvironmentId });
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
  turnId: ChatTurnId | null = null,
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

export async function discoverDefaultChatProviders(): Promise<ChatSettingsRead> {
  return parseChatSettingsRead(await invoke<unknown>("chat_discover_default_providers"));
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

export async function listChatChannels(projectId: string, archived = false): Promise<ChatChannelRead[]> {
  return parseChatChannels(await invoke<unknown>("chat_list_channels", {
    dbUrl: await ensureDbUrl(), projectId, archived,
  }));
}

export async function listChatNavigationChannels(): Promise<ChatChannelRead[]> {
  return parseChatChannels(await invoke<unknown>("chat_list_navigation_channels", {
    dbUrl: await ensureDbUrl(),
  }));
}

export async function searchChatChannels(
  projectId: string,
  query: string,
  archived = false,
  limit = 100,
): Promise<ChatChannelRead[]> {
  return parseChatChannels(await invoke<unknown>("chat_search_channels", {
    dbUrl: await ensureDbUrl(), projectId, query, archived, limit,
  }));
}

export async function readChatChannel(channelId: ChatChannelId): Promise<ChatChannelRead> {
  return parseChatChannel(await invoke<unknown>("chat_read_channel", {
    dbUrl: await ensureDbUrl(), channelId,
  }));
}

export async function createChatChannel(request: CreateChatChannelRequest): Promise<ChatChannelRead> {
  return parseChatChannel(await invoke<unknown>("chat_create_channel", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function updateChatChannelDetails(
  request: UpdateChatChannelDetailsRequest,
): Promise<ChatChannelRead> {
  return parseChatChannel(await invoke<unknown>("chat_update_channel_details", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function archiveChatChannel(
  channelId: ChatChannelId,
  expectedRevision: number,
): Promise<ChatChannelRead> {
  return parseChatChannel(await invoke<unknown>("chat_archive_channel", {
    dbUrl: await ensureDbUrl(), channelId, expectedRevision,
  }));
}

export async function restoreChatChannel(
  channelId: ChatChannelId,
  expectedRevision: number,
): Promise<ChatChannelRead> {
  return parseChatChannel(await invoke<unknown>("chat_restore_channel", {
    dbUrl: await ensureDbUrl(), channelId, expectedRevision,
  }));
}

export async function setChatChannelRead(channelId: ChatChannelId, read: boolean): Promise<ChatChannelRead> {
  return parseChatChannel(await invoke<unknown>("chat_set_channel_read", {
    dbUrl: await ensureDbUrl(), channelId, read,
  }));
}

export async function listChatTeammates(archived = false): Promise<ChatAiTeammateRead[]> {
  return parseChatAiTeammates(await invoke<unknown>("chat_list_teammates", {
    dbUrl: await ensureDbUrl(), archived,
  }));
}

export async function readChatTeammate(teammateId: ChatParticipantId): Promise<ChatAiTeammateRead> {
  return parseChatAiTeammate(await invoke<unknown>("chat_read_teammate", {
    dbUrl: await ensureDbUrl(), teammateId,
  }));
}

export async function createChatTeammate(request: CreateChatTeammateRequest): Promise<ChatAiTeammateRead> {
  return parseChatAiTeammate(await invoke<unknown>("chat_create_teammate", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function updateChatTeammateProfile(
  request: UpdateChatTeammateProfileRequest,
): Promise<ChatAiTeammateRead> {
  return parseChatAiTeammate(await invoke<unknown>("chat_update_teammate_profile", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function archiveChatTeammate(
  teammateId: ChatParticipantId,
  expectedRevision: number,
  archived: boolean,
): Promise<ChatAiTeammateRead> {
  return parseChatAiTeammate(await invoke<unknown>("chat_archive_teammate", {
    dbUrl: await ensureDbUrl(), teammateId, expectedRevision, archived,
  }));
}

export async function publishChatTeammatePolicy(
  request: PublishChatTeammatePolicyRequest,
): Promise<ChatTeammatePolicyRead> {
  return parseChatTeammatePolicy(await invoke<unknown>("chat_publish_teammate_policy", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function listChatChannelMemberships(
  channelId: ChatChannelId,
): Promise<ChatConversationMembershipRead[]> {
  return parseChatConversationMemberships(await invoke<unknown>("chat_list_channel_memberships", {
    dbUrl: await ensureDbUrl(), channelId,
  }));
}

export async function readChatProjectPrimaryWorkingFolder(
  projectId: string,
): Promise<ChatProjectPrimaryWorkingFolderRead> {
  return parseChatProjectPrimaryWorkingFolder(await invoke<unknown>(
    "chat_read_project_primary_working_folder",
    { dbUrl: await ensureDbUrl(), projectId },
  ));
}

export async function setChatProjectPrimaryWorkingFolder(
  projectId: string,
  workingFolderId: ProjectWorkingFolderId,
  expectedRevision: number,
): Promise<ChatProjectPrimaryWorkingFolderRead> {
  return parseChatProjectPrimaryWorkingFolder(await invoke<unknown>(
    "chat_set_project_primary_working_folder",
    { dbUrl: await ensureDbUrl(), projectId, workingFolderId, expectedRevision },
  ));
}

export async function upsertChatTeammateMembership(
  request: UpsertChatTeammateMembershipRequest,
): Promise<ChatConversationMembershipRead> {
  const memberships = parseChatConversationMemberships([
    await invoke<unknown>("chat_upsert_teammate_membership", {
      dbUrl: await ensureDbUrl(), request,
    }),
  ]);
  return memberships[0];
}

export async function removeChatTeammateMembership(
  teammateId: ChatParticipantId,
  channelId: ChatChannelId,
  expectedRevision: number,
  stopActiveWork: boolean,
): Promise<void> {
  await invoke("chat_remove_teammate_membership", {
    dbUrl: await ensureDbUrl(), teammateId, channelId, expectedRevision, stopActiveWork,
  });
}

export async function postChatMessage(request: PostChatMessageRequest): Promise<PostChatMessageResult> {
  return parsePostChatMessageResult(await invoke<unknown>("chat_post_message", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function readChatChannelPage(
  channelId: ChatChannelId,
  cursor: string | null = null,
  limit = 50,
): Promise<ChatChannelPageRead> {
  return parseChatChannelPage(await invoke<unknown>("chat_read_channel_page", {
    dbUrl: await ensureDbUrl(), channelId, cursor, limit,
  }));
}

export async function readChatReplyThreadPage(
  replyThreadId: ChatReplyThreadId,
  cursor: string | null = null,
  limit = 50,
): Promise<ChatReplyThreadPageRead> {
  return parseChatReplyThreadPage(await invoke<unknown>("chat_read_reply_thread_page", {
    dbUrl: await ensureDbUrl(), replyThreadId, cursor, limit,
  }));
}

export async function searchChatMessages(
  query: string,
  projectId: string | null = null,
  limit = 50,
): Promise<ChatMessageSearchResultRead[]> {
  return parseChatMessageSearchResults(await invoke<unknown>("chat_search_messages", {
    dbUrl: await ensureDbUrl(), query, projectId, limit,
  }));
}

export async function cancelChatAssignment(
  assignmentId: ChatWorkAssignmentId,
  expectedRevision: number,
): Promise<import("$lib/chat/contracts").ChatWorkAssignmentRead> {
  return parseChatWorkAssignment(await invoke<unknown>("chat_cancel_assignment", {
    dbUrl: await ensureDbUrl(), assignmentId, expectedRevision,
  }));
}

export async function retryChatAssignment(
  assignmentId: ChatWorkAssignmentId,
  expectedRevision: number,
): Promise<import("$lib/chat/contracts").ChatWorkAssignmentRead> {
  return parseChatWorkAssignment(await invoke<unknown>("chat_retry_assignment", {
    dbUrl: await ensureDbUrl(), assignmentId, expectedRevision,
  }));
}

export async function recoverChatAssignmentDispatchJobs(): Promise<number> {
  return invoke<number>("chat_recover_assignment_dispatch_jobs", { dbUrl: await ensureDbUrl() });
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

export async function listChatThreadWindow(
  workingFolderId: ProjectWorkingFolderId | null,
  archived: boolean,
  limit: number,
): Promise<ChatThreadShellRead[]> {
  return parseChatThreadShells(await invoke<unknown>("chat_list_thread_window", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    archived,
    limit,
  }));
}

export async function readChatThreadShell(threadId: ChatThreadId): Promise<ChatThreadShellRead> {
  return parseChatThreadShell(await invoke<unknown>("chat_read_thread_shell", {
    dbUrl: await ensureDbUrl(),
    threadId,
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

export async function forkChatThread(
  sourceThreadId: ChatThreadId,
  newThreadId: ChatThreadId,
  title: string,
  lastProviderTurnId: string | null = null,
): Promise<ChatThreadShellRead> {
  return parseChatThreadShell(await invoke<unknown>("chat_fork_thread", {
    dbUrl: await ensureDbUrl(),
    request: { sourceThreadId, newThreadId, title, lastProviderTurnId },
  }));
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
  executionEnvironmentId: string | null = null,
): Promise<ProjectWorkingFolderPathPage> {
  return parseProjectWorkingFolderPathPage(await invoke<unknown>("chat_search_working_folder_paths", {
    dbUrl: await ensureDbUrl(),
    request: { workingFolderId, query, includeIgnored, cursor, limit, executionEnvironmentId },
  }));
}

export async function validateChatWorkingFolderMentions(
  workingFolderId: ProjectWorkingFolderId,
  relativePaths: string[],
  executionEnvironmentId: string | null = null,
): Promise<void> {
  await invoke("chat_validate_working_folder_mentions", {
    dbUrl: await ensureDbUrl(), workingFolderId, relativePaths, executionEnvironmentId,
  });
}

export async function listChatPromptCatalog(
  workingFolderId: ProjectWorkingFolderId,
  providerInstanceId: ProviderInstanceId,
  threadId: ChatThreadId | null,
): Promise<ChatPromptCatalogEntry[]> {
  return parseChatPromptCatalog(await invoke<unknown>("chat_list_prompt_catalog", {
    dbUrl: await ensureDbUrl(), workingFolderId, providerInstanceId, threadId,
  }));
}

export async function readChatInteractionState(threadId: ChatThreadId): Promise<ChatInteractionStateRead> {
  return parseChatInteractionState(await invoke<unknown>("chat_read_interaction_state", {
    dbUrl: await ensureDbUrl(), threadId,
  }));
}

export async function compactChatContext(threadId: ChatThreadId): Promise<void> {
  await invoke("chat_compact_context", {
    dbUrl: await ensureDbUrl(), threadId, clientCommandId: crypto.randomUUID(),
  });
}

export async function readChatMcpStatus(
  workingFolderId: ProjectWorkingFolderId,
  providerInstanceId: ProviderInstanceId,
  threadId: ChatThreadId | null,
): Promise<McpStatusRead> {
  return parseMcpStatus(await invoke<unknown>("chat_read_mcp_status", {
    dbUrl: await ensureDbUrl(), workingFolderId, providerInstanceId, threadId,
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
