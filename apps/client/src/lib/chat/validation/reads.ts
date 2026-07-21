import {
  CHAT_ERROR_CODES,
  CHAT_THREAD_STATES,
  CHAT_TURN_STATES,
  PROVIDER_SESSION_STATES,
  type ChatChangeNotification,
  type ChatError,
  type ChatThreadShellRead,
  type ChatProjectShellRead,
  type ChatTimelineItemRead,
  type ChatTimelinePageRead,
  type DriverOperationReceipt,
  type ProviderHistoryItem,
  type ProviderHistoryPage,
  type ProviderSessionSnapshot,
  type TurnDispatchReceipt,
} from "../contracts";
import { parseModelOptionSelection, parseProviderCapabilities } from "./provider";
import {
  readArray,
  readBoolean,
  readEnum,
  readIdentifier,
  readJsonValue,
  readNullable,
  readNonNegativeSafeInteger,
  readRecord,
  readString,
  readStringArray,
  readTurnModeSnapshot,
  readUtcTimestamp,
  readVersionedJson,
} from "./readers";

export function parseChatError(value: unknown, label = "Chat error"): ChatError {
  const record = readRecord(value, label);
  return {
    code: readEnum(record.code, CHAT_ERROR_CODES, `${label}.code`),
    message: readString(record.message, `${label}.message`),
    field: readNullable(record.field, `${label}.field`, readString),
    recoverable: readBoolean(record.recoverable, `${label}.recoverable`),
    details: readNullable(record.details, `${label}.details`, readJsonValue),
  };
}

export function parseChatThreadShells(value: unknown, label = "Chat thread shells"): ChatThreadShellRead[] {
  return readArray(value, label, parseChatThreadShell);
}

export function parseChatProjectShell(value: unknown, label = "Chat project shell"): ChatProjectShellRead {
  const record = readRecord(value, label);
  return {
    projectId: readNullable(record.projectId, `${label}.projectId`, readString),
    workspaceId: readIdentifier(record.workspaceId, `${label}.workspaceId`),
    workspaceName: readString(record.workspaceName, `${label}.workspaceName`),
    workspaceArchivedAt: readNullable(record.workspaceArchivedAt, `${label}.workspaceArchivedAt`, readUtcTimestamp),
    activeThreadCount: readNonNegativeSafeInteger(record.activeThreadCount, `${label}.activeThreadCount`),
    archivedThreadCount: readNonNegativeSafeInteger(record.archivedThreadCount, `${label}.archivedThreadCount`),
  };
}

export function parseChatProjectShells(value: unknown): ChatProjectShellRead[] {
  return readArray(value, "Chat project shells", parseChatProjectShell);
}

export function parseProviderSessionSnapshot(value: unknown, label = "provider session"): ProviderSessionSnapshot {
  const record = readRecord(value, label);
  return {
    sessionId: readIdentifier(record.sessionId, `${label}.sessionId`),
    state: readEnum(record.state, PROVIDER_SESSION_STATES, `${label}.state`),
    providerThreadId: readNullable(record.providerThreadId, `${label}.providerThreadId`, readIdentifier),
    continuationGroupId: readIdentifier(record.continuationGroupId, `${label}.continuationGroupId`),
    resumeCursor: readNullable(record.resumeCursor, `${label}.resumeCursor`, readVersionedJson),
    effectiveModes: readTurnModeSnapshot(record.effectiveModes, `${label}.effectiveModes`),
    capabilities: parseProviderCapabilities(record.capabilities, `${label}.capabilities`),
    startedAt: readUtcTimestamp(record.startedAt, `${label}.startedAt`),
  };
}

export function parseTurnDispatchReceipt(value: unknown, label = "turn receipt"): TurnDispatchReceipt {
  const record = readRecord(value, label);
  return {
    turnId: readIdentifier(record.turnId, `${label}.turnId`),
    state: readEnum(record.state, CHAT_TURN_STATES, `${label}.state`),
    providerTurnId: readNullable(record.providerTurnId, `${label}.providerTurnId`, readIdentifier),
    acceptedAt: readUtcTimestamp(record.acceptedAt, `${label}.acceptedAt`),
  };
}

export function parseDriverOperationReceipt(value: unknown, label = "driver receipt"): DriverOperationReceipt {
  const record = readRecord(value, label);
  return {
    accepted: readBoolean(record.accepted, `${label}.accepted`),
    operationId: readString(record.operationId, `${label}.operationId`),
    detail: readNullable(record.detail, `${label}.detail`, readString),
  };
}

