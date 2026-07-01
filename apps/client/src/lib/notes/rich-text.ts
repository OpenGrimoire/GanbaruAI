import { Temporal } from "@js-temporal/polyfill";
import type {
  NotesDateMentionRichText,
  NotesDateMentionValue,
  NotesEquationRichText,
  NotesPageMentionRichText,
  NotesColor,
  NotesRichText,
  NotesRichTextAnnotations,
  NotesRichTextLink,
  NotesTextRichText,
} from "./types";

const DEFAULT_RICH_TEXT_ANNOTATIONS: NotesRichTextAnnotations = {
  bold: false,
  italic: false,
  strikethrough: false,
  underline: false,
  code: false,
  color: "default",
};

const MENTION_QUERY_LIMIT = 80;
const LINK_SCHEME_PATTERN = /^[a-z][a-z0-9+.-]*:/iu;
const EMAIL_ADDRESS_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/u;

export interface NotesPageMentionTarget {
  kind: "page";
  id: string;
  title: string;
  subtitle?: string;
  iconText?: string | null;
}

export interface NotesDateMentionTarget {
  kind: "date";
  id: string;
  title: string;
  subtitle: string;
  iconText?: string | null;
  date: NotesDateMentionValue;
  reminder: boolean;
}

export type NotesMentionTarget = NotesPageMentionTarget | NotesDateMentionTarget;

export interface NotesDateMentionLabels {
  today: string;
  tomorrow: string;
  yesterday: string;
  nextWeek: string;
  date: string;
  reminder: string;
  remindTitle: (dateLabel: string) => string;
}

export interface NotesMentionQuery {
  start: number;
  end: number;
  query: string;
}

export interface NotesRichTextLinkRange {
  start: number;
  end: number;
  url: string | null;
}

export type NotesRichTextAnnotationName =
  | "bold"
  | "italic"
  | "strikethrough"
  | "underline"
  | "code";

export type NotesRichTextAnnotationPatch =
  Partial<Pick<NotesRichTextAnnotations, NotesRichTextAnnotationName | "color">>;

export interface NotesRichTextAnnotationRange {
  start: number;
  end: number;
  annotations: NotesRichTextAnnotations;
}

export type NotesInlineEquationConversionError =
  | "selection_required"
  | "invalid_expression";

export type NotesInlineEquationConversionPlan =
  | {
    type: "convert";
    start: number;
    end: number;
    expression: string;
    cursor: number;
  }
  | {
    type: "error";
    reason: NotesInlineEquationConversionError;
    start: number;
    end: number;
  };

interface DateMentionCandidate {
  id: string;
  aliases: readonly string[];
  date: Temporal.PlainDate;
  title: string;
}

const WEEKDAY_ALIASES: readonly {
  dayOfWeek: number;
  aliases: readonly string[];
}[] = [
  { dayOfWeek: 1, aliases: ["monday", "mon", "lunes", "lun"] },
  { dayOfWeek: 2, aliases: ["tuesday", "tue", "martes", "mar"] },
  { dayOfWeek: 3, aliases: ["wednesday", "wed", "miercoles", "miércoles", "mie", "mié"] },
  { dayOfWeek: 4, aliases: ["thursday", "thu", "jueves", "jue"] },
  { dayOfWeek: 5, aliases: ["friday", "fri", "viernes", "vie"] },
  { dayOfWeek: 6, aliases: ["saturday", "sat", "sabado", "sábado", "sab", "sáb"] },
  { dayOfWeek: 7, aliases: ["sunday", "sun", "domingo", "dom"] },
];

const ISO_DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/u;

export function createTextRichText(content: string): NotesTextRichText {
  return {
    type: "text",
    text: {
      content,
      link: null,
    },
    annotations: { ...DEFAULT_RICH_TEXT_ANNOTATIONS },
    plain_text: content,
    href: null,
  };
}

export function createLinkedTextRichText(
  content: string,
  url: string,
): NotesTextRichText {
  return {
    type: "text",
    text: {
      content,
      link: { url },
    },
    annotations: { ...DEFAULT_RICH_TEXT_ANNOTATIONS },
    plain_text: content,
    href: url,
  };
}

export function createPageMentionRichText(
  pageId: string,
  title: string,
  href: string | null = null,
): NotesPageMentionRichText {
  const plainText = title.trim() || pageId;
  return {
    type: "mention",
    mention: {
      type: "page",
      page: {
        id: pageId,
      },
    },
    annotations: { ...DEFAULT_RICH_TEXT_ANNOTATIONS },
    plain_text: plainText,
    href,
  };
}

