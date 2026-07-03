import { render } from "svelte/server";
import { describe, expect, it } from "vitest";
import { createTextRichText } from "$lib/notes/rich-text";
import type { NotesRichText } from "$lib/notes/types";
import NotesRichTextInline from "./NotesRichTextInline.svelte";

function renderedText(richText: readonly NotesRichText[]): string {
  const { body } = render(NotesRichTextInline, {
    props: {
      richText,
    },
  });
  return body
    .replace(/<!--[\s\S]*?-->/gu, "")
    .replace(/<[^>]*>/gu, "");
}

describe("NotesRichTextInline", () => {
  it("renders no text nodes for empty rich text", () => {
    expect(renderedText([createTextRichText("")])).toBe("");
  });

  it("renders visible plain text without template whitespace", () => {
    expect(renderedText([createTextRichText("Example")])).toBe("Example");
  });
});
