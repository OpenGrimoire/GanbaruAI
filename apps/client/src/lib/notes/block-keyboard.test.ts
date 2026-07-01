import { describe, expect, it } from "vitest";
import {
  planNotesKeyboardAction,
  type NotesKeyboardPlanInput,
} from "./block-keyboard";
import type { NotesBlockType } from "./types";

function plan(input: Partial<NotesKeyboardPlanInput>) {
  return planNotesKeyboardAction({
    key: "Enter",
    shiftKey: false,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    text: "",
    selectionStart: 0,
    selectionEnd: 0,
    blockType: "paragraph",
    previousBlockType: null,
    isOnlyBlock: false,
    ...input,
  });
}

describe("notes keyboard planning", () => {
  it("splits rich text blocks on Enter", () => {
    expect(plan({ key: "Enter", text: "Hello" })).toEqual({
      type: "split_text_block",
      selectionStart: 0,
      selectionEnd: 0,
      text: "Hello",
      preventDefault: true,
    });
    expect(
      plan({
        key: "Enter",
        text: "Hello",
        selectionStart: 2,
        selectionEnd: 4,
        blockType: "heading_2",
      }),
    ).toEqual({
      type: "split_text_block",
      selectionStart: 2,
      selectionEnd: 4,
      text: "Hello",
      preventDefault: true,
    });
  });

  it("allows newline insertion for Shift+Enter and code Enter", () => {
    expect(plan({ key: "Enter", shiftKey: true })).toEqual({
      type: "insert_newline",
      preventDefault: false,
    });
    expect(plan({ key: "Enter", blockType: "code", text: "let x = 1;" })).toEqual({
      type: "insert_newline",
      preventDefault: false,
    });
  });

  it("creates a paragraph after code on Ctrl+Enter", () => {
    expect(plan({ key: "Enter", blockType: "code", ctrlKey: true })).toEqual({
      type: "create_sibling",
      preventDefault: true,
    });
  });

  it("converts empty list-like blocks to paragraphs on Enter", () => {
    expect(plan({ key: "Enter", blockType: "bulleted_list_item", text: "" })).toEqual({
      type: "convert_to_paragraph",
      preventDefault: true,
    });
    expect(plan({ key: "Enter", blockType: "callout", text: "" })).toEqual({
      type: "convert_to_paragraph",
      preventDefault: true,
    });
  });

  it("plans Backspace deletion, only-block recovery, and merge", () => {
    expect(plan({ key: "Backspace", text: "" })).toEqual({
      type: "delete_block",
      preventDefault: true,
    });
    expect(plan({ key: "Backspace", text: "", isOnlyBlock: true })).toEqual({
      type: "convert_to_paragraph",
      preventDefault: true,
    });
    expect(
      plan({
        key: "Backspace",
        text: "Text",
        selectionStart: 0,
        selectionEnd: 0,
        previousBlockType: "paragraph",
      }),
    ).toEqual({
      type: "merge_with_previous",
      preventDefault: true,
    });
  });

  it("plans Backspace delete and merge for all text-editable block types", () => {
    const textEditableTypes: readonly NotesBlockType[] = [
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
      "template",
      "button",
      "code",
    ];

    for (const blockType of textEditableTypes) {
      expect(plan({ key: "Backspace", blockType, text: "" })).toEqual({
        type: "delete_block",
        preventDefault: true,
      });
      expect(
        plan({
          key: "Backspace",
          blockType,
          text: "Text",
          selectionStart: 0,
          selectionEnd: 0,
          previousBlockType: "paragraph",
        }),
      ).toEqual({
        type: "merge_with_previous",
        preventDefault: true,
      });
    }
  });

  it("plans Tab nesting and Shift+Tab outdent", () => {
    expect(plan({ key: "Tab" })).toEqual({ type: "nest", preventDefault: true });
    expect(plan({ key: "Tab", shiftKey: true })).toEqual({
      type: "outdent",
      preventDefault: true,
    });
  });

  it("plans primary modifier movement shortcuts", () => {
    expect(plan({ key: "ArrowUp", ctrlKey: true, shiftKey: true })).toEqual({
      type: "move_up",
      preventDefault: true,
    });
    expect(plan({ key: "ArrowDown", metaKey: true, shiftKey: true })).toEqual({
      type: "move_down",
      preventDefault: true,
    });
  });

  it("opens slash commands only at the start of an empty block", () => {
    expect(plan({ key: "/" })).toEqual({
      type: "open_slash_menu",
      preventDefault: false,
    });
    expect(plan({ key: "/", text: "x", selectionStart: 1, selectionEnd: 1 })).toEqual({
      type: "none",
    });
  });

  it("applies text shortcuts on Space or Enter", () => {
    expect(plan({ key: " ", text: "##" })).toEqual({
      type: "apply_text_shortcut",
      blockType: "heading_2",
      preventDefault: true,
    });
    expect(plan({ key: " ", text: "####" })).toEqual({
      type: "apply_text_shortcut",
      blockType: "heading_4",
      preventDefault: true,
    });
    expect(plan({ key: " ", text: ">" })).toEqual({
      type: "apply_text_shortcut",
      blockType: "toggle",
      preventDefault: true,
    });
    expect(plan({ key: " ", text: "\"" })).toEqual({
      type: "apply_text_shortcut",
      blockType: "quote",
      preventDefault: true,
    });
    expect(plan({ key: "Enter", text: "```" })).toEqual({
      type: "apply_text_shortcut",
      blockType: "code",
      preventDefault: true,
    });
  });

  it("modifies toggle open state on Ctrl+Enter", () => {
    expect(plan({ key: "Enter", ctrlKey: true, blockType: "toggle" })).toEqual({
      type: "toggle_block_open",
      preventDefault: true,
    });
  });
});
