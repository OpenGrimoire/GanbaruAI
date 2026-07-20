import type {
  ApprovalDecisionKind,
  ChatAttachmentId,
  ChatCheckpointId,
  ChatCommandId,
  ChatRequestId,
  ChatThreadId,
  ChatTurnId,
  ChatWorkspaceId,
  ContinuationGroupId,
  ModelId,
  ProviderInstanceId,
  ProviderRequestId,
  ProviderSessionId,
  ProviderThreadId,
  RepositoryKind,
  TurnModeSnapshot,
  VersionedJson,
} from "./common";
import type { ModelOptionSelection } from "./provider";

export interface ChatCommandContext {
  clientCommandId: ChatCommandId;
  expectedThreadRevision: number | null;
}

export interface VerifiedWorkspaceContext {
  workspaceId: ChatWorkspaceId;
  canonicalPath: string;
  repositoryKind: RepositoryKind;
  repositoryIdentity: string | null;
}

export interface PromptAttachmentReference {
  attachmentId: ChatAttachmentId;
  kind: string;
  displayName: string;
  managedRelativePath: string;
  mimeType: string | null;
  byteSize: number;
}

export interface WorkspaceMentionReference {
  relativePath: string;
  kind: string;
}

export interface StartSessionRequest {
  threadId: ChatThreadId;
  workspace: VerifiedWorkspaceContext;
  providerInstanceId: ProviderInstanceId;
  modes: TurnModeSnapshot;
  modelId: ModelId | null;
  modelOptions: ModelOptionSelection[];
}

export interface ResumeSessionRequest {
  threadId: ChatThreadId;
  workspace: VerifiedWorkspaceContext;
  providerInstanceId: ProviderInstanceId;
  providerThreadId: ProviderThreadId;
  continuationGroupId: ContinuationGroupId;
  resumeCursor: VersionedJson;
  modes: TurnModeSnapshot;
}

export interface SendTurnRequest {
  command: ChatCommandContext;
  sessionId: ProviderSessionId;
  turnId: ChatTurnId;
  prompt: string;
  attachments: PromptAttachmentReference[];
  mentions: WorkspaceMentionReference[];
  modelId: ModelId | null;
  modelOptions: ModelOptionSelection[];
  modes: TurnModeSnapshot;
  developerInstructions: string | null;
}

export interface SteerTurnRequest {
  command: ChatCommandContext;
  sessionId: ProviderSessionId;
  turnId: ChatTurnId;
  prompt: string;
}

export interface InterruptTurnRequest {
  command: ChatCommandContext;
  sessionId: ProviderSessionId;
  turnId: ChatTurnId;
}

export interface ApprovalDecision {
  kind: ApprovalDecisionKind;
  providerOptionId: string | null;
  updatedToolInput: VersionedJson | null;
}

export interface ResolveApprovalRequest {
  command: ChatCommandContext;
  sessionId: ProviderSessionId;
  requestId: ChatRequestId;
  providerRequestId: ProviderRequestId;
  decision: ApprovalDecision;
}

export interface UserInputAnswer {
  questionId: string;
  selectedOptionIds: string[];
  freeFormText: string | null;
}

export interface ResolveUserInputRequest {
  command: ChatCommandContext;
  sessionId: ProviderSessionId;
  requestId: ChatRequestId;
  providerRequestId: ProviderRequestId;
  answers: UserInputAnswer[];
}

export interface RollbackRequest {
  command: ChatCommandContext;
  sessionId: ProviderSessionId;
  checkpointId: ChatCheckpointId | null;
  providerCursor: VersionedJson | null;
  targetTurnId: ChatTurnId | null;
}

export interface ReadHistoryRequest {
  sessionId: ProviderSessionId;
  cursor: string | null;
  limit: number;
}

export interface StopSessionRequest {
  sessionId: ProviderSessionId;
  force: boolean;
}

export interface ContinuationGroupRequest {
  providerInstanceId: ProviderInstanceId;
  normalizedProviderHome: string | null;
  accountIdentity: string | null;
  serverIdentity: string | null;
  providerFields: Record<string, string>;
}
