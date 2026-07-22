import type {
  ChatThreadShellRead,
  ChatWorkspaceRead,
  ProviderInstanceRead,
} from "./contracts";
import type { Project, ProjectGroup } from "$lib/projects/types";

export type ChatFirstUseState =
  | { kind: "no_provider" }
  | { kind: "no_workspace" }
  | { kind: "select_workspace" }
  | { kind: "missing_binding"; workspace: ChatWorkspaceRead }
  | { kind: "provider_unavailable"; provider: ProviderInstanceRead | null; providerInstanceId: string }
  | { kind: "no_thread"; workspace: ChatWorkspaceRead }
  | { kind: "archived_thread"; thread: ChatThreadShellRead }
  | { kind: "conversation"; thread: ChatThreadShellRead };

export type ChatThreadStatus = "waiting_answer" | "waiting_approval" | "working" | "error" | "unread" | "idle" | "archived";

export interface ChatRailWorkspace {
  workspace: ChatWorkspaceRead;
  showSubdivision: boolean;
  threads: ChatThreadShellRead[];
}

export interface ChatRailProject {
  project: Project;
  workspaces: ChatRailWorkspace[];
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
  standalone: ChatRailWorkspace[];
  retainedThreadId: string | null;
}

export interface ChatFirstUseInput {
  providers: ProviderInstanceRead[];
  workspaces: ChatWorkspaceRead[];
  selectedWorkspaceId: string | null;
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
  const enabledProviders = input.providers.filter((provider) => provider.configuration.enabled);
  if (enabledProviders.length === 0 && !selectedThread) return { kind: "no_provider" };
  const activeWorkspaces = input.workspaces.filter((entry) => entry.workspace.archivedAt === null);
  if (activeWorkspaces.length === 0) return { kind: "no_workspace" };
  if (!input.selectedWorkspaceId) return { kind: "select_workspace" };
  const workspace = activeWorkspaces.find((entry) => entry.workspace.id === input.selectedWorkspaceId);
  if (!workspace) return { kind: "select_workspace" };
  if (workspace.bindingStatus !== "available") return { kind: "missing_binding", workspace };

  const providerId = selectedThread?.providerInstanceId ?? null;
  const provider = providerId
    ? input.providers.find((entry) => entry.configuration.instanceId === providerId) ?? null
    : enabledProviders.some((entry) => entry.lastProbe?.state === "healthy")
      ? null
      : enabledProviders[0] ?? null;
  if (provider && (!provider.configuration.enabled || provider.lastProbe?.state !== "healthy")) {
    return { kind: "provider_unavailable", provider, providerInstanceId: provider.configuration.instanceId };
  }
  if (selectedThread && providerId && !provider) {
    return { kind: "provider_unavailable", provider: null, providerInstanceId: providerId };
  }
  if (selectedThread) return { kind: "conversation", thread: selectedThread };
  return { kind: "no_thread", workspace };
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
  workspaces: readonly ChatWorkspaceRead[],
  threads: readonly ChatThreadShellRead[],
  selectedThreadId: string | null,
): ChatRailModel {
  const activeWorkspaces = workspaces.filter((entry) => entry.workspace.archivedAt === null);
  const projectModels = projects
    .map((project) => {
      const projectWorkspaces = activeWorkspaces.filter((entry) => entry.workspace.projectId === project.id);
      return {
        project,
        workspaces: projectWorkspaces.map((workspace) => railWorkspace(
          workspace,
          threads,
          projectWorkspaces.length > 1,
        )),
      };
    })
    .filter((project) => project.workspaces.length > 0);
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
    standalone: activeWorkspaces
      .filter((entry) => entry.workspace.projectId === null)
      .map((workspace) => railWorkspace(workspace, threads, true)),
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
  workspace: ChatWorkspaceRead,
  threads: readonly ChatThreadShellRead[],
  showSubdivision: boolean,
): ChatRailWorkspace {
  return {
    workspace,
    showSubdivision,
    threads: threads
      .filter((thread) => thread.workspaceId === workspace.workspace.id && thread.archivedAt === null)
      .sort((left, right) => right.lastActivityAt.localeCompare(left.lastActivityAt)),
  };
}

function normalizeSearch(value: string): string {
  return value.trim().toLocaleLowerCase().normalize("NFKD").replace(/\p{Diacritic}/gu, "");
}
