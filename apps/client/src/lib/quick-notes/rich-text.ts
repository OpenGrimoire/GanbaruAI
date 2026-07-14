import {
  EMPTY_QUICK_NOTE_FORMATTING,
  QUICK_NOTE_BODY_MAX_CHARS,
  type QuickNoteFormatting,
  type QuickNoteFormattingName,
  type QuickNoteTextRun,
} from "./types";

export interface QuickNoteSelection {
  start: number;
  end: number;
}

function sameFormatting(left: QuickNoteTextRun, right: QuickNoteTextRun): boolean {
  return left.bold === right.bold
    && left.italic === right.italic
    && left.underline === right.underline;
}

export function normalizeQuickNoteRuns(
  runs: readonly QuickNoteTextRun[],
): QuickNoteTextRun[] {
  const normalized: QuickNoteTextRun[] = [];
  for (const run of runs) {
    if (!run.content) continue;
    const next = { ...run };
    const previous = normalized.at(-1);
    if (previous && sameFormatting(previous, next)) {
      previous.content += next.content;
    } else {
      normalized.push(next);
    }
  }
  return normalized;
}

export function quickNotePlainText(runs: readonly QuickNoteTextRun[]): string {
  return runs.map((run) => run.content).join("");
}

export function quickNoteCharacterCount(runs: readonly QuickNoteTextRun[]): number {
  return Array.from(quickNotePlainText(runs)).length;
}

export function quickNoteFormattingAt(
  runs: readonly QuickNoteTextRun[],
  offset: number,
): QuickNoteFormatting {
  let cursor = 0;
  let previous = EMPTY_QUICK_NOTE_FORMATTING;
  for (const run of runs) {
    const end = cursor + run.content.length;
    const formatting = {
      bold: run.bold,
      italic: run.italic,
      underline: run.underline,
    };
    if (offset >= cursor && offset < end) return formatting;
    if (offset === end) previous = formatting;
    cursor = end;
  }
  return { ...previous };
}

export function quickNoteFormattingForSelection(
  runs: readonly QuickNoteTextRun[],
  selection: QuickNoteSelection,
): QuickNoteFormatting {
  const start = Math.min(selection.start, selection.end);
  const end = Math.max(selection.start, selection.end);
  if (start === end) return quickNoteFormattingAt(runs, start);
  let cursor = 0;
  let common: QuickNoteFormatting | null = null;
  for (const run of runs) {
    const runEnd = cursor + run.content.length;
    if (end <= cursor) break;
    if (start < runEnd && end > cursor) {
      const formatting = { bold: run.bold, italic: run.italic, underline: run.underline };
      common = common
        ? {
            bold: common.bold && formatting.bold,
            italic: common.italic && formatting.italic,
            underline: common.underline && formatting.underline,
          }
        : formatting;
    }
    cursor = runEnd;
  }
  return common ?? { ...EMPTY_QUICK_NOTE_FORMATTING };
}

function sliceRun(run: QuickNoteTextRun, start: number, end: number): QuickNoteTextRun | null {
  const content = run.content.slice(start, end);
  return content ? { ...run, content } : null;
}

function runsBeforeOffset(
  runs: readonly QuickNoteTextRun[],
  offset: number,
): QuickNoteTextRun[] {
  const output: QuickNoteTextRun[] = [];
  let cursor = 0;
  for (const run of runs) {
    const end = cursor + run.content.length;
    if (cursor >= offset) break;
    const slice = sliceRun(run, 0, Math.min(run.content.length, offset - cursor));
    if (slice) output.push(slice);
    cursor = end;
  }
  return output;
}

function runsAfterOffset(
  runs: readonly QuickNoteTextRun[],
  offset: number,
): QuickNoteTextRun[] {
  const output: QuickNoteTextRun[] = [];
  let cursor = 0;
  for (const run of runs) {
    const end = cursor + run.content.length;
    if (end > offset) {
      const slice = sliceRun(run, Math.max(0, offset - cursor), run.content.length);
      if (slice) output.push(slice);
    }
    cursor = end;
  }
  return output;
}

