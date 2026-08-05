import {
  CHAT_APPROVAL_POLICIES,
  CHAT_PARTICIPANT_KINDS,
  CHAT_SCHEDULED_MESSAGE_STATES,
  CHAT_TEAMMATE_CONFIGURATION_STATES,
  CHAT_WORK_ASSIGNMENT_STATES,
  type ChatAgentRunRead,
  type ChatAiTeammateRead,
  type ChatChannelPageRead,
  type ChatConversationMembershipRead,
  type ChatMessageRead,
  type ChatMessageSearchResultRead,
  type ChatParticipantMentionRead,
  type ChatParticipantRead,
  type ChatProjectPrimaryWorkingFolderRead,
  type ChatReplyThreadPageRead,
  type ChatReplyThreadSummaryRead,
  type ChatResourceReferenceRead,
  type ChatScheduledMessageDispatchRead,
  type ChatScheduledMessageRead,
  type ChatTeammatePolicyRead,
  type ChatWorkAssignmentRead,
  type ChatWorkingFolderGrantRead,
  type PostChatMessageResult,
} from "../contracts";
import { parseModelOptionSelection } from "./provider";
import {
  readArray,
  readBoolean,
  readEnum,
  readIdentifier,
  readNullable,
  readNonNegativeSafeInteger,
  readRecord,
  readString,
  readUtcTimestamp,
  readVersionedJson,
} from "./readers";

const AGENT_RUN_STATES = [
  "queued", "starting", "working", "waiting", "completed", "failed", "cancelled",
] as const;

export function parseChatParticipant(value: unknown, label = "Chat participant"): ChatParticipantRead {
  const record = readRecord(value, label);
  return {
    id: readIdentifier(record.id, `${label}.id`),
    kind: readEnum(record.kind, CHAT_PARTICIPANT_KINDS, `${label}.kind`),
    displayName: readString(record.displayName, `${label}.displayName`),
    handle: readNullable(record.handle, `${label}.handle`, readString),
    avatar: readVersionedJson(record.avatar, `${label}.avatar`),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
    archivedAt: readNullable(record.archivedAt, `${label}.archivedAt`, readUtcTimestamp),
  };
}

export function parseChatTeammatePolicy(value: unknown, label = "Chat teammate policy"): ChatTeammatePolicyRead {
  const record = readRecord(value, label);
  return {
    id: readIdentifier(record.id, `${label}.id`),
    teammateId: readIdentifier(record.teammateId, `${label}.teammateId`),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
    providerInstanceId: readIdentifier(record.providerInstanceId, `${label}.providerInstanceId`),
    providerManagedModel: readBoolean(record.providerManagedModel, `${label}.providerManagedModel`),
    modelId: readNullable(record.modelId, `${label}.modelId`, readIdentifier),
    modelOptions: readArray(record.modelOptions, `${label}.modelOptions`, parseModelOptionSelection),
    effort: readNullable(record.effort, `${label}.effort`, readString),
    speed: readNullable(record.speed, `${label}.speed`, readString),
    providerOptions: readVersionedJson(record.providerOptions, `${label}.providerOptions`),
    createdAt: readUtcTimestamp(record.createdAt, `${label}.createdAt`),
  };
}

export function parseChatAiTeammate(value: unknown, label = "Chat AI teammate"): ChatAiTeammateRead {
  const record = readRecord(value, label);
  return {
    participant: parseChatParticipant(record.participant, `${label}.participant`),
    purpose: readString(record.purpose, `${label}.purpose`),
    instructions: readString(record.instructions, `${label}.instructions`),
    configurationState: readEnum(
      record.configurationState,
      CHAT_TEAMMATE_CONFIGURATION_STATES,
      `${label}.configurationState`,
    ),
    latestPolicy: readNullable(record.latestPolicy, `${label}.latestPolicy`, parseChatTeammatePolicy),
    channelCount: readNonNegativeSafeInteger(record.channelCount, `${label}.channelCount`),
  };
}

export function parseChatAiTeammates(value: unknown, label = "Chat AI teammates"): ChatAiTeammateRead[] {
  return readArray(value, label, parseChatAiTeammate);
}

function parseWorkingFolderGrant(value: unknown, label: string): ChatWorkingFolderGrantRead {
  const record = readRecord(value, label);
  return {
    workingFolderId: readIdentifier(record.workingFolderId, `${label}.workingFolderId`),
    displayName: readString(record.displayName, `${label}.displayName`),
    isDefault: readBoolean(record.isDefault, `${label}.isDefault`),
  };
}

