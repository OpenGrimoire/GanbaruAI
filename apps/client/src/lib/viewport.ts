export type ViewportOrientation = "portrait" | "landscape";

export const ANDROID_INSETS_EVENT_NAME = "ganbaru:android-insets";

export interface SystemBarInsets {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

export interface PointerCapabilities {
  coarsePointer: boolean;
  hoverAvailable: boolean;
}

interface MediaQueryState {
  readonly matches: boolean;
}

const MAX_SYSTEM_BAR_INSET_CSS_PIXELS = 4096;

/** Parse the bounded, read-only Android system-bar bridge response. */
export function parseAndroidSystemBarInsets(value: unknown): SystemBarInsets | null {
  if (typeof value !== "string") return null;
  const parts = value.split(",");
  if (parts.length !== 4 || parts.some((part) => part.trim() === "")) return null;
  const values = parts.map(Number);
  if (
    values.some(
      (inset) =>
        !Number.isFinite(inset) ||
        inset < 0 ||
        inset > MAX_SYSTEM_BAR_INSET_CSS_PIXELS,
    )
  ) {
    return null;
  }
  const [top, right, bottom, left] = values;
  return { top, right, bottom, left };
}

/**
 * Returns the part of the layout viewport hidden below the visual viewport.
 *
 * This is a WebView fallback for keyboard avoidance. Native Android insets
 * can override the corresponding CSS variable once the Android bridge is
 * available.
 */
export function calculateKeyboardInset(
  layoutHeight: number,
  visualHeight: number,
  visualOffsetTop: number,
): number {
  if (![layoutHeight, visualHeight, visualOffsetTop].every(Number.isFinite)) return 0;
  return Math.max(0, Math.round(layoutHeight - visualHeight - visualOffsetTop));
}

/** Return a stable orientation for the current drawable viewport. */
export function viewportOrientation(width: number, height: number): ViewportOrientation {
  return width > height ? "landscape" : "portrait";
}

/** Resolve pointer capabilities with a touch-safe fallback when media queries are unavailable. */
export function resolvePointerCapabilities(
  coarsePointerQuery: MediaQueryState | null,
  hoverQuery: MediaQueryState | null,
  maxTouchPoints: number,
): PointerCapabilities {
  const touchAvailable = Number.isFinite(maxTouchPoints) && maxTouchPoints > 0;
  return {
    coarsePointer: coarsePointerQuery?.matches ?? touchAvailable,
    hoverAvailable: hoverQuery?.matches ?? !touchAvailable,
  };
}
