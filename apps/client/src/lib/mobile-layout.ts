export const MOBILE_NAVIGATION_RAIL_MIN_WIDTH = 600;

export type MobileNavigationPresentation = "top" | "rail";

/** Select primary navigation from stable layout width, independent of the input method. */
export function mobileNavigationPresentation(
  layoutWidth: number,
): MobileNavigationPresentation {
  return Number.isFinite(layoutWidth) && layoutWidth >= MOBILE_NAVIGATION_RAIL_MIN_WIDTH
    ? "rail"
    : "top";
}
