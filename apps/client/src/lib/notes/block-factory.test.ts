import { describe, expect, it } from "vitest";
import {
  blockConvertedToType,
  blockWithHeadingToggleable,
} from "./block-factory";
import type {
  NotesBlock,
  NotesRichText,
  NotesRichTextAnnotations,
} from "./types";

const baseAnnotations: NotesRichTextAnnotations = {
  bold: false,
  italic: false,
  strikethrough: false,
  underline: false,
  code: false,
  color: "default",
};

function richTextFixture(): NotesRichText[] {
  return [
    {
      type: "text",
      text: {
        content: "Docs",
        link: { url: "https://example.com" },
      },
      annotations: {
        ...baseAnnotations,
        bold: true,
        color: "blue",
      },
      plain_text: "Docs",
      href: "https://example.com",
    },
    {
      type: "equation",
      equation: {
        expression: "x+1",
      },
      annotations: baseAnnotations,
      plain_text: "x+1",
      href: null,
    },
  ];
}

function paragraphBlock(richText: NotesRichText[]): NotesBlock {
  return {
    object: "block",
    id: "block-1",
    parent: {
      type: "page_id",
      page_id: "page-1",
    },
    created_time: "2026-01-01T00:00:00.000Z",
    last_edited_time: "2026-01-01T00:00:00.000Z",
    has_children: false,
    in_trash: false,
    type: "paragraph",
    source_provider: null,
    source_object_id: null,
    source_last_edited_time: null,
    paragraph: {
      rich_text: richText,
      color: "blue",
    },
  };
}

describe("notes block factory conversions", () => {
  it("preserves rich text objects when converting to another text block", () => {
    const richText = richTextFixture();
    const converted = blockConvertedToType(paragraphBlock(richText), "heading_2");

    expect(converted.type).toBe("heading_2");
    if (converted.type !== "heading_2") throw new Error("Expected heading 2 conversion");
    expect(converted.heading_2.rich_text).toEqual(richText);
    expect(converted.heading_2.color).toBe("blue");
  });

  it("preserves rich text objects when converting to a toggle heading", () => {
    const richText = richTextFixture();
    const converted = blockWithHeadingToggleable(paragraphBlock(richText), "heading_3", true);

    expect(converted.type).toBe("heading_3");
    if (converted.type !== "heading_3") throw new Error("Expected heading 3 conversion");
    expect(converted.heading_3.rich_text).toEqual(richText);
    expect(converted.heading_3.color).toBe("blue");
    expect(converted.heading_3.is_toggleable).toBe(true);
    expect(converted.heading_3.ganbaru_open).toBe(true);
  });
});
