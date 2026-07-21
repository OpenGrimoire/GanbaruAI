import type { ChatWorkspaceId, RepositoryKind, UtcTimestamp } from "./common";

export const WORKSPACE_BINDING_STATUSES = [
  "unbound",
  "available",
  "missing",
  "repository_mismatch",
] as const;
export type WorkspaceBindingStatus = (typeof WORKSPACE_BINDING_STATUSES)[number];

export interface LogicalChatWorkspace {
  id: ChatWorkspaceId;
  projectId: string | null;
  displayName: string;
  repositoryKind: RepositoryKind;
  repositoryIdentity: string | null;
  createdAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
  archivedAt: UtcTimestamp | null;
  revision: number;
}

export interface CreateChatWorkspaceRequest {
  id: ChatWorkspaceId;
  projectId: string | null;
  displayName: string;
}

export interface ChatWorkspaceRead {
  workspace: LogicalChatWorkspace;
  bindingStatus: WorkspaceBindingStatus;
  canonicalPath: string | null;
  lastVerifiedAt: UtcTimestamp | null;
  currentBranch: string | null;
}
