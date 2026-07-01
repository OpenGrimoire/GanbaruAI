import { describe, expect, it } from "vitest";
import {
  unsupportedBlockPlainText,
  unsupportedBlockTypeName,
  unsupportedBlockWarnings,
} from "./unsupported";

describe("notes unsupported blocks", () => {
  it("uses block_type as the visible imported kind", () => {
    expect(unsupportedBlockTypeName({ block_type: "form", source_type: "notion" })).toBe("form");
  });

  it("falls back to source_type when block_type is missing", () => {
    expect(unsupportedBlockTypeName({ source_type: "notion_template" })).toBe("notion_template");
  });

  it("builds searchable text from imported kind and warnings", () => {
    expect(
      unsupportedBlockPlainText({
        block_type: "button",
        warnings: ["Action content is not exposed"],
      }),
    ).toBe("button Action content is not exposed");
  });

  it("drops empty warning text", () => {
    expect(unsupportedBlockWarnings({ warnings: ["", "  ", "Preserved raw payload"] })).toEqual([
      "Preserved raw payload",
    ]);
  });
});
