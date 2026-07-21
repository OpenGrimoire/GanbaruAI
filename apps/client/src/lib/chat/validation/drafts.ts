import type { ChatDraftRead } from "../contracts";
import { INTERACTION_MODES, SAFETY_MODES } from "../contracts";
import {
  readArray,
  readEnum,
  readIdentifier,
  readNullable,
  readRecord,
  readString,
  readUtcTimestamp,
  readVersionedJson,
} from "./readers";

export function parseChatDraftRead(value: unknown): ChatDraftRead {
  const record = readRecord(value, "Chat draft");
  const mentions = readVersionedJson(record.mentions, "Chat draft.mentions");
  if (!Array.isArray(mentions.value)) throw new Error("Chat draft.mentions.value must be an array");
  return {
    id: readIdentifier(record.id, "Chat draft.id"),
    workspaceId: readIdentifier(record.workspaceId, "Chat draft.workspaceId"),
    threadId: readNullable(record.threadId, "Chat draft.threadId", readIdentifier),
    text: readString(record.text, "Chat draft.text"),
    attachmentIds: readArray(record.attachmentIds, "Chat draft.attachmentIds", readIdentifier),
    mentions,
    providerInstanceId: readNullable(record.providerInstanceId, "Chat draft.providerInstanceId", readIdentifier),
    modelSelection: readNullable(record.modelSelection, "Chat draft.modelSelection", readVersionedJson),
    safetyMode: readNullable(record.safetyMode, "Chat draft.safetyMode", (entry, label) => readEnum(entry, SAFETY_MODES, label)),
    interactionMode: readNullable(record.interactionMode, "Chat draft.interactionMode", (entry, label) => readEnum(entry, INTERACTION_MODES, label)),
    sentSnapshot: readNullable(record.sentSnapshot, "Chat draft.sentSnapshot", readVersionedJson),
    updatedAt: readUtcTimestamp(record.updatedAt, "Chat draft.updatedAt"),
  };
}
