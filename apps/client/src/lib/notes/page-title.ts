import { richTextPlainText } from "./block-factory";
import { parseNotesRichTextArray } from "./block-validation";
import type { NotesPage } from "./types";

export function notesPageTitle(page: NotesPage, fallback = ""): string {
  const title = page.properties.title;
  if (typeof title !== "object" || title === null || Array.isArray(title)) return fallback;
  if (!Object.hasOwn(title, "title")) return fallback;
  const titleRecord = title as Record<string, unknown>;
  try {
    return richTextPlainText(parseNotesRichTextArray(titleRecord.title, "page.properties.title.title"))
      .trim() || fallback;
  } catch {
    return fallback;
  }
}
