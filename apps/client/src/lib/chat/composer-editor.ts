import {
  chatComposerDocumentVersioned,
  chatComposerActiveMarks,
  chatComposerMarkdown,
  chatComposerMarksAt,
  chatComposerPlainText,
  normalizeChatComposerDocument,
  replaceChatComposerText,
  toggleChatComposerMark,
  type ChatComposerDocument,
  type ChatComposerMark,
  type ChatComposerSelection,
  type ChatComposerTextRun,
} from "$lib/chat/composer-rich-text";

export interface ChatComposerEditorChange {
  document: ChatComposerDocument;
  markdown: string;
  plainText: string;
  richContent: ReturnType<typeof chatComposerDocumentVersioned>;
}

export interface ChatComposerEditorCallbacks {
  onChange(change: ChatComposerEditorChange): void;
  onSelectionChange(selection: ChatComposerSelection, activeMarks: ChatComposerMark[]): void;
}

interface EditorSnapshot {
  document: ChatComposerDocument;
  selection: ChatComposerSelection;
  storedMarks: ChatComposerMark[] | null;
}

const HISTORY_LIMIT = 100;

/**
 * Owns every descendant of one Chat contenteditable root.
 *
 * The browser may mutate the root during input, especially during IME composition. The controller
 * reads that mutation into a typed document, then normalizes the DOM only when composition is not
 * active. Svelte must never render children inside the root.
 */
export class ChatComposerEditor {
  private document: ChatComposerDocument;
  private composing = false;
  private storedMarks: ChatComposerMark[] | null = null;
  private pendingNativeHistory: EditorSnapshot | null = null;
  private readonly undoStack: EditorSnapshot[] = [];
  private readonly redoStack: EditorSnapshot[] = [];

  public constructor(
    private readonly root: HTMLDivElement,
    document: ChatComposerDocument,
    private readonly callbacks: ChatComposerEditorCallbacks,
  ) {
    this.document = normalizeChatComposerDocument(document);
    this.render();
  }

  /** Returns whether an IME currently owns the browser DOM. */
  public get isComposing(): boolean {
    return this.composing;
  }

  /** Returns the visible text without Markdown formatting delimiters. */
  public plainText(): string {
    return chatComposerPlainText(this.document);
  }

  /** Returns the current visible-text selection. */
  public selection(): ChatComposerSelection {
    return readEditorSelection(this.root, this.document) ?? {
      start: this.plainText().length,
      end: this.plainText().length,
    };
  }

  /** Focuses the editor and optionally restores a visible-text selection. */
  public focus(selection?: ChatComposerSelection): void {
    this.root.focus();
    if (selection) this.restoreSelection(selection);
  }

  /** Replaces editor state after an external draft or layout change. */
  public setDocument(document: ChatComposerDocument): void {
    const normalized = normalizeChatComposerDocument(document);
    if (sameDocument(this.document, normalized)) return;
    const selection = this.selection();
    this.document = normalized;
    this.storedMarks = null;
    this.pendingNativeHistory = null;
    this.undoStack.length = 0;
    this.redoStack.length = 0;
    this.render();
    if (this.root.ownerDocument.activeElement === this.root) this.restoreSelection(selection);
    this.notifySelection(selection);
  }

  /** Captures native editing intent before the browser changes the DOM. */
  public handleBeforeInput(event: InputEvent): boolean {
    if (event.isComposing || this.composing) return false;
    if (event.inputType === "historyUndo") {
      event.preventDefault();
      this.undo();
      return true;
    }
    if (event.inputType === "historyRedo") {
      event.preventDefault();
      this.redo();
      return true;
    }
    if (event.inputType === "formatBold" || event.inputType === "formatItalic") {
      event.preventDefault();
      this.toggleMark(event.inputType === "formatBold" ? "bold" : "italic");
      return true;
    }
    if (event.inputType === "insertText" && event.data !== null && this.storedMarks !== null) {
      event.preventDefault();
      this.replaceSelection(event.data, this.storedMarks);
      return true;
    }
    this.pendingNativeHistory = this.snapshot();
    return false;
  }

  /** Reconciles a completed native DOM input into editor state. */
  public handleInput(): void {
    const nextDocument = readChatComposerDocument(this.root);
    const selection = readEditorSelection(this.root, nextDocument) ?? {
      start: chatComposerPlainText(nextDocument).length,
      end: chatComposerPlainText(nextDocument).length,
    };
    const changed = !sameDocument(this.document, nextDocument);
    this.document = nextDocument;
    if (this.composing) {
      if (changed) this.notifyChange();
      this.notifySelection(selection);
      return;
    }
    if (changed && this.pendingNativeHistory) this.pushUndo(this.pendingNativeHistory);
    this.pendingNativeHistory = null;
    this.storedMarks = null;
    this.render();
    this.restoreSelection(selection);
    if (changed) this.notifyChange();
    this.notifySelection(selection);
  }

