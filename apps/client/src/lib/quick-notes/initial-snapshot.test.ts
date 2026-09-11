import { afterEach, describe, expect, it } from "vitest";
import {
  cacheQuickNotesViewWindow,
  invalidateQuickNotesInitialSnapshot,
  readQuickNotesViewWindow,
} from "./initial-snapshot";

afterEach(() => invalidateQuickNotesInitialSnapshot());

describe("Quick notes resident view cache", () => {
  it("keeps collection and tag windows isolated", () => {
    cacheQuickNotesViewWindow("active", "tag-a", { notes: [], nextCursor: "tag-cursor" });
    cacheQuickNotesViewWindow("archive", null, { notes: [], nextCursor: "archive-cursor" });

    expect(readQuickNotesViewWindow("active", "tag-a")?.nextCursor).toBe("tag-cursor");
    expect(readQuickNotesViewWindow("active", "tag-b")).toBeNull();
    expect(readQuickNotesViewWindow("archive", null)?.nextCursor).toBe("archive-cursor");
  });

  it("returns copies and clears every view on invalidation", () => {
    const source = { notes: [], nextCursor: "cursor" };
    cacheQuickNotesViewWindow("trash", null, source);
    const first = readQuickNotesViewWindow("trash", null);
    const second = readQuickNotesViewWindow("trash", null);

    expect(first).toEqual(source);
    expect(first).not.toBe(second);
    invalidateQuickNotesInitialSnapshot();
    expect(readQuickNotesViewWindow("trash", null)).toBeNull();
  });
});
