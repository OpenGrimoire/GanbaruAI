import {
  canBlockHaveChildren,
  isTextEditableBlock,
} from "./block-factory";
import {
  replaceRichTextRange,
  richTextPlainText,
} from "./rich-text";
import type { NotesBlock, NotesBlockType, NotesRichText } from "./types";

export function notesBackspaceCanMergeBlockTypes(
  sourceType: NotesBlockType,
  targetType: NotesBlockType,
): boolean {
  return isTextEditableBlock(sourceType) && isTextEditableBlock(targetType);
}

export function mergeRichTextForBackspace(
  targetRichText: readonly NotesRichText[],
  sourceRichText: readonly NotesRichText[],
): NotesRichText[] {
  const targetText = richTextPlainText(targetRichText);
  return replaceRichTextRange(targetRichText, targetText.length, targetText.length, sourceRichText);
}

export function notesParentCanAcceptBlockType(
  parentBlock: NotesBlock | null,
  childType: NotesBlockType,
): boolean {
  if (parentBlock?.type === "column_list") return childType === "column";
  if (parentBlock?.type === "table") return childType === "table_row";
  if (parentBlock?.type === "tab") return childType === "paragraph";
  if (childType === "column" || childType === "table_row") return false;
  return parentBlock === null || canBlockHaveChildren(parentBlock);
}
