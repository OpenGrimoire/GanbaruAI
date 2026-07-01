import { describe, expect, it } from "vitest";
import { blockPlainText, createBlockWrite } from "./block-factory";
import { buildNotesChildIdsByParent, type NotesTreeState } from "./block-tree";
import {
  createNotesUndoSnapshot,
  notesUndoShortcutAction,
  parseNotesUndoStateJson,
  recordNotesUndoEntry,
  serializeNotesUndoState,
  type NotesUndoEntry,
  type NotesUndoSnapshot,
  type NotesUndoState,
} from "./undo-history";
import type { NotesBlock, NotesBlockWrite, NotesParent } from "./types";

const now = "2026-07-01T01:00:00.000Z";
const pageId = "11111111-1111-4111-8111-111111111111";
const otherPageId = "22222222-2222-4222-8222-222222222222";
const blockA = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const blockB = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";

function blockFromWrite(write: NotesBlockWrite, parent: NotesParent): NotesBlock {
  if (write.type !== "paragraph") throw new Error("fixture only supports paragraph blocks");
  return {
    object: "block",
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
    type: "paragraph",
    paragraph: write.paragraph,
  };
}

function paragraph(id: string, parent: NotesParent, text: string): NotesBlock {
  return blockFromWrite(createBlockWrite(id, "paragraph", text), parent);
}

function state(blocks: readonly NotesBlock[]): NotesTreeState {
  return {
    blocksById: Object.fromEntries(blocks.map((block) => [block.id, block])),
    childIdsByParentId: buildNotesChildIdsByParent(blocks),
  };
}

function pageSnapshot(text: string): NotesUndoSnapshot {
  const snapshot = createNotesUndoSnapshot(
    pageId,
    state([paragraph(blockA, { type: "page_id", page_id: pageId }, text)]),
    blockA,
  );
  if (!snapshot) throw new Error("snapshot fixture must create a snapshot");
  return snapshot;
}

function entry(
  before: NotesUndoSnapshot,
  after: NotesUndoSnapshot,
  id = "entry",
): NotesUndoEntry {
  return {
    id,
    kind: "typing",
    groupKey: "typing:block",
    before,
    after,
    createdAt: 1,
    updatedAt: 1,
  };
}

describe("notes undo history", () => {
  it("snapshots proxy-wrapped blocks from reactive state", () => {
    const block = new Proxy(
      paragraph(blockA, { type: "page_id", page_id: pageId }, "Reactive"),
      {},
    );
    const snapshot = createNotesUndoSnapshot(pageId, state([block]), blockA);

    expect(snapshot?.blocks).toHaveLength(1);
    expect(snapshot?.blocks[0]).not.toBe(block);
    expect(blockPlainText(snapshot?.blocks[0] ?? block)).toBe("Reactive");
  });

  it("groups typing by block within the typing window", () => {
    const initial: NotesUndoState = { undo: [], redo: [] };
    const first = recordNotesUndoEntry(initial, {
      id: "one",
      kind: "typing",
      groupKey: `typing:${blockA}`,
      before: pageSnapshot("a"),
      after: pageSnapshot("ab"),
      now: 1000,
    });
    const second = recordNotesUndoEntry(first, {
      id: "two",
      kind: "typing",
      groupKey: `typing:${blockA}`,
      before: pageSnapshot("ab"),
      after: pageSnapshot("abc"),
      now: 1600,
    });

    expect(second.undo).toHaveLength(1);
    expect(blockPlainText(second.undo[0].before.blocks[0])).toBe("a");
    expect(blockPlainText(second.undo[0].after.blocks[0])).toBe("abc");
    expect(second.redo).toEqual([]);
  });

  it("keeps discrete operations as separate undo entries", () => {
    const first = recordNotesUndoEntry({ undo: [], redo: [] }, {
      id: "one",
      kind: "formatting",
      before: pageSnapshot("a"),
      after: pageSnapshot("ab"),
      now: 1000,
    });
    const second = recordNotesUndoEntry(first, {
      id: "two",
      kind: "link",
      before: pageSnapshot("ab"),
      after: pageSnapshot("abc"),
      now: 1100,
    });

    expect(second.undo.map((undoEntry) => undoEntry.kind)).toEqual(["formatting", "link"]);
  });

  it("clears redo when a new undo entry is recorded", () => {
    const before = pageSnapshot("a");
    const after = pageSnapshot("ab");
    const stateWithRedo: NotesUndoState = {
      undo: [],
      redo: [entry(before, after)],
    };
    const next = recordNotesUndoEntry(stateWithRedo, {
      id: "new",
      kind: "typing",
      groupKey: `typing:${blockA}`,
      before,
      after,
      now: 1000,
    });

    expect(next.undo).toHaveLength(1);
    expect(next.redo).toEqual([]);
  });

  it("recognizes standard undo and redo shortcuts", () => {
    expect(notesUndoShortcutAction({
      key: "z",
      ctrlKey: true,
      metaKey: false,
      shiftKey: false,
      altKey: false,
    })).toBe("undo");
    expect(notesUndoShortcutAction({
      key: "Z",
      ctrlKey: false,
      metaKey: true,
      shiftKey: true,
      altKey: false,
    })).toBe("redo");
    expect(notesUndoShortcutAction({
      key: "y",
      ctrlKey: true,
      metaKey: false,
      shiftKey: false,
      altKey: false,
    })).toBe("redo");
    expect(notesUndoShortcutAction({
      key: "z",
      ctrlKey: true,
      metaKey: false,
      shiftKey: false,
      altKey: true,
    })).toBeNull();
  });

  it("round trips serialized undo state and rejects unknown operation kinds", () => {
    const undoState: NotesUndoState = {
      undo: [entry(pageSnapshot("a"), pageSnapshot("ab"))],
      redo: [],
    };
    const parsed = parseNotesUndoStateJson(serializeNotesUndoState(undoState));
    expect(parsed.undo).toHaveLength(1);
    expect(blockPlainText(parsed.undo[0].after.blocks[0])).toBe("ab");

    const invalid = JSON.parse(serializeNotesUndoState(undoState)) as Record<string, unknown>;
    const undo = invalid.undo;
    if (!Array.isArray(undo) || typeof undo[0] !== "object" || undo[0] === null) {
      throw new Error("serialized undo fixture must include an entry");
    }
    (undo[0] as Record<string, unknown>).kind = "unknown";
    expect(() => parseNotesUndoStateJson(JSON.stringify(invalid))).toThrow(
      "undo[0].kind is not a supported undo kind",
    );
  });

  it("includes moved extra blocks outside the selected page tree", () => {
    const moved = paragraph(blockB, { type: "page_id", page_id: otherPageId }, "Moved");
    const snapshot = createNotesUndoSnapshot(
      pageId,
      state([paragraph(blockA, { type: "page_id", page_id: pageId }, "A")]),
      null,
      [moved],
    );

    expect(snapshot?.blocks.map((block) => block.id)).toEqual([blockA, blockB]);
    expect(snapshot?.childIdsByParentId[otherPageId]).toEqual([blockB]);
  });
});
