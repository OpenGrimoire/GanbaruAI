import { invoke } from "@tauri-apps/api/core";
import type { ProfileImageAsset } from "$lib/api/profile-image";

/** Open the desktop image picker and copy the selection into managed profile assets. */
export async function pickProfileImageFile(title: string): Promise<ProfileImageAsset | null> {
  return invoke<ProfileImageAsset | null>("profile_image_pick_file", { title });
}
