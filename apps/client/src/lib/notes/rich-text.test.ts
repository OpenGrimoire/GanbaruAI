import { describe, expect, it } from "vitest";
import { Temporal } from "@js-temporal/polyfill";
import {
  applyRichTextAnnotations,
  applyRichTextLink,
  buildDateMentionTargets,
  createEquationRichText,
  createLinkedTextRichText,
  createDateMentionValue,
  createDateMentionRichText,
  createPageMentionRichText,
  createTextRichText,
  detectPageMentionQuery,
  filterPageMentionTargets,
  insertDateMentionRichText,
  insertEquationRichText,
  insertPageMentionRichText,
  normalizeRichTextEquationExpression,
  normalizeRichTextLinkUrl,
  replacePlainTextPreservingRichText,
  richTextAnnotationTogglePatch,
  richTextAnnotationsForSelection,
  richTextColorPatch,
  richTextHasVisibleFormatting,
  richTextLinkRangeForSelection,
  richTextPlainText,
} from "./rich-text";

const pageId = "11111111-1111-4111-8111-111111111111";

describe("notes rich text helpers", () => {
  it("inserts page mentions into a plain text range", () => {
    const richText = insertPageMentionRichText(
      [createTextRichText("See @tar today")],
      4,
      8,
      pageId,
      "Target page",
      "http://localhost:1420/?view=notes#notes?page=11111111-1111-4111-8111-111111111111",
    );

    expect(richTextPlainText(richText)).toBe("See Target page today");
    expect(richText[1]).toMatchObject({
      type: "mention",
      mention: { type: "page", page: { id: pageId } },
      plain_text: "Target page",
    });
  });

  it("inserts date mentions into a plain text range", () => {
    const richText = insertDateMentionRichText(
      [createTextRichText("Due @today")],
      4,
      10,
      createDateMentionValue("2026-06-30"),
      "Today",
    );

    expect(richTextPlainText(richText)).toBe("Due Today");
    expect(richText[1]).toMatchObject({
      type: "mention",
      mention: {
        type: "date",
        date: {
          start: "2026-06-30",
          end: null,
          time_zone: null,
        },
      },
      plain_text: "Today",
    });
  });

  it("applies and removes links over text ranges", () => {
    const linked = applyRichTextLink(
      [createTextRichText("Read the docs")],
      5,
      13,
      "example.com/docs",
    );

    expect(richTextPlainText(linked)).toBe("Read the docs");
    expect(linked[1]).toMatchObject({
      type: "text",
      text: { content: "the docs", link: { url: "https://example.com/docs" } },
      href: "https://example.com/docs",
    });

    const unlinked = applyRichTextLink(linked, 5, 13, null);
    expect(unlinked).toEqual([createTextRichText("Read the docs")]);
  });

  it("applies annotations over text ranges and merges compatible spans", () => {
    const richText = applyRichTextAnnotations(
      [createTextRichText("Make this bold")],
      5,
      14,
      { bold: true, color: "blue" },
    );

    expect(richTextPlainText(richText)).toBe("Make this bold");
    expect(richText[1]).toMatchObject({
      type: "text",
      text: { content: "this bold" },
      annotations: { bold: true, color: "blue" },
    });

    const unbold = applyRichTextAnnotations(richText, 5, 14, { bold: false, color: "default" });
    expect(unbold).toEqual([createTextRichText("Make this bold")]);
  });

  it("inserts inline equations into text ranges", () => {
    const richText = insertEquationRichText(
      [createTextRichText("Use e=mc^2 here")],
      4,
      10,
      " e=mc^2 ",
    );

    expect(richTextPlainText(richText)).toBe("Use e=mc^2 here");
    expect(richText[1]).toEqual(createEquationRichText("e=mc^2"));
  });

  it("normalizes inline equation expressions", () => {
    expect(normalizeRichTextEquationExpression("  \\frac{a}{b}  ")).toBe("\\frac{a}{b}");
    expect(normalizeRichTextEquationExpression("")).toBeNull();
    expect(normalizeRichTextEquationExpression("bad\u0008")).toBeNull();
  });

  it("keeps mentions intact when applying annotations around them", () => {
    const mention = createPageMentionRichText(pageId, "Target page", null);
    const richText = applyRichTextAnnotations(
      [createTextRichText("See "), mention, createTextRichText(" soon")],
      0,
      20,
      { italic: true },
    );

    expect(richTextPlainText(richText)).toBe("See Target page soon");
    expect(richText[1]).toEqual(mention);
    expect(richText[0]).toMatchObject({ annotations: { italic: true } });
    expect(richText[2]).toMatchObject({ annotations: { italic: true } });
  });

  it("reads annotation state from selections and collapsed cursors", () => {
    const linked = applyRichTextAnnotations(
      [createTextRichText("Read docs today")],
      5,
      9,
      { code: true, color: "red_background" },
    );

    expect(richTextAnnotationsForSelection(linked, 6, 6).annotations).toMatchObject({
      code: true,
      color: "red_background",
    });
    expect(richTextAnnotationsForSelection(linked, 5, 9).annotations).toMatchObject({
      code: true,
      color: "red_background",
    });
    expect(richTextAnnotationsForSelection(linked, 0, 9).annotations).toMatchObject({
      code: false,
      color: "default",
    });
  });

  it("builds annotation patches for toolbar actions", () => {
    const annotations = richTextAnnotationsForSelection(
      applyRichTextAnnotations([createTextRichText("Bold")], 0, 4, { bold: true }),
      0,
      4,
    ).annotations;

    expect(richTextAnnotationTogglePatch(annotations, "bold")).toEqual({ bold: false });
    expect(richTextColorPatch("green")).toEqual({ color: "green" });
  });

  it("detects visible inline formatting", () => {
    expect(richTextHasVisibleFormatting([createTextRichText("Plain")])).toBe(false);
    expect(
      richTextHasVisibleFormatting(
        applyRichTextAnnotations([createTextRichText("Styled")], 0, 6, { underline: true }),
      ),
    ).toBe(true);
    expect(richTextHasVisibleFormatting([createPageMentionRichText(pageId, "Page", null)])).toBe(true);
    expect(richTextHasVisibleFormatting([createEquationRichText("e=mc^2")])).toBe(true);
  });

  it("detects every rich text shape rendered by the focused editor", () => {
    expect(
      richTextHasVisibleFormatting(
        applyRichTextAnnotations([createTextRichText("Bold")], 0, 4, { bold: true }),
      ),
    ).toBe(true);
    expect(
      richTextHasVisibleFormatting(
        applyRichTextAnnotations([createTextRichText("Italic")], 0, 6, { italic: true }),
      ),
    ).toBe(true);
    expect(
      richTextHasVisibleFormatting(
        applyRichTextAnnotations([createTextRichText("Strike")], 0, 6, {
          strikethrough: true,
        }),
      ),
    ).toBe(true);
    expect(
      richTextHasVisibleFormatting(
        applyRichTextAnnotations([createTextRichText("Code")], 0, 4, { code: true }),
      ),
    ).toBe(true);
    expect(
      richTextHasVisibleFormatting(
        applyRichTextAnnotations([createTextRichText("Blue")], 0, 4, { color: "blue" }),
      ),
    ).toBe(true);
    expect(
      richTextHasVisibleFormatting(
        applyRichTextAnnotations([createTextRichText("Highlight")], 0, 9, {
          color: "yellow_background",
        }),
      ),
    ).toBe(true);
    expect(richTextHasVisibleFormatting([createLinkedTextRichText("Link", "https://example.com")]))
      .toBe(true);
    expect(richTextHasVisibleFormatting([createPageMentionRichText(pageId, "Page", null)]))
      .toBe(true);
    expect(
      richTextHasVisibleFormatting([
        createDateMentionRichText(createDateMentionValue("2026-06-30"), "Today"),
      ]),
    ).toBe(true);
    expect(richTextHasVisibleFormatting([createEquationRichText("e=mc^2")])).toBe(true);
  });

  it("finds an existing link range from a collapsed cursor", () => {
    const linked = [
      createTextRichText("Read "),
      createLinkedTextRichText("the docs", "https://example.com/docs"),
      createTextRichText(" today"),
    ];

    expect(richTextLinkRangeForSelection(linked, 8, 8)).toEqual({
      start: 5,
      end: 13,
      url: "https://example.com/docs",
    });
  });

  it("normalizes supported links and rejects unsafe schemes", () => {
    expect(normalizeRichTextLinkUrl("example.com")).toBe("https://example.com/");
    expect(normalizeRichTextLinkUrl("javascript:alert(1)")).toBeNull();
  });

  it("builds reminder mention targets from reminder queries", () => {
    const targets = buildDateMentionTargets("remind tomorrow", {
      today: Temporal.PlainDate.from("2026-06-30"),
      locale: "en",
      labels: {
        today: "Today",
        tomorrow: "Tomorrow",
        yesterday: "Yesterday",
        nextWeek: "Next week",
        date: "Date",
        reminder: "Reminder",
        remindTitle: (label) => `Remind ${label}`,
      },
    });

    expect(targets[0]).toMatchObject({
      kind: "date",
      title: "Remind Tomorrow",
      subtitle: "Reminder",
      date: {
        start: "2026-07-01",
        ganbaru_reminder: { enabled: true },
      },
      reminder: true,
    });
  });

  it("preserves page mentions when surrounding text changes", () => {
    const mention = createPageMentionRichText(pageId, "Target page", null);
    const richText = replacePlainTextPreservingRichText(
      [createTextRichText("See "), mention],
      "Please see Target page soon",
    );

    expect(richTextPlainText(richText)).toBe("Please see Target page soon");
    expect(richText.some((item) => item.type === "mention")).toBe(true);
  });

  it("collapses mentions back to text when the visible label is removed", () => {
    const richText = replacePlainTextPreservingRichText(
      [createTextRichText("See "), createPageMentionRichText(pageId, "Target page", null)],
      "See target",
    );

    expect(richText).toHaveLength(1);
    expect(richText[0]?.type).toBe("text");
  });

  it("detects page mention queries at the cursor", () => {
    expect(detectPageMentionQuery("See @tar", 8, 8)).toEqual({
      start: 4,
      end: 8,
      query: "tar",
    });
    expect(detectPageMentionQuery("email@example.com", 17, 17)).toBeNull();
  });

  it("filters mention targets with prefix matches first", () => {
    expect(
      filterPageMentionTargets(
        [
          { id: "b", kind: "page", title: "Roadmap" },
          { id: "a", kind: "page", title: "Daily roadmap" },
          { id: "c", kind: "page", title: "Archive" },
        ],
        "road",
      ).map((target) => target.id),
    ).toEqual(["b", "a"]);
  });
});
