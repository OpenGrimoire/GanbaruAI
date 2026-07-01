import type { NotesRichTextAnnotationName } from "./rich-text";

export interface NotesRichTextFormattingShortcutInput {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
}

/**
 * Resolve a selected-text formatting shortcut to the annotation it toggles.
 */
export function notesRichTextFormattingShortcutAnnotationName(
  input: NotesRichTextFormattingShortcutInput,
): NotesRichTextAnnotationName | null {
  if (!(input.ctrlKey || input.metaKey) || input.altKey) return null;
  const key = input.key.toLowerCase();
  if (!input.shiftKey && key === "b") return "bold";
  if (!input.shiftKey && key === "i") return "italic";
  if (!input.shiftKey && key === "u") return "underline";
  if (input.shiftKey && key === "s") return "strikethrough";
  if (!input.shiftKey && key === "e") return "code";
  return null;
}