export function replaceQuickNoteRange(
  runs: readonly QuickNoteTextRun[],
  selection: QuickNoteSelection,
  replacement: readonly QuickNoteTextRun[],
): QuickNoteTextRun[] {
  const textLength = quickNotePlainText(runs).length;
  const start = Math.max(0, Math.min(selection.start, selection.end, textLength));
  const end = Math.max(start, Math.min(Math.max(selection.start, selection.end), textLength));
  return normalizeQuickNoteRuns([
    ...runsBeforeOffset(runs, start),
    ...replacement,
    ...runsAfterOffset(runs, end),
  ]);
}

export function insertQuickNoteText(
  runs: readonly QuickNoteTextRun[],
  selection: QuickNoteSelection,
  content: string,
  formatting: QuickNoteFormatting,
): { runs: QuickNoteTextRun[]; selection: QuickNoteSelection } {
  const replacement = content ? [{ content, ...formatting }] : [];
  const nextRuns = replaceQuickNoteRange(runs, selection, replacement);
  const cursor = Math.min(selection.start, selection.end) + content.length;
  return { runs: nextRuns, selection: { start: cursor, end: cursor } };
}

export function toggleQuickNoteFormatting(
  runs: readonly QuickNoteTextRun[],
  selection: QuickNoteSelection,
  name: QuickNoteFormattingName,
): QuickNoteTextRun[] {
  const start = Math.min(selection.start, selection.end);
  const end = Math.max(selection.start, selection.end);
  if (start === end) return normalizeQuickNoteRuns(runs);
  const active = quickNoteFormattingForSelection(runs, selection)[name];
  const replacement: QuickNoteTextRun[] = [];
  let cursor = 0;
  for (const run of runs) {
    const runEnd = cursor + run.content.length;
    if (runEnd > start && cursor < end) {
      const middleStart = Math.max(0, start - cursor);
      const middleEnd = Math.min(run.content.length, end - cursor);
      const before = sliceRun(run, 0, middleStart);
      const middle = sliceRun(run, middleStart, middleEnd);
      const after = sliceRun(run, middleEnd, run.content.length);
      if (before) replacement.push(before);
      if (middle) replacement.push({ ...middle, [name]: !active });
      if (after) replacement.push(after);
    } else {
      replacement.push({ ...run });
    }
    cursor = runEnd;
  }
  return normalizeQuickNoteRuns(replacement);
}

function previousCodePointOffset(text: string, offset: number): number {
  const previous = Array.from(text.slice(0, offset)).at(-1);
  return Math.max(0, offset - (previous?.length ?? 1));
}

function nextCodePointOffset(text: string, offset: number): number {
  const next = Array.from(text.slice(offset)).at(0);
  return Math.min(text.length, offset + (next?.length ?? 1));
}

function previousWordOffset(text: string, offset: number): number {
  const before = text.slice(0, offset);
  const withoutTrailingSpace = before.replace(/\s+$/u, "");
  return Math.max(0, withoutTrailingSpace.search(/[^\s]*$/u));
}

function nextWordOffset(text: string, offset: number): number {
  const after = text.slice(offset);
  const leadingSpaceLength = after.match(/^\s*/u)?.[0].length ?? 0;
  const wordLength = after.slice(leadingSpaceLength).match(/^[^\s]*/u)?.[0].length ?? 0;
  return Math.min(text.length, offset + leadingSpaceLength + wordLength);
}

export function applyQuickNoteBeforeInput(
  runs: readonly QuickNoteTextRun[],
  selection: QuickNoteSelection,
  inputType: string,
  data: string | null,
  formatting: QuickNoteFormatting,
): { runs: QuickNoteTextRun[]; selection: QuickNoteSelection } | null {
  const text = quickNotePlainText(runs);
  const start = Math.min(selection.start, selection.end);
  const end = Math.max(selection.start, selection.end);
  if ([
    "insertText",
    "insertReplacementText",
    "insertFromDrop",
    "insertFromYank",
    "insertFromDictation",
  ].includes(inputType) && data !== null) {
    return insertQuickNoteText(runs, selection, data, formatting);
  }
  if (inputType === "insertParagraph" || inputType === "insertLineBreak") {
    return insertQuickNoteText(runs, selection, "\n", formatting);
  }
  if (inputType === "deleteContentBackward") {
    const deleteStart = start === end ? previousCodePointOffset(text, start) : start;
    return insertQuickNoteText(runs, { start: deleteStart, end }, "", formatting);
  }
  if (inputType === "deleteContentForward") {
    const deleteEnd = start === end ? nextCodePointOffset(text, end) : end;
    return insertQuickNoteText(runs, { start, end: deleteEnd }, "", formatting);
  }
  if (inputType === "deleteWordBackward") {
    const deleteStart = start === end ? previousWordOffset(text, start) : start;
    return insertQuickNoteText(runs, { start: deleteStart, end }, "", formatting);
  }
  if (inputType === "deleteWordForward") {
    const deleteEnd = start === end ? nextWordOffset(text, end) : end;
    return insertQuickNoteText(runs, { start, end: deleteEnd }, "", formatting);
  }
  if (inputType === "deleteByCut" || inputType === "deleteContent") {
    return insertQuickNoteText(runs, selection, "", formatting);
  }
  return null;
}

