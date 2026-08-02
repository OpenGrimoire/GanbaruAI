import type {
  ChatChannelId,
  ChatConversationId,
  UtcTimestamp,
} from "./common";
import type {
  ChatConversationMembershipRead,
  ChatWorkAssignmentState,
} from "./organizational";

export interface ChatChannelRead {
  id: ChatChannelId;
  conversationId: ChatConversationId;
  projectId: string;
  name: string;
  topic: string;
  isDefault: boolean;
  memberships: ChatConversationMembershipRead[];
  messageCount: number;
  unreadCount: number;
  latestPreview: string | null;
  lastActivityAt: UtcTimestamp;
  attentionState: ChatWorkAssignmentState | null;
  revision: number;
  archivedAt: UtcTimestamp | null;
  createdAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
}

export interface CreateChatChannelRequest {
  id: ChatChannelId;
  projectId: string;
  name: string;
  topic: string;
}

export interface UpdateChatChannelDetailsRequest {
  channelId: ChatChannelId;
  name: string;
  topic: string;
  expectedRevision: number;
}
