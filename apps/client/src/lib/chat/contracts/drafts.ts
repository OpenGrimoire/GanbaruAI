import type {
  ChatAttachmentId,
  ChatThreadId,
  ProjectWorkingFolderId,
  InteractionMode,
  ProviderInstanceId,
  SafetyMode,
  UtcTimestamp,
  VersionedJson,
} from "./common";

export const CHAT_DRAFT_SCHEMA_VERSION = 1;

export interface ChatDraftMention {
  relativePath: string;
  kind: "file" | "directory";
  ignored: boolean;
}

export interface ChatDraftModelSelection {
  modelId: string | null;
  providerManaged: boolean;
  options: VersionedJson;
}

export interface ChatSentDraftSnapshot {
  text: string;
  attachmentIds: ChatAttachmentId[];
  mentions: ChatDraftMention[];
}

export interface ChatDraftRead {
  id: string;
  workingFolderId: ProjectWorkingFolderId;
  threadId: ChatThreadId | null;
  text: string;
  richContent: VersionedJson | null;
  attachmentIds: ChatAttachmentId[];
  mentions: VersionedJson;
  providerInstanceId: ProviderInstanceId | null;
  modelSelection: VersionedJson | null;
  safetyMode: SafetyMode | null;
  interactionMode: InteractionMode | null;
  sentSnapshot: VersionedJson | null;
  updatedAt: UtcTimestamp;
}

export type SaveChatDraftRequest = Omit<ChatDraftRead, "updatedAt">;
