// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import { ChatComposerEditor } from "./composer-editor";
import { chatComposerDocumentFromText } from "./composer-rich-text";

describe("ChatComposerEditor", () => {
  it("maps a browser selection to visible-text offsets", () => {
    const root = document.createElement("div");
    document.body.append(root);
    const editor = new ChatComposerEditor(root, chatComposerDocumentFromText("Review the calendar"), {
      onChange: vi.fn(),
      onSelectionChange: vi.fn(),
    });
    root.focus();
    setSelection(root, 11, 19);

    expect(document.getSelection()?.toString()).toBe("calendar");
    expect(editor.selection()).toEqual({ start: 11, end: 19 });
    root.remove();
  });

  it("renders a soft break as two sibling editor lines", () => {
    const root = document.createElement("div");
    document.body.append(root);
    const onChange = vi.fn();
    const editor = new ChatComposerEditor(root, chatComposerDocumentFromText("Example"), {
      onChange,
      onSelectionChange: vi.fn(),
    });
    root.focus();
    setSelection(root, 7, 7);
    editor.handleKeydown(new KeyboardEvent("keydown", {
      key: "Enter",
      shiftKey: true,
      cancelable: true,
    }));

    expect(root.querySelectorAll(":scope > [data-chat-composer-line]")).toHaveLength(2);
    expect(onChange).toHaveBeenLastCalledWith(expect.objectContaining({ plainText: "Example\n" }));
    root.remove();
  });

  it("advances the caret against the new document after native input", () => {
    const root = document.createElement("div");
    document.body.append(root);
    const editor = new ChatComposerEditor(root, chatComposerDocumentFromText("Example"), {
      onChange: vi.fn(),
      onSelectionChange: vi.fn(),
    });
    root.focus();
    setSelection(root, 7, 7);
    editor.handleBeforeInput(new InputEvent("beforeinput", { data: "x", inputType: "insertText" }));
    const line = root.querySelector<HTMLElement>("[data-chat-composer-line]");
    if (!line) throw new Error("Editor line did not render");
    line.textContent = "Examplex";
    setSelection(root, 8, 8);
    editor.handleInput();

    expect(editor.plainText()).toBe("Examplex");
    expect(editor.selection()).toEqual({ start: 8, end: 8 });
    root.remove();
  });

  it("anchors the caret to the empty editor line after an external draft clear", () => {
    const root = document.createElement("div");
    root.tabIndex = 0;
    document.body.append(root);
    const onSelectionChange = vi.fn();
    const editor = new ChatComposerEditor(root, chatComposerDocumentFromText("Example"), {
      onChange: vi.fn(),
      onSelectionChange,
    });
    root.focus();
    setSelection(root, 7, 7);

    editor.setDocument(chatComposerDocumentFromText(""));

    const sentinel = root.querySelector<HTMLElement>("[data-chat-composer-sentinel]");
    expect(sentinel?.textContent).toBe("\u200b");
    expect(document.getSelection()?.anchorNode).toBe(sentinel?.firstChild);
    expect(editor.selection()).toEqual({ start: 0, end: 0 });
    expect(onSelectionChange).toHaveBeenLastCalledWith({ start: 0, end: 0 }, []);
    root.remove();
  });

  it("preserves whitespace-only native input", () => {
    const root = document.createElement("div");
    document.body.append(root);
    const editor = new ChatComposerEditor(root, chatComposerDocumentFromText(""), {
      onChange: vi.fn(),
      onSelectionChange: vi.fn(),
    });
    root.focus();
    const line = root.querySelector<HTMLElement>("[data-chat-composer-line]");
    if (!line) throw new Error("Editor line did not render");
    line.textContent = "   ";
    setSelection(root, 3, 3);
    editor.handleInput();

    expect(editor.plainText()).toBe("   ");
    expect(editor.selection()).toEqual({ start: 3, end: 3 });
    root.remove();
  });

  it("commits one undo step after IME composition", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const editor = new ChatComposerEditor(root, chatComposerDocumentFromText("abc"), {
      onChange: vi.fn(),
      onSelectionChange: vi.fn(),
    });
    root.focus();
    setSelection(root, 3, 3);
    editor.handleCompositionStart();
    const line = root.querySelector<HTMLElement>("[data-chat-composer-line]");
    if (!line) throw new Error("Editor line did not render");
    line.textContent = "abcñ";
    setSelection(root, 4, 4);
    editor.handleInput();
    editor.handleCompositionEnd();
    await Promise.resolve();
    editor.handleKeydown(new KeyboardEvent("keydown", { key: "z", ctrlKey: true, cancelable: true }));

    expect(editor.plainText()).toBe("abc");
    root.remove();
  });
});

function setSelection(root: HTMLDivElement, start: number, end: number): void {
  const text = root.querySelector("[data-chat-composer-line]")?.firstChild;
  if (!(text instanceof Text)) throw new Error("Editor text did not render");
  const range = document.createRange();
  range.setStart(text, start);
  range.setEnd(text, end);
  const selection = document.getSelection();
  if (!selection) throw new Error("Document selection is unavailable");
  selection.removeAllRanges();
  selection.addRange(range);
}
