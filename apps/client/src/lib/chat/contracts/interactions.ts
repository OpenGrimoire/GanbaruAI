import type {
  ChatAttachmentId,
  ChatThreadId,
  ProjectWorkingFolderId,
  ProviderSessionState,
  UtcTimestamp,
  VersionedJson,
} from "./common";
import type { ProviderCapabilities } from "./provider";
import type { AccountStatusEvent, RateLimitStatusEvent, ThreadUsageUpdatedEvent } from "./events";

export interface ChatAttachmentRead {
  id: ChatAttachmentId;
  workingFolderId: ProjectWorkingFolderId;
  kind: "image" | "text_snippet";
  originalDisplayName: string;
  mimeType: string;
  byteSize: number;
  sha256: string;
  managedRelativePath: string;
  signatureKind: string;
  createdAt: UtcTimestamp;
}

export interface ProjectWorkingFolderPathRead {
  relativePath: string;
  displayName: string;
  kind: "file" | "directory";
  ignored: boolean;
}

export interface ProjectWorkingFolderPathPage {
  entries: ProjectWorkingFolderPathRead[];
  nextCursor: string | null;
}

export interface ChatPromptCatalogEntry {
  value: string;
  label: string;
  description: string | null;
  kind: "skill" | "command";
  stale: boolean;
}

export interface ChatPendingRequestRead {
  id: string;
  turnId: string | null;
  providerRequestId: string;
  requestKind: "approval" | "user_input";
  safeDisplay: VersionedJson;
  allowedDecisions: VersionedJson;
  openedAt: UtcTimestamp;
}

export interface ChatQueuedFollowupRead {
  id: string;
  threadId: ChatThreadId;
  text: string;
  providerInstanceId: string;
  modelSelection: VersionedJson;
  safetyMode: import("./common").SafetyMode;
  interactionMode: import("./common").InteractionMode;
  attachmentIds: ChatAttachmentId[];
  mentions: VersionedJson;
  createdAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
}

export interface ChatInteractionStateRead {
  sessionState: ProviderSessionState;
  activeTurnId: string | null;
  capabilities: ProviderCapabilities;
  pendingRequest: ChatPendingRequestRead | null;
  queuedFollowup: ChatQueuedFollowupRead | null;
  usage: ThreadUsageUpdatedEvent | null;
  accountStatus: AccountStatusEvent | null;
  rateLimitStatus: RateLimitStatusEvent | null;
  automaticCompactionReported: boolean;
}

export interface ChatUserInputDraftRead {
  requestId: string;
  answers: VersionedJson;
  updatedAt: UtcTimestamp;
}

export interface SaveQueuedFollowupRequest {
  id: string;
  threadId: ChatThreadId;
  text: string;
  providerInstanceId: string;
  modelSelection: VersionedJson;
  safetyMode: import("./common").SafetyMode;
  interactionMode: import("./common").InteractionMode;
  attachmentIds: ChatAttachmentId[];
  mentions: VersionedJson;
}

export interface SendChatTurnCommand {
  command: import("./commands").ChatCommandContext;
  workingFolderId: ProjectWorkingFolderId;
  threadId: ChatThreadId | null;
  newThreadId: ChatThreadId | null;
  turnId: string;
  messageId: string;
  providerInstanceId: string;
  providerManagedModel: boolean;
  modelId: string | null;
  modelOptions: import("./provider").ModelOptionSelection[];
  modes: import("./common").TurnModeSnapshot;
  prompt: string;
  attachmentIds: ChatAttachmentId[];
  mentions: import("./commands").WorkspaceMentionReference[];
}

export interface SendChatTurnResult {
  thread: import("./reads").ChatThreadShellRead;
  dispatch: import("./reads").TurnDispatchReceipt | null;
  launchError: import("./common").ChatError | null;
}

export interface SteerChatTurnCommand {
  command: import("./commands").ChatCommandContext;
  threadId: ChatThreadId;
  messageId: string;
  prompt: string;
}

export interface ResolveChatApprovalCommand {
  command: import("./commands").ChatCommandContext;
  threadId: ChatThreadId;
  requestId: string;
  providerRequestId: string;
  decision: import("./commands").ApprovalDecision;
}

export interface ResolveChatUserInputCommand {
  command: import("./commands").ChatCommandContext;
  threadId: ChatThreadId;
  requestId: string;
  providerRequestId: string;
  answers: import("./commands").UserInputAnswer[];
}
