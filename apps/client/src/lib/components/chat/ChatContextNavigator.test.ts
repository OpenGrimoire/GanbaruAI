// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type {
  ChatThreadShellRead,
  ProjectWorkingFolderRead,
} from "$lib/chat/contracts";
import { getChat } from "$lib/stores/chat.svelte";
import ChatContextNavigator from "./ChatContextNavigator.svelte";

const timestamp = "2026-07-26T12:00:00.000Z";

class ResizeObserverMock implements ResizeObserver {
  constructor(_callback: ResizeObserverCallback) {}

  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

function workingFolder(
  id: string,
  overrides: Partial<ProjectWorkingFolderRead> = {},
): ProjectWorkingFolderRead {
  return {
    workingFolder: {
      id,
      projectId: "project-1",
      displayName: id,
      kind: id === "managed" ? "managed" : "external",
      managedRelativePath: id === "managed" ? "projects/project-1" : null,
      sortOrder: id === "managed" ? 0 : 10,
      repositoryKind: "git",
      repositoryIdentity: `git-sha256:${id}`,
      createdAt: timestamp,
      updatedAt: timestamp,
      archivedAt: null,
      revision: 1,
    },
    bindingStatus: "available",
    canonicalPath: `/work/${id}`,
    lastVerifiedAt: timestamp,
    currentBranch: "feat/chat-shell",
    ...overrides,
  };
}

function thread(
  id: string,
  workingFolderId: string,
  lastActivityAt: string,
): ChatThreadShellRead {
  return {
    id,
    workingFolderId,
    projectId: "project-1",
    title: id,
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
    lastActivityAt,
    unreadAt: null,
    archivedAt: null,
  };
}

describe("ChatContextNavigator", () => {
  const mounted: { target: HTMLDivElement; component: ReturnType<typeof mount> }[] = [];

  beforeEach(() => {
    vi.stubGlobal("ResizeObserver", ResizeObserverMock);
    const chat = getChat();
    chat.workingFolders = [
      workingFolder("external"),
      workingFolder("managed"),
      workingFolder("other-project", {
        workingFolder: {
          ...workingFolder("other-project").workingFolder,
          projectId: "project-2",
        },
      }),
    ];
    chat.activeThreads = [
      thread("older", "external", "2026-07-26T10:00:00.000Z"),
      thread("newer", "external", "2026-07-26T11:00:00.000Z"),
    ];
    chat.archivedThreads = [];
    chat.selectedWorkingFolderId = "managed";
    chat.selectedThreadId = null;
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

  function setup(mode: "folders" | "threads", workingFolderId = "external") {
    const target = document.createElement("div");
    target.className = "chat-workspace";
    document.body.append(target);
    const onFolderSelected = vi.fn();
    const onThreadSelected = vi.fn();
    const onNewChat = vi.fn();
    const onAddFolder = vi.fn();
    const component = mount(ChatContextNavigator, {
      target,
      props: {
        mode,
        projectId: "project-1",
        workingFolderId,
        panelMaxHeight: 320,
        onFolderSelected,
        onThreadSelected,
        onNewChat,
        onAddFolder,
      },
    });
    mounted.push({ target, component });
    return { target, onFolderSelected, onThreadSelected, onNewChat, onAddFolder };
  }

  it("opens recent chats beside a focused folder and keeps click fallback", async () => {
    const { target, onFolderSelected, onThreadSelected, onNewChat, onAddFolder } = setup("folders");
    const folderRows = [...target.querySelectorAll<HTMLButtonElement>("[data-chat-folder-id]")];

    expect(folderRows.map((row) => row.dataset.chatFolderId)).toEqual(["managed", "external"]);
    expect(target.querySelector('[data-chat-folder-id="other-project"]')).toBeNull();
    expect(target.querySelector<HTMLElement>("[data-chat-context-navigator='folders']")?.style.height).toBe("156px");

    target.querySelector<HTMLButtonElement>("[data-chat-add-folder]")?.click();
    expect(onAddFolder).toHaveBeenCalledOnce();

    const external = target.querySelector<HTMLButtonElement>('[data-chat-folder-id="external"]');
    external?.blur();
    external?.focus();
    await tick();
    await tick();

    const adjacentPanel = document.querySelector('[data-chat-folder-thread-panel="external"]');
    const chatRows = [...(adjacentPanel?.querySelectorAll<HTMLElement>("[data-chat-thread-id]") ?? [])];
    expect(chatRows.map((row) => row.dataset.chatThreadId)).toEqual(["newer", "older"]);

    chatRows[0]?.click();
    expect(onThreadSelected).toHaveBeenCalledWith(expect.objectContaining({ id: "newer" }));

    external?.click();
    expect(onFolderSelected).toHaveBeenCalledWith("external");

    external?.blur();
    external?.focus();
    await tick();
    document.querySelector<HTMLButtonElement>('[data-chat-folder-thread-panel="external"] [data-chat-new-folder-id="external"]')?.click();
    expect(onNewChat).toHaveBeenCalledWith("external");
  });

  it("filters sibling chats and starts a draft in the current folder", async () => {
    const { target, onThreadSelected, onNewChat } = setup("threads");
    const input = target.querySelector<HTMLInputElement>("input");
    if (!input) throw new Error("Chat navigator search did not render");

    input.value = "new";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await tick();

    const rows = [...target.querySelectorAll<HTMLButtonElement>("[data-chat-thread-id]")];
    expect(rows.map((row) => row.dataset.chatThreadId)).toEqual(["newer"]);
    rows[0]?.click();
    expect(onThreadSelected).toHaveBeenCalledWith(expect.objectContaining({ id: "newer" }));

    target.querySelector<HTMLButtonElement>('[data-chat-new-folder-id="external"]')?.click();
    expect(onNewChat).toHaveBeenCalledWith("external");
  });
});