function formattingFromElement(element: Element, inherited: QuickNoteFormatting): QuickNoteFormatting {
  const tag = element.tagName;
  const style = element.getAttribute("style")?.toLowerCase() ?? "";
  return {
    bold: inherited.bold || tag === "B" || tag === "STRONG" || /font-weight\s*:\s*(bold|[6-9]00)/u.test(style),
    italic: inherited.italic || tag === "I" || tag === "EM" || /font-style\s*:\s*italic/u.test(style),
    underline: inherited.underline || tag === "U" || /text-decoration[^:]*:\s*[^;]*underline/u.test(style),
  };
}

export function quickNoteRunsFromHtml(html: string): QuickNoteTextRun[] {
  const document = new DOMParser().parseFromString(html, "text/html");
  const output: QuickNoteTextRun[] = [];
  const ignored = new Set(["SCRIPT", "STYLE", "NOSCRIPT", "SVG", "IMG", "VIDEO", "AUDIO"]);
  const blocks = new Set(["DIV", "P", "LI", "H1", "H2", "H3", "H4", "H5", "H6", "BLOCKQUOTE"]);

  function append(content: string, formatting: QuickNoteFormatting): void {
    if (content) output.push({ content, ...formatting });
  }

  function visit(node: Node, formatting: QuickNoteFormatting): void {
    if (node.nodeType === Node.TEXT_NODE) {
      append(node.textContent ?? "", formatting);
      return;
    }
    if (!(node instanceof Element) || ignored.has(node.tagName)) return;
    if (node.tagName === "BR") {
      append("\n", formatting);
      return;
    }
    const nextFormatting = formattingFromElement(node, formatting);
    const block = blocks.has(node.tagName);
    if (block && output.length > 0 && !quickNotePlainText(output).endsWith("\n")) append("\n", formatting);
    for (const child of node.childNodes) visit(child, nextFormatting);
    if (block && output.length > 0 && !quickNotePlainText(output).endsWith("\n")) append("\n", formatting);
  }

  for (const child of document.body.childNodes) visit(child, EMPTY_QUICK_NOTE_FORMATTING);
  const normalized = normalizeQuickNoteRuns(output);
  const text = quickNotePlainText(normalized).replace(/\n+$/u, "");
  if (!text) return [];
  return replaceQuickNoteRange(normalized, { start: text.length, end: quickNotePlainText(normalized).length }, []);
}

export function quickNoteBodyWithinLimit(runs: readonly QuickNoteTextRun[]): boolean {
  return quickNoteCharacterCount(runs) <= QUICK_NOTE_BODY_MAX_CHARS;
}

export function reconcileQuickNotePlainText(
  runs: readonly QuickNoteTextRun[],
  nextText: string,
): QuickNoteTextRun[] {
  const current = quickNotePlainText(runs);
  if (current === nextText) return normalizeQuickNoteRuns(runs);
  let prefix = 0;
  while (prefix < current.length && prefix < nextText.length && current[prefix] === nextText[prefix]) {
    prefix += 1;
  }
  let currentSuffix = current.length;
  let nextSuffix = nextText.length;
  while (
    currentSuffix > prefix
    && nextSuffix > prefix
    && current[currentSuffix - 1] === nextText[nextSuffix - 1]
  ) {
    currentSuffix -= 1;
    nextSuffix -= 1;
  }
  const formatting = quickNoteFormattingAt(runs, prefix);
  return replaceQuickNoteRange(
    runs,
    { start: prefix, end: currentSuffix },
    nextSuffix > prefix ? [{ content: nextText.slice(prefix, nextSuffix), ...formatting }] : [],
  );
}