export function parseChatConversationMembership(
  value: unknown,
  label = "Chat conversation membership",
): ChatConversationMembershipRead {
  const record = readRecord(value, label);
  return {
    conversationId: readIdentifier(record.conversationId, `${label}.conversationId`),
    participant: parseChatParticipant(record.participant, `${label}.participant`),
    addressable: readBoolean(record.addressable, `${label}.addressable`),
    approvalPolicy: readEnum(record.approvalPolicy, CHAT_APPROVAL_POLICIES, `${label}.approvalPolicy`),
    workingFolderGrants: readArray(
      record.workingFolderGrants,
      `${label}.workingFolderGrants`,
      parseWorkingFolderGrant,
    ),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
    removedAt: readNullable(record.removedAt, `${label}.removedAt`, readUtcTimestamp),
  };
}

export function parseChatConversationMemberships(
  value: unknown,
  label = "Chat conversation memberships",
): ChatConversationMembershipRead[] {
  return readArray(value, label, parseChatConversationMembership);
}

export function parseChatProjectPrimaryWorkingFolder(
  value: unknown,
  label = "Chat project primary working folder",
): ChatProjectPrimaryWorkingFolderRead {
  const record = readRecord(value, label);
  return {
    projectId: readIdentifier(record.projectId, `${label}.projectId`),
    workingFolderId: readIdentifier(record.workingFolderId, `${label}.workingFolderId`),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
  };
}

function parseParticipantMention(value: unknown, label: string): ChatParticipantMentionRead {
  const record = readRecord(value, label);
  return {
    participantId: readIdentifier(record.participantId, `${label}.participantId`),
    participantKind: readEnum(record.participantKind, CHAT_PARTICIPANT_KINDS, `${label}.participantKind`),
    handleSnapshot: readNullable(record.handleSnapshot, `${label}.handleSnapshot`, readString),
    labelSnapshot: readString(record.labelSnapshot, `${label}.labelSnapshot`),
    startOffset: readNonNegativeSafeInteger(record.startOffset, `${label}.startOffset`),
    endOffset: readNonNegativeSafeInteger(record.endOffset, `${label}.endOffset`),
  };
}

function parseResourceReference(value: unknown, label: string): ChatResourceReferenceRead {
  const record = readRecord(value, label);
  return {
    workingFolderId: readIdentifier(record.workingFolderId, `${label}.workingFolderId`),
    kind: readEnum(record.kind, ["file", "folder"] as const, `${label}.kind`),
    relativePath: readString(record.relativePath, `${label}.relativePath`),
    displayLabel: readString(record.displayLabel, `${label}.displayLabel`),
  };
}

export function parseChatReplyThreadSummary(
  value: unknown,
  label = "Chat reply thread summary",
): ChatReplyThreadSummaryRead {
  const record = readRecord(value, label);
  return {
    id: readIdentifier(record.id, `${label}.id`),
    replyCount: readNonNegativeSafeInteger(record.replyCount, `${label}.replyCount`),
    lastActivityAt: readUtcTimestamp(record.lastActivityAt, `${label}.lastActivityAt`),
    participants: readArray(record.participants, `${label}.participants`, parseChatParticipant),
    unread: readBoolean(record.unread, `${label}.unread`),
    workState: readNullable(
      record.workState,
      `${label}.workState`,
      (entry, entryLabel) => readEnum(entry, CHAT_WORK_ASSIGNMENT_STATES, entryLabel),
    ),
  };
}

export function parseChatMessage(value: unknown, label = "Chat message"): ChatMessageRead {
  const record = readRecord(value, label);
  return {
    itemId: readIdentifier(record.itemId, `${label}.itemId`),
    conversationId: readIdentifier(record.conversationId, `${label}.conversationId`),
    replyThreadId: readNullable(record.replyThreadId, `${label}.replyThreadId`, readIdentifier),
    revisionId: readIdentifier(record.revisionId, `${label}.revisionId`),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
    author: parseChatParticipant(record.author, `${label}.author`),
    normalizedMarkdown: readString(record.normalizedMarkdown, `${label}.normalizedMarkdown`),
    richContent: readVersionedJson(record.richContent, `${label}.richContent`),
    mentions: readArray(record.mentions, `${label}.mentions`, parseParticipantMention),
    attachmentIds: readArray(record.attachmentIds, `${label}.attachmentIds`, readIdentifier),
    resourceReferences: readArray(record.resourceReferences, `${label}.resourceReferences`, parseResourceReference),
    replyThread: readNullable(record.replyThread, `${label}.replyThread`, parseChatReplyThreadSummary),
    ordinal: readNonNegativeSafeInteger(record.ordinal, `${label}.ordinal`),
    editedAt: readNullable(record.editedAt, `${label}.editedAt`, readUtcTimestamp),
    createdAt: readUtcTimestamp(record.createdAt, `${label}.createdAt`),
  };
}

