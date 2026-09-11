import { describe, expect, it } from "vitest";
import { textInputCaretIndexForPointer } from "$lib/utils/text-input-caret";

describe("textInputCaretIndexForPointer", () => {
  const base = {
    inputLeft: 100,
    borderLeftWidth: 1,
    paddingLeft: 10,
    scrollLeft: 0,
    text: "Sleep",
    measureText: (text: string) => text.length * 10,
  };

  it("places clicks before the visible text at the start", () => {
    expect(textInputCaretIndexForPointer({
      ...base,
      clientX: 108,
    })).toBe(0);
  });

  it("places clicks after the visible text at the end", () => {
    expect(textInputCaretIndexForPointer({
      ...base,
      clientX: 170,
    })).toBe(5);
  });

  it("places contour clicks at the nearest horizontal text position", () => {
    expect(textInputCaretIndexForPointer({
      ...base,
      clientX: 132,
    })).toBe(2);
  });

  it("uses the nearest character boundary for normal text clicks too", () => {
    expect(textInputCaretIndexForPointer({
      ...base,
      clientX: 146,
    })).toBe(4);
  });

  it("accounts for horizontal input scroll", () => {
    expect(textInputCaretIndexForPointer({
      ...base,
      clientX: 115,
      scrollLeft: 20,
    })).toBe(2);
  });

  it("places empty text at the only valid offset", () => {
    expect(textInputCaretIndexForPointer({
      ...base,
      clientX: 130,
      text: "",
    })).toBe(0);
  });
});