export function createDateMentionValue(
  start: string,
  reminder = false,
): NotesDateMentionValue {
  return {
    start,
    end: null,
    time_zone: null,
    ...(reminder ? { ganbaru_reminder: { enabled: true } } : {}),
  };
}

export function createDateMentionRichText(
  date: NotesDateMentionValue,
  title: string,
): NotesDateMentionRichText {
  return {
    type: "mention",
    mention: {
      type: "date",
      date,
    },
    annotations: { ...DEFAULT_RICH_TEXT_ANNOTATIONS },
    plain_text: title.trim() || date.start,
    href: null,
  };
}

export function normalizeRichTextEquationExpression(expression: string): string | null {
  const trimmed = expression.trim();
  if (!trimmed) return null;
  if (trimmed.length > 2048 || /[\u0000-\u001f]/u.test(trimmed)) return null;
  return trimmed;
}

export function planRichTextEquationConversion(
  text: string,
  selectionStart: number,
  selectionEnd: number,
): NotesInlineEquationConversionPlan {
  const rawStart = Math.min(selectionStart, selectionEnd);
  const rawEnd = Math.max(selectionStart, selectionEnd);
  const start = Math.max(0, Math.min(rawStart, text.length));
  const end = Math.max(start, Math.min(rawEnd, text.length));
  if (start === end) {
    return { type: "error", reason: "selection_required", start, end };
  }
  const expression = normalizeRichTextEquationExpression(text.slice(start, end));
  if (!expression) {
    return { type: "error", reason: "invalid_expression", start, end };
  }
  return {
    type: "convert",
    start,
    end,
    expression,
    cursor: start + expression.length,
  };
}

export function createEquationRichText(expression: string): NotesEquationRichText {
  const normalizedExpression = normalizeRichTextEquationExpression(expression) ?? "e=mc^2";
  return {
    type: "equation",
    equation: {
      expression: normalizedExpression,
    },
    annotations: { ...DEFAULT_RICH_TEXT_ANNOTATIONS },
    plain_text: normalizedExpression,
    href: null,
  };
}

export function richTextPlainText(richText: readonly NotesRichText[]): string {
  return richText.map((item) => item.plain_text).join("");
}

export function isPageMentionRichText(item: NotesRichText): item is NotesPageMentionRichText {
  return item.type === "mention" && item.mention.type === "page";
}

export function isDateMentionRichText(item: NotesRichText): item is NotesDateMentionRichText {
  return item.type === "mention" && item.mention.type === "date";
}

function cloneRichText(item: NotesRichText): NotesRichText {
  return structuredClone(item);
}

function richTextLinksEqual(
  left: NotesRichTextLink | null,
  right: NotesRichTextLink | null,
): boolean {
  return (left?.url ?? null) === (right?.url ?? null);
}

function annotationsEqual(
  left: NotesRichTextAnnotations,
  right: NotesRichTextAnnotations,
): boolean {
  return left.bold === right.bold
    && left.italic === right.italic
    && left.strikethrough === right.strikethrough
    && left.underline === right.underline
    && left.code === right.code
    && left.color === right.color;
}

export function defaultRichTextAnnotations(): NotesRichTextAnnotations {
  return { ...DEFAULT_RICH_TEXT_ANNOTATIONS };
}

function cloneTextRichTextWithContent(
  item: NotesTextRichText,
  content: string,
): NotesTextRichText {
  const clone: NotesTextRichText = structuredClone(item);
  return {
    ...clone,
    text: {
      ...item.text,
      content,
    },
    plain_text: content,
  };
}

function appendTextItem(items: NotesRichText[], item: NotesTextRichText): void {
  if (!item.plain_text) return;
  const previous = items.at(-1);
  if (
    previous?.type === "text"
    && annotationsEqual(previous.annotations, item.annotations)
    && richTextLinksEqual(previous.text.link, item.text.link)
    && previous.href === item.href
  ) {
    previous.text.content += item.text.content;
    previous.plain_text += item.plain_text;
    return;
  }
  items.push(cloneRichText(item));
}

function appendText(items: NotesRichText[], content: string): void {
  if (!content) return;
  appendTextItem(items, createTextRichText(content));
}