export function parseChatChannelPage(value: unknown, label = "Chat channel page"): ChatChannelPageRead {
  const record = readRecord(value, label);
  return {
    channelId: readIdentifier(record.channelId, `${label}.channelId`),
    messages: readArray(record.messages, `${label}.messages`, parseChatMessage),
    previousCursor: readNullable(record.previousCursor, `${label}.previousCursor`, readString),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
  };
}

export function parseChatWorkAssignment(
  value: unknown,
  label = "Chat work assignment",
): ChatWorkAssignmentRead {
  const record = readRecord(value, label);
  return {
    id: readIdentifier(record.id, `${label}.id`),
    replyThreadId: readIdentifier(record.replyThreadId, `${label}.replyThreadId`),
    teammate: parseChatParticipant(record.teammate, `${label}.teammate`),
    triggeringMessageItemId: readIdentifier(record.triggeringMessageItemId, `${label}.triggeringMessageItemId`),
    previousAssignmentId: readNullable(record.previousAssignmentId, `${label}.previousAssignmentId`, readIdentifier),
    state: readEnum(record.state, CHAT_WORK_ASSIGNMENT_STATES, `${label}.state`),
    stateReason: readNullable(record.stateReason, `${label}.stateReason`, readString),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
    settledAt: readNullable(record.settledAt, `${label}.settledAt`, readUtcTimestamp),
    createdAt: readUtcTimestamp(record.createdAt, `${label}.createdAt`),
    updatedAt: readUtcTimestamp(record.updatedAt, `${label}.updatedAt`),
  };
}

function parseAgentRun(value: unknown, label: string): ChatAgentRunRead {
  const record = readRecord(value, label);
  return {
    id: readIdentifier(record.id, `${label}.id`),
    assignmentId: readIdentifier(record.assignmentId, `${label}.assignmentId`),
    projectId: readIdentifier(record.projectId, `${label}.projectId`),
    workingFolderId: readIdentifier(record.workingFolderId, `${label}.workingFolderId`),
    teammatePolicyRevisionId: readIdentifier(record.teammatePolicyRevisionId, `${label}.teammatePolicyRevisionId`),
    effort: readNullable(record.effort, `${label}.effort`, readString),
    providerExecutionTurnId: readIdentifier(record.providerExecutionTurnId, `${label}.providerExecutionTurnId`),
    providerExecutionThreadId: readNullable(
      record.providerExecutionThreadId,
      `${label}.providerExecutionThreadId`,
      readIdentifier,
    ),
    state: readEnum(record.state, AGENT_RUN_STATES, `${label}.state`),
    runOrdinal: readNonNegativeSafeInteger(record.runOrdinal, `${label}.runOrdinal`),
    createdAt: readUtcTimestamp(record.createdAt, `${label}.createdAt`),
    updatedAt: readUtcTimestamp(record.updatedAt, `${label}.updatedAt`),
  };
}

export function parseChatReplyThreadPage(
  value: unknown,
  label = "Chat reply thread page",
): ChatReplyThreadPageRead {
  const record = readRecord(value, label);
  return {
    thread: parseChatReplyThreadSummary(record.thread, `${label}.thread`),
    rootMessage: parseChatMessage(record.rootMessage, `${label}.rootMessage`),
    replies: readArray(record.replies, `${label}.replies`, parseChatMessage),
    assignment: readNullable(record.assignment, `${label}.assignment`, parseChatWorkAssignment),
    agentRuns: readArray(record.agentRuns, `${label}.agentRuns`, parseAgentRun),
    previousCursor: readNullable(record.previousCursor, `${label}.previousCursor`, readString),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
  };
}

