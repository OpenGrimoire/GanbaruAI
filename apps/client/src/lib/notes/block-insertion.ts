import type { NotesBlockType } from "./types";

export const NOTES_INSERTABLE_BLOCK_TYPES = [
  "paragraph",
  "heading_1",
  "heading_2",
  "heading_3",
  "heading_4",
  "bulleted_list_item",
  "numbered_list_item",
  "to_do",
  "toggle",
  "callout",
  "quote",
  "child_page",
  "breadcrumb",
  "table_of_contents",
  "column_list",
  "table",
  "tab",
  "image",
  "video",
  "audio",
  "file",
  "pdf",
  "bookmark",
  "link_preview",
  "template",
  "button",
  "embed",
  "equation",
  "divider",
  "code",
] as const satisfies readonly NotesBlockType[];

export type NotesInsertableBlockType = (typeof NOTES_INSERTABLE_BLOCK_TYPES)[number];

export function isNotesInsertableBlockType(value: unknown): value is NotesInsertableBlockType {
  return (
    typeof value === "string"
    && NOTES_INSERTABLE_BLOCK_TYPES.includes(value as NotesInsertableBlockType)
  );
}

export function notesInsertableBlockTypes(): readonly NotesInsertableBlockType[] {
  return NOTES_INSERTABLE_BLOCK_TYPES;
}
