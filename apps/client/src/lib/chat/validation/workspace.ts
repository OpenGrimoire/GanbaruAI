import {
  REPOSITORY_KINDS,
  WORKSPACE_BINDING_STATUSES,
  type ChatWorkspaceRead,
  type LogicalChatWorkspace,
} from "../contracts";
import {
  readEnum,
  readIdentifier,
  readNonNegativeSafeInteger,
  readNullable,
  readRecord,
  readString,
  readUtcTimestamp,
} from "./readers";

export function parseLogicalChatWorkspace(value: unknown, label = "workspace"): LogicalChatWorkspace {
  const record = readRecord(value, label);
  return {
    id: readIdentifier(record.id, `${label}.id`),
    projectId: readNullable(record.projectId, `${label}.projectId`, readIdentifier),
    displayName: readString(record.displayName, `${label}.displayName`),
    repositoryKind: readEnum(record.repositoryKind, REPOSITORY_KINDS, `${label}.repositoryKind`),
    repositoryIdentity: readNullable(record.repositoryIdentity, `${label}.repositoryIdentity`, readIdentifier),
    createdAt: readUtcTimestamp(record.createdAt, `${label}.createdAt`),
    updatedAt: readUtcTimestamp(record.updatedAt, `${label}.updatedAt`),
    archivedAt: readNullable(record.archivedAt, `${label}.archivedAt`, readUtcTimestamp),
    revision: readNonNegativeSafeInteger(record.revision, `${label}.revision`),
  };
}

export function parseChatWorkspaceRead(value: unknown, label = "workspaceRead"): ChatWorkspaceRead {
  const record = readRecord(value, label);
  return {
    workspace: parseLogicalChatWorkspace(record.workspace, `${label}.workspace`),
    bindingStatus: readEnum(record.bindingStatus, WORKSPACE_BINDING_STATUSES, `${label}.bindingStatus`),
    canonicalPath: readNullable(record.canonicalPath, `${label}.canonicalPath`, readString),
    lastVerifiedAt: readNullable(record.lastVerifiedAt, `${label}.lastVerifiedAt`, readUtcTimestamp),
    currentBranch: readNullable(record.currentBranch, `${label}.currentBranch`, readString),
  };
}

export function parseChatWorkspaceReads(value: unknown): ChatWorkspaceRead[] {
  if (!Array.isArray(value)) throw new Error("Chat workspaces must be an array");
  return value.map((workspace, index) => parseChatWorkspaceRead(workspace, `workspaces[${index}]`));
}
