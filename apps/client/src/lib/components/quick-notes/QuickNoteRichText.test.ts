import { render } from "svelte/server";
import { describe, expect, it } from "vitest";
import QuickNoteRichText from "./QuickNoteRichText.svelte";

const plain = (content: string) => ({ content, bold: false, italic: false, underline: false });

function renderedHtml(content: string): string {
  return render(QuickNoteRichText, { props: { runs: content ? [plain(content)] : [] } }).body;
}

describe("QuickNoteRichText", () => {
  it("always renders a normal-height editable line", () => {
    expect(renderedHtml("")).toContain('data-notes-editor-sentinel="empty-line"');
  });

  it("renders every newline as an explicit editor line", () => {
    const html = renderedHtml("first\n\n");
    expect(html.match(/data-notes-editor-line="true"/gu)).toHaveLength(3);
    expect(html.match(/data-notes-editor-sentinel="empty-line"/gu)).toHaveLength(2);
  });
});
