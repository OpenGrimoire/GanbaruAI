import type {
  ChatChannelId,
  ChatThreadId,
  ModelId,
  ProjectWorkingFolderId,
  ProviderInstanceId,
  UtcTimestamp,
} from "./common";
import type { ModelOptionSelection } from "./provider";
import type { ChatThreadShellRead, ChatTimelineItemRead, ChatTimelineTurnRead } from "./reads";

export interface ChatChannelTarget {
  workingFolderId: ProjectWorkingFolderId | null;
  providerInstanceId: ProviderInstanceId | null;
  providerManagedModel: boolean;
  modelId: ModelId | null;
  modelOptions: ModelOptionSelection[];
}

export interface ChatChannelRead {
  id: ChatChannelId;
  projectId: string;
  name: string;
  topic: string;
  isDefault: boolean;
  target: ChatChannelTarget;
  currentThread: ChatThreadShellRead | null;
  sessionCount: number;
  messageCount: number;
  latestPreview: string | null;
  lastActivityAt: UtcTimestamp;
  unreadAt: UtcTimestamp | null;
  revision: number;
  archivedAt: UtcTimestamp | null;
  createdAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
}

export interface ChatChannelSessionRead {
  channelId: ChatChannelId;
  ordinal: number;
  isCurrent: boolean;
  createdAt: UtcTimestamp;
  thread: ChatThreadShellRead;
}

export interface ChatChannelTimelinePageRead {
  channelId: ChatChannelId;
  sessions: ChatChannelSessionRead[];
  items: ChatTimelineItemRead[];
  turns: ChatTimelineTurnRead[];
  previousCursor: string | null;
  revision: number;
}

export interface CreateChatChannelRequest {
  id: ChatChannelId;
  projectId: string;
  name: string;
  topic: string;
  target: ChatChannelTarget;
}

export interface UpdateChatChannelDetailsRequest {
  channelId: ChatChannelId;
  name: string;
  topic: string;
  expectedRevision: number;
}

export interface UpdateChatChannelTargetRequest {
  channelId: ChatChannelId;
  target: ChatChannelTarget;
  expectedRevision: number;
}

export interface ChatChannelSessionBoundary {
  channelId: ChatChannelId;
  threadId: ChatThreadId;
  ordinal: number;
  createdAt: UtcTimestamp;
  target: ChatChannelTarget;
}