export function parsePostChatMessageResult(
  value: unknown,
  label = "Post Chat message result",
): PostChatMessageResult {
  const record = readRecord(value, label);
  return {
    message: parseChatMessage(record.message, `${label}.message`),
    replyThreadId: readNullable(record.replyThreadId, `${label}.replyThreadId`, readIdentifier),
    assignment: readNullable(record.assignment, `${label}.assignment`, parseChatWorkAssignment),
    assignmentInputQueued: readBoolean(record.assignmentInputQueued, `${label}.assignmentInputQueued`),
  };
}

export function parseChatScheduledMessage(
  value: unknown,
  label = "Chat scheduled message",
): ChatScheduledMessageRead {
  const record = readRecord(value, label);
  return {
    id: readIdentifier(record.id, `${label}.id`),
    channelId: readIdentifier(record.channelId, `${label}.channelId`),
    replyThreadId: readNullable(record.replyThreadId, `${label}.replyThreadId`, readIdentifier),
    normalizedMarkdown: readString(record.normalizedMarkdown, `${label}.normalizedMarkdown`),
    richContent: readVersionedJson(record.richContent, `${label}.richContent`),
    attachmentIds: readArray(record.attachmentIds, `${label}.attachmentIds`, readIdentifier),
    participantMentions: readArray(
      record.participantMentions,
      `${label}.participantMentions`,
      parseParticipantMention,
    ),
    resourceReferences: readArray(
      record.resourceReferences,
      `${label}.resourceReferences`,
      parseResourceReference,
    ),
    alsoSendToChannel: readBoolean(record.alsoSendToChannel, `${label}.alsoSendToChannel`),
    state: readEnum(record.state, CHAT_SCHEDULED_MESSAGE_STATES, `${label}.state`),
    scheduledFor: readUtcTimestamp(record.scheduledFor, `${label}.scheduledFor`),
    lastError: readNullable(record.lastError, `${label}.lastError`, readString),
    createdAt: readUtcTimestamp(record.createdAt, `${label}.createdAt`),
  };
}

export function parseChatScheduledMessages(
  value: unknown,
  label = "Chat scheduled messages",
): ChatScheduledMessageRead[] {
  return readArray(value, label, parseChatScheduledMessage);
}

export function parseChatScheduledMessageDispatch(
  value: unknown,
  label = "Chat scheduled message dispatch",
): ChatScheduledMessageDispatchRead {
  const record = readRecord(value, label);
  return {
    processedCount: readNonNegativeSafeInteger(record.processedCount, `${label}.processedCount`),
    dispatchedCount: readNonNegativeSafeInteger(record.dispatchedCount, `${label}.dispatchedCount`),
    dispatchedChannelIds: readArray(record.dispatchedChannelIds, `${label}.dispatchedChannelIds`, readIdentifier),
    nextDispatchAt: readNullable(record.nextDispatchAt, `${label}.nextDispatchAt`, readUtcTimestamp),
  };
}

export function parseChatMessageSearchResult(
  value: unknown,
  label = "Chat message search result",
): ChatMessageSearchResultRead {
  const record = readRecord(value, label);
  return {
    projectId: readIdentifier(record.projectId, `${label}.projectId`),
    channelId: readIdentifier(record.channelId, `${label}.channelId`),
    channelName: readString(record.channelName, `${label}.channelName`),
    conversationId: readIdentifier(record.conversationId, `${label}.conversationId`),
    replyThreadId: readNullable(record.replyThreadId, `${label}.replyThreadId`, readIdentifier),
    messageItemId: readIdentifier(record.messageItemId, `${label}.messageItemId`),
    ordinal: readNonNegativeSafeInteger(record.ordinal, `${label}.ordinal`),
    authorParticipantId: readIdentifier(record.authorParticipantId, `${label}.authorParticipantId`),
    authorKind: readEnum(record.authorKind, CHAT_PARTICIPANT_KINDS, `${label}.authorKind`),
    authorDisplayName: readString(record.authorDisplayName, `${label}.authorDisplayName`),
    excerpt: readString(record.excerpt, `${label}.excerpt`),
    createdAt: readUtcTimestamp(record.createdAt, `${label}.createdAt`),
  };
}

export function parseChatMessageSearchResults(
  value: unknown,
  label = "Chat message search results",
): ChatMessageSearchResultRead[] {
  return readArray(value, label, parseChatMessageSearchResult);
}
