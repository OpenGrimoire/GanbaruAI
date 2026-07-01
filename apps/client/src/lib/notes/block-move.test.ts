import { describe, expect, it } from "vitest";
import { notesMoveToPageTargets } from "./block-move";
import { createBlockWrite } from "./block-factory";
import type {
  NotesBlock,
  NotesBlockWrite,
  NotesPage,
  NotesParent,
  NotesRichText,
} from "./types";

const now = "2026-06-30T09:00:00.000Z";

function richText(text: string): NotesRichText {
  return {
    type: "text",
    text: { content: text, link: null },
    annotations: {
      bold: false,
      italic: false,
      strikethrough: false,
      underline: false,
      code: false,
      color: "default",
    },
    plain_text: text,
    href: null,
  };
}

function page(id: string, title: string, inTrash = false): NotesPage {
  return {
    object: "page",
    id,
    created_time: now,
    last_edited_time: now,
    parent: { type: "workspace", workspace: true },
    in_trash: inTrash,
    archived: false,
    icon: null,
    cover: null,
    properties: {
      title: {
        id: "title",
        type: "title",
        title: title ? [richText(title)] : [],
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

function blockFromWrite(write: NotesBlockWrite, parent: NotesParent): NotesBlock {
  const base = {
    object: "block" as const,
    id: write.id,
    parent,
    created_time: now,
    last_edited_time: now,
    has_children: false,
    in_trash: false,
    archived: false,
    source_provider: null,
    source_object_id: null,
    source_last_edited_time: null,
  };
  if (write.type === "child_page") return { ...base, type: write.type, child_page: write.child_page };
  if (write.type === "paragraph") return { ...base, type: write.type, paragraph: write.paragraph };
  throw new Error("fixture supports paragraph and child_page blocks only");
}

describe("notes block move targets", () => {
  it("excludes the current page and trashed pages", () => {
    const block = blockFromWrite(
      createBlockWrite("block", "paragraph", "Move me"),
      { type: "page_id", page_id: "current" },
    );

    expect(
      notesMoveToPageTargets(
        [
          page("current", "Current"),
          page("target", "Target"),
          page("trashed", "Trashed", true),
        ],
        block,
        "current",
        "Untitled",
      ),
    ).toEqual([{ id: "target", title: "Target" }]);
  });

  it("excludes a child page block's own page", () => {
    const block = blockFromWrite(
      createBlockWrite("child-page", "child_page", "Nested"),
      { type: "page_id", page_id: "current" },
    );

    expect(
      notesMoveToPageTargets(
        [
          page("child-page", "Nested"),
          page("target", "Target"),
        ],
        block,
        "current",
        "Untitled",
      ),
    ).toEqual([{ id: "target", title: "Target" }]);
  });

  it("uses the untitled label for blank page titles", () => {
    const block = blockFromWrite(
      createBlockWrite("block", "paragraph", "Move me"),
      { type: "page_id", page_id: "current" },
    );

    expect(
      notesMoveToPageTargets([page("target", "")], block, "current", "Untitled"),
    ).toEqual([{ id: "target", title: "Untitled" }]);
  });
});
