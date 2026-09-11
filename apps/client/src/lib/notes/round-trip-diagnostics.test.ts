import { describe, expect, it } from "vitest";
import {
  buildRoundTripNotesSourceHref,
  classifyRoundTripDiagnostic,
  roundTripCategoryCounts,
  toRoundTripDiagnosticItem,
} from "./round-trip-diagnostics";

describe("round-trip diagnostics", () => {
  it("classifies preserved approximated skipped and unsupported consequences", () => {
    expect(classifyRoundTripDiagnostic("markdown_inline_image_preserved_as_text", "info")).toBe(
      "preserved",
    );
    expect(classifyRoundTripDiagnostic("markdown_heading_depth_approximated", "warning")).toBe(
      "approximated",
    );
    expect(classifyRoundTripDiagnostic("html_media_reference_skipped", "warning")).toBe(
      "skipped",
    );
    expect(classifyRoundTripDiagnostic("markdown_html_unsupported", "warning")).toBe(
      "unsupported",
    );
    expect(classifyRoundTripDiagnostic("custom_backend_warning", "warning")).toBe("warning");
  });

  it("counts normalized diagnostic categories in display order", () => {
    const diagnostics = [
      toRoundTripDiagnosticItem({
        code: "markdown_html_unsupported",
        severity: "warning",
        message: "HTML is unsupported.",
      }),
      toRoundTripDiagnosticItem({
        code: "markdown_heading_depth_approximated",
        severity: "warning",
        message: "Heading was approximated.",
      }),
      toRoundTripDiagnosticItem({
        code: "html_media_reference_skipped",
        severity: "warning",
        message: "Media reference was skipped.",
      }),
    ];

    expect(roundTripCategoryCounts(diagnostics)).toEqual([
      { category: "unsupported", count: 1 },
      { category: "skipped", count: 1 },
      { category: "approximated", count: 1 },
    ]);
  });

  it("builds local Notes links only when source ids are navigable", () => {
    expect(
      buildRoundTripNotesSourceHref("http://localhost:1420/?view=calendar", {
        pageId: "11111111-1111-4111-8111-111111111111",
        blockId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      }),
    ).toBe(
      "http://localhost:1420/?view=notes#notes?page=11111111-1111-4111-8111-111111111111&block=aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    );
    expect(
      buildRoundTripNotesSourceHref("http://localhost:1420/?view=calendar", {
        pageId: "not-a-page",
        blockId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      }),
    ).toBeNull();
  });
});
