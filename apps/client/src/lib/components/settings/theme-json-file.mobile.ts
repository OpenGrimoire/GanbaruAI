export const THEME_JSON_FILE_SAVE_AVAILABLE = false;

/** Reject unsupported direct file writes until the mobile content-URI adapter exists. */
export function saveThemeJsonFile(
  _defaultName: string,
  _contents: string,
): Promise<boolean> {
  return Promise.reject(new Error("Theme JSON file export is unavailable on mobile."));
}
