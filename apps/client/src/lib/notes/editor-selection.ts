export interface NotesTextSelection {
  start: number;
  end: number;
}

export type NotesSelectionFallback = "start" | "end";

export interface NotesFocusSelectionInput {
  requestedSelection: NotesTextSelection | null;
  currentSelection: NotesTextSelection | null;
  textLength: number;
  fallback: NotesSelectionFallback;
}

export interface NotesSelectionViewportRect {
  top: number;
  right: number;
  bottom: number;
  left: number;
  width: number;
  height: number;
}

const TEXT_NODE = 3;
const ELEMENT_NODE = 1;
const DOCUMENT_FRAGMENT_NODE = 11;
const BROWSER_FILLER_TEXT_PATTERN = /^[\u00a0\u200b\ufeff]*$/u;

interface SelectionControl {
  selectionStart: number | null;
  selectionEnd: number | null;
}

/**
 * Read a normalized text selection from a textarea or input-like control.
 */
export function notesTextSelectionFromControl(control: SelectionControl): NotesTextSelection {
  const start = control.selectionStart ?? 0;
  const end = control.selectionEnd ?? start;
  return {
    start: Math.min(start, end),
    end: Math.max(start, end),
  };
}

/**
 * Keep a text selection inside the current plain text bounds.
 */
export function clampNotesTextSelection(
  selection: NotesTextSelection,
  textLength: number,
): NotesTextSelection {
  const max = Math.max(0, textLength);
  const start = Math.min(Math.max(0, selection.start), max);
  const end = Math.min(Math.max(0, selection.end), max);
  return {
    start: Math.min(start, end),
    end: Math.max(start, end),
  };
}

/**
 * Choose the selection to restore when a rich text editor regains focus.
 */
export function notesSelectionForFocus(
  input: NotesFocusSelectionInput,
): NotesTextSelection {
  const fallbackOffset = input.fallback === "start" ? 0 : input.textLength;
  return clampNotesTextSelection(
    input.requestedSelection
      ?? input.currentSelection
      ?? { start: fallbackOffset, end: fallbackOffset },
    input.textLength,
  );
}

function isElementNode(node: Node): node is Element {
  return node.nodeType === ELEMENT_NODE;
}

function isNotesEditorSentinelElement(node: Node): boolean {
  return isElementNode(node)
    && (
      node.hasAttribute("data-notes-editor-sentinel")
      || node.getAttribute("data-notes-trailing-line-sentinel") === "true"
    );
}

function nodeHasEditablePlainText(node: Node): boolean {
  if (node.nodeType === TEXT_NODE) {
    const text = node.textContent ?? "";
    return text.length > 0 && !BROWSER_FILLER_TEXT_PATTERN.test(text);
  }
  if (!isElementNode(node) && node.nodeType !== DOCUMENT_FRAGMENT_NODE) return false;
  if (isNotesEditorSentinelElement(node)) return false;
  if (isElementNode(node) && node.tagName === "BR") return false;
  return Array.from(node.childNodes).some(nodeHasEditablePlainText);
}

function appendEditablePlainText(node: Node, output: string[], root: Node): void {
  if (node.nodeType === TEXT_NODE) {
    output.push(node.textContent ?? "");
    return;
  }

  if (!isElementNode(node) && node.nodeType !== DOCUMENT_FRAGMENT_NODE) return;

  if (isElementNode(node)) {
    if (isNotesEditorSentinelElement(node)) return;
    if (node.tagName === "BR") {
      output.push("\n");
      return;
    }
    if (node !== root && ["DIV", "P"].includes(node.tagName)) {
      const previous = output.at(-1);
      if (output.length > 0 && previous !== "\n") output.push("\n");
    }
  }

  node.childNodes.forEach((child) => appendEditablePlainText(child, output, root));
}

/**
 * Read only the plain text represented by the rich editable Notes surface.
 */
export function notesPlainTextFromEditableRoot(root: HTMLElement | DocumentFragment): string {
  if (!nodeHasEditablePlainText(root)) return "";
  const output: string[] = [];
  root.childNodes.forEach((child) => appendEditablePlainText(child, output, root));
  return output.join("").replace(/\u00a0/gu, " ");
}

