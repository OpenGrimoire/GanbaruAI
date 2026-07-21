import type {
  ChatActivityId,
  ChatThreadId,
  ChatThreadState,
  ChatTurnId,
  ChatTurnState,
  ChatWorkspaceId,
  ContinuationGroupId,
  ModelId,
  ProviderFamilyId,
  ProviderInstanceId,
  ProviderItemId,
  ProviderSessionId,
  ProviderSessionState,
  ProviderThreadId,
  ProviderTurnId,
  TurnModeSnapshot,
  UtcTimestamp,
  VersionedJson,
} from "./common";
import type { ModelOptionSelection, ProviderCapabilities } from "./provider";

export interface ProviderSessionSnapshot {
  sessionId: ProviderSessionId;
  state: ProviderSessionState;
  providerThreadId: ProviderThreadId | null;
  continuationGroupId: ContinuationGroupId;
  resumeCursor: VersionedJson | null;
  effectiveModes: TurnModeSnapshot;
  capabilities: ProviderCapabilities;
  startedAt: UtcTimestamp;
}

export interface TurnDispatchReceipt {
  turnId: ChatTurnId;
  state: ChatTurnState;
  providerTurnId: ProviderTurnId | null;
  acceptedAt: UtcTimestamp;
}

export interface DriverOperationReceipt {
  accepted: boolean;
  operationId: string;
  detail: string | null;
}

export interface ProviderHistoryItem {
  providerItemId: ProviderItemId | null;
  providerTurnId: ProviderTurnId | null;
  kind: string;
  data: VersionedJson;
}

export interface ProviderHistoryPage {
  items: ProviderHistoryItem[];
  nextCursor: string | null;
}

export interface ChatThreadShellRead {
  id: ChatThreadId;
  workspaceId: ChatWorkspaceId;
  projectId: string | null;
  title: string;
  providerFamilyId: ProviderFamilyId;
  providerInstanceId: ProviderInstanceId;
  providerThreadId: ProviderThreadId | null;
  modelId: ModelId | null;
  modelOptions: ModelOptionSelection[];
  modes: TurnModeSnapshot;
  state: ChatThreadState;
  latestTurnState: ChatTurnState | null;
  latestPreview: string | null;
  messageCount: number;
  revision: number;
  lastEventSequence: number;
  lastActivityAt: UtcTimestamp;
  unreadAt: UtcTimestamp | null;
  archivedAt: UtcTimestamp | null;
}

export interface ChatProjectShellRead {
  projectId: string | null;
  workspaceId: ChatWorkspaceId;
  workspaceName: string;
  workspaceArchivedAt: UtcTimestamp | null;
  activeThreadCount: number;
  archivedThreadCount: number;
}

export interface ChatChangeNotification {
  threadId: ChatThreadId;
  sequence: number;
  revision: number;
  changedProjectionKeys: string[];
}

export interface ChatTimelineItemRead {
  activityId: ChatActivityId;
  turnId: ChatTurnId | null;
  sequenceAnchor: number;
  kind: string;
  data: VersionedJson;
}

export interface ChatTimelinePageRead {
  threadId: ChatThreadId;
  items: ChatTimelineItemRead[];
  previousCursor: string | null;
  nextCursor: string | null;
  threadRevision: number;
}
