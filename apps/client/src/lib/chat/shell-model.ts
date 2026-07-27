import type {
  ChatThreadShellRead,
  ProjectWorkingFolderRead,
  ProviderInstanceRead,
} from "./contracts";

export type ChatFirstUseState =
  | { kind: "no_provider" }
  | { kind: "no_project" }
  | { kind: "archived_project" }
  | { kind: "missing_binding"; workingFolder: ProjectWorkingFolderRead }
  | { kind: "provider_unavailable"; provider: ProviderInstanceRead | null; providerInstanceId: string }
  | { kind: "no_thread"; workingFolder: ProjectWorkingFolderRead }
  | { kind: "archived_thread"; thread: ChatThreadShellRead }
  | { kind: "conversation"; thread: ChatThreadShellRead };

export type ChatThreadStatus = "waiting_answer" | "waiting_approval" | "working" | "error" | "unread" | "idle" | "archived";

export interface ChatRailFolder {
  workingFolder: ProjectWorkingFolderRead;
  threads: ChatThreadShellRead[];
  selected: boolean;
  hasDraft: boolean;
}

export interface ChatRailModel {
  folders: ChatRailFolder[];
}

export interface ChatFirstUseInput {
  providers: ProviderInstanceRead[];
  workingFolders: ProjectWorkingFolderRead[];
  selectedProjectId: string | null;
  selectedProjectArchived: boolean;
  selectedWorkingFolderId: string | null;
  selectedThreadId: string | null;
  threads: ChatThreadShellRead[];
}

export interface ChatSearchPartitions {
  active: ChatThreadShellRead[];
  archived: ChatThreadShellRead[];
}

export function resolveChatFirstUseState(input: ChatFirstUseInput): ChatFirstUseState {
  const selectedThread = input.selectedThreadId
    ? input.threads.find((thread) => thread.id === input.selectedThreadId) ?? null
    : null;
  if (selectedThread?.archivedAt) return { kind: "archived_thread", thread: selectedThread };
  if (!input.selectedProjectId) return { kind: "no_project" };
  if (input.selectedProjectArchived && !selectedThread) return { kind: "archived_project" };
  if (selectedThread) return { kind: "conversation", thread: selectedThread };
  const enabledProviders = input.providers.filter((provider) => provider.configuration.enabled);
  if (enabledProviders.length === 0) return { kind: "no_provider" };
  const activeWorkingFolders = input.workingFolders.filter((entry) => (
    entry.workingFolder.projectId === input.selectedProjectId
      && entry.workingFolder.archivedAt === null
  ));
  const workingFolder = activeWorkingFolders.find(
    (entry) => entry.workingFolder.id === input.selectedWorkingFolderId,
  ) ?? activeWorkingFolders.find((entry) => entry.workingFolder.kind === "managed") ?? null;
  if (!workingFolder) return { kind: "no_project" };
  if (workingFolder.bindingStatus !== "available") {
    return { kind: "missing_binding", workingFolder };
  }

  const provider = enabledProviders.some((entry) => entry.lastProbe?.state === "healthy")
    ? null
    : enabledProviders[0] ?? null;
  if (provider && (!provider.configuration.enabled || provider.lastProbe?.state !== "healthy")) {
    return { kind: "provider_unavailable", provider, providerInstanceId: provider.configuration.instanceId };
  }
  return { kind: "no_thread", workingFolder };
}

export function threadStatus(thread: ChatThreadShellRead): ChatThreadStatus {
  if (thread.archivedAt) return "archived";
  if (thread.latestTurnState === "waiting_for_user_input") return "waiting_answer";
  if (thread.latestTurnState === "waiting_for_approval") return "waiting_approval";
  if (["pending", "dispatching", "active"].includes(thread.latestTurnState ?? "")) return "working";
  if (thread.state === "error" || thread.latestTurnState === "failed") return "error";
  if (thread.unreadAt) return "unread";
  return "idle";
}

export function buildChatRailModel(
  workingFolders: readonly ProjectWorkingFolderRead[],
  threads: readonly ChatThreadShellRead[],
  selectedProjectId: string | null,
  selectedWorkingFolderId: string | null,
  selectedThreadId: string | null,
): ChatRailModel {
  return {
    folders: contextualChatFolders(
      workingFolders,
      threads,
      selectedProjectId,
      selectedWorkingFolderId,
    ).map((workingFolder) => ({
      workingFolder,
      threads: siblingChatThreads(
        threads,
        selectedProjectId,
        workingFolder.workingFolder.id,
      ),
      selected: workingFolder.workingFolder.id === selectedWorkingFolderId,
      hasDraft: selectedThreadId === null
        && workingFolder.workingFolder.id === selectedWorkingFolderId,
    })),
  };
}

