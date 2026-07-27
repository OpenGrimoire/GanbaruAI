// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { getChat } from "$lib/stores/chat.svelte";
import ChatWorkspaceHeader from "./ChatWorkspaceHeader.svelte";

const projectState = vi.hoisted(() => {
  const group = {
    id: "group-1",
    name: "Work",
    icon: "📁",
    sortOrder: 0,
    collapsed: false,
    createdAt: "2026-07-26T12:00:00.000Z",
    updatedAt: "2026-07-26T12:00:00.000Z",
  };
  const project = {
    id: "project-1",
    groupId: group.id,
    name: "Ganbaru",
    icon: "💬",
    sortOrder: 0,
    status: "active" as const,
    defaultEventName: null,
    defaultEventTimeMode: "timed" as const,
    defaultEventDurationMinutes: null,
    defaultPomodoroMode: "preset" as const,
    defaultIdleSettingsSource: "global" as const,
    defaultIdlePauseEnabled: true,
    defaultIdleThresholdMinutes: 5 as const,
    createdAt: "2026-07-26T12:00:00.000Z",
    updatedAt: "2026-07-26T12:00:00.000Z",
  };
  const store = {
    selectedProjectId: project.id as string | null,
    selectedProject: project,
    selectedGroup: group,
    projects: [project],
    groups: [group],
    customEmojis: [],
    loading: false,
    loaded: true,
    loadError: null,
    projectById: (id: string | null | undefined) => id === project.id ? project : undefined,
    groupById: (id: string | null | undefined) => id === group.id ? group : undefined,
    visibleGroups: () => [group],
    projectsForGroup: (groupId: string) => groupId === group.id ? [project] : [],
    projectsForGroupIncludingInactive: (groupId: string) => groupId === group.id ? [project] : [],
    selectProject: vi.fn(async (_id: string) => undefined),
    addGroup: vi.fn(async () => group),
    addProject: vi.fn(async () => project),
    ensureLoaded: vi.fn(async () => undefined),
  };
  return { store };
});

vi.mock("$lib/stores/projects.svelte", () => ({
  getProjects: () => projectState.store,
}));

class ResizeObserverMock implements ResizeObserver {
  constructor(_callback: ResizeObserverCallback) {}

  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

describe("ChatWorkspaceHeader", () => {
  const mounted: { target: HTMLDivElement; component: ReturnType<typeof mount> }[] = [];

  beforeEach(() => {
    vi.stubGlobal("ResizeObserver", ResizeObserverMock);
    const chat = getChat();
    chat.workingFolders = [{
      workingFolder: {
        id: "managed",
        projectId: "project-1",
        displayName: "Ganbaru files",
        kind: "managed",
        managedRelativePath: "projects/project-1",
        sortOrder: 0,
        repositoryKind: "git",
        repositoryIdentity: "git-sha256:ganbaru",
        createdAt: "2026-07-26T12:00:00.000Z",
        updatedAt: "2026-07-26T12:00:00.000Z",
        archivedAt: null,
        revision: 1,
      },
      bindingStatus: "available",
      canonicalPath: "/work/ganbaru",
      lastVerifiedAt: "2026-07-26T12:00:00.000Z",
      currentBranch: "feat/chat-shell",
    }];
    chat.activeThreads = [];
    chat.archivedThreads = [];
    chat.selectedWorkingFolderId = "managed";
    chat.selectedThreadId = null;
    chat.inspectorOpen = false;
  });

  afterEach(async () => {
    while (mounted.length > 0) {
      const entry = mounted.pop();
      if (!entry) continue;
      await unmount(entry.component);
      entry.target.remove();
    }
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  function setup(
    explorerExpanded: boolean,
    reserveGlobalActions = true,
    editingTitle = false,
  ): HTMLDivElement {
    const target = document.createElement("div");
    target.className = "chat-workspace";
    document.body.append(target);
    const component = mount(ChatWorkspaceHeader, {
      target,
      props: {
        explorerExpanded,
        showRailButton: false,
        reserveGlobalActions,
        editingTitle,
        onOpenRail: vi.fn(),
      },
    });
    mounted.push({ target, component });
    return target;
  }

  it("uses the shared Project navigator from both project identity triggers", async () => {
    const target = setup(true);
    const groupTrigger = target.querySelector<HTMLButtonElement>("[data-chat-group-trigger]");
    const projectTrigger = target.querySelector<HTMLButtonElement>("[data-chat-project-trigger]");

    groupTrigger?.click();
    await tick();
    expect(document.querySelector(".project-picker-panel")).not.toBeNull();
    expect(document.querySelector<HTMLInputElement>('.project-picker-panel input')?.placeholder).toBe("Search projects...");

    projectTrigger?.click();
    await tick();
    expect(document.querySelector(".project-picker-panel")).not.toBeNull();
    expect(document.querySelector(".project-picker-panel")?.textContent).toContain("Ganbaru");
  });

  it("shows the resource path only when the explorer is collapsed", () => {
    const expanded = setup(true);
    expect(expanded.querySelector("[data-chat-folder-trigger]")).toBeNull();
    expect(expanded.querySelector("[data-chat-thread-trigger]")).toBeNull();
    expect(expanded.querySelectorAll("[data-chat-context-chevron]")).toHaveLength(1);

    const collapsed = setup(false);
    expect(collapsed.querySelector("[data-chat-folder-trigger]")?.textContent).toContain("Ganbaru files");
    expect(collapsed.querySelector("[data-chat-thread-trigger]")?.textContent).toContain("New chat");
    expect(collapsed.querySelectorAll("[data-chat-context-chevron]")).toHaveLength(1);
    expect(collapsed.querySelector("[data-chat-thread-trigger] [data-chat-context-chevron]")).not.toBeNull();
    expect(collapsed.querySelector("[data-chat-new-button] svg")?.classList.contains("lucide-plus")).toBe(true);
  });

  it("reveals the chat segment editor when renaming from an expanded explorer", () => {
    const chat = getChat();
    chat.activeThreads = [{
      id: "thread-1",
      workingFolderId: "managed",
      projectId: "project-1",
      title: "Shell redesign",
      providerFamilyId: "codex",
      providerInstanceId: "codex-local",
      providerThreadId: null,
      modelId: null,
      modelOptions: [],
      modes: { safetyMode: "ask_for_approval", interactionMode: "build" },
      state: "idle",
      latestTurnState: null,
      latestPreview: null,
      messageCount: 1,
      revision: 1,
      lastEventSequence: 1,
      lastActivityAt: "2026-07-26T12:00:00.000Z",
      unreadAt: null,
      archivedAt: null,
    }];
    chat.selectedThreadId = "thread-1";
    const target = setup(true, true, true);

    expect(target.querySelector("[data-chat-title-editor]")).not.toBeNull();
    expect(target.querySelector("[data-chat-folder-trigger]")).not.toBeNull();
  });

});
