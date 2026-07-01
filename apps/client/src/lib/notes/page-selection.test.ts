import { describe, expect, it } from "vitest";
import {
  filterNotesPagesByTitle,
  nextSelectedNotesPageId,
  parseStoredNotesPageId,
} from "./page-selection";

describe("notes page selection", () => {
  it("parses stored page ids defensively", () => {
    expect(parseStoredNotesPageId(" page-a ")).toBe("page-a");
    expect(parseStoredNotesPageId(" ")).toBe(null);
    expect(parseStoredNotesPageId(1)).toBe(null);
  });

  it("selects the next nearby page after a page is removed", () => {
    const pages = [{ id: "a" }, { id: "b" }, { id: "c" }];
    expect(nextSelectedNotesPageId(pages, "b")).toBe("c");
    expect(nextSelectedNotesPageId(pages, "c")).toBe("b");
    expect(nextSelectedNotesPageId([{ id: "a" }], "a")).toBe(null);
  });

  it("filters pages by normalized title", () => {
    const pages = [
      { id: "a", title: "Daily notes" },
      { id: "b", title: "Project index" },
    ];
    expect(filterNotesPagesByTitle(pages, " DAILY ", (page) => page.title)).toEqual([pages[0]]);
    expect(filterNotesPagesByTitle(pages, "", (page) => page.title)).toEqual(pages);
  });
});
