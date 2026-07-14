// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import type {
  NotesPageSummaryWindow,
  NotesSearchWindow,
} from "$lib/notes/types";

const backend = vi.hoisted(() => {
  function queue<T>() {
    const resolvers: Array<(value: T) => void> = [];
    return {
      next: vi.fn(() => new Promise<T>((resolve) => resolvers.push(resolve))),
      resolve(index: number, value: T): void {
        resolvers[index]?.(value);
      },
    };
  }
  return {
    archive: queue<NotesPageSummaryWindow>(),
    search: queue<NotesSearchWindow>(),
  };
});

vi.mock("$lib/api/notes", async (importOriginal) => {
  const actual = await importOriginal<typeof import("$lib/api/notes")>();
  return {
    ...actual,
    listArchivedNotesPages: backend.archive.next,
    listTrashedNotesPages: vi.fn(),
    searchNotes: backend.search.next,
  };
});

describe("Notes query controllers", () => {
  it("rejects a stale archive reload", async () => {
    const { createNotesArchiveController } = await import("./notes-store-archive.svelte");
    const controller = createNotesArchiveController();
    const stale = controller.reloadArchivedPages("old");
    const current = controller.reloadArchivedPages("new");
    backend.archive.resolve(1, { pages: [], next_cursor: "current", total_count: 0 });
    await current;
    backend.archive.resolve(0, { pages: [], next_cursor: "stale", total_count: 0 });
    await stale;

    expect(controller.archiveHasMore).toBe(true);
    const more = controller.loadMoreArchivedPages();
    expect(backend.archive.next).toHaveBeenLastCalledWith({
      cursor: "current",
      query: "new",
    });
    backend.archive.resolve(2, { pages: [], next_cursor: null, total_count: 0 });
    await more;
  });

  it("rejects stale search results", async () => {
    const { createNotesSearchController } = await import("./notes-store-search.svelte");
    const controller = createNotesSearchController();
    const stale = controller.search("old");
    const current = controller.search("new");
    backend.search.resolve(1, { results: [], next_cursor: "current" });
    await current;
    backend.search.resolve(0, { results: [], next_cursor: "stale" });
    await stale;

    expect(controller.hasMore).toBe(true);
    const more = controller.loadMoreSearchResults();
    expect(backend.search.next).toHaveBeenLastCalledWith("new", 20, false, "current");
    backend.search.resolve(2, { results: [], next_cursor: null });
    await more;
  });
});
