// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import {
  applyQuickNoteBeforeInput,
  normalizeQuickNoteRuns,
  quickNotePlainText,
  quickNoteRunsFromHtml,
  toggleQuickNoteFormatting,
} from "./rich-text";

const plain = (content: string) => ({ content, bold: false, italic: false, underline: false });

describe("Quick note rich text", () => {
  it("merges adjacent formatting and removes empty runs", () => {
    expect(normalizeQuickNoteRuns([plain("a"), plain(""), plain("b")])).toEqual([plain("ab")]);
  });

  it("splits and merges runs when formatting a selection", () => {
    expect(toggleQuickNoteFormatting([plain("hello")], { start: 1, end: 4 }, "bold")).toEqual([
      plain("h"),
      { ...plain("ell"), bold: true },
      plain("o"),
    ]);
  });

  it("applies controlled text insertion and Unicode deletion", () => {
    const inserted = applyQuickNoteBeforeInput([plain("ac")], { start: 1, end: 1 }, "insertText", "b", {
      bold: true,
      italic: false,
      underline: false,
    });
    expect(inserted && quickNotePlainText(inserted.runs)).toBe("abc");
    const deleted = applyQuickNoteBeforeInput([plain("a😀b")], { start: 3, end: 3 }, "deleteContentBackward", null, {
      bold: false,
      italic: false,
      underline: false,
    });
    expect(deleted && quickNotePlainText(deleted.runs)).toBe("ab");
  });

  it("inserts visible trailing lines and handles word deletion", () => {
    const newline = applyQuickNoteBeforeInput([plain("hello")], { start: 5, end: 5 }, "insertParagraph", null, {
      bold: false,
      italic: false,
      underline: false,
    });
    expect(newline && quickNotePlainText(newline.runs)).toBe("hello\n");
    expect(newline?.selection).toEqual({ start: 6, end: 6 });

    const deleted = applyQuickNoteBeforeInput([plain("one two")], { start: 7, end: 7 }, "deleteWordBackward", null, {
      bold: false,
      italic: false,
      underline: false,
    });
    expect(deleted && quickNotePlainText(deleted.runs)).toBe("one ");
    expect(deleted?.selection).toEqual({ start: 4, end: 4 });
  });

  it("accepts browser replacement and drag insertion input types", () => {
    const replacement = applyQuickNoteBeforeInput([plain("teh")], { start: 0, end: 3 }, "insertReplacementText", "the", {
      bold: false,
      italic: false,
      underline: false,
    });
    expect(replacement && quickNotePlainText(replacement.runs)).toBe("the");
    const dropped = applyQuickNoteBeforeInput([plain("ac")], { start: 1, end: 1 }, "insertFromDrop", "b", {
      bold: false,
      italic: false,
      underline: false,
    });
    expect(dropped && quickNotePlainText(dropped.runs)).toBe("abc");
  });

  it("sanitizes pasted HTML to supported formatting", () => {
    const runs = quickNoteRunsFromHtml("<p><strong>Bold</strong> <a href='https://example.com'><u>link</u></a><script>bad()</script></p>");
    expect(quickNotePlainText(runs)).toBe("Bold link");
    expect(runs.some((run) => run.bold)).toBe(true);
    expect(runs.some((run) => run.underline)).toBe(true);
    expect(quickNotePlainText(runs)).not.toContain("bad");
  });
});
