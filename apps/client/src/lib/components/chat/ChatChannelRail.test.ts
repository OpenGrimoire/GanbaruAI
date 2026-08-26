// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ChatChannelRead } from "$lib/chat/contracts";
import { saveChatSidebarSections } from "$lib/chat/channel-sections";
import { getChat } from "$lib/stores/chat.svelte";
import { setActiveVaultIdentity } from "$lib/vault/active-vault";
import ChatChannelRail from "./ChatChannelRail.svelte";

const projectState = vi.hoisted(() => ({
  store: {
    selectedProjectId: "project-1" as string | null,
    projects: [{ id: "project-1", name: "Ganbaru" }],
  },
}));

vi.mock("$lib/stores/projects.svelte", () => ({
  getProjects: () => projectState.store,
}));

describe("ChatChannelRail", () => {
  let target: HTMLDivElement;
  let component: ReturnType<typeof mount> | null;

  beforeEach(() => {
    localStorage.clear();
    setActiveVaultIdentity("vault-a");
    projectState.store.selectedProjectId = "project-1";
    const chat = getChat();
    chat.activeChannels = [channel("channel-general", "general"), channel("channel-design", "design")];
    chat.archivedChannels = [];
    chat.channelsLoading = false;
    chat.selectedChannelId = "channel-general";
    chat.channelArchiveOpen = false;
    saveChatSidebarSections("project-1", [{
      id: "section-design",
      name: "Design",
      collapsed: false,
      channelIds: ["channel-design"],
    }]);
    target = document.createElement("div");
    document.body.appendChild(target);
    component = null;
  });

  afterEach(async () => {
    if (component) await unmount(component);
    target.remove();
    setActiveVaultIdentity(null);
    vi.restoreAllMocks();
  });

  it("uses right-side disclosure chevrons and collapses every channel section", async () => {
    component = mount(ChatChannelRail, {
      target,
      props: {
        expanded: true,
        showCollapsedStrip: true,
        onExpand: vi.fn(),
        onCollapse: vi.fn(),
      },
    });
    await tick();

    const toggles = [...target.querySelectorAll<HTMLButtonElement>(".section-toggle")];
    expect(toggles.map((toggle) => toggle.textContent?.trim())).toEqual([
      "Channels",
      "Design",
      "Direct messages",
    ]);
    for (const toggle of toggles) {
      expect(toggle.firstElementChild?.tagName).toBe("SPAN");
      expect(toggle.lastElementChild?.classList.contains("section-chevron")).toBe(true);
      expect(toggle.getAttribute("aria-expanded")).toBe("true");
    }

    toggles[0]?.click();
    toggles[1]?.click();
    toggles[2]?.click();
    await tick();

    expect(toggles.map((toggle) => toggle.getAttribute("aria-expanded"))).toEqual([
      "false",
      "false",
      "false",
    ]);
    expect(toggles.every((toggle) => toggle.classList.contains("collapsed"))).toBe(true);
    expect(target.querySelectorAll(".channel-row")).toHaveLength(0);
    expect(target.textContent).not.toContain("No direct messages yet.");
  });

  it("closes the mobile surface after navigating to a channel", async () => {
    const chat = getChat();
    const selectChannel = vi.spyOn(chat, "selectChannel").mockResolvedValue();
    const onCollapse = vi.fn();
    component = mount(ChatChannelRail, {
      target,
      props: {
        presentation: "surface",
        expanded: true,
        showCollapsedStrip: false,
        onExpand: vi.fn(),
        onCollapse,
      },
    });
    await tick();

    const designChannel = [...target.querySelectorAll<HTMLButtonElement>(".channel-row")]
      .find((button) => button.textContent?.trim() === "design");
    designChannel?.click();

    await vi.waitFor(() => {
      expect(selectChannel).toHaveBeenCalledWith("channel-design");
      expect(onCollapse).toHaveBeenCalledOnce();
    });
  });
});

function channel(id: string, name: string): ChatChannelRead {
  const timestamp = "2026-08-05T12:00:00.000Z";
  return {
    id,
    conversationId: `conversation-${id}`,
    projectId: "project-1",
    name,
    topic: `${name} channel`,
    isDefault: name === "general",
    memberships: [],
    messageCount: 0,
    unreadCount: 0,
    latestPreview: null,
    lastActivityAt: timestamp,
    attentionState: null,
    revision: 1,
    archivedAt: null,
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}
