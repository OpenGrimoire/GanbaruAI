import type {
  ChatChannelCapabilities,
  ChatFolderCapability,
  ChatHistoryBoundary,
  ChatRuntimeApprovalPolicy,
  ChatTeammateChannelAccessInput,
} from "$lib/chat/contracts";

export type ChatChannelCapabilityPreset =
  | "contextSource"
  | "isolatedResponder"
  | "collaborator"
  | "custom";

export type SelectionState = "none" | "some" | "all";

const CAPABILITY_RANK: Record<ChatFolderCapability, number> = {
  none: 0,
  read: 1,
  edit: 2,
  execute: 3,
  publish: 4,
};

/** Returns whether a requested folder capability fits inside a profile ceiling. */
export function folderCapabilityFits(
  requested: ChatFolderCapability,
  ceiling: ChatFolderCapability,
): boolean {
  return CAPABILITY_RANK[requested] <= CAPABILITY_RANK[ceiling];
}

/** Returns a deterministic preset name for independent channel capabilities. */
export function channelCapabilityPreset(
  capabilities: ChatChannelCapabilities,
): ChatChannelCapabilityPreset {
  if (capabilities.readHistory && capabilities.participate) return "collaborator";
  if (capabilities.readHistory) return "contextSource";
  if (capabilities.participate) return "isolatedResponder";
  return "custom";
}

/** Returns the capability pair represented by a quick preset. */
export function capabilitiesForPreset(
  preset: Exclude<ChatChannelCapabilityPreset, "custom">,
): ChatChannelCapabilities {
  if (preset === "contextSource") return { readHistory: true, participate: false };
  if (preset === "isolatedResponder") return { readHistory: false, participate: true };
  return { readHistory: true, participate: true };
}

/** Resolves an inherited runtime approval without letting an absent override widen it. */
export function resolveRuntimeApproval(
  teammateDefault: ChatRuntimeApprovalPolicy,
  channelOverride: ChatRuntimeApprovalPolicy | null,
  folderOverride: ChatRuntimeApprovalPolicy | null,
): { policy: ChatRuntimeApprovalPolicy; source: "teammate" | "channel" | "folder" } {
  if (folderOverride) return { policy: folderOverride, source: "folder" };
  if (channelOverride) return { policy: channelOverride, source: "channel" };
  return { policy: teammateDefault, source: "teammate" };
}

/** Calculates the native tri-state value for a bounded group of selectable channels. */
export function selectionState(ids: readonly string[], selectedIds: ReadonlySet<string>): SelectionState {
  if (ids.length === 0 || ids.every((id) => !selectedIds.has(id))) return "none";
  if (ids.every((id) => selectedIds.has(id))) return "all";
  return "some";
}

/** Updates a bounded selection. Future channels are intentionally unaffected. */
export function toggleSelectionGroup(
  ids: readonly string[],
  selectedIds: ReadonlySet<string>,
  selected: boolean,
): Set<string> {
  const next = new Set(selectedIds);
  for (const id of ids) {
    if (selected) next.add(id);
    else next.delete(id);
  }
  return next;
}

/** Creates a stable access snapshot for dirty checks and optimistic replacement. */
export function teammateAccessDraftSnapshot(
  channels: readonly ChatTeammateChannelAccessInput[],
): string {
  return JSON.stringify(
    [...channels]
      .map((channel) => ({
        ...channel,
        capabilities: {
          readHistory: channel.capabilities.readHistory,
          participate: channel.capabilities.participate,
        },
        historyBoundary: normalizeHistoryBoundary(channel.historyBoundary),
        folderGrants: [...channel.folderGrants]
          .map((grant) => ({ ...grant }))
          .sort((left, right) => left.workingFolderId.localeCompare(right.workingFolderId)),
      }))
      .sort((left, right) => left.channelId.localeCompare(right.channelId)),
  );
}

function normalizeHistoryBoundary(boundary: ChatHistoryBoundary): ChatHistoryBoundary {
  return boundary.kind === "entire"
    ? { kind: "entire" }
    : boundary.lowerOrdinal === undefined
      ? { kind: "fromGrant" }
      : { kind: "fromGrant", lowerOrdinal: boundary.lowerOrdinal };
}

/** Returns all configuration errors that must block an atomic access save. */
export function teammateAccessDraftErrors(
  channels: readonly ChatTeammateChannelAccessInput[],
  profileCeilings: ReadonlyMap<string, ChatFolderCapability>,
): string[] {
  const errors: string[] = [];
  const channelIds = new Set<string>();
  for (const channel of channels) {
    if (channelIds.has(channel.channelId)) errors.push(`Duplicate channel ${channel.channelId}`);
    channelIds.add(channel.channelId);
    if (!channel.capabilities.readHistory && channel.historyBoundary.kind !== "entire") {
      errors.push(`Channel ${channel.channelId} has a history boundary without history access`);
    }
    const ceiling = profileCeilings.get(channel.accessProfileId);
    if (!ceiling) errors.push(`Channel ${channel.channelId} uses an unavailable access profile`);
    const folderIds = new Set<string>();
    let defaultCount = 0;
    for (const grant of channel.folderGrants) {
      if (folderIds.has(grant.workingFolderId)) {
        errors.push(`Channel ${channel.channelId} grants folder ${grant.workingFolderId} twice`);
      }
      folderIds.add(grant.workingFolderId);
      if (grant.isDefault) defaultCount += 1;
      if (ceiling && !folderCapabilityFits(grant.capability, ceiling)) {
        errors.push(`Folder ${grant.workingFolderId} exceeds its access profile`);
      }
    }
    if (defaultCount > 1) errors.push(`Channel ${channel.channelId} has multiple default folders`);
  }
  return errors;
}
