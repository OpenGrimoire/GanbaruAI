import { richTextPlainText } from "./block-factory";
import type { NotesBookmarkBlockPayload } from "./types";

export function bookmarkCaptionPlainText(bookmark: NotesBookmarkBlockPayload): string {
  return richTextPlainText(bookmark.caption);
}

export function canOpenBookmarkUrl(url: string): boolean {
  const trimmed = url.trim();
  if (!trimmed) return false;
  try {
    const parsed = new URL(trimmed);
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch {
    return false;
  }
}