function appendRichTextSlice(
  output: NotesRichText[],
  item: NotesRichText,
  startOffset: number,
  endOffset: number,
): void {
  const itemText = item.plain_text;
  if (startOffset >= endOffset) return;
  if (startOffset === 0 && endOffset === itemText.length) {
    output.push(cloneRichText(item));
    return;
  }
  if (item.type === "text") {
    appendTextItem(output, cloneTextRichTextWithContent(item, itemText.slice(startOffset, endOffset)));
  }
}

function richTextSlice(
  richText: readonly NotesRichText[],
  start: number,
  end: number,
): NotesRichText[] {
  const output: NotesRichText[] = [];
  let cursor = 0;
  for (const item of richText) {
    const itemText = item.plain_text;
    const itemStart = cursor;
    const itemEnd = itemStart + itemText.length;
    cursor = itemEnd;
    if (end <= itemStart) break;
    if (start >= itemEnd) continue;
    appendRichTextSlice(
      output,
      item,
      Math.max(0, start - itemStart),
      Math.min(itemText.length, end - itemStart),
    );
  }
  return output;
}

export function insertPageMentionRichText(
  richText: readonly NotesRichText[],
  start: number,
  end: number,
  pageId: string,
  title: string,
  href: string | null = null,
): NotesRichText[] {
  const plainText = richTextPlainText(richText);
  const safeStart = Math.max(0, Math.min(start, plainText.length));
  const safeEnd = Math.max(safeStart, Math.min(end, plainText.length));
  return [
    ...richTextSlice(richText, 0, safeStart),
    createPageMentionRichText(pageId, title, href),
    ...richTextSlice(richText, safeEnd, plainText.length),
  ];
}

export function insertDateMentionRichText(
  richText: readonly NotesRichText[],
  start: number,
  end: number,
  date: NotesDateMentionValue,
  title: string,
): NotesRichText[] {
  const plainText = richTextPlainText(richText);
  const safeStart = Math.max(0, Math.min(start, plainText.length));
  const safeEnd = Math.max(safeStart, Math.min(end, plainText.length));
  return [
    ...richTextSlice(richText, 0, safeStart),
    createDateMentionRichText(date, title),
    ...richTextSlice(richText, safeEnd, plainText.length),
  ];
}

export function insertEquationRichText(
  richText: readonly NotesRichText[],
  start: number,
  end: number,
  expression: string,
): NotesRichText[] {
  const normalizedExpression = normalizeRichTextEquationExpression(expression);
  if (!normalizedExpression) return richText.map(cloneRichText);
  const plainText = richTextPlainText(richText);
  const safeStart = Math.max(0, Math.min(start, plainText.length));
  const safeEnd = Math.max(safeStart, Math.min(end, plainText.length));
  if (safeStart === safeEnd) return richText.map(cloneRichText);
  return [
    ...richTextSlice(richText, 0, safeStart),
    createEquationRichText(normalizedExpression),
    ...richTextSlice(richText, safeEnd, plainText.length),
  ];
}

export function normalizeRichTextLinkUrl(rawUrl: string): string | null {
  const trimmed = rawUrl.trim();
  if (!trimmed) return null;
  if (trimmed.length > 2048 || /[\u0000-\u001f]/u.test(trimmed)) return null;
  const hasScheme = LINK_SCHEME_PATTERN.test(trimmed);
  if (!hasScheme && EMAIL_ADDRESS_PATTERN.test(trimmed)) return `mailto:${trimmed}`;
  if (!hasScheme && trimmed.includes("@")) return null;
  const candidate = hasScheme ? trimmed : `https://${trimmed}`;
  try {
    const parsed = new URL(candidate);
    if (parsed.protocol === "mailto:") {
      const address = parsed.pathname.trim();
      if (!EMAIL_ADDRESS_PATTERN.test(address)) return null;
      return parsed.toString();
    }
    if (!["http:", "https:"].includes(parsed.protocol)) return null;
    return parsed.toString();
  } catch {
    return null;
  }
}

function itemLinkUrl(item: NotesRichText): string | null {
  return item.type === "text" ? item.text.link?.url ?? null : null;
}

function itemAnnotations(item: NotesRichText): NotesRichTextAnnotations {
  return item.annotations;
}

