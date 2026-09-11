import { describe, expect, it } from "vitest";
import { quickNoteViewIndexForKey, quickNoteViewShortcut } from "$lib/quick-notes/tags";

describe("Quick notes tag shortcuts", () => {
  it("maps All and nine tag views to 1 through 9 and then 0", () => {
    expect(Array.from({ length: 10 }, (_, index) => quickNoteViewShortcut(index)))
      .toEqual(["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"]);
  });

  it("maps numeric keys back to view positions", () => {
    expect(quickNoteViewIndexForKey("1")).toBe(0);
    expect(quickNoteViewIndexForKey("9")).toBe(8);
    expect(quickNoteViewIndexForKey("0")).toBe(9);
  });

  it("rejects unsupported positions and keys", () => {
    expect(quickNoteViewShortcut(-1)).toBeNull();
    expect(quickNoteViewShortcut(10)).toBeNull();
    expect(quickNoteViewShortcut(1.5)).toBeNull();
    expect(quickNoteViewIndexForKey("Digit1")).toBeNull();
    expect(quickNoteViewIndexForKey("a")).toBeNull();
  });
});
