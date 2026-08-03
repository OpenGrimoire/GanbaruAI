import type {
  ChatAgentRunId,
  ChatAttachmentId,
  ChatChannelId,
  ChatCommandId,
  ChatConversationId,
  ChatConversationItemId,
  ChatMessageRevisionId,
  ChatParticipantId,
  ChatReplyThreadId,
  ChatTeammatePolicyRevisionId,
  ChatThreadId,
  ChatTurnId,
  ChatWorkAssignmentId,
  ModelId,
  ProjectWorkingFolderId,
  ProviderInstanceId,
  UtcTimestamp,
  VersionedJson,
} from "./common";
import type { ModelOptionSelection } from "./provider";

export const CHAT_PARTICIPANT_KINDS = ["local_user", "ai_teammate", "human"] as const;
export type ChatParticipantKind = (typeof CHAT_PARTICIPANT_KINDS)[number];

export const CHAT_TEAMMATE_CONFIGURATION_STATES = [
  "healthy",
  "needs_setup",
  "provider_unavailable",
  "folder_access_missing",
] as const;
export type ChatTeammateConfigurationState = (typeof CHAT_TEAMMATE_CONFIGURATION_STATES)[number];

export const CHAT_WORK_ASSIGNMENT_STATES = [
  "queued",
  "working",
  "waiting_for_answer",
  "waiting_for_approval",
  "ready_for_review",
  "completed",
  "failed",
  "cancelled",
] as const;
export type ChatWorkAssignmentState = (typeof CHAT_WORK_ASSIGNMENT_STATES)[number];

export const CHAT_APPROVAL_POLICIES = [
  "ask_for_approval",
  "approve_for_me",
  "full_access",
  "custom",
] as const;
export type ChatApprovalPolicy = (typeof CHAT_APPROVAL_POLICIES)[number];

export interface ChatParticipantRead {
  id: ChatParticipantId;
  kind: ChatParticipantKind;
  displayName: string;
  handle: string | null;
  avatar: VersionedJson;
  revision: number;
  archivedAt: UtcTimestamp | null;
}

export interface ChatTeammatePolicyRead {
  id: ChatTeammatePolicyRevisionId;
  teammateId: ChatParticipantId;
  revision: number;
  providerInstanceId: ProviderInstanceId;
  providerManagedModel: boolean;
  modelId: ModelId | null;
  modelOptions: ModelOptionSelection[];
  effort: string | null;
  speed: string | null;
  providerOptions: VersionedJson;
  createdAt: UtcTimestamp;
}

export interface ChatAiTeammateRead {
  participant: ChatParticipantRead;
  purpose: string;
  instructions: string;
  configurationState: ChatTeammateConfigurationState;
  latestPolicy: ChatTeammatePolicyRead | null;
  channelCount: number;
}

export interface ChatWorkingFolderGrantRead {
  workingFolderId: ProjectWorkingFolderId;
  displayName: string;
  isDefault: boolean;
}

export interface ChatConversationMembershipRead {
  conversationId: ChatConversationId;
  participant: ChatParticipantRead;
  addressable: boolean;
  approvalPolicy: ChatApprovalPolicy;
  workingFolderGrants: ChatWorkingFolderGrantRead[];
  revision: number;
  removedAt: UtcTimestamp | null;
}

export interface ChatProjectPrimaryWorkingFolderRead {
  projectId: string;
  workingFolderId: ProjectWorkingFolderId;
  revision: number;
}

export interface ChatParticipantMentionRead {
  participantId: ChatParticipantId;
  participantKind: ChatParticipantKind;
  handleSnapshot: string | null;
  labelSnapshot: string;
  startOffset: number;
  endOffset: number;
}

export interface ChatParticipantMentionInput extends ChatParticipantMentionRead {}

export interface ChatResourceReferenceRead {
  workingFolderId: ProjectWorkingFolderId;
  kind: "file" | "folder";
  relativePath: string;
  displayLabel: string;
}

export interface ChatResourceReferenceInput extends ChatResourceReferenceRead {}

export interface ChatReplyThreadSummaryRead {
  id: ChatReplyThreadId;
  replyCount: number;
  lastActivityAt: UtcTimestamp;
  participants: ChatParticipantRead[];
  unread: boolean;
  workState: ChatWorkAssignmentState | null;
}

