import { describe, expect, it } from "vitest";
import {
  notesBlockMarker,
  notesRichTextEditorClass,
  notesRichTextPreviewClass,
  notesTextareaClass,
} from "./block-editor-ui";

describe("notes block editor UI helpers", () => {
  it("keeps heading and code editor classes distinct", () => {
    expect(notesTextareaClass("heading_1")).toContain("text-[1.45rem]");
    expect(notesTextareaClass("code")).toContain("font-mono");
  });

  it("adds preview-only affordances on top of text block classes", () => {
    expect(notesRichTextPreviewClass("paragraph")).toContain("cursor-text");
    expect(notesRichTextPreviewClass("paragraph")).toContain("whitespace-pre-wrap");
  });

  it("adds rich editor affordances on top of text block classes", () => {
    expect(notesRichTextEditorClass("paragraph")).toContain("notes-rich-text-editor");
    expect(notesRichTextEditorClass("paragraph")).toContain("break-words");
    expect(notesRichTextEditorClass("paragraph")).not.toContain("focus-visible:ring");
    expect(notesRichTextEditorClass("code")).toContain("font-mono");
  });

  it("returns stable visible markers for list-like blocks", () => {
    expect(notesBlockMarker("bulleted_list_item")).toBe("•");
    expect(notesBlockMarker("numbered_list_item")).toBe("1.");
    expect(notesBlockMarker("paragraph")).toBe("");
  });
});
