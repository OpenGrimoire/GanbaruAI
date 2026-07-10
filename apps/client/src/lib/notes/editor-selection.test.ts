// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import {
  clampNotesTextSelection,
  notesEditableOffsetFromDomPoint,
  notesEditableSelectionViewportRect,
  notesPlainTextFromEditableRoot,
  notesSelectionForFocus,
  notesTextSelectionFromEditableRoot,
  notesTextSelectionFromControl,
  planNotesSelectionReconciliation,
  restoreNotesEditableSelection,
} from "./editor-selection";

describe("notes editor selection helpers", () => {
  it("lets a new explicit caret request replace the previous selected range", () => {
    expect(
      planNotesSelectionReconciliation({
        focusRequestIsNew: true,
        focusRequestedForEditor: true,
        requestedSelection: { start: 8, end: 8 },
        currentSelection: { start: 2, end: 5 },
        textLength: 12,
        editorActive: true,
      }),
    ).toEqual({
      focusEditor: true,
      selection: { start: 8, end: 8 },
    });
  });

  it("preserves the collapsed caret after the explicit request has been applied", () => {
    expect(
      planNotesSelectionReconciliation({
        focusRequestIsNew: false,
        focusRequestedForEditor: true,
        requestedSelection: { start: 8, end: 8 },
        currentSelection: { start: 8, end: 8 },
        textLength: 12,
        editorActive: true,
      }),
    ).toEqual({
      focusEditor: false,
      selection: { start: 8, end: 8 },
    });
  });

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

  it("prefers requested focus selections over remembered editor selections", () => {
    expect(notesSelectionForFocus({
      requestedSelection: { start: 2, end: 5 },
      currentSelection: { start: 8, end: 8 },
      textLength: 10,
      fallback: "end",
    })).toEqual({ start: 2, end: 5 });
  });

  it("restores remembered selections instead of falling back to the text end", () => {
    expect(notesSelectionForFocus({
      requestedSelection: null,
      currentSelection: { start: 3, end: 7 },
      textLength: 12,
      fallback: "end",
    })).toEqual({ start: 3, end: 7 });
  });

  it("uses explicit start or end fallbacks only when no selection is known", () => {
    expect(notesSelectionForFocus({
      requestedSelection: null,
      currentSelection: null,
      textLength: 12,
      fallback: "start",
    })).toEqual({ start: 0, end: 0 });
    expect(notesSelectionForFocus({
      requestedSelection: null,
      currentSelection: null,
      textLength: 12,
      fallback: "end",
    })).toEqual({ start: 12, end: 12 });
  });

  it("clamps remembered focus selections to reloaded text bounds", () => {
    expect(notesSelectionForFocus({
      requestedSelection: null,
      currentSelection: { start: 4, end: 20 },
      textLength: 9,
      fallback: "end",
    })).toEqual({ start: 4, end: 9 });
  });

  it("reads plain text from a rich editable surface without keeping markup", () => {
    const root = document.createElement("div");
    root.innerHTML = "<span>Hello</span><br><span>world</span>";

    expect(notesPlainTextFromEditableRoot(root)).toBe("Hello\nworld");
  });

  it("reads soft line wrappers as plain text newlines", () => {
    const root = document.createElement("div");
    root.innerHTML = [
      "<div data-notes-editor-line=\"true\"><span>Hello</span></div>",
      "<div data-notes-editor-line=\"true\"><span>world</span></div>",
    ].join("");

    expect(notesPlainTextFromEditableRoot(root)).toBe("Hello\nworld");
  });

  it("keeps trailing text-node soft line breaks", () => {
    const root = document.createElement("div");
    const span = document.createElement("span");
    span.textContent = "Hello\n";
    root.append(span);

    expect(notesPlainTextFromEditableRoot(root)).toBe("Hello\n");
  });

  it("ignores trailing line sentinels while preserving the soft line break", () => {
    const root = document.createElement("div");
    root.innerHTML = [
      "<span>Hello\n</span>",
      "<span data-notes-editor-sentinel=\"trailing-line\">&#8203;</span>",
    ].join("");

    expect(notesPlainTextFromEditableRoot(root)).toBe("Hello\n");
  });

  it("keeps sentinel-backed soft line breaks without visible text", () => {
    const root = document.createElement("div");
    root.innerHTML = [
      "<div data-notes-editor-line=\"true\"><span data-notes-editor-sentinel=\"empty-line\">&#8203;</span></div>",
      "<div data-notes-editor-line=\"true\"><span data-notes-editor-sentinel=\"empty-line\">&#8203;</span></div>",
    ].join("");

    expect(notesPlainTextFromEditableRoot(root)).toBe("\n");
  });

  it("maps trailing line sentinels to the previous text offset", () => {
    const root = document.createElement("div");
    root.innerHTML = [
      "<span>Hello\n</span>",
      "<span data-notes-editor-sentinel=\"trailing-line\">&#8203;</span>",
    ].join("");
    const sentinel = root.querySelector("[data-notes-editor-sentinel]");
    expect(sentinel).toBeInstanceOf(HTMLElement);
    if (!(sentinel instanceof HTMLElement)) return;

    expect(notesEditableOffsetFromDomPoint(root, sentinel.firstChild ?? sentinel, 1)).toBe(6);
  });

  it("maps explicit trailing line sentinels to the previous text offset", () => {
    const root = document.createElement("div");
    root.innerHTML = [
      "<span>Hello</span>",
      "<br>",
      "<span data-notes-editor-sentinel=\"trailing-line\">&#8203;</span>",
    ].join("");
    const sentinel = root.querySelector("[data-notes-editor-sentinel]");
    expect(sentinel).toBeInstanceOf(HTMLElement);
    if (!(sentinel instanceof HTMLElement)) return;

    expect(notesEditableOffsetFromDomPoint(root, sentinel.firstChild ?? sentinel, 1)).toBe(6);
  });

  it("maps DOM points around soft line breaks to plain text offsets", () => {
    const root = document.createElement("div");
    root.innerHTML = "<span>Hello</span><br><span>world</span>";
    const firstText = root.querySelector("span")?.firstChild;
    const secondText = root.querySelectorAll("span")[1]?.firstChild;
    expect(firstText).toBeInstanceOf(Text);
    expect(secondText).toBeInstanceOf(Text);
    if (!(firstText instanceof Text) || !(secondText instanceof Text)) return;

    expect(notesEditableOffsetFromDomPoint(root, firstText, 5)).toBe(5);
    expect(notesEditableOffsetFromDomPoint(root, root, 2)).toBe(6);
    expect(notesEditableOffsetFromDomPoint(root, secondText, 0)).toBe(6);
  });

  it("maps DOM points around line-wrapped soft breaks to plain text offsets", () => {
    const root = document.createElement("div");
    root.innerHTML = [
      "<div data-notes-editor-line=\"true\"><span>Hello</span></div>",
      "<div data-notes-editor-line=\"true\"><span>world</span></div>",
    ].join("");
    const firstText = root.querySelector("span")?.firstChild;
    const secondText = root.querySelectorAll("span")[1]?.firstChild;
    expect(firstText).toBeInstanceOf(Text);
    expect(secondText).toBeInstanceOf(Text);
    if (!(firstText instanceof Text) || !(secondText instanceof Text)) return;

    expect(notesEditableOffsetFromDomPoint(root, firstText, 5)).toBe(5);
    expect(notesEditableOffsetFromDomPoint(root, secondText, 0)).toBe(6);
  });

  it("treats browser filler markup in an empty editable surface as empty text", () => {
    const root = document.createElement("div");

    root.innerHTML = "<br>";
    expect(notesPlainTextFromEditableRoot(root)).toBe("");

    root.innerHTML = "<span></span>";
    expect(notesPlainTextFromEditableRoot(root)).toBe("");

    root.innerHTML = "&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;";
    expect(notesPlainTextFromEditableRoot(root)).toBe("");

    root.innerHTML = "<span data-notes-editor-sentinel=\"empty-line\">&#8203;</span>";
    expect(notesPlainTextFromEditableRoot(root)).toBe("");

    root.innerHTML = "<div><br></div>";
    expect(notesPlainTextFromEditableRoot(root)).toBe("");
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

  it("restores selections after a trailing soft line break", () => {
    const root = document.createElement("div");
    root.innerHTML = [
      "<span>Hello\n</span>",
      "<span data-notes-editor-sentinel=\"trailing-line\">&#8203;</span>",
    ].join("");
    document.body.append(root);

    expect(restoreNotesEditableSelection(root, { start: 6, end: 6 })).toBe(true);
    expect(notesTextSelectionFromEditableRoot(root)).toEqual({ start: 6, end: 6 });
    root.remove();
  });

  it("restores selections after line-wrapped soft breaks", () => {
    const root = document.createElement("div");
    root.innerHTML = [
      "<div data-notes-editor-line=\"true\"><span>Hello</span></div>",
      "<div data-notes-editor-line=\"true\"><span>world</span></div>",
    ].join("");
    document.body.append(root);

    expect(restoreNotesEditableSelection(root, { start: 6, end: 6 })).toBe(true);
    expect(notesTextSelectionFromEditableRoot(root)).toEqual({ start: 6, end: 6 });
    const selection = document.getSelection();
    expect(selection?.anchorNode).toBe(root.querySelectorAll("span")[1]?.firstChild);
    expect(selection?.anchorOffset).toBe(0);
    root.remove();
  });

  it("restores selections inside line-wrapped empty soft lines", () => {
    const root = document.createElement("div");
    root.innerHTML = [
      "<div data-notes-editor-line=\"true\"><span>Hello</span></div>",
      "<div data-notes-editor-line=\"true\"><span data-notes-editor-sentinel=\"empty-line\">&#8203;</span></div>",
    ].join("");
    document.body.append(root);

    expect(restoreNotesEditableSelection(root, { start: 6, end: 6 })).toBe(true);
    expect(notesTextSelectionFromEditableRoot(root)).toEqual({ start: 6, end: 6 });
    const sentinel = root.querySelector("[data-notes-editor-sentinel=\"empty-line\"]");
    const selection = document.getSelection();
    expect(sentinel).toBeInstanceOf(HTMLElement);
    if (!(sentinel instanceof HTMLElement)) return;
    expect(selection?.anchorNode).toBe(sentinel.firstChild);
    expect(selection?.anchorOffset).toBe(0);
    root.remove();
  });

  it("restores remembered selections after rich editable markup is replaced", () => {
    const root = document.createElement("div");
    root.innerHTML = "<span>Alpha </span><span><strong>beta</strong></span>";
    document.body.append(root);

    const rememberedSelection = { start: 2, end: 10 };
    root.innerHTML = "<span>Alpha </span><span class=\"mention\">beta</span>";

    expect(restoreNotesEditableSelection(root, rememberedSelection)).toBe(true);
    expect(notesTextSelectionFromEditableRoot(root)).toEqual(rememberedSelection);
    root.remove();
  });

  it("reads a viewport rectangle from a non-collapsed editable selection", () => {
    const root = document.createElement("div");
    root.textContent = "Selected text";
    document.body.append(root);
    const textNode = root.firstChild;
    expect(textNode).toBeInstanceOf(Text);
    if (!(textNode instanceof Text)) return;

    const range = document.createRange();
    range.setStart(textNode, 0);
    range.setEnd(textNode, 8);
    const rect = {
      top: 10,
      right: 90,
      bottom: 30,
      left: 20,
      width: 70,
      height: 20,
    } as DOMRect;
    Object.defineProperty(range, "getBoundingClientRect", { value: () => rect });
    const selection = document.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(range);

    expect(notesEditableSelectionViewportRect(root)).toEqual({
      top: 10,
      right: 90,
      bottom: 30,
      left: 20,
      width: 70,
      height: 20,
    });
    root.remove();
  });
});