export interface ChatMessageRead {
  itemId: ChatConversationItemId;
  conversationId: ChatConversationId;
  replyThreadId: ChatReplyThreadId | null;
  revisionId: ChatMessageRevisionId;
  revision: number;
  author: ChatParticipantRead;
  normalizedMarkdown: string;
  richContent: VersionedJson;
  mentions: ChatParticipantMentionRead[];
  attachmentIds: ChatAttachmentId[];
  resourceReferences: ChatResourceReferenceRead[];
  replyThread: ChatReplyThreadSummaryRead | null;
  ordinal: number;
  editedAt: UtcTimestamp | null;
  createdAt: UtcTimestamp;
}

export interface ChatChannelPageRead {
  channelId: ChatChannelId;
  messages: ChatMessageRead[];
  previousCursor: string | null;
  revision: number;
}

export interface ChatWorkAssignmentRead {
  id: ChatWorkAssignmentId;
  replyThreadId: ChatReplyThreadId;
  teammate: ChatParticipantRead;
  triggeringMessageItemId: ChatConversationItemId;
  previousAssignmentId: ChatWorkAssignmentId | null;
  state: ChatWorkAssignmentState;
  stateReason: string | null;
  revision: number;
  settledAt: UtcTimestamp | null;
  createdAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
}

export interface ChatAgentRunRead {
  id: ChatAgentRunId;
  assignmentId: ChatWorkAssignmentId;
  projectId: string;
  workingFolderId: ProjectWorkingFolderId;
  teammatePolicyRevisionId: ChatTeammatePolicyRevisionId;
  effort: string | null;
  providerExecutionTurnId: ChatTurnId;
  providerExecutionThreadId: ChatThreadId | null;
  state: "queued" | "starting" | "working" | "waiting" | "completed" | "failed" | "cancelled";
  runOrdinal: number;
  createdAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
}

export interface ChatReplyThreadPageRead {
  thread: ChatReplyThreadSummaryRead;
  rootMessage: ChatMessageRead;
  replies: ChatMessageRead[];
  assignment: ChatWorkAssignmentRead | null;
  agentRuns: ChatAgentRunRead[];
  previousCursor: string | null;
  revision: number;
}

export interface ChatMessageSearchResultRead {
  projectId: string;
  channelId: ChatChannelId;
  channelName: string;
  conversationId: ChatConversationId;
  replyThreadId: ChatReplyThreadId | null;
  messageItemId: ChatConversationItemId;
  ordinal: number;
  authorParticipantId: ChatParticipantId;
  authorKind: ChatParticipantKind;
  authorDisplayName: string;
  excerpt: string;
  createdAt: UtcTimestamp;
}

export interface ChatTeammatePolicyInput {
  providerInstanceId: ProviderInstanceId;
  providerManagedModel: boolean;
  modelId: ModelId | null;
  modelOptions: ModelOptionSelection[];
  effort: string | null;
  speed: string | null;
  providerOptions: VersionedJson;
}

export interface ChatTeammateMembershipInput {
  channelId: ChatChannelId;
  addressable: boolean;
  approvalPolicy: ChatApprovalPolicy;
  workingFolderIds: ProjectWorkingFolderId[];
  defaultWorkingFolderId: ProjectWorkingFolderId;
}

export interface CreateChatTeammateRequest {
  teammateId: ChatParticipantId;
  displayName: string;
  handle: string;
  avatar: VersionedJson;
  purpose: string;
  instructions: string;
  policy: ChatTeammatePolicyInput;
  memberships: ChatTeammateMembershipInput[];
}

export interface UpdateChatTeammateProfileRequest {
  teammateId: ChatParticipantId;
  displayName: string;
  handle: string;
  avatar: VersionedJson;
  purpose: string;
  instructions: string;
  expectedRevision: number;
}

export interface PublishChatTeammatePolicyRequest {
  teammateId: ChatParticipantId;
  policy: ChatTeammatePolicyInput;
}

export interface UpsertChatTeammateMembershipRequest {
  teammateId: ChatParticipantId;
  membership: ChatTeammateMembershipInput;
  expectedRevision: number | null;
}

export interface PostChatMessageRequest {
  clientCommandId: ChatCommandId;
  channelId: ChatChannelId;
  replyThreadId: ChatReplyThreadId | null;
  normalizedMarkdown: string;
  richContent: VersionedJson;
  attachmentIds: ChatAttachmentId[];
  participantMentions: ChatParticipantMentionInput[];
  resourceReferences: ChatResourceReferenceInput[];
  alsoSendToChannel: boolean;
}

export interface PostChatMessageResult {
  message: ChatMessageRead;
  replyThreadId: ChatReplyThreadId;
  assignment: ChatWorkAssignmentRead | null;
  assignmentInputQueued: boolean;
}
