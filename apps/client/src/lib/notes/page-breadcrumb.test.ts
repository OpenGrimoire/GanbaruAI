import { describe, expect, it } from "vitest";
import { createRichText } from "./block-factory";
import { buildNotesPageBreadcrumb } from "./page-breadcrumb";
import type { NotesPage, NotesParent } from "./types";

const now = "2026-06-30T12:00:00.000Z";

function page(id: string, title: string, parent: NotesParent): NotesPage {
  return {
    object: "page",
    id,
    created_time: now,
    last_edited_time: now,
    parent,
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

describe("notes page breadcrumb", () => {
  it("builds a workspace-prefixed page ancestor chain", () => {
    const root = page("root", "Root", { type: "workspace", workspace: true });
    const child = page("child", "Child", { type: "page_id", page_id: root.id });
    const leaf = page("leaf", "Leaf", { type: "page_id", page_id: child.id });

    expect(
      buildNotesPageBreadcrumb(leaf, [child, root], "Workspace", "Untitled").map((item) => [
        item.id,
        item.title,
        item.current,
      ]),
    ).toEqual([
      [null, "Workspace", false],
      ["root", "Root", false],
      ["child", "Child", false],
      ["leaf", "Leaf", true],
    ]);
  });

  it("renders the current page even when an ancestor is not loaded", () => {
    const leaf = page("leaf", "Leaf", { type: "page_id", page_id: "missing" });

    expect(
      buildNotesPageBreadcrumb(leaf, [], "Workspace", "Untitled").map((item) => item.title),
    ).toEqual(["Workspace", "Leaf"]);
  });

  it("stops at cycles instead of repeating pages", () => {
    const first = page("first", "First", { type: "page_id", page_id: "second" });
    const second = page("second", "Second", { type: "page_id", page_id: "first" });

    expect(
      buildNotesPageBreadcrumb(first, [second], "Workspace", "Untitled").map((item) => item.id),
    ).toEqual([null, "second", "first"]);
  });
});