  /** Starts an IME transaction without normalizing its temporary DOM. */
  public handleCompositionStart(): void {
    if (!this.composing) this.pendingNativeHistory = this.snapshot();
    this.composing = true;
  }

  /** Finishes an IME transaction after the browser commits its text. */
  public handleCompositionEnd(): void {
    this.composing = false;
    queueMicrotask(() => {
      const nextDocument = readChatComposerDocument(this.root);
      const selection = readEditorSelection(this.root, nextDocument) ?? {
        start: chatComposerPlainText(nextDocument).length,
        end: chatComposerPlainText(nextDocument).length,
      };
      const changed = !sameDocument(this.document, nextDocument);
      this.document = nextDocument;
      if (this.pendingNativeHistory && !sameDocument(this.pendingNativeHistory.document, nextDocument)) {
        this.pushUndo(this.pendingNativeHistory);
      }
      this.pendingNativeHistory = null;
      this.storedMarks = null;
      this.render();
      this.restoreSelection(selection);
      if (changed) this.notifyChange();
      this.notifySelection(selection);
    });
  }

  /** Handles formatting, history, and soft-break shortcuts before the parent handles sending. */
  public handleKeydown(event: KeyboardEvent): boolean {
    if (this.composing || event.isComposing || event.keyCode === 229) return false;
    const command = (event.ctrlKey || event.metaKey) && !event.altKey;
    if (command && event.key.toLowerCase() === "b") {
      event.preventDefault();
      this.toggleMark("bold");
      return true;
    }
    if (command && event.key.toLowerCase() === "i") {
      event.preventDefault();
      this.toggleMark("italic");
      return true;
    }
    if (command && event.key.toLowerCase() === "z") {
      event.preventDefault();
      if (event.shiftKey) this.redo();
      else this.undo();
      return true;
    }
    if (command && event.key.toLowerCase() === "y") {
      event.preventDefault();
      this.redo();
      return true;
    }
    if (event.key === "Enter" && event.shiftKey) {
      event.preventDefault();
      this.replaceSelection("\n");
      return true;
    }
    return false;
  }

  /** Synchronizes the active mark controls after pointer or keyboard selection changes. */
  public handleSelectionChange(): void {
    this.notifySelection(this.selection());
  }

  /** Inserts plain clipboard text and strips untrusted clipboard markup. */
  public insertPlainText(text: string): void {
    this.replaceSelection(text.replace(/\r\n?/gu, "\n"));
  }

  /** Replaces an explicit visible-text range, used by mentions and commands. */
  public replaceRange(start: number, end: number, text: string): void {
    const selection = { start, end };
    this.applyReplacement(selection, text, chatComposerMarksAt(this.document, start));
  }

  /** Appends unformatted text while retaining the current rich content. */
  public appendPlainText(text: string): void {
    const offset = this.plainText().length;
    this.applyReplacement({ start: offset, end: offset }, text, []);
  }

  /** Toggles bold or italic for the current selection or future typed text. */
  public toggleMark(mark: ChatComposerMark): void {
    const selection = this.selection();
    if (selection.start === selection.end) {
      const current = this.storedMarks ?? chatComposerMarksAt(this.document, selection.start);
      this.storedMarks = current.includes(mark)
        ? current.filter((candidate) => candidate !== mark)
        : [...current, mark];
      this.notifySelection(selection);
      this.root.focus();
      return;
    }
    this.pushUndo(this.snapshot());
    this.document = toggleChatComposerMark(this.document, selection, mark);
    this.storedMarks = null;
    this.render();
    this.restoreSelection(selection);
    this.notifyChange();
    this.notifySelection(selection);
  }

  private replaceSelection(text: string, marks?: readonly ChatComposerMark[]): void {
    const selection = this.selection();
    this.applyReplacement(
      selection,
      text,
      marks ?? this.storedMarks ?? chatComposerMarksAt(this.document, selection.start),
    );
  }

  private applyReplacement(
    selection: ChatComposerSelection,
    text: string,
    marks: readonly ChatComposerMark[],
  ): void {
    this.pushUndo(this.snapshot());
    const replacement = replaceChatComposerText(this.document, selection, text, marks);
    this.document = replacement.document;
    this.render();
    this.restoreSelection(replacement.selection);
    this.notifyChange();
    this.notifySelection(replacement.selection);
  }

  private undo(): void {
    const previous = this.undoStack.pop();
    if (!previous) return;
    this.redoStack.push(this.snapshot());
    this.restoreSnapshot(previous);
  }

  private redo(): void {
    const next = this.redoStack.pop();
    if (!next) return;
    this.undoStack.push(this.snapshot());
    this.restoreSnapshot(next);
  }

