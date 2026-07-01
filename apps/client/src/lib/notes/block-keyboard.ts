import {
  blockTypeForTextShortcut,
  isTextShortcutTriggerKey,
} from "./block-shortcuts";
import {
  notesEmptyEnterReturnsParagraph,
  notesEnterSplitsRichTextBlock,
} from "./block-enter";
import { notesBackspaceCanMergeBlockTypes } from "./block-backspace";
import type { NotesBlockType } from "./types";

export type NotesKeyboardAction =
  | { type: "none" }
  | { type: "open_slash_menu"; preventDefault: false }
  | { type: "insert_newline"; preventDefault: false }
  | { type: "create_sibling"; preventDefault: true }
  | {
    type: "split_text_block";
    selectionStart: number;
    selectionEnd: number;
    preventDefault: true;
  }
  | { type: "convert_to_paragraph"; preventDefault: true }
  | { type: "apply_text_shortcut"; blockType: NotesBlockType; preventDefault: true }
  | { type: "toggle_block_open"; preventDefault: true }
  | { type: "delete_block"; preventDefault: true }
  | { type: "merge_with_previous"; preventDefault: true }
  | { type: "nest"; preventDefault: true }
  | { type: "outdent"; preventDefault: true }
  | { type: "move_up"; preventDefault: true }
  | { type: "move_down"; preventDefault: true };

export interface NotesKeyboardPlanInput {
  key: string;
  shiftKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  text: string;
  selectionStart: number;
  selectionEnd: number;
  blockType: NotesBlockType;
  previousBlockType: NotesBlockType | null;
  isOnlyBlock: boolean;
}

/** Plan editor behavior from key state without touching the DOM. */
export function planNotesKeyboardAction(input: NotesKeyboardPlanInput): NotesKeyboardAction {
  if (input.altKey) return { type: "none" };

  const primaryModifier = input.ctrlKey || input.metaKey;
  if (primaryModifier && input.shiftKey && input.key === "ArrowUp") {
    return { type: "move_up", preventDefault: true };
  }
  if (primaryModifier && input.shiftKey && input.key === "ArrowDown") {
    return { type: "move_down", preventDefault: true };
  }

  if (input.metaKey) return { type: "none" };

  if (
    input.key === "/"
    && !input.ctrlKey
    && !input.shiftKey
    && input.selectionStart === 0
    && input.selectionEnd === 0
    && input.text.length === 0
  ) {
    return { type: "open_slash_menu", preventDefault: false };
  }

  if (input.key === "Tab" && !input.ctrlKey) {
    return input.shiftKey
      ? { type: "outdent", preventDefault: true }
      : { type: "nest", preventDefault: true };
  }

  if (input.key === "Enter") {
    if (input.ctrlKey && input.blockType === "toggle") {
      return { type: "toggle_block_open", preventDefault: true };
    }
    if (input.blockType === "code" && !input.ctrlKey) {
      return { type: "insert_newline", preventDefault: false };
    }
    if (input.blockType === "code" && input.ctrlKey) {
      return { type: "create_sibling", preventDefault: true };
    }
    if (input.shiftKey) {
      return { type: "insert_newline", preventDefault: false };
    }
    if (input.blockType === "paragraph" && isTextShortcutTriggerKey(input.key)) {
      const shortcutType = blockTypeForTextShortcut(input.text);
      if (shortcutType) {
        return {
          type: "apply_text_shortcut",
          blockType: shortcutType,
          preventDefault: true,
        };
      }
    }
    if (input.text.trim().length === 0 && notesEmptyEnterReturnsParagraph(input.blockType)) {
      return { type: "convert_to_paragraph", preventDefault: true };
    }
    if (notesEnterSplitsRichTextBlock(input.blockType)) {
      return {
        type: "split_text_block",
        selectionStart: input.selectionStart,
        selectionEnd: input.selectionEnd,
        preventDefault: true,
      };
    }
    return { type: "create_sibling", preventDefault: true };
  }

  if (
    input.key === " "
    && !input.ctrlKey
    && !input.shiftKey
    && input.blockType === "paragraph"
  ) {
    const shortcutType = blockTypeForTextShortcut(input.text);
    if (shortcutType) {
      return {
        type: "apply_text_shortcut",
        blockType: shortcutType,
        preventDefault: true,
      };
    }
  }

  if (input.key === "Backspace" && !input.ctrlKey && !input.shiftKey) {
    if (input.text.length === 0) {
      return input.isOnlyBlock
        ? { type: "convert_to_paragraph", preventDefault: true }
        : { type: "delete_block", preventDefault: true };
    }
    if (
      input.selectionStart === 0
      && input.selectionEnd === 0
      && input.previousBlockType
      && notesBackspaceCanMergeBlockTypes(input.blockType, input.previousBlockType)
    ) {
      return { type: "merge_with_previous", preventDefault: true };
    }
  }

  return { type: "none" };
}