function annotationsForRange(
  richText: readonly NotesRichText[],
  start: number,
  end: number,
): NotesRichTextAnnotations {
  let cursor = 0;
  let commonAnnotations: NotesRichTextAnnotations | null = null;
  for (const item of richText) {
    const itemStart = cursor;
    const itemEnd = itemStart + item.plain_text.length;
    cursor = itemEnd;
    if (end <= itemStart) break;
    if (start >= itemEnd || item.type !== "text") continue;
    if (!commonAnnotations) {
      commonAnnotations = { ...itemAnnotations(item) };
      continue;
    }
    const annotations = itemAnnotations(item);
    commonAnnotations = {
      bold: commonAnnotations.bold && annotations.bold,
      italic: commonAnnotations.italic && annotations.italic,
      strikethrough: commonAnnotations.strikethrough && annotations.strikethrough,
      underline: commonAnnotations.underline && annotations.underline,
      code: commonAnnotations.code && annotations.code,
      color: commonAnnotations.color === annotations.color ? commonAnnotations.color : "default",
    };
  }
  return commonAnnotations ?? defaultRichTextAnnotations();
}

function annotationsAtCursor(
  richText: readonly NotesRichText[],
  cursorPosition: number,
): NotesRichTextAnnotations {
  let cursor = 0;
  let previousTextAnnotations: NotesRichTextAnnotations | null = null;
  for (const item of richText) {
    const itemStart = cursor;
    const itemEnd = itemStart + item.plain_text.length;
    cursor = itemEnd;
    if (item.type !== "text") continue;
    if (cursorPosition >= itemStart && cursorPosition < itemEnd) {
      return { ...itemAnnotations(item) };
    }
    if (cursorPosition === itemEnd) previousTextAnnotations = itemAnnotations(item);
    if (cursorPosition < itemEnd) break;
  }
  return previousTextAnnotations ? { ...previousTextAnnotations } : defaultRichTextAnnotations();
}

function commonLinkUrlInRange(
  richText: readonly NotesRichText[],
  start: number,
  end: number,
): string | null {
  let cursor = 0;
  let commonUrl: string | null | undefined;
  for (const item of richText) {
    const itemStart = cursor;
    const itemEnd = itemStart + item.plain_text.length;
    cursor = itemEnd;
    if (end <= itemStart) break;
    if (start >= itemEnd || item.type !== "text") continue;
    const url = itemLinkUrl(item);
    if (!url) continue;
    if (commonUrl === undefined) {
      commonUrl = url;
      continue;
    }
    if (commonUrl !== url) return null;
  }
  return commonUrl ?? null;
}

export function richTextLinkRangeForSelection(
  richText: readonly NotesRichText[],
  selectionStart: number,
  selectionEnd: number,
): NotesRichTextLinkRange {
  const plainText = richTextPlainText(richText);
  const safeStart = Math.max(0, Math.min(selectionStart, plainText.length));
  const safeEnd = Math.max(safeStart, Math.min(selectionEnd, plainText.length));
  if (safeStart !== safeEnd) {
    return {
      start: safeStart,
      end: safeEnd,
      url: commonLinkUrlInRange(richText, safeStart, safeEnd),
    };
  }

  let rangeCursor = 0;
  const ranges = richText.map((item) => {
    const start = rangeCursor;
    const end = start + item.plain_text.length;
    rangeCursor = end;
    return { start, end, url: itemLinkUrl(item) };
  });
  const rangeIndex = ranges.findIndex(
    (range) =>
      range.url !== null
      && (
        (safeStart >= range.start && safeStart < range.end)
        || (safeStart === range.end && range.end > range.start)
      ),
  );
  if (rangeIndex !== -1) {
    const url = ranges[rangeIndex]?.url ?? null;
    let rangeStart = ranges[rangeIndex]?.start ?? safeStart;
    let rangeEnd = ranges[rangeIndex]?.end ?? safeStart;
    for (let index = rangeIndex - 1; index >= 0; index -= 1) {
      const range = ranges[index];
      if (!range || range.url !== url || range.end !== rangeStart) break;
      rangeStart = range.start;
    }
    for (let index = rangeIndex + 1; index < ranges.length; index += 1) {
      const range = ranges[index];
      if (!range || range.url !== url || range.start !== rangeEnd) break;
      rangeEnd = range.end;
    }
    return { start: rangeStart, end: rangeEnd, url };
  }

  return { start: safeStart, end: safeEnd, url: null };
}