export function chatHeaderShowsResourcePath(explorerExpanded: boolean): boolean {
  return !explorerExpanded;
}

export function chatHeaderActionInset(
  headerRight: number,
  fixedActionsLeft: number,
  edgeGap: number,
): number {
  const safeGap = Math.max(0, edgeGap);
  return Math.max(safeGap, headerRight - fixedActionsLeft + safeGap);
}

export function chatNavigationFolders(
  workingFolders: readonly ProjectWorkingFolderRead[],
  selectedProjectId: string | null,
): ProjectWorkingFolderRead[] {
  return sortChatFolders(workingFolders.filter((entry) => (
    entry.workingFolder.projectId === selectedProjectId
      && entry.workingFolder.archivedAt === null
  )));
}

export function siblingChatThreads(
  threads: readonly ChatThreadShellRead[],
  selectedProjectId: string | null,
  workingFolderId: string | null,
): ChatThreadShellRead[] {
  return threads
    .filter((thread) => (
      thread.projectId === selectedProjectId
        && thread.workingFolderId === workingFolderId
        && thread.archivedAt === null
    ))
    .sort((left, right) => right.lastActivityAt.localeCompare(left.lastActivityAt));
}

export function contextualChatFolders(
  workingFolders: readonly ProjectWorkingFolderRead[],
  threads: readonly ChatThreadShellRead[],
  selectedProjectId: string | null,
  selectedWorkingFolderId: string | null,
): ProjectWorkingFolderRead[] {
  const activeThreadFolderIds = new Set(
    threads
      .filter((thread) => thread.projectId === selectedProjectId && thread.archivedAt === null)
      .map((thread) => thread.workingFolderId),
  );
  return sortChatFolders(workingFolders.filter((entry) => (
    entry.workingFolder.projectId === selectedProjectId
      && (
        entry.workingFolder.kind === "managed"
        || entry.workingFolder.id === selectedWorkingFolderId
        || activeThreadFolderIds.has(entry.workingFolder.id)
      )
  )));
}

export function filterThreadTitles(
  threads: readonly ChatThreadShellRead[],
  query: string,
): ChatThreadShellRead[] {
  const normalized = normalizeSearch(query);
  if (!normalized) return [...threads];
  return threads.filter((thread) => normalizeSearch(thread.title).includes(normalized));
}

export function partitionThreadSearchResults(
  threads: readonly ChatThreadShellRead[],
): ChatSearchPartitions {
  return {
    active: threads.filter((thread) => thread.archivedAt === null),
    archived: threads.filter((thread) => thread.archivedAt !== null),
  };
}

export function filterArchivedThreads(
  threads: readonly ChatThreadShellRead[],
  query: string,
  projectId: string,
): ChatThreadShellRead[] {
  return filterThreadTitles(
    threads.filter((thread) => thread.archivedAt !== null),
    query,
  ).filter((thread) => !projectId || thread.projectId === projectId);
}

export function nextThreadIndex(currentIndex: number, itemCount: number, direction: "next" | "previous"): number {
  if (itemCount <= 0) return -1;
  if (currentIndex < 0) return direction === "next" ? 0 : itemCount - 1;
  return (currentIndex + (direction === "next" ? 1 : -1) + itemCount) % itemCount;
}

function sortChatFolders(
  workingFolders: readonly ProjectWorkingFolderRead[],
): ProjectWorkingFolderRead[] {
  return [...workingFolders].sort((left, right) => {
    if (left.workingFolder.kind !== right.workingFolder.kind) {
      return left.workingFolder.kind === "managed" ? -1 : 1;
    }
    return left.workingFolder.sortOrder - right.workingFolder.sortOrder
      || left.workingFolder.displayName.localeCompare(right.workingFolder.displayName)
      || left.workingFolder.id.localeCompare(right.workingFolder.id);
  });
}

function normalizeSearch(value: string): string {
  return value.trim().toLocaleLowerCase().normalize("NFKD").replace(/\p{Diacritic}/gu, "");
}
