// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import {
  clampNotesTextSelection,
  notesPlainTextFromEditableRoot,
  notesTextSelectionFromEditableRoot,
  notesTextSelectionFromControl,
  restoreNotesEditableSelection,
} from "./editor-selection";

describe("notes editor selection helpers", () => {
  it("normalizes reversed control selections", () => {
    expect(notesTextSelectionFromControl({ selectionStart: 8, selectionEnd: 3 })).toEqual({
      start: 3,
      end: 8,
    });
  });

  it("treats missing selection ends as a collapsed selection", () => {
    expect(notesTextSelectionFromControl({ selectionStart: 4, selectionEnd: null })).toEqual({
      start: 4,
      end: 4,
    });
  });

  it("clamps selections inside the current text length", () => {
    expect(clampNotesTextSelection({ start: -2, end: 12 }, 5)).toEqual({
      start: 0,
      end: 5,
    });
  });

  it("reads plain text from a rich editable surface without keeping markup", () => {
    const root = document.createElement("div");
    root.innerHTML = "<span>Hello</span><br><span>world</span>";

    expect(notesPlainTextFromEditableRoot(root)).toBe("Hello\nworld");
  });

  it("normalizes editable block wrappers as line breaks", () => {
    const root = document.createElement("div");
    root.innerHTML = "<div>First</div><div><span>Second</span></div>";

    expect(notesPlainTextFromEditableRoot(root)).toBe("First\nSecond");
  });

  it("maps rich editable selections to plain text offsets", () => {
    const root = document.createElement("div");
    root.innerHTML = "<span>Hello </span><span><strong>world</strong></span>";
    document.body.append(root);

    const firstText = root.querySelector("span")?.firstChild;
    const strongText = root.querySelector("strong")?.firstChild;
    expect(firstText).toBeInstanceOf(Text);
    expect(strongText).toBeInstanceOf(Text);
    if (!(firstText instanceof Text) || !(strongText instanceof Text)) return;

    const range = document.createRange();
    range.setStart(firstText, 3);
    range.setEnd(strongText, 2);
    const selection = document.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(range);

    expect(notesTextSelectionFromEditableRoot(root)).toEqual({ start: 3, end: 8 });
    root.remove();
  });

  it("restores rich editable selections from plain text offsets", () => {
    const root = document.createElement("div");
    root.innerHTML = "<span>Hello </span><span><em>world</em></span>";
    document.body.append(root);

    expect(restoreNotesEditableSelection(root, { start: 6, end: 11 })).toBe(true);
    expect(notesTextSelectionFromEditableRoot(root)).toEqual({ start: 6, end: 11 });
    root.remove();
  });
});
