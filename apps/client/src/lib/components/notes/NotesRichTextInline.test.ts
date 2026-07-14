import { render } from "svelte/server";
import { describe, expect, it } from "vitest";
import { createTextRichText } from "$lib/notes/rich-text";
import type { NotesRichText } from "$lib/notes/types";
import NotesRichTextInline from "./NotesRichTextInline.svelte";

function renderedText(richText: readonly NotesRichText[]): string {
  const body = renderedHtml(richText);
  const lineMatches = [...body.matchAll(/<div[^>]*data-notes-editor-line="true"[^>]*>([\s\S]*?)<\/div>/giu)];
  if (lineMatches.length > 0) {
    return lineMatches
      .map((match) => (match[1] ?? "").replace(/<[^>]*>/gu, ""))
      .join("\n");
  }
  return body.replace(/<[^>]*>/gu, "");
}

function renderedHtml(richText: readonly NotesRichText[]): string {
  const { body } = render(NotesRichTextInline, {
    props: {
      richText,
    },
  });
  return body.replace(/<!--[\s\S]*?-->/gu, "");
}

describe("NotesRichTextInline", () => {
  it("renders no text nodes for empty rich text", () => {
    expect(renderedText([createTextRichText("")])).toBe("");
  });

  it("renders visible plain text without template whitespace", () => {
    expect(renderedText([createTextRichText("Example")])).toBe("Example");
  });

  it("renders soft newlines as explicit line boundaries", () => {
    expect(renderedText([createTextRichText("Example\nExample")])).toBe("Example\nExample");
    expect(renderedHtml([createTextRichText("Example\nExample")]).match(/data-notes-editor-line="true"/gu)).toHaveLength(2);
  });

  it("renders a zero-width trailing sentinel without extra whitespace", () => {
    expect(renderedText([createTextRichText("Example\n")])).toBe("Example\n\u200b");
  });
});