export function richTextAnnotationsForSelection(
  richText: readonly NotesRichText[],
  selectionStart: number,
  selectionEnd: number,
): NotesRichTextAnnotationRange {
  const plainText = richTextPlainText(richText);
  const safeStart = Math.max(0, Math.min(selectionStart, plainText.length));
  const safeEnd = Math.max(safeStart, Math.min(selectionEnd, plainText.length));
  return {
    start: safeStart,
    end: safeEnd,
    annotations: safeStart === safeEnd
      ? annotationsAtCursor(richText, safeStart)
      : annotationsForRange(richText, safeStart, safeEnd),
  };
}

export function applyRichTextLink(
  richText: readonly NotesRichText[],
  start: number,
  end: number,
  url: string | null,
): NotesRichText[] {
  const plainText = richTextPlainText(richText);
  const safeStart = Math.max(0, Math.min(start, plainText.length));
  const safeEnd = Math.max(safeStart, Math.min(end, plainText.length));
  if (safeStart === safeEnd) return richText.map(cloneRichText);
  const normalizedUrl = url === null ? null : normalizeRichTextLinkUrl(url);
  if (url !== null && normalizedUrl === null) return richText.map(cloneRichText);
  const output: NotesRichText[] = [];
  let cursor = 0;
  for (const item of richText) {
    const itemText = item.plain_text;
    const itemStart = cursor;
    const itemEnd = itemStart + itemText.length;
    cursor = itemEnd;
    if (itemEnd <= safeStart || itemStart >= safeEnd) {
      output.push(cloneRichText(item));
      continue;
    }
    appendRichTextSlice(output, item, 0, Math.max(0, safeStart - itemStart));
    const middleStart = Math.max(0, safeStart - itemStart);
    const middleEnd = Math.min(itemText.length, safeEnd - itemStart);
    if (item.type === "text") {
      const nextText = cloneTextRichTextWithContent(
        item,
        itemText.slice(middleStart, middleEnd),
      );
      nextText.text.link = normalizedUrl ? { url: normalizedUrl } : null;
      nextText.href = normalizedUrl;
      appendTextItem(output, nextText);
    } else {
      appendRichTextSlice(output, item, middleStart, middleEnd);
    }
    appendRichTextSlice(output, item, Math.min(itemText.length, safeEnd - itemStart), itemText.length);
  }
  return output.length > 0 ? output : [createTextRichText("")];
}

export function applyRichTextAnnotations(
  richText: readonly NotesRichText[],
  start: number,
  end: number,
  patch: NotesRichTextAnnotationPatch,
): NotesRichText[] {
  const plainText = richTextPlainText(richText);
  const safeStart = Math.max(0, Math.min(start, plainText.length));
  const safeEnd = Math.max(safeStart, Math.min(end, plainText.length));
  if (safeStart === safeEnd) return richText.map(cloneRichText);
  const output: NotesRichText[] = [];
  let cursor = 0;
  for (const item of richText) {
    const itemText = item.plain_text;
    const itemStart = cursor;
    const itemEnd = itemStart + itemText.length;
    cursor = itemEnd;
    if (itemEnd <= safeStart || itemStart >= safeEnd) {
      output.push(cloneRichText(item));
      continue;
    }
    appendRichTextSlice(output, item, 0, Math.max(0, safeStart - itemStart));
    const middleStart = Math.max(0, safeStart - itemStart);
    const middleEnd = Math.min(itemText.length, safeEnd - itemStart);
    if (item.type === "text") {
      const nextText = cloneTextRichTextWithContent(
        item,
        itemText.slice(middleStart, middleEnd),
      );
      nextText.annotations = {
        ...nextText.annotations,
        ...patch,
      };
      appendTextItem(output, nextText);
    } else {
      appendRichTextSlice(output, item, middleStart, middleEnd);
    }
    appendRichTextSlice(
      output,
      item,
      Math.min(itemText.length, safeEnd - itemStart),
      itemText.length,
    );
  }
  return output.length > 0 ? output : [createTextRichText("")];
}

export function richTextHasVisibleFormatting(
  richText: readonly NotesRichText[],
): boolean {
  return richText.some((item) => {
    if (item.type !== "text") return true;
    return item.text.link !== null
      || item.href !== null
      || item.annotations.bold
      || item.annotations.italic
      || item.annotations.strikethrough
      || item.annotations.underline
      || item.annotations.code
      || item.annotations.color !== "default";
  });
}

