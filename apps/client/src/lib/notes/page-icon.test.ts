import { describe, expect, it } from "vitest";
import { createNotesEmojiPageIcon, notesPageIconText } from "./page-icon";

describe("notes page icons", () => {
  it("creates an emoji icon payload", () => {
    expect(createNotesEmojiPageIcon(" 📌 ")).toEqual({ type: "emoji", emoji: "📌" });
  });

  it("rejects blank emoji icons", () => {
    expect(() => createNotesEmojiPageIcon(" ")).toThrow("page icon emoji must not be empty");
  });

  it("returns display text only for emoji icons", () => {
    expect(notesPageIconText({ type: "emoji", emoji: "📌" })).toBe("📌");
    expect(notesPageIconText({ type: "external", external: { url: "https://example.com/a.png" } })).toBeNull();
    expect(notesPageIconText(null)).toBeNull();
  });
});
