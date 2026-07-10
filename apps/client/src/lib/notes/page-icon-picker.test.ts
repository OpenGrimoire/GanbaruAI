import { describe, expect, it } from "vitest";
import {
  notesPageIconFromPickerValue,
  notesPageIconPickerValue,
} from "./page-icon-picker";

const customEmojis = [{
  id: "focus",
  name: "Focus",
  assetPath: `project-icons/${"a".repeat(64)}.png`,
}];

describe("Notes page icon picker adapter", () => {
  it("round trips emoji and named Lucide icons", () => {
    expect(notesPageIconFromPickerValue("emoji:📌", customEmojis)).toEqual({
      type: "emoji",
      emoji: "📌",
    });
    expect(notesPageIconFromPickerValue("lucide:book-open:blue", customEmojis)).toEqual({
      type: "icon",
      icon: { name: "book-open", color: "blue" },
    });
    expect(notesPageIconPickerValue({
      type: "icon",
      icon: { name: "book-open", color: "purple" },
    })).toBe("lucide:book-open:purple");
  });

  it("maps the shared event palette to Notes icon colors", () => {
    expect(notesPageIconFromPickerValue("lucide:heart:3", customEmojis)).toEqual({
      type: "icon",
      icon: { name: "heart", color: "red" },
    });
    expect(notesPageIconFromPickerValue("lucide:leaf:13", customEmojis)).toEqual({
      type: "icon",
      icon: { name: "leaf", color: "green" },
    });
  });

  it("resolves reusable custom emoji metadata", () => {
    expect(notesPageIconFromPickerValue("custom-emoji:focus", customEmojis)).toEqual({
      type: "custom_emoji",
      custom_emoji: {
        id: "focus",
        name: "Focus",
        url: `ganbaru-asset:project-icons/${"a".repeat(64)}.png`,
        ganbaru_asset_path: `project-icons/${"a".repeat(64)}.png`,
      },
    });
  });

  it("keeps image uploads on the dedicated Notes asset path", () => {
    expect(() => notesPageIconFromPickerValue(
      `asset:project-icons/${"b".repeat(64)}.png`,
      customEmojis,
    )).toThrow("Notes asset adapter");
  });
});
