import {
  createBlockUpdate,
  createBlockWrite,
  createCodePayload,
  createTodoPayload,
} from "./block-factory";
import type { NotesBlockType, NotesBlockUpdate, NotesBlockWrite } from "./types";

export const NOTES_CLIPBOARD_MAX_TEXT_LENGTH = 64 * 1024;
export const NOTES_CLIPBOARD_MAX_BLOCKS = 101;

type NotesPastedTextBlockType =
  | "paragraph"
  | "heading_1"
  | "heading_2"
  | "heading_3"
  | "heading_4"
  | "bulleted_list_item"
  | "numbered_list_item"
  | "to_do"
  | "toggle"
  | "quote"
  | "divider"
  | "code";

interface NotesPastedBlockSegment {
  type: NotesPastedTextBlockType;
  content: string;
  checked?: boolean;
  language?: string;
}

export interface NotesPlainTextPastePlan {
  currentUpdate: NotesBlockUpdate;
  appendedBlocks: NotesBlockWrite[];
  focusBlockId: string;
  focusOffset: number;
}

export interface NotesPlainTextPastePlanInput {
  currentBlockId: string;
  currentBlockType: NotesBlockType;
  currentText: string;
  selectionStart: number;
  selectionEnd: number;
  plainText: string;
  createId: () => string;
}

function containsUnsupportedControlCharacter(text: string): boolean {
  return [...text].some((character) => {
    if (character === "\n" || character === "\t") return false;
    return character.charCodeAt(0) < 32 || character.charCodeAt(0) === 127;
  });
}

export function normalizeNotesClipboardPlainText(plainText: string): string | null {
  if (!plainText || plainText.length > NOTES_CLIPBOARD_MAX_TEXT_LENGTH) return null;
  const normalized = plainText.replace(/\r\n?/gu, "\n");
  if (!normalized || containsUnsupportedControlCharacter(normalized)) return null;
  return normalized;
}

function markdownPrefixSegment(line: string): NotesPastedBlockSegment | null {
  const trimmedRight = line.trimEnd();
  if (/^---+$/u.test(trimmedRight.trim())) {
    return { type: "divider", content: "" };
  }

  const heading = /^(#{1,4})\s+(.*)$/u.exec(trimmedRight);
  if (heading) {
    const level = heading[1].length;
    const content = heading[2] ?? "";
    if (level === 1) return { type: "heading_1", content };
    if (level === 2) return { type: "heading_2", content };
    if (level === 3) return { type: "heading_3", content };
    return { type: "heading_4", content };
  }

  const bullet = /^[-*+]\s+(.*)$/u.exec(trimmedRight);
  if (bullet) return { type: "bulleted_list_item", content: bullet[1] ?? "" };

  const numbered = /^(?:\d+|[aAiI])\.\s+(.*)$/u.exec(trimmedRight);
  if (numbered) return { type: "numbered_list_item", content: numbered[1] ?? "" };

  const checkedTodo = /^(?:\[[xX]\]|☑)\s+(.*)$/u.exec(trimmedRight);
  if (checkedTodo) {
    return { type: "to_do", content: checkedTodo[1] ?? "", checked: true };
  }

  const todo = /^(?:\[\]|\[ \]|☐)\s+(.*)$/u.exec(trimmedRight);
  if (todo) return { type: "to_do", content: todo[1] ?? "", checked: false };

  const toggle = /^>\s+(.*)$/u.exec(trimmedRight);
  if (toggle) return { type: "toggle", content: toggle[1] ?? "" };

  const quote = /^"\s+(.*)$/u.exec(trimmedRight);
  if (quote) return { type: "quote", content: quote[1] ?? "" };

  return null;
}

function languageFromFence(line: string): string {
  const language = line.trim().slice(3).trim();
  return language || "plain text";
}

function parsePastedTextSegments(
  text: string,
  allowMarkdownForFirstLine: boolean,
): NotesPastedBlockSegment[] {
  const lines = text.split("\n");
  const segments: NotesPastedBlockSegment[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index] ?? "";
    if (line.trim().startsWith("```")) {
      const language = languageFromFence(line);
      const codeLines: string[] = [];
      index += 1;
      while (index < lines.length && !(lines[index] ?? "").trim().startsWith("```")) {
        codeLines.push(lines[index] ?? "");
        index += 1;
      }
      segments.push({ type: "code", content: codeLines.join("\n"), language });
      continue;
    }

    const markdownSegment = index === 0 && !allowMarkdownForFirstLine
      ? null
      : markdownPrefixSegment(line);
    segments.push(markdownSegment ?? { type: "paragraph", content: line });
  }
  return segments;
}

function segmentAsPlainText(segment: NotesPastedBlockSegment): string {
  if (segment.type === "divider") return "---";
  return segment.content;
}

function capSegments(segments: readonly NotesPastedBlockSegment[]): NotesPastedBlockSegment[] {
  if (segments.length <= NOTES_CLIPBOARD_MAX_BLOCKS) return [...segments];
  const kept = segments.slice(0, NOTES_CLIPBOARD_MAX_BLOCKS - 1);
  const overflow = segments
    .slice(NOTES_CLIPBOARD_MAX_BLOCKS - 1)
    .map(segmentAsPlainText)
    .join("\n");
  return [...kept, { type: "paragraph", content: overflow }];
}

