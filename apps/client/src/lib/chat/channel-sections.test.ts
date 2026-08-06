// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { setActiveVaultIdentity } from "$lib/vault/active-vault";
import {
  moveChatChannelToSection,
  normalizeChatSidebarSections,
  readLastChatChannelId,
  readChatSidebarSections,
  saveLastChatChannelId,
  saveChatSidebarSections,
} from "./channel-sections";

describe("Chat channel sections", () => {
  beforeEach(() => {
    localStorage.clear();
    setActiveVaultIdentity("vault-a");
  });

  afterEach(() => setActiveVaultIdentity(null));

  it("round trips validated custom sections", () => {
    saveChatSidebarSections("project", [{ id: "section", name: "Work", collapsed: true, channelIds: ["a"] }]);
    expect(readChatSidebarSections("project")).toEqual([
      { id: "section", name: "Work", collapsed: true, channelIds: ["a"] },
    ]);
  });

  it("removes stale and duplicate assignments", () => {
    expect(normalizeChatSidebarSections([
      { id: "a", name: "A", collapsed: false, channelIds: ["one", "missing"] },
      { id: "b", name: "B", collapsed: false, channelIds: ["one", "two"] },
    ], ["one", "two"])).toEqual([
      { id: "a", name: "A", collapsed: false, channelIds: ["one"] },
      { id: "b", name: "B", collapsed: false, channelIds: ["two"] },
    ]);
  });

  it("moves a channel between custom and fixed sections", () => {
    const sections = [{ id: "a", name: "A", collapsed: false, channelIds: ["one"] }];
    expect(moveChatChannelToSection(sections, "two", "a")[0]?.channelIds).toEqual(["one", "two"]);
    expect(moveChatChannelToSection(sections, "one", null)[0]?.channelIds).toEqual([]);
  });

  it("remembers the last selected channel per project", () => {
    saveLastChatChannelId("one", "channel:a");
    saveLastChatChannelId("two", "channel:b");
    expect(readLastChatChannelId("one")).toBe("channel:a");
    expect(readLastChatChannelId("two")).toBe("channel:b");
  });

  it("isolates stable project IDs between vaults", () => {
    saveChatSidebarSections("project-routine-learning", [
      { id: "section-a", name: "Vault A", collapsed: false, channelIds: [] },
    ]);
    saveLastChatChannelId("project-routine-learning", "channel:a");

    setActiveVaultIdentity("vault-b");
    expect(readChatSidebarSections("project-routine-learning")).toEqual([]);
    expect(readLastChatChannelId("project-routine-learning")).toBeNull();

    setActiveVaultIdentity("vault-a");
    expect(readChatSidebarSections("project-routine-learning")).toEqual([
      { id: "section-a", name: "Vault A", collapsed: false, channelIds: [] },
    ]);
    expect(readLastChatChannelId("project-routine-learning")).toBe("channel:a");
  });

  it("removes obsolete unscoped preferences instead of assigning them to a new vault", () => {
    const legacyKey = "ganbaru.chat.channel-sections.v1:project-routine-learning";
    localStorage.setItem(legacyKey, JSON.stringify([
      { id: "section-old", name: "Old vault", collapsed: false, channelIds: [] },
    ]));

    expect(readChatSidebarSections("project-routine-learning")).toEqual([]);
    expect(localStorage.getItem(legacyKey)).toBeNull();
  });
});
