// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import type { NotesBlock } from "$lib/notes/types";
import { createNotesBlockNavigationController } from "./notes-block-navigation-controller";

function paragraph(id: string, text: string): NotesBlock {
  return {
    object: "block",
    id,
    parent: { type: "page_id", page_id: "page-1" },
    created_time: "2026-01-01T00:00:00Z",
    last_edited_time: "2026-01-01T00:00:00Z",
    has_children: false,
    in_trash: false,
    type: "paragraph",
    source_provider: null,
    source_object_id: null,
    source_last_edited_time: null,
    paragraph: { rich_text: [], color: "default" },
  };
}

describe("Notes block navigation controller", () => {
  it("moves from the last visual position of one editor to the next rendered block", () => {
    Range.prototype.getBoundingClientRect = () => ({
      width: 0,
      height: 0,
    } as DOMRect);
    Range.prototype.getClientRects = () => [] as unknown as DOMRectList;
    const list = document.createElement("div");
    list.innerHTML = `
      <div data-notes-selectable-block-id="first"><div contenteditable="true" role="textbox" data-notes-block-id="first">a</div></div>
      <div data-notes-selectable-block-id="second"><div contenteditable="true" role="textbox" data-notes-block-id="second"></div></div>
    `;
    document.body.append(list);
    const editor = list.querySelector<HTMLElement>("[data-notes-block-id='first']")!;
    const text = editor.firstChild!;
    const range = document.createRange();
    range.setStart(text, 1);
    range.collapse(true);
    const selection = document.getSelection()!;
    selection.removeAllRanges();
    selection.addRange(range);
    const requestFocus = vi.fn();
    const blocks = new Map([
      ["first", paragraph("first", "a")],
      ["second", paragraph("second", "")],
    ]);
    const controller = createNotesBlockNavigationController({
      readListElement: () => list,
      readRenderedBlockIds: () => ["first", "second"],
      readBlock: (id) => blocks.get(id),
      requestFocus,
    });
    const event = new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true });
    Object.defineProperty(event, "target", { value: editor });
    expect(controller.handleKeydown(event, "first")).toBe(true);
    expect(requestFocus).toHaveBeenCalledWith("second", { start: 0, end: 0 });
  });
});
