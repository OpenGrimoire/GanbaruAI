import type { ChatApprovalPolicy, ModelOptionSelection } from "$lib/chat/contracts";

export interface TeammateProfileDraftSnapshotInput {
  displayName: string;
  role: string;
  instructions: string;
  providerId: string;
  modelId: string;
  modelOptions: readonly ModelOptionSelection[];
  effort: string;
}

export interface TeammateMembershipDraftSnapshotInput {
  channelId: string;
  approvalPolicy: ChatApprovalPolicy;
  folderIds: readonly string[];
  defaultFolderId: string;
}

/**
 * Creates a canonical snapshot of persisted teammate profile and execution fields.
 */
export function teammateProfileDraftSnapshot(
  input: TeammateProfileDraftSnapshotInput,
): string {
  const modelOptions = [...input.modelOptions].sort((left, right) => left.key.localeCompare(right.key));
  return JSON.stringify({
    displayName: input.displayName.trim(),
    role: input.role.trim(),
    instructions: input.instructions.trim(),
    providerId: input.providerId,
    modelId: input.modelId,
    modelOptions,
    effort: input.effort,
  });
}

/**
 * Creates a canonical snapshot of the membership fields saved for one channel.
 */
export function teammateMembershipDraftSnapshot(
  input: TeammateMembershipDraftSnapshotInput,
): string {
  const folderIds = [...new Set([
    ...input.folderIds,
    input.defaultFolderId,
  ].filter(Boolean))].sort();
  return JSON.stringify({
    channelId: input.channelId,
    approvalPolicy: input.approvalPolicy,
    folderIds,
    defaultFolderId: input.defaultFolderId,
  });
}
