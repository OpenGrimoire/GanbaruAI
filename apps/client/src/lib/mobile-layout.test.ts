import { describe, expect, it } from "vitest";
import {
  MOBILE_NAVIGATION_RAIL_MIN_WIDTH,
  mobileNavigationShowsLabels,
  mobileNavigationPresentation,
} from "./mobile-layout";

describe("mobileNavigationPresentation", () => {
  it("keeps compact Android windows on bottom navigation", () => {
    expect(mobileNavigationPresentation(MOBILE_NAVIGATION_RAIL_MIN_WIDTH - 1)).toBe("bottom");
  });

  it("uses a rail from the Android medium-width boundary", () => {
    expect(mobileNavigationPresentation(MOBILE_NAVIGATION_RAIL_MIN_WIDTH)).toBe("rail");
    expect(mobileNavigationPresentation(840)).toBe("rail");
  });

  it("falls back safely for invalid measurements", () => {
    expect(mobileNavigationPresentation(Number.NaN)).toBe("bottom");
    expect(mobileNavigationPresentation(Number.POSITIVE_INFINITY)).toBe("bottom");
  });
});

describe("mobileNavigationShowsLabels", () => {
  it("removes colliding bottom labels at larger interface scales", () => {
    expect(mobileNavigationShowsLabels("bottom", 1.1)).toBe(true);
    expect(mobileNavigationShowsLabels("bottom", 1.25)).toBe(false);
    expect(mobileNavigationShowsLabels("bottom", 1.5)).toBe(false);
  });

  it("keeps labels on the wider navigation rail", () => {
    expect(mobileNavigationShowsLabels("rail", 1.5)).toBe(true);
  });
});
