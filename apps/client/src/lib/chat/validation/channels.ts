import type {
  ChatChannelRead,
  ChatChannelSessionRead,
  ChatChannelTarget,
  ChatChannelTimelinePageRead,
} from "../contracts";
import { parseModelOptionSelection } from "./provider";
import { parseChatThreadShell } from "./reads";
import { parseChatTimelineItem, parseChatTimelineTurn } from "./reads";
import {
  readArray,
  readBoolean,
  readIdentifier,
  readNullable,
  readNonNegativeSafeInteger,
  readRecord,
  readString,
  readUtcTimestamp,
} from "./readers";

export function parseChatChannelTarget(value: unknown, label = "Chat channel target"): ChatChannelTarget {
  const record = readRecord(value, label);
  return {
    workingFolderId: readNullable(record.workingFolderId, `${label}.workingFolderId`, readIdentifier),
    providerInstanceId: readNullable(record.providerInstanceId, `${label}.providerInstanceId`, readIdentifier),
    providerManagedModel: readBoolean(record.providerManagedModel, `${label}.providerManagedModel`),
    modelId: readNullable(record.modelId, `${label}.modelId`, readIdentifier),
    modelOptions: readArray(record.modelOptions, `${label}.modelOptions`, parseModelOptionSelection),
  };
}

export function parseChatChannel(value: unknown, label = "Chat channel"): ChatChannelRead {
  const record = readRecord(value, label);
  return {
    id: readIdentifier(record.id, `${label}.id`),
    projectId: readIdentifier(record.projectId, `${label}.projectId`),
    name: readString(record.name, `${label}.name`),
    topic: readString(record.topic, `${label}.topic`),
    isDefault: readBoolean(record.isDefault, `${label}.isDefault`),
    target: parseChatChannelTarget(record.target, `${label}.target`),
    currentThread: readNullable(record.currentThread, `${label}.currentThread`, parseChatThreadShell),
    sessionCount: readNonNegativeSafeInteger(record.sessionCount, `${label}.sessionCount`),
    messageCount: readNonNegativeSafeInteger(record.messageCount, `${label}.messageCount`),
    latestPreview: readNullable(record.latestPreview, `${label}.latestPreview`, readString),
    lastActivityAt: readUtcTimestamp(record.lastActivityAt, `${label}.lastActivityAt`),
    unreadAt: readNullable(record.unreadAt, `${label}.unreadAt`, readUtcTimestamp),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
    archivedAt: readNullable(record.archivedAt, `${label}.archivedAt`, readUtcTimestamp),
    createdAt: readUtcTimestamp(record.createdAt, `${label}.createdAt`),
    updatedAt: readUtcTimestamp(record.updatedAt, `${label}.updatedAt`),
  };
}

export function parseChatChannels(value: unknown, label = "Chat channels"): ChatChannelRead[] {
  return readArray(value, label, parseChatChannel);
}

export function parseChatChannelSession(value: unknown, label = "Chat channel session"): ChatChannelSessionRead {
  const record = readRecord(value, label);
  return {
    channelId: readIdentifier(record.channelId, `${label}.channelId`),
    ordinal: readNonNegativeSafeInteger(record.ordinal, `${label}.ordinal`),
    isCurrent: readBoolean(record.isCurrent, `${label}.isCurrent`),
    createdAt: readUtcTimestamp(record.createdAt, `${label}.createdAt`),
    thread: parseChatThreadShell(record.thread, `${label}.thread`),
  };
}

export function parseChatChannelSessions(value: unknown, label = "Chat channel sessions"): ChatChannelSessionRead[] {
  return readArray(value, label, parseChatChannelSession);
}

export function parseChatChannelTimelinePage(
  value: unknown,
  label = "Chat channel timeline page",
): ChatChannelTimelinePageRead {
  const record = readRecord(value, label);
  return {
    channelId: readIdentifier(record.channelId, `${label}.channelId`),
    sessions: readArray(record.sessions, `${label}.sessions`, parseChatChannelSession),
    items: readArray(record.items, `${label}.items`, parseChatTimelineItem),
    turns: readArray(record.turns, `${label}.turns`, parseChatTimelineTurn),
    previousCursor: readNullable(record.previousCursor, `${label}.previousCursor`, readString),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
  };
}
