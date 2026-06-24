import { describe, expect, it } from "vitest";
import {
  applyProjectEmojiSkinTone,
  cleanupProjectIconRecentValues,
  filterProjectEmojiEntries,
  filterProjectLucideIcons,
  prependProjectIconRecentValue,
  projectEmojiSkinToneFromEmoji,
  projectIconVirtualWindow,
  stripProjectEmojiSkinTone,
} from "./project-icon-picker";
import {
  PROJECT_EMOJI_CATALOG_VERSION,
  PROJECT_EMOJI_ENTRIES,
  PROJECT_EMOJI_SKIN_TONE_BASES,
  type ProjectEmojiEntry,
} from "./project-emoji-catalog";
import type { ProjectLucideIconEntry } from "./project-lucide-catalog.generated";

const emojiEntries: readonly ProjectEmojiEntry[] = [
  { emoji: "🚀", name: "rocket", category: "travel", terms: "🚀 rocket" },
  { emoji: "✅", name: "check", category: "symbols", terms: "✅ check done" },
  { emoji: "☕", name: "coffee", category: "food", terms: "☕ coffee drink" },
];

const lucideEntries: readonly ProjectLucideIconEntry[] = [
  { slug: "rocket", label: "Rocket", category: "Travel", terms: "rocket travel", iconNode: [] },
  { slug: "folder", label: "Folder", category: "File icons", terms: "folder file", iconNode: [] },
  { slug: "check", label: "Check", category: "Mathematics", terms: "check done", iconNode: [] },
];

describe("project icon picker helpers", () => {
  it("filters emoji by category and search terms", () => {
    expect(filterProjectEmojiEntries(emojiEntries, "coffee", "all").map((entry) => entry.emoji)).toEqual(["☕"]);
    expect(filterProjectEmojiEntries(emojiEntries, "", "symbols").map((entry) => entry.emoji)).toEqual(["✅"]);
  });

  it("filters Lucide icons by category and search terms", () => {
    expect(filterProjectLucideIcons(lucideEntries, "file", "all").map((entry) => entry.slug)).toEqual(["folder"]);
    expect(filterProjectLucideIcons(lucideEntries, "", "Travel").map((entry) => entry.slug)).toEqual(["rocket"]);
  });

  it("cleans recent values by deduplicating and dropping missing custom emoji", () => {
    expect(cleanupProjectIconRecentValues([
      "emoji:🚀",
      "emoji:🚀",
      "custom-emoji:missing",
      "custom-emoji:kept",
      "none",
    ], new Set(["kept"]))).toEqual(["emoji:🚀", "custom-emoji:kept"]);
  });

  it("prepends recent values without keeping stale duplicates", () => {
    expect(prependProjectIconRecentValue(
      ["emoji:🚀", "lucide:folder"],
      { kind: "emoji", emoji: "🚀" },
      new Set(),
    )).toEqual(["emoji:🚀", "lucide:folder"]);
  });

  it("computes a bounded virtual grid window", () => {
    expect(projectIconVirtualWindow(100, 5, 36, 72, 72, 1)).toEqual({
      startIndex: 5,
      endIndex: 30,
      beforeHeight: 36,
      afterHeight: 504,
    });
  });

  it("applies skin tone only to supported emoji", () => {
    expect(applyProjectEmojiSkinTone("👍", "medium")).toBe("👍🏽");
    expect(applyProjectEmojiSkinTone("👍🏽", "default")).toBe("👍");
    expect(applyProjectEmojiSkinTone("👨‍💻", "medium")).toBe("👨🏽‍💻");
    expect(applyProjectEmojiSkinTone("🚀", "dark")).toBe("🚀");
    expect(stripProjectEmojiSkinTone("🙏🏿")).toBe("🙏");
    expect(projectEmojiSkinToneFromEmoji("👏🏼")).toBe("medium-light");
  });

  it("uses the generated Unicode emoji catalog", () => {
    expect(PROJECT_EMOJI_CATALOG_VERSION).toBe("17.0");
    expect(PROJECT_EMOJI_ENTRIES.length).toBeGreaterThan(1_500);
    expect(PROJECT_EMOJI_ENTRIES.filter((entry) => entry.category === "flags").length).toBeGreaterThan(250);
    expect(PROJECT_EMOJI_ENTRIES.some((entry) => entry.emoji === "🇿🇼" && entry.name === "flag: Zimbabwe")).toBe(true);
    expect(PROJECT_EMOJI_SKIN_TONE_BASES).toContain("👨‍💻");
  });
});
