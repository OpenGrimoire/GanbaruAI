export const COMPOSER_MIN_VISIBLE_LINES = 2;
export const COMPOSER_MAX_VISIBLE_LINES = 6;

export interface ComposerTextareaLayout {
  height: number;
  overflowing: boolean;
}

/** Clamps an auto-growing textarea to the configured whole-line viewport. */
export function composerTextareaLayout(
  contentHeight: number,
  lineHeight: number,
): ComposerTextareaLayout {
  const minimumHeight = COMPOSER_MIN_VISIBLE_LINES * lineHeight;
  const maximumHeight = COMPOSER_MAX_VISIBLE_LINES * lineHeight;
  return {
    height: Math.min(maximumHeight, Math.max(minimumHeight, contentHeight)),
    overflowing: contentHeight > maximumHeight,
  };
}

/** Resolves the native scroll position that fully reveals one caret line. */
export function composerScrollTopForCaret(
  currentScrollTop: number,
  caretTop: number,
  lineHeight: number,
  scrollHeight: number,
  clientHeight: number,
): number {
  const maximumScrollTop = Math.max(0, scrollHeight - clientHeight);
  const caretBottom = caretTop + lineHeight;
  if (caretTop < currentScrollTop) {
    return Math.min(maximumScrollTop, Math.max(0, caretTop));
  }
  if (caretBottom > currentScrollTop + clientHeight) {
    return Math.min(maximumScrollTop, Math.max(0, caretBottom - clientHeight));
  }
  return Math.min(maximumScrollTop, Math.max(0, currentScrollTop));
}

/** Reveals the current contenteditable caret on a complete line. */
export function revealContenteditableComposerCaret(root: HTMLDivElement): void {
  const selection = root.ownerDocument.getSelection();
  if (!selection || selection.rangeCount === 0) return;
  const range = selection.getRangeAt(0).cloneRange();
  if (!root.contains(range.endContainer)) return;
  range.collapse(false);
  const rootBounds = root.getBoundingClientRect();
  const fallbackLine = composerLineElement(range.endContainer, root);
  const fallbackBounds = fallbackLine?.getBoundingClientRect();
  const caretBounds = typeof range.getBoundingClientRect === "function"
    ? range.getBoundingClientRect()
    : null;
  const caretTop = (caretBounds && (caretBounds.height > 0 || caretBounds.top !== 0)
    ? caretBounds.top
    : fallbackBounds?.top ?? rootBounds.top) - rootBounds.top + root.scrollTop;
  revealComposerCaret(root, caretTop);
}

/** Reveals a textarea caret after a controlled text mutation such as paste. */
export function revealTextareaComposerCaret(
  textarea: HTMLTextAreaElement,
  caretOffset: number,
): void {
  const ownerDocument = textarea.ownerDocument;
  const ownerWindow = ownerDocument.defaultView;
  if (!ownerWindow || !ownerDocument.body) return;
  const computed = ownerWindow.getComputedStyle(textarea);
  const mirror = ownerDocument.createElement("div");
  const marker = ownerDocument.createElement("span");
  mirror.style.position = "fixed";
  mirror.style.top = "0";
  mirror.style.left = "-10000px";
  mirror.style.boxSizing = "border-box";
  mirror.style.width = `${textarea.clientWidth}px`;
  mirror.style.visibility = "hidden";
  mirror.style.pointerEvents = "none";
  mirror.style.whiteSpace = "pre-wrap";
  mirror.style.overflowWrap = "break-word";
  mirror.style.wordBreak = "break-word";
  mirror.style.fontFamily = computed.fontFamily;
  mirror.style.fontSize = computed.fontSize;
  mirror.style.fontStyle = computed.fontStyle;
  mirror.style.fontWeight = computed.fontWeight;
  mirror.style.letterSpacing = computed.letterSpacing;
  mirror.style.lineHeight = computed.lineHeight;
  mirror.style.padding = computed.padding;
  mirror.style.tabSize = computed.tabSize;
  marker.textContent = "\u200b";
  const offset = Math.min(textarea.value.length, Math.max(0, caretOffset));
  mirror.append(ownerDocument.createTextNode(textarea.value.slice(0, offset)), marker);
  ownerDocument.body.append(mirror);
  const caretTop = marker.getBoundingClientRect().top - mirror.getBoundingClientRect().top;
  mirror.remove();
  revealComposerCaret(textarea, caretTop);
}

function revealComposerCaret(element: HTMLElement, caretTop: number): void {
  const lineHeight = composerLineHeight(element);
  const next = composerScrollTopForCaret(
    element.scrollTop,
    caretTop,
    lineHeight,
    element.scrollHeight,
    element.clientHeight,
  );
  if (Math.abs(element.scrollTop - next) > 0.25) element.scrollTop = next;
}

function composerLineHeight(element: HTMLElement): number {
  const value = element.ownerDocument.defaultView?.getComputedStyle(element).lineHeight ?? "";
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 1;
}

function composerLineElement(node: Node, root: HTMLDivElement): HTMLElement | null {
  const element = node instanceof HTMLElement ? node : node.parentElement;
  const line = element?.closest<HTMLElement>("[data-chat-composer-line]") ?? null;
  return line && root.contains(line) ? line : null;
}
