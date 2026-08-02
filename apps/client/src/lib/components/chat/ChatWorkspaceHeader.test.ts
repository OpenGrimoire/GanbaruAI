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
    chat.activeChannels = [{
      id: "channel-general",
      conversationId: "conversation-general",
      projectId: "project-1",
      name: "general",
      topic: "Project coordination",
      isDefault: true,
      memberships: [],
      messageCount: 0,
      unreadCount: 0,
      latestPreview: null,
      lastActivityAt: "2026-07-26T12:00:00.000Z",
      attentionState: null,
      revision: 1,
      archivedAt: null,
      createdAt: "2026-07-26T12:00:00.000Z",
      updatedAt: "2026-07-26T12:00:00.000Z",
    }];
    chat.archivedChannels = [];
    chat.selectedChannelId = "channel-general";
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

  it("leaves Chat synchronization to the workspace project observer", async () => {
    const chat = getChat();
    const syncProjectSelection = vi.spyOn(chat, "syncProjectSelection");
    const target = setup(true);
    target.querySelector<HTMLButtonElement>("[data-chat-project-trigger]")?.click();
    await tick();
    const projectButton = [...document.querySelectorAll<HTMLButtonElement>(
      ".project-picker-panel button",
    )].find((button) => button.textContent?.includes("Ganbaru"));

    projectButton?.click();
    await vi.waitFor(() => expect(projectState.store.selectProject).toHaveBeenCalledWith("project-1"));

    expect(syncProjectSelection).not.toHaveBeenCalled();
  });

  it("keeps the organization and channel breadcrumb stable across explorer states", () => {
    const expanded = setup(true);
    expect(expanded.querySelector("[data-chat-group-trigger]")?.textContent).toContain("Work");
    expect(expanded.querySelector("[data-chat-project-trigger]")?.textContent).toContain("Ganbaru");
    expect(expanded.querySelector("[data-chat-channel-trigger]")?.textContent).toContain("general");

    const collapsed = setup(false);
    expect(collapsed.querySelector("[data-chat-channel-trigger]")?.textContent).toContain("general");
    expect(collapsed.querySelector("[data-chat-new-channel-button]")).not.toBeNull();
  });

  it("reveals the chat segment editor when renaming from an expanded explorer", () => {
    const chat = getChat();
    chat.activeChannels = chat.activeChannels.map((channel) => ({
      ...channel,
      id: "channel-shell-redesign",
      name: "shell-redesign",
      isDefault: false,
    }));
    chat.selectedChannelId = "channel-shell-redesign";
    const target = setup(true, true, true);

    expect(target.querySelector("[data-chat-title-editor]")).not.toBeNull();
    expect(target.querySelector("[data-chat-project-trigger]")).not.toBeNull();
  });

});
