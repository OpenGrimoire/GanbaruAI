// @vitest-environment jsdom

import { beforeEach, describe, expect, it } from "vitest";
import {
  moveChatChannelToSection,
  normalizeChatSidebarSections,
  readLastChatChannelId,
  readChatSidebarSections,
  saveLastChatChannelId,
  saveChatSidebarSections,
} from "./channel-sections";

describe("Chat channel sections", () => {
  beforeEach(() => localStorage.clear());

  it("round trips validated personal sections", () => {
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
});