function parseProviderHistoryItem(value: unknown, label: string): ProviderHistoryItem {
  const record = readRecord(value, label);
  return {
    providerItemId: readNullable(record.providerItemId, `${label}.providerItemId`, readIdentifier),
    providerTurnId: readNullable(record.providerTurnId, `${label}.providerTurnId`, readIdentifier),
    kind: readString(record.kind, `${label}.kind`),
    data: readVersionedJson(record.data, `${label}.data`),
  };
}

export function parseProviderHistoryPage(value: unknown, label = "provider history"): ProviderHistoryPage {
  const record = readRecord(value, label);
  return {
    items: readArray(record.items, `${label}.items`, parseProviderHistoryItem),
    nextCursor: readNullable(record.nextCursor, `${label}.nextCursor`, readString),
  };
}

export function parseChatThreadShell(value: unknown, label = "Chat thread shell"): ChatThreadShellRead {
  const record = readRecord(value, label);
  return {
    id: readIdentifier(record.id, `${label}.id`),
    workspaceId: readIdentifier(record.workspaceId, `${label}.workspaceId`),
    projectId: readNullable(record.projectId, `${label}.projectId`, readString),
    title: readString(record.title, `${label}.title`),
    providerFamilyId: readIdentifier(record.providerFamilyId, `${label}.providerFamilyId`),
    providerInstanceId: readIdentifier(record.providerInstanceId, `${label}.providerInstanceId`),
    providerThreadId: readNullable(record.providerThreadId, `${label}.providerThreadId`, readIdentifier),
    modelId: readNullable(record.modelId, `${label}.modelId`, readIdentifier),
    modelOptions: readArray(record.modelOptions, `${label}.modelOptions`, parseModelOptionSelection),
    modes: readTurnModeSnapshot(record.modes, `${label}.modes`),
    state: readEnum(record.state, CHAT_THREAD_STATES, `${label}.state`),
    latestTurnState: readNullable(
      record.latestTurnState,
      `${label}.latestTurnState`,
      (entry, entryLabel) => readEnum(entry, CHAT_TURN_STATES, entryLabel),
    ),
    latestPreview: readNullable(record.latestPreview, `${label}.latestPreview`, readString),
    messageCount: readNonNegativeSafeInteger(record.messageCount, `${label}.messageCount`),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
    lastEventSequence: readNonNegativeSafeInteger(record.lastEventSequence, `${label}.lastEventSequence`),
    lastActivityAt: readUtcTimestamp(record.lastActivityAt, `${label}.lastActivityAt`),
    unreadAt: readNullable(record.unreadAt, `${label}.unreadAt`, readUtcTimestamp),
    archivedAt: readNullable(record.archivedAt, `${label}.archivedAt`, readUtcTimestamp),
  };
}

export function parseChatChangeNotification(value: unknown, label = "Chat change notification"): ChatChangeNotification {
  const record = readRecord(value, label);
  return {
    threadId: readIdentifier(record.threadId, `${label}.threadId`),
    sequence: readNonNegativeSafeInteger(record.sequence, `${label}.sequence`),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
    changedProjectionKeys: readStringArray(record.changedProjectionKeys, `${label}.changedProjectionKeys`),
  };
}

function parseChatTimelineItem(value: unknown, label: string): ChatTimelineItemRead {
  const record = readRecord(value, label);
  return {
    activityId: readIdentifier(record.activityId, `${label}.activityId`),
    turnId: readNullable(record.turnId, `${label}.turnId`, readIdentifier),
    sequenceAnchor: readNonNegativeSafeInteger(record.sequenceAnchor, `${label}.sequenceAnchor`),
    kind: readString(record.kind, `${label}.kind`),
    data: readVersionedJson(record.data, `${label}.data`),
  };
}

export function parseChatTimelinePage(value: unknown, label = "Chat timeline page"): ChatTimelinePageRead {
  const record = readRecord(value, label);
  return {
    threadId: readIdentifier(record.threadId, `${label}.threadId`),
    items: readArray(record.items, `${label}.items`, parseChatTimelineItem),
    previousCursor: readNullable(record.previousCursor, `${label}.previousCursor`, readString),
    nextCursor: readNullable(record.nextCursor, `${label}.nextCursor`, readString),
    threadRevision: readNonNegativeSafeInteger(record.threadRevision, `${label}.threadRevision`),
  };
}
