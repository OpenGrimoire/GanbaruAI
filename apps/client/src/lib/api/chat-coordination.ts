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
  ScheduleChatMessageRequest,
  ChatScheduledMessageDispatchRead,
  ChatScheduledMessageId,
  ChatScheduledMessageRead,
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
  parseChatScheduledMessage,
  parseChatScheduledMessageDispatch,
  parseChatScheduledMessages,
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

export async function scheduleChatMessage(
  request: ScheduleChatMessageRequest,
): Promise<ChatScheduledMessageRead> {
  return parseChatScheduledMessage(await invoke<unknown>("chat_schedule_message", {
    dbUrl: await ensureDbUrl(), request,
  }));
}

export async function listScheduledChatMessages(
  channelId: ChatChannelId,
  replyThreadId: ChatReplyThreadId | null,
): Promise<ChatScheduledMessageRead[]> {
  return parseChatScheduledMessages(await invoke<unknown>("chat_list_scheduled_messages", {
    dbUrl: await ensureDbUrl(), channelId, replyThreadId,
  }));
}

export async function cancelScheduledChatMessage(
  scheduledMessageId: ChatScheduledMessageId,
): Promise<void> {
  await invoke("chat_cancel_scheduled_message", {
    dbUrl: await ensureDbUrl(), scheduledMessageId,
  });
}

export async function retryScheduledChatMessage(
  scheduledMessageId: ChatScheduledMessageId,
): Promise<ChatScheduledMessageRead> {
  return parseChatScheduledMessage(await invoke<unknown>("chat_retry_scheduled_message", {
    dbUrl: await ensureDbUrl(), scheduledMessageId,
  }));
}

export async function sendScheduledChatMessageNow(
  scheduledMessageId: ChatScheduledMessageId,
): Promise<PostChatMessageResult> {
  return parsePostChatMessageResult(await invoke<unknown>("chat_send_scheduled_message_now", {
    dbUrl: await ensureDbUrl(), scheduledMessageId,
  }));
}

export async function dispatchDueScheduledChatMessages(): Promise<ChatScheduledMessageDispatchRead> {
  return parseChatScheduledMessageDispatch(await invoke<unknown>("chat_dispatch_due_scheduled_messages", {
    dbUrl: await ensureDbUrl(),
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
