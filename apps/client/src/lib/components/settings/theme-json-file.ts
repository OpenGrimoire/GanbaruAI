import { invoke } from "@tauri-apps/api/core";

export const THEME_JSON_FILE_SAVE_AVAILABLE = true;

/** Save theme JSON through the desktop native path dialog. */
export function saveThemeJsonFile(
  defaultName: string,
  contents: string,
): Promise<boolean> {
  return invoke<boolean>("vault_pick_and_write_theme_json", {
    defaultName,
    contents,
  });
}