  private restoreSnapshot(snapshot: EditorSnapshot): void {
    this.document = snapshot.document;
    this.storedMarks = snapshot.storedMarks;
    this.pendingNativeHistory = null;
    this.render();
    this.restoreSelection(snapshot.selection);
    this.notifyChange();
    this.notifySelection(snapshot.selection);
  }

  private snapshot(): EditorSnapshot {
    return {
      document: this.document,
      selection: this.selection(),
      storedMarks: this.storedMarks ? [...this.storedMarks] : null,
    };
  }

  private pushUndo(snapshot: EditorSnapshot): void {
    const previous = this.undoStack.at(-1);
    if (!previous || !sameDocument(previous.document, snapshot.document)
      || previous.selection.start !== snapshot.selection.start || previous.selection.end !== snapshot.selection.end) {
      this.undoStack.push(snapshot);
      if (this.undoStack.length > HISTORY_LIMIT) this.undoStack.shift();
    }
    this.redoStack.length = 0;
  }

  private render(): void {
    renderChatComposerDocument(this.root, this.document);
  }

  private restoreSelection(selection: ChatComposerSelection): void {
    if (this.root.ownerDocument.activeElement !== this.root) this.root.focus({ preventScroll: true });
    restoreEditorSelection(this.root, this.document, selection);
  }

  private notifyChange(): void {
    this.callbacks.onChange({
      document: this.document,
      markdown: chatComposerMarkdown(this.document),
      plainText: chatComposerPlainText(this.document),
      richContent: chatComposerDocumentVersioned(this.document),
    });
  }

  private notifySelection(selection: ChatComposerSelection): void {
    const marks = this.storedMarks ?? chatComposerActiveMarks(this.document, selection);
    this.callbacks.onSelectionChange(selection, marks);
  }
}

/** Reads the supported rich-text subset from browser-owned editor DOM. */
export function readChatComposerDocument(root: ParentNode): ChatComposerDocument {
  const meaningful = [...root.childNodes];
  const blockChildren = meaningful.filter(isEditorBlock);
  if (blockChildren.length > 0 && blockChildren.length === meaningful.length) {
    return normalizeChatComposerDocument({
      lines: blockChildren.flatMap((block) => readInlineLines(block)),
    });
  }
  return normalizeChatComposerDocument({ lines: readInlineLines(root) });
}

/** Renders a composer document without assigning HTML strings. */
export function renderChatComposerDocument(root: HTMLDivElement, document: ChatComposerDocument): void {
  const owner = root.ownerDocument;
  const normalized = normalizeChatComposerDocument(document);
  const lines = normalized.lines.map((line) => {
    const lineElement = owner.createElement("div");
    lineElement.dataset.chatComposerLine = "true";
    if (line.runs.length === 0) {
      const sentinel = owner.createElement("br");
      sentinel.dataset.chatComposerSentinel = "true";
      lineElement.append(sentinel);
      return lineElement;
    }
    for (const run of line.runs) lineElement.append(renderRun(owner, run));
    return lineElement;
  });
  root.replaceChildren(...lines);
  root.dataset.empty = chatComposerPlainText(normalized).length === 0 ? "true" : "false";
}

function readEditorSelection(
  root: HTMLDivElement,
  document: ChatComposerDocument,
): ChatComposerSelection | null {
  const selection = root.ownerDocument.getSelection();
  if (!selection?.anchorNode || !selection.focusNode) return null;
  if (!root.contains(selection.anchorNode) || !root.contains(selection.focusNode)) return null;
  const length = chatComposerPlainText(document).length;
  const anchor = Math.min(length, domPointOffset(root, selection.anchorNode, selection.anchorOffset));
  const focus = Math.min(length, domPointOffset(root, selection.focusNode, selection.focusOffset));
  return { start: Math.min(anchor, focus), end: Math.max(anchor, focus) };
}

function restoreEditorSelection(
  root: HTMLDivElement,
  document: ChatComposerDocument,
  selection: ChatComposerSelection,
): void {
  const browserSelection = root.ownerDocument.getSelection();
  if (!browserSelection) return;
  const length = chatComposerPlainText(document).length;
  const start = domPointAtOffset(root, document, Math.min(length, Math.max(0, selection.start)));
  const end = domPointAtOffset(root, document, Math.min(length, Math.max(0, selection.end)));
  const range = root.ownerDocument.createRange();
  range.setStart(start.node, start.offset);
  range.setEnd(end.node, end.offset);
  browserSelection.removeAllRanges();
  browserSelection.addRange(range);
}

