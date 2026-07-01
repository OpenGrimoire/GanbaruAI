import type { NotesBlock, NotesPage } from "./types";

export interface NotesMoveToPageTarget {
  id: string;
  title: string;
}

/** Return pages that can receive this block through the handle move menu. */
export function notesMoveToPageTargets(
  pages: readonly NotesPage[],
  block: NotesBlock,
  currentPageId: string,
  untitledLabel: string,
): NotesMoveToPageTarget[] {
  return pages
    .filter((page) => !page.in_trash)
    .filter((page) => page.id !== currentPageId)
    .filter((page) => !(block.type === "child_page" && page.id === block.id))
    .map((page) => ({
      id: page.id,
      title: pageTitleForMoveTarget(page, untitledLabel),
    }))
    .sort((left, right) => left.title.localeCompare(right.title, undefined, { sensitivity: "base" }));
}

function pageTitleForMoveTarget(page: NotesPage, untitledLabel: string): string {
  const trimmed = pageTitleCache(page);
  return trimmed || untitledLabel;
}

function pageTitleCache(page: NotesPage): string {
  const title = page.properties.title;
  if (!isTitleProperty(title)) return "";
  return title.title
    .map((item) => item.plain_text)
    .join("")
    .trim();
}

function isTitleProperty(value: unknown): value is { title: { plain_text: string }[] } {
  if (!value || typeof value !== "object" || !("title" in value)) return false;
  const title = (value as { title: unknown }).title;
  return Array.isArray(title)
    && title.every(
      (item) =>
        !!item
        && typeof item === "object"
        && "plain_text" in item
        && typeof (item as { plain_text: unknown }).plain_text === "string",
    );
}
