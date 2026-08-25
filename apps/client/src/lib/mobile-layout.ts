export const MOBILE_NAVIGATION_RAIL_MIN_WIDTH = 600;
export const MOBILE_NAVIGATION_COMPACT_LABEL_SCALE = 1.25;

export type MobileNavigationPresentation = "bottom" | "rail";

/** Select primary navigation from stable layout width, independent of the input method. */
export function mobileNavigationPresentation(
  layoutWidth: number,
): MobileNavigationPresentation {
  return Number.isFinite(layoutWidth) && layoutWidth >= MOBILE_NAVIGATION_RAIL_MIN_WIDTH
    ? "rail"
    : "bottom";
}

/** Keep rail labels, but simplify bottom navigation when interface scaling makes them collide. */
export function mobileNavigationShowsLabels(
  presentation: MobileNavigationPresentation,
  interfaceScale: number,
): boolean {
  return presentation === "rail"
    || !Number.isFinite(interfaceScale)
    || interfaceScale < MOBILE_NAVIGATION_COMPACT_LABEL_SCALE;
}