export function richTextAnnotationTogglePatch(
  annotations: NotesRichTextAnnotations,
  name: NotesRichTextAnnotationName,
): NotesRichTextAnnotationPatch {
  switch (name) {
    case "bold":
      return { bold: !annotations.bold };
    case "italic":
      return { italic: !annotations.italic };
    case "strikethrough":
      return { strikethrough: !annotations.strikethrough };
    case "underline":
      return { underline: !annotations.underline };
    case "code":
      return { code: !annotations.code };
  }
}

export function richTextColorPatch(color: NotesColor): NotesRichTextAnnotationPatch {
  return { color };
}

export function replacePlainTextPreservingRichText(
  richText: readonly NotesRichText[],
  nextPlainText: string,
): NotesRichText[] {
  const anchors = richText.filter((item) => item.type !== "text");
  if (anchors.length === 0) return [createTextRichText(nextPlainText)];

  const output: NotesRichText[] = [];
  let cursor = 0;
  for (const anchor of anchors) {
    const anchorText = anchor.plain_text;
    if (!anchorText) return [createTextRichText(nextPlainText)];
    const anchorIndex = nextPlainText.indexOf(anchorText, cursor);
    if (anchorIndex === -1) return [createTextRichText(nextPlainText)];
    appendText(output, nextPlainText.slice(cursor, anchorIndex));
    output.push(cloneRichText(anchor));
    cursor = anchorIndex + anchorText.length;
  }
  appendText(output, nextPlainText.slice(cursor));
  return output.length > 0 ? output : [createTextRichText("")];
}

