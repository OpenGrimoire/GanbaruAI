import { describe, expect, it } from "vitest";
import { createRichText } from "./block-factory";
import { planNotesSidebarNavigation } from "./page-sidebar";
import { notesPageTitle } from "./page-title";
import type { NotesPage, NotesParent } from "./types";

const now = "2026-07-01T12:00:00.000Z";

function page(id: string, title: string, parent: NotesParent): NotesPage {
  return {
    object: "page",
    id,
    created_time: now,
    last_edited_time: now,
    parent,
    folder_id: null,
    in_trash: false,
    archived: false,
    icon: null,
    cover: null,
    properties: {
      title: {
        id: "title",
        type: "title",
        title: [createRichText(title)],
      },
    },
    url: null,
    public_url: null,
    source_provider: null,
    source_object_id: null,
    source_workspace_id: null,
    source_last_edited_time: null,
  };
}

function plan(
  overrides: Partial<Parameters<typeof planNotesSidebarNavigation>[0]> = {},
) {
  const pages = [
    page("root", "Root", { type: "workspace", workspace: true }),
    page("child", "Child", { type: "page_id", page_id: "root" }),
    page("other", "Other", { type: "workspace", workspace: true }),
    page("later", "Later", { type: "workspace", workspace: true }),
  ];
  return planNotesSidebarNavigation({
    pages,
    favoritePageIds: ["missing", "root"],
    recentPageIds: ["child", "root", "later", "other"],
    expandedPageIds: ["root"],
    pageIdsWithChildren: ["root"],
    missingParentPageIds: [],
    trashedParentPageIds: [],
    activePageId: null,
    search: "",
    titleForPage: (candidate) => notesPageTitle(candidate, "Untitled"),
    ...overrides,
  });
}

describe("notes sidebar navigation plan", () => {
  it("builds favorites, recents, and page tree from active pages", () => {
    const result = plan();

    expect(result.showNavigationSections).toBe(true);
    expect(result.favoritePages.map((candidate) => candidate.id)).toEqual(["root"]);
    expect(result.recentPages.map((candidate) => candidate.id)).toEqual([
      "child",
      "later",
      "other",
    ]);
    expect(result.treeItems.map((item) => [item.page.id, item.depth])).toEqual([
      ["root", 0],
      ["child", 1],
      ["other", 0],
      ["later", 0],
    ]);
    expect(result.showPagesHeading).toBe(true);
  });

  it("reveals the active page through closed ancestors", () => {
    const result = plan({
      activePageId: "child",
      expandedPageIds: [],
    });

    expect(result.treeItems.map((item) => item.page.id)).toEqual([
      "root",
      "child",
      "other",
      "later",
    ]);
  });

  it("hides navigation sections while the sidebar search is active", () => {
    const result = plan({ search: " root " });

    expect(result.searchQuery).toBe("root");
    expect(result.showNavigationSections).toBe(false);
    expect(result.favoritePages).toEqual([]);
    expect(result.recentPages).toEqual([]);
    expect(result.treeItems).toEqual([]);
    expect(result.showPagesHeading).toBe(false);
  });

  it("limits recents after dropping favorites and stale ids", () => {
    const result = plan({
      favoritePageIds: ["other"],
      recentPageIds: ["missing", "child", "other", "root", "later"],
      recentLimit: 2,
    });

    expect(result.recentPages.map((candidate) => candidate.id)).toEqual(["child", "root"]);
  });

  it("reports unavailable parent states for loaded section pages", () => {
    const orphan = page("orphan", "Orphan", { type: "page_id", page_id: "missing" });
    const trashedChild = page("trashed-child", "Trashed child", {
      type: "page_id",
      page_id: "trashed-parent",
    });
    const result = plan({
      pages: [orphan, trashedChild],
      favoritePageIds: ["orphan"],
      recentPageIds: ["trashed-child"],
      expandedPageIds: [],
      pageIdsWithChildren: [],
      missingParentPageIds: ["missing"],
      trashedParentPageIds: ["trashed-parent"],
    });

    expect(result.parentStatusByPageId).toEqual({
      orphan: "missing",
      "trashed-child": "trashed",
    });
    expect(result.treeItems.map((item) => [item.page.id, item.parentStatus])).toEqual([
      ["orphan", "missing"],
      ["trashed-child", "trashed"],
    ]);
  });
});
