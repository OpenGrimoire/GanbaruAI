import { describe, expect, it } from "vitest";
import { createBlockWrite } from "./block-factory";
import { buildNotesChildIdsByParent, flattenNotesBlockTree } from "./block-tree";
import { flattenNotesBlockOutlines, notesBlockOutlineFromBlock } from "./block-outline";
import type { NotesBlock } from "./types";

const pageId = "10000000-0000-4000-8000-000000000001";

function paragraph(id: string, parent: NotesBlock["parent"], hasChildren = false): NotesBlock {
  const write = createBlockWrite(id, "paragraph", id);
  if (write.type !== "paragraph") throw new Error("expected paragraph");
  return {
    object: "block",
    id,
    parent,
    created_time: "2026-01-01T00:00:00Z",
    last_edited_time: "2026-01-01T00:00:00Z",
    has_children: hasChildren,
    in_trash: false,
    archived: false,
    source_provider: null,
    source_object_id: null,
    source_last_edited_time: null,
    type: "paragraph",
    paragraph: write.paragraph,
  };
}

describe("Notes block outline", () => {
  it("matches hydrated tree order and depth for nested blocks", () => {
    const rootA = paragraph("10000000-0000-4000-8000-000000000002", { type: "page_id", page_id: pageId }, true);
    const child = paragraph("10000000-0000-4000-8000-000000000003", { type: "block_id", block_id: rootA.id });
    const rootB = paragraph("10000000-0000-4000-8000-000000000004", { type: "page_id", page_id: pageId });
    const blocks = [rootA, child, rootB];
    const outlines = [
      notesBlockOutlineFromBlock(rootB, pageId, 2_000),
      notesBlockOutlineFromBlock(child, pageId, 1_000),
      notesBlockOutlineFromBlock(rootA, pageId, 1_000),
    ];
    const outlineItems = flattenNotesBlockOutlines(outlines, pageId);
    const hydrated = flattenNotesBlockTree({
      blocksById: Object.fromEntries(blocks.map((block) => [block.id, block])),
      childIdsByParentId: buildNotesChildIdsByParent(blocks),
    }, pageId);
    expect(outlineItems.map((item) => [item.outline.id, item.depth])).toEqual(
      hydrated.map((item) => [item.block.id, item.depth]),
    );
  });
});
