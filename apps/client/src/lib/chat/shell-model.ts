import type {
  ChatThreadShellRead,
  ProjectWorkingFolderRead,
  ProviderInstanceRead,
} from "./contracts";
import type { Project, ProjectGroup } from "$lib/projects/types";

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

export interface ChatRailWorkspace {
  workingFolder: ProjectWorkingFolderRead;
  showSubdivision: boolean;
  threads: ChatThreadShellRead[];
}

export interface ChatRailProject {
  project: Project;
  workingFolders: ChatRailWorkspace[];
}

export interface ChatRailGroup {
  id: string;
  label: string;
  collapsed: boolean;
  hidden: boolean;
  archived: boolean;
  projects: ChatRailProject[];
}

export interface ChatRailModel {
  groups: ChatRailGroup[];
  retainedThreadId: string | null;
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
  groups: readonly ProjectGroup[],
  projects: readonly Project[],
  workingFolders: readonly ProjectWorkingFolderRead[],
  threads: readonly ChatThreadShellRead[],
  selectedProjectId: string | null,
  selectedThreadId: string | null,
): ChatRailModel {
  const activeWorkspaces = workingFolders.filter((entry) => (
    entry.workingFolder.archivedAt === null
      && entry.workingFolder.projectId === selectedProjectId
  ));
  const projectModels = projects
    .filter((project) => project.id === selectedProjectId)
    .map((project) => {
      const projectWorkspaces = activeWorkspaces.filter((entry) => (
        entry.workingFolder.projectId === project.id
          && (
            entry.workingFolder.kind === "managed"
            || threads.some((thread) => (
              thread.workingFolderId === entry.workingFolder.id && thread.archivedAt === null
            ))
          )
      ));
      return {
        project,
        workingFolders: projectWorkspaces.map((workspace) => railWorkspace(
          workspace,
          threads,
          projectWorkspaces.length > 1,
        )),
      };
    })
    .filter((project) => project.workingFolders.length > 0);
  const orderedGroups = [...groups]
    .sort((left, right) => left.sortOrder - right.sortOrder)
    .map((group) => ({
      id: group.id,
      label: group.name,
      collapsed: group.collapsed,
      hidden: Boolean(group.hiddenAt),
      archived: Boolean(group.archivedAt),
      projects: projectModels
        .filter((entry) => entry.project.groupId === group.id)
        .sort((left, right) => left.project.sortOrder - right.project.sortOrder),
    }))
    .filter((group) => group.projects.length > 0);
  const ungrouped = projectModels.filter((entry) => !groups.some((group) => group.id === entry.project.groupId));
  if (ungrouped.length > 0) {
    orderedGroups.push({
      id: "ungrouped",
      label: "Projects",
      collapsed: false,
      hidden: false,
      archived: false,
      projects: ungrouped,
    });
  }
  const retainedThreadId = selectedThreadId && threads.some((thread) => thread.id === selectedThreadId)
    ? selectedThreadId
    : null;
  return {
    groups: orderedGroups,
    retainedThreadId,
  };
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

function railWorkspace(
  workingFolder: ProjectWorkingFolderRead,
  threads: readonly ChatThreadShellRead[],
  showSubdivision: boolean,
): ChatRailWorkspace {
  return {
    workingFolder,
    showSubdivision,
    threads: threads
      .filter((thread) => thread.workingFolderId === workingFolder.workingFolder.id && thread.archivedAt === null)
      .sort((left, right) => right.lastActivityAt.localeCompare(left.lastActivityAt)),
  };
}

function normalizeSearch(value: string): string {
  return value.trim().toLocaleLowerCase().normalize("NFKD").replace(/\p{Diacritic}/gu, "");
}
