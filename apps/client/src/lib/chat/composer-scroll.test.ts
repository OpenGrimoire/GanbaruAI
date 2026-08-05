import { describe, expect, it } from "vitest";
import {
  composerScrollTopForCaret,
  composerTextareaLayout,
} from "./composer-scroll";

describe("Chat composer scrolling", () => {
  it("uses exactly two through six complete line heights", () => {
    expect(composerTextareaLayout(22, 22)).toEqual({ height: 44, overflowing: false });
    expect(composerTextareaLayout(88, 22)).toEqual({ height: 88, overflowing: false });
    expect(composerTextareaLayout(132, 22)).toEqual({ height: 132, overflowing: false });
    expect(composerTextareaLayout(154, 22)).toEqual({ height: 132, overflowing: true });
  });

  it("normalizes rounded browser measurements when shrinking to the base height", () => {
    expect(composerTextareaLayout(45, 22.4).height).toBeCloseTo(44.8);
    expect(composerTextareaLayout(67, 22.4).height).toBeCloseTo(67.2);
    expect(composerTextareaLayout(67, 22.4).overflowing).toBe(false);
  });

  it("reveals the complete caret line below the viewport", () => {
    expect(composerScrollTopForCaret(22, 176, 22, 264, 132)).toBe(66);
  });

  it("reveals the complete caret line above the viewport without snapping", () => {
    expect(composerScrollTopForCaret(88, 43, 22, 264, 132)).toBe(43);
  });

  it("reaches the true end of an uneven scroll range", () => {
    expect(composerScrollTopForCaret(0, 208, 22, 230, 132)).toBe(98);
  });
});