function domPointOffset(root: HTMLDivElement, node: Node, offset: number): number {
  try {
    const range = root.ownerDocument.createRange();
    range.setStart(root, 0);
    range.setEnd(node, offset);
    const fragment = range.cloneContents();
    const holder = root.ownerDocument.createElement("div");
    holder.append(fragment);
    let length = chatComposerPlainText(readChatComposerDocument(holder)).length;
    if (node === root && offset > 0 && offset < root.childNodes.length) length += 1;
    return length;
  } catch {
    return 0;
  }
}

function domPointAtOffset(
  root: HTMLDivElement,
  document: ChatComposerDocument,
  offset: number,
): { node: Node; offset: number } {
  const normalized = normalizeChatComposerDocument(document);
  let remaining = offset;
  for (let index = 0; index < normalized.lines.length; index += 1) {
    const line = normalized.lines[index];
    const lineElement = root.children.item(index);
    if (!(lineElement instanceof HTMLElement)) return { node: root, offset: root.childNodes.length };
    const length = line.runs.reduce((total, run) => total + run.text.length, 0);
    if (remaining <= length) return textPointAtOffset(lineElement, remaining);
    remaining -= length;
    if (index < normalized.lines.length - 1) remaining -= 1;
  }
  return { node: root, offset: root.childNodes.length };
}

function textPointAtOffset(root: HTMLElement, offset: number): { node: Node; offset: number } {
  const walker = root.ownerDocument.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  let remaining = offset;
  let last: Text | null = null;
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    if (!(node instanceof Text)) continue;
    last = node;
    const length = node.data.length;
    if (remaining <= length) return { node, offset: remaining };
    remaining -= length;
  }
  return last ? { node: last, offset: last.data.length } : { node: root, offset: 0 };
}

function readInlineLines(root: ParentNode): import("$lib/chat/composer-rich-text").ChatComposerLine[] {
  const meaningful = [...root.childNodes];
  if (meaningful.length === 0 || (meaningful.length === 1 && meaningful[0] instanceof HTMLBRElement)) {
    return [{ runs: [] }];
  }
  const lines: import("$lib/chat/composer-rich-text").ChatComposerLine[] = [{ runs: [] }];
  for (const node of root.childNodes) appendInlineNode(node, [], lines);
  return lines;
}

function appendInlineNode(
  node: Node,
  inheritedMarks: ChatComposerMark[],
  lines: import("$lib/chat/composer-rich-text").ChatComposerLine[],
): void {
  if (node instanceof Text) {
    const text = node.data.replaceAll("\u200b", "");
    if (text.length > 0) appendRun(lines.at(-1), { text, marks: inheritedMarks });
    return;
  }
  if (!(node instanceof HTMLElement)) return;
  if (node instanceof HTMLBRElement) {
    if (!node.dataset.chatComposerSentinel) lines.push({ runs: [] });
    return;
  }
  const marks = elementMarks(node, inheritedMarks);
  for (const child of node.childNodes) appendInlineNode(child, marks, lines);
}

function appendRun(
  line: import("$lib/chat/composer-rich-text").ChatComposerLine | undefined,
  run: ChatComposerTextRun,
): void {
  if (!line) return;
  const previous = line.runs.at(-1);
  if (previous && sameMarks(previous.marks, run.marks)) previous.text += run.text;
  else line.runs.push({ text: run.text, marks: [...run.marks] });
}

function elementMarks(element: HTMLElement, inherited: ChatComposerMark[]): ChatComposerMark[] {
  const marks = new Set(inherited);
  const tag = element.tagName;
  const weight = element.style.fontWeight;
  if (tag === "B" || tag === "STRONG" || weight === "bold" || Number.parseInt(weight, 10) >= 600) {
    marks.add("bold");
  }
  if (tag === "I" || tag === "EM" || element.style.fontStyle === "italic") marks.add("italic");
  return [...marks].sort((left, right) => left.localeCompare(right));
}

function renderRun(owner: Document, run: ChatComposerTextRun): Node {
  let node: Node = owner.createTextNode(run.text);
  if (run.marks.includes("italic")) {
    const italic = owner.createElement("em");
    italic.append(node);
    node = italic;
  }
  if (run.marks.includes("bold")) {
    const bold = owner.createElement("strong");
    bold.append(node);
    node = bold;
  }
  return node;
}

function isEditorBlock(node: Node): node is HTMLElement {
  return node instanceof HTMLElement && (node.dataset.chatComposerLine === "true" || node.tagName === "DIV" || node.tagName === "P");
}

function sameDocument(left: ChatComposerDocument, right: ChatComposerDocument): boolean {
  return JSON.stringify(normalizeChatComposerDocument(left)) === JSON.stringify(normalizeChatComposerDocument(right));
}

function sameMarks(left: readonly ChatComposerMark[], right: readonly ChatComposerMark[]): boolean {
  return left.length === right.length && left.every((mark) => right.includes(mark));
}
