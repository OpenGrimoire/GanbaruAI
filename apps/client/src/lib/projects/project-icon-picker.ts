import {
  parseProjectIcon,
  projectIconRecentValue,
  serializeProjectIcon,
  type ProjectIconValue,
} from "$lib/projects/project-icons";
import type {
  ProjectEmojiCategoryId,
  ProjectEmojiEntry,
} from "$lib/projects/project-emoji-catalog";
import {
  PROJECT_EMOJI_SKIN_TONE_BASES,
  PROJECT_EMOJI_SKIN_TONE_VARIANTS,
} from "$lib/projects/project-emoji-catalog";
import type {
  ProjectLucideCategory,
  ProjectLucideIconEntry,
} from "$lib/projects/project-lucide-catalog.generated";

export const PROJECT_ICON_RECENT_LIMIT = 12;
export const PROJECT_EMOJI_SKIN_TONES = [
  "default",
  "light",
  "medium-light",
  "medium",
  "medium-dark",
  "dark",
] as const;

export type ProjectEmojiSkinTone = (typeof PROJECT_EMOJI_SKIN_TONES)[number];

const PROJECT_EMOJI_SKIN_TONE_MODIFIERS: Record<ProjectEmojiSkinTone, string> = {
  default: "",
  light: "🏻",
  "medium-light": "🏼",
  medium: "🏽",
  "medium-dark": "🏾",
  dark: "🏿",
};

const PROJECT_EMOJI_SKIN_TONE_PATTERN = /[\u{1f3fb}-\u{1f3ff}]/gu;
const PROJECT_EMOJI_SKIN_TONE_SUPPORTED = new Set<string>(PROJECT_EMOJI_SKIN_TONE_BASES);
type ProjectEmojiVariantSkinTone = Exclude<ProjectEmojiSkinTone, "default">;

export interface ProjectIconVirtualWindow {
  startIndex: number;
  endIndex: number;
  beforeHeight: number;
  afterHeight: number;
}

export function stripProjectEmojiSkinTone(emoji: string): string {
  return emoji.replace(PROJECT_EMOJI_SKIN_TONE_PATTERN, "");
}

export function projectEmojiSkinToneFromEmoji(emoji: string): ProjectEmojiSkinTone {
  for (const skinTone of PROJECT_EMOJI_SKIN_TONES) {
    if (skinTone !== "default" && emoji.includes(PROJECT_EMOJI_SKIN_TONE_MODIFIERS[skinTone])) {
      return skinTone;
    }
  }
  return "default";
}

export function applyProjectEmojiSkinTone(
  emoji: string,
  skinTone: ProjectEmojiSkinTone,
): string {
  const baseEmoji = stripProjectEmojiSkinTone(emoji);
  if (skinTone === "default" || !PROJECT_EMOJI_SKIN_TONE_SUPPORTED.has(baseEmoji)) {
    return baseEmoji;
  }
  return PROJECT_EMOJI_SKIN_TONE_VARIANTS[baseEmoji]?.[skinTone as ProjectEmojiVariantSkinTone] ?? baseEmoji;
}

function normalizedQuery(query: string): string {
  return query.trim().toLowerCase();
}

function matchesQuery(terms: string, query: string): boolean {
  const normalized = normalizedQuery(query);
  return !normalized || terms.toLowerCase().includes(normalized);
}

export function filterProjectEmojiEntries(
  entries: readonly ProjectEmojiEntry[],
  query: string,
  category: ProjectEmojiCategoryId | "all",
): ProjectEmojiEntry[] {
  return entries.filter((entry) =>
    (category === "all"
      || entry.category === category
      || (category === "symbols" && entry.category === "flags"))
    && matchesQuery(entry.terms, query)
  );
}

export function filterProjectLucideIcons(
  entries: readonly ProjectLucideIconEntry[],
  query: string,
  category: ProjectLucideCategory | "all",
): ProjectLucideIconEntry[] {
  return entries.filter((entry) =>
    (category === "all" || entry.category === category)
    && matchesQuery(entry.terms, query)
  );
}

export function cleanupProjectIconRecentValues(
  values: readonly string[],
  customEmojiIds: ReadonlySet<string>,
  limit = PROJECT_ICON_RECENT_LIMIT,
): string[] {
  const recent: string[] = [];
  const seen = new Set<string>();
  for (const rawValue of values) {
    const value = parseProjectIcon(rawValue);
    if (value.kind === "none") continue;
    if (value.kind === "custom-emoji" && !customEmojiIds.has(value.id)) continue;
    const serialized = serializeProjectIcon(value);
    if (seen.has(serialized)) continue;
    seen.add(serialized);
    recent.push(serialized);
    if (recent.length >= limit) break;
  }
  return recent;
}

export function prependProjectIconRecentValue(
  values: readonly string[],
  value: ProjectIconValue,
  customEmojiIds: ReadonlySet<string>,
  limit = PROJECT_ICON_RECENT_LIMIT,
): string[] {
  const recentValue = projectIconRecentValue(value);
  if (!recentValue) {
    return cleanupProjectIconRecentValues(values, customEmojiIds, limit);
  }
  return cleanupProjectIconRecentValues([recentValue, ...values], customEmojiIds, limit);
}

export function projectIconVirtualWindow(
  itemCount: number,
  columnCount: number,
  rowHeight: number,
  viewportHeight: number,
  scrollTop: number,
  overscanRows = 2,
): ProjectIconVirtualWindow {
  const safeColumnCount = Math.max(1, Math.floor(columnCount));
  const safeRowHeight = Math.max(1, rowHeight);
  const rowCount = Math.ceil(itemCount / safeColumnCount);
  const firstVisibleRow = Math.floor(Math.max(0, scrollTop) / safeRowHeight);
  const visibleRowCount = Math.ceil(Math.max(0, viewportHeight) / safeRowHeight);
  const startRow = Math.max(0, firstVisibleRow - overscanRows);
  const endRow = Math.min(rowCount, firstVisibleRow + visibleRowCount + overscanRows + 1);
  const startIndex = Math.min(itemCount, startRow * safeColumnCount);
  const endIndex = Math.min(itemCount, endRow * safeColumnCount);
  return {
    startIndex,
    endIndex,
    beforeHeight: startRow * safeRowHeight,
    afterHeight: Math.max(0, (rowCount - endRow) * safeRowHeight),
  };
}
