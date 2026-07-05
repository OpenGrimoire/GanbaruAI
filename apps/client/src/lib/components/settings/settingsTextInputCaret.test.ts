import { describe, expect, it } from "vitest";
import { settingsTextInputPointerMissesText } from "./settingsTextInputCaret";

describe("settingsTextInputPointerMissesText", () => {
  const base = {
    inputLeft: 100,
    borderLeftWidth: 1,
    paddingLeft: 10,
    scrollLeft: 0,
    textWidth: 40,
    tolerancePx: 0,
  };

  it("treats clicks before the visible text as misses", () => {
    expect(settingsTextInputPointerMissesText({
      ...base,
      clientX: 108,
    })).toBe(true);
  });

  it("treats clicks after the visible text as misses", () => {
    expect(settingsTextInputPointerMissesText({
      ...base,
      clientX: 152,
    })).toBe(true);
  });

  it("keeps normal text clicks under the visible text", () => {
    expect(settingsTextInputPointerMissesText({
      ...base,
      clientX: 130,
    })).toBe(false);
  });

  it("accounts for horizontal input scroll", () => {
    expect(settingsTextInputPointerMissesText({
      ...base,
      clientX: 78,
      scrollLeft: 35,
    })).toBe(false);
  });

  it("treats empty text as a miss so the caret goes to the only valid offset", () => {
    expect(settingsTextInputPointerMissesText({
      ...base,
      clientX: 130,
      textWidth: 0,
    })).toBe(true);
  });
});
