import { describe, expect, it } from "vitest";
import {
  MOBILE_NAVIGATION_RAIL_MIN_WIDTH,
  mobileNavigationPresentation,
} from "./mobile-layout";

describe("mobileNavigationPresentation", () => {
  it("keeps compact Android navigation in the top bar", () => {
    expect(mobileNavigationPresentation(MOBILE_NAVIGATION_RAIL_MIN_WIDTH - 1)).toBe("top");
  });

  it("uses a rail from the Android medium-width boundary", () => {
    expect(mobileNavigationPresentation(MOBILE_NAVIGATION_RAIL_MIN_WIDTH)).toBe("rail");
    expect(mobileNavigationPresentation(840)).toBe("rail");
  });

  it("falls back safely for invalid measurements", () => {
    expect(mobileNavigationPresentation(Number.NaN)).toBe("top");
    expect(mobileNavigationPresentation(Number.POSITIVE_INFINITY)).toBe("top");
  });
});