export function notesEditableOffsetFromDomPoint(
  root: HTMLElement,
  node: Node,
  offset: number,
): number | null {
  const range = root.ownerDocument.createRange();
  range.selectNodeContents(root);
  try {
    range.setEnd(node, offset);
  } catch {
    return null;
  }
  return notesPlainTextFromEditableRoot(range.cloneContents()).length;
}

/**
 * Read a normalized selection from a rich editable Notes surface.
 */
export function notesTextSelectionFromEditableRoot(root: HTMLElement): NotesTextSelection | null {
  const selection = root.ownerDocument.getSelection();
  const anchorNode = selection?.anchorNode ?? null;
  const focusNode = selection?.focusNode ?? null;
  if (!selection || selection.rangeCount === 0 || !anchorNode || !focusNode) return null;
  if (!root.contains(anchorNode) || !root.contains(focusNode)) return null;

  const anchor = notesEditableOffsetFromDomPoint(root, anchorNode, selection.anchorOffset);
  const focus = notesEditableOffsetFromDomPoint(root, focusNode, selection.focusOffset);
  if (anchor === null || focus === null) return null;

  return {
    start: Math.min(anchor, focus),
    end: Math.max(anchor, focus),
  };
}

function rectFromDomRect(rect: DOMRect): NotesSelectionViewportRect | null {
  if (rect.width <= 0 && rect.height <= 0) return null;
  return {
    top: rect.top,
    right: rect.right,
    bottom: rect.bottom,
    left: rect.left,
    width: rect.width,
    height: rect.height,
  };
}

export function notesEditableSelectionViewportRect(
  root: HTMLElement,
): NotesSelectionViewportRect | null {
  const selection = root.ownerDocument.getSelection();
  const anchorNode = selection?.anchorNode ?? null;
  const focusNode = selection?.focusNode ?? null;
  if (!selection || selection.rangeCount === 0 || !anchorNode || !focusNode) return null;
  if (selection.isCollapsed || !root.contains(anchorNode) || !root.contains(focusNode)) {
    return null;
  }
  const range = selection.getRangeAt(0);
  const boundingRect = rectFromDomRect(range.getBoundingClientRect());
  if (boundingRect) return boundingRect;
  for (const rect of Array.from(range.getClientRects())) {
    const usableRect = rectFromDomRect(rect);
    if (usableRect) return usableRect;
  }
  return null;
}

interface EditableDomPoint {
  node: Node;
  offset: number;
}

function findEditableDomPoint(root: HTMLElement, textOffset: number): EditableDomPoint {
  let remaining = Math.max(0, textOffset);
  let fallback: EditableDomPoint = { node: root, offset: 0 };

  function visit(parent: Node): EditableDomPoint | null {
    const children = Array.from(parent.childNodes);
    for (let index = 0; index < children.length; index += 1) {
      const child = children[index];
      if (!child) continue;

      if (child.nodeType === TEXT_NODE) {
        const text = child.textContent ?? "";
        if (remaining <= text.length) return { node: child, offset: remaining };
        remaining -= text.length;
        fallback = { node: child, offset: text.length };
        continue;
      }

      if (isNotesEditorSentinelElement(child)) {
        continue;
      }

      if (isElementNode(child) && child.tagName === "BR") {
        if (remaining === 0) return { node: parent, offset: index };
        remaining -= 1;
        fallback = { node: parent, offset: index + 1 };
        continue;
      }

      if (child.nodeType === ELEMENT_NODE || child.nodeType === DOCUMENT_FRAGMENT_NODE) {
        const nested = visit(child);
        if (nested) return nested;
        const childIndex = children.indexOf(child);
        fallback = { node: parent, offset: childIndex + 1 };
      }
    }
    return null;
  }

  return visit(root) ?? fallback;
}

/**
 * Restore a Notes text selection inside a rich editable surface.
 */
export function restoreNotesEditableSelection(
  root: HTMLElement,
  selection: NotesTextSelection,
): boolean {
  const textLength = notesPlainTextFromEditableRoot(root).length;
  const safeSelection = clampNotesTextSelection(selection, textLength);
  const start = findEditableDomPoint(root, safeSelection.start);
  const end = findEditableDomPoint(root, safeSelection.end);
  const range = root.ownerDocument.createRange();
  const windowSelection = root.ownerDocument.getSelection();
  if (!windowSelection) return false;

  range.setStart(start.node, start.offset);
  range.setEnd(end.node, end.offset);
  windowSelection.removeAllRanges();
  windowSelection.addRange(range);
  return true;
}
