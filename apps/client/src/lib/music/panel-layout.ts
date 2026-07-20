export interface SidePlaylistPanelMeasurements {
  mediaWidth: number;
  headerHeight: number;
  controlsHeight: number;
  sideBySide: boolean;
}

/**
 * Returns the panel height that fits a 16:9 media surface without vertical gaps.
 */
export function fittedSidePlaylistPanelHeight(
  measurements: SidePlaylistPanelMeasurements,
): number | null {
  const { mediaWidth, headerHeight, controlsHeight, sideBySide } = measurements;
  if (!sideBySide) return null;
  if (![mediaWidth, headerHeight, controlsHeight].every(Number.isFinite)) return null;
  if (mediaWidth <= 0 || headerHeight < 0 || controlsHeight < 0) return null;
  return Math.ceil(headerHeight + controlsHeight + (mediaWidth * 9) / 16);
}