function segmentCanCarrySuffix(segment: NotesPastedBlockSegment): boolean {
  return segment.type !== "divider";
}

function appendSuffixToSegments(
  segments: readonly NotesPastedBlockSegment[],
  suffix: string,
): NotesPastedBlockSegment[] {
  if (!suffix) return [...segments];
  const next = [...segments];
  const last = next.at(-1);
  if (!last) return [{ type: "paragraph", content: suffix }];
  if (!segmentCanCarrySuffix(last)) {
    next.push({ type: "paragraph", content: suffix });
    return next;
  }
  next[next.length - 1] = {
    ...last,
    content: `${last.content}${suffix}`,
  };
  return next;
}

function canConvertCurrentBlockFromSegment(
  currentBlockType: NotesBlockType,
  prefix: string,
  _segment: NotesPastedBlockSegment,
): boolean {
  if (currentBlockType !== "paragraph") return false;
  if (prefix.trim().length > 0) return false;
  return true;
}

function createUpdateForSegment(
  currentBlockType: NotesBlockType,
  prefix: string,
  segment: NotesPastedBlockSegment,
): NotesBlockUpdate {
  if (canConvertCurrentBlockFromSegment(currentBlockType, prefix, segment)) {
    if (segment.type === "to_do") {
      return {
        type: "to_do",
        to_do: createTodoPayload(segment.content, segment.checked ?? false),
      };
    }
    if (segment.type === "code") {
      return {
        type: "code",
        code: createCodePayload(segment.content, segment.language),
      };
    }
    return createBlockUpdate(segment.type, segment.content);
  }
  return createBlockUpdate(currentBlockType, `${prefix}${segmentAsPlainText(segment)}`);
}

function createWriteForSegment(
  id: string,
  segment: NotesPastedBlockSegment,
): NotesBlockWrite {
  if (segment.type === "to_do") {
    return {
      id,
      type: "to_do",
      to_do: createTodoPayload(segment.content, segment.checked ?? false),
    };
  }
  if (segment.type === "code") {
    return {
      id,
      type: "code",
      code: createCodePayload(segment.content, segment.language),
    };
  }
  return createBlockWrite(id, segment.type, segment.content);
}

export function shouldHandleNotesPlainTextPaste(input: {
  currentBlockType: NotesBlockType;
  currentText: string;
  selectionStart: number;
  plainText: string;
}): boolean {
  if (input.currentBlockType === "code") return false;
  const normalizedText = normalizeNotesClipboardPlainText(input.plainText);
  if (!normalizedText) return false;
  return shouldPlanPaste(
    normalizedText,
    input.currentText,
    input.selectionStart,
    input.currentBlockType,
  );
}

function shouldPlanPaste(
  normalizedText: string,
  currentText: string,
  selectionStart: number,
  currentBlockType: NotesBlockType,
): boolean {
  if (normalizedText.includes("\n")) return true;
  if (currentBlockType !== "paragraph") return false;
  if (currentText.slice(0, selectionStart).trim().length > 0) return false;
  return markdownPrefixSegment(normalizedText) !== null;
}

export function planNotesPlainTextPaste(
  input: NotesPlainTextPastePlanInput,
): NotesPlainTextPastePlan | null {
  if (input.currentBlockType === "code") return null;
  const normalizedText = normalizeNotesClipboardPlainText(input.plainText);
  if (!normalizedText) return null;
  const start = Math.max(0, Math.min(input.selectionStart, input.selectionEnd));
  const end = Math.min(
    input.currentText.length,
    Math.max(input.selectionStart, input.selectionEnd),
  );
  if (!shouldPlanPaste(normalizedText, input.currentText, start, input.currentBlockType)) {
    return null;
  }

  const prefix = input.currentText.slice(0, start);
  const suffix = input.currentText.slice(end);
  const allowMarkdownForFirstLine = prefix.trim().length === 0;
  const segments = capSegments(
    appendSuffixToSegments(
      parsePastedTextSegments(normalizedText, allowMarkdownForFirstLine),
      suffix,
    ),
  );
  const [firstSegment, ...remainingSegments] = segments;
  if (!firstSegment) return null;
  const appendedBlocks = remainingSegments.map((segment) => {
    const id = input.createId();
    return createWriteForSegment(id, segment);
  });
  const focusSegment = segments.at(-1) ?? firstSegment;
  const focusSegmentPasteEnd = Math.max(
    0,
    segmentAsPlainText(focusSegment).length - suffix.length,
  );
  const focusOffset = appendedBlocks.length > 0
    ? focusSegmentPasteEnd
    : canConvertCurrentBlockFromSegment(input.currentBlockType, prefix, firstSegment)
      ? focusSegmentPasteEnd
      : prefix.length + focusSegmentPasteEnd;
  return {
    currentUpdate: createUpdateForSegment(input.currentBlockType, prefix, firstSegment),
    appendedBlocks,
    focusBlockId: appendedBlocks.at(-1)?.id ?? input.currentBlockId,
    focusOffset,
  };
}