export function detectPageMentionQuery(
  text: string,
  selectionStart: number,
  selectionEnd: number,
): NotesMentionQuery | null {
  if (selectionStart !== selectionEnd) return null;
  const cursor = Math.max(0, Math.min(selectionStart, text.length));
  const lineStart = text.lastIndexOf("\n", cursor - 1) + 1;
  const atIndex = text.lastIndexOf("@", cursor - 1);
  if (atIndex < lineStart) return null;
  const before = atIndex === 0 ? "" : text.at(atIndex - 1) ?? "";
  if (before && !/[\s([{:;,]/u.test(before)) return null;
  const query = text.slice(atIndex + 1, cursor);
  if (query.length > MENTION_QUERY_LIMIT || query.includes("@")) return null;
  return { start: atIndex, end: cursor, query };
}

export function filterPageMentionTargets(
  targets: readonly NotesPageMentionTarget[],
  query: string,
  limit = 8,
): NotesPageMentionTarget[] {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const matches = normalizedQuery
    ? targets.filter((target) => target.title.toLocaleLowerCase().includes(normalizedQuery))
    : [...targets];
  return matches
    .sort((left, right) => {
      const leftTitle = left.title.toLocaleLowerCase();
      const rightTitle = right.title.toLocaleLowerCase();
      const leftStarts = normalizedQuery && leftTitle.startsWith(normalizedQuery);
      const rightStarts = normalizedQuery && rightTitle.startsWith(normalizedQuery);
      if (leftStarts !== rightStarts) return leftStarts ? -1 : 1;
      return left.title.localeCompare(right.title);
    })
    .slice(0, limit);
}

function normalizeDateMentionQuery(query: string): string {
  return query
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .trim()
    .replace(/\s+/gu, " ")
    .toLocaleLowerCase();
}

function splitReminderQuery(query: string): { reminder: boolean; dateQuery: string } {
  const normalized = normalizeDateMentionQuery(query);
  for (const prefix of ["remind me", "remind", "reminder", "recordar", "recuerdame", "recuérdame"]) {
    if (normalized === prefix) return { reminder: true, dateQuery: "" };
    if (normalized.startsWith(`${prefix} `)) {
      return { reminder: true, dateQuery: normalized.slice(prefix.length + 1).trim() };
    }
  }
  return { reminder: false, dateQuery: normalized };
}

function dateFromIsoQuery(query: string): Temporal.PlainDate | null {
  if (!ISO_DATE_PATTERN.test(query)) return null;
  try {
    return Temporal.PlainDate.from(query);
  } catch {
    return null;
  }
}

function formatPlainDate(date: Temporal.PlainDate, locale: string): string {
  return new Intl.DateTimeFormat(locale, {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(date.year, date.month - 1, date.day));
}

function formatWeekday(date: Temporal.PlainDate, locale: string): string {
  return new Intl.DateTimeFormat(locale, { weekday: "long" }).format(
    new Date(date.year, date.month - 1, date.day),
  );
}

function nextWeekday(today: Temporal.PlainDate, dayOfWeek: number, forceNext: boolean): Temporal.PlainDate {
  const offset = (dayOfWeek - today.dayOfWeek + 7) % 7;
  return today.add({ days: offset === 0 && forceNext ? 7 : offset });
}

function baseDateMentionCandidates(
  today: Temporal.PlainDate,
  labels: NotesDateMentionLabels,
): DateMentionCandidate[] {
  return [
    {
      id: "today",
      aliases: ["today", "tod", "hoy"],
      date: today,
      title: labels.today,
    },
    {
      id: "tomorrow",
      aliases: ["tomorrow", "tom", "manana", "mañana"],
      date: today.add({ days: 1 }),
      title: labels.tomorrow,
    },
    {
      id: "yesterday",
      aliases: ["yesterday", "yest", "ayer"],
      date: today.subtract({ days: 1 }),
      title: labels.yesterday,
    },
    {
      id: "next-week",
      aliases: ["next week", "week", "proxima semana", "próxima semana", "semana"],
      date: today.add({ weeks: 1 }),
      title: labels.nextWeek,
    },
  ];
}

function weekdayDateMentionCandidates(
  query: string,
  today: Temporal.PlainDate,
  locale: string,
): DateMentionCandidate[] {
  const forceNext = query.startsWith("next ") || query.startsWith("proximo ") || query.startsWith("proxima ");
  const normalizedWeekdayQuery = forceNext ? query.replace(/^(next|proximo|proxima)\s+/u, "") : query;
  return WEEKDAY_ALIASES
    .filter((weekday) =>
      weekday.aliases.some((alias) =>
        alias.includes(normalizedWeekdayQuery) || normalizedWeekdayQuery.includes(alias)
      )
    )
    .map((weekday) => {
      const date = nextWeekday(today, weekday.dayOfWeek, forceNext);
      return {
        id: `${forceNext ? "next-" : ""}weekday-${weekday.dayOfWeek}`,
        aliases: weekday.aliases,
        date,
        title: formatWeekday(date, locale),
      };
    });
}

function dateMentionCandidateMatches(candidate: DateMentionCandidate, query: string): boolean {
  if (!query) return true;
  const title = normalizeDateMentionQuery(candidate.title);
  return title.includes(query)
    || candidate.date.toString().includes(query)
    || candidate.aliases.some((alias) => alias.includes(query) || query.includes(alias));
}

function deduplicateDateCandidates(
  candidates: readonly DateMentionCandidate[],
): DateMentionCandidate[] {
  const seen = new Set<string>();
  const output: DateMentionCandidate[] = [];
  for (const candidate of candidates) {
    const key = candidate.date.toString();
    if (seen.has(key)) continue;
    seen.add(key);
    output.push(candidate);
  }
  return output;
}

export function buildDateMentionTargets(
  query: string,
  options: {
    today: Temporal.PlainDate;
    locale: string;
    labels: NotesDateMentionLabels;
    limit?: number;
  },
): NotesDateMentionTarget[] {
  const { reminder, dateQuery } = splitReminderQuery(query);
  const isoDate = dateFromIsoQuery(dateQuery);
  const isoCandidate: DateMentionCandidate[] = isoDate
    ? [{
        id: "iso-date",
        aliases: [isoDate.toString()],
        date: isoDate,
        title: formatPlainDate(isoDate, options.locale),
      }]
    : [];
  const baseCandidates = baseDateMentionCandidates(options.today, options.labels);
  const weekdayCandidates = dateQuery
    ? weekdayDateMentionCandidates(dateQuery, options.today, options.locale)
    : [];
  const candidates = deduplicateDateCandidates([
    ...isoCandidate,
    ...weekdayCandidates,
    ...baseCandidates.filter((candidate) => dateMentionCandidateMatches(candidate, dateQuery)),
  ]);
  return candidates.slice(0, options.limit ?? 6).map((candidate) => {
    const title = reminder ? options.labels.remindTitle(candidate.title) : candidate.title;
    return {
      kind: "date",
      id: `${reminder ? "reminder" : "date"}:${candidate.date.toString()}`,
      title,
      subtitle: reminder ? options.labels.reminder : options.labels.date,
      iconText: null,
      date: createDateMentionValue(candidate.date.toString(), reminder),
      reminder,
    };
  });
}
