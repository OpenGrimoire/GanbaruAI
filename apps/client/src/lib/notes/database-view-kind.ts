export const NOTES_DATABASE_VIEW_KINDS = [
  "table",
  "board",
  "gallery",
  "list",
  "calendar",
  "timeline",
] as const;

export type NotesDatabaseViewKind = (typeof NOTES_DATABASE_VIEW_KINDS)[number];
