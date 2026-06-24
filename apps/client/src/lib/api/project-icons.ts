import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";

export interface ProjectIconAsset {
  relativePath: string;
}

const projectIconAssetUrls = new Map<string, string>();

export async function pickProjectIconImageFile(): Promise<ProjectIconAsset | null> {
  return invoke<ProjectIconAsset | null>("project_icon_pick_image_file");
}

export async function saveProjectIconImageDataUrl(dataUrl: string): Promise<ProjectIconAsset> {
  return invoke<ProjectIconAsset>("project_icon_save_image_data_url", { dataUrl });
}

export async function downloadProjectIconImageUrl(url: string): Promise<ProjectIconAsset> {
  return invoke<ProjectIconAsset>("project_icon_download_image_url", { url });
}

export async function projectIconAssetUrl(relativePath: string): Promise<string> {
  const cached = projectIconAssetUrls.get(relativePath);
  if (cached) return cached;
  const assetUrl = await invoke<string>("project_icon_asset_data_url", { relativePath });
  projectIconAssetUrls.set(relativePath, assetUrl);
  return assetUrl;
}

export async function deleteProjectIconAssetsIfUnreferenced(relativePaths: string[]): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("project_icon_delete_assets_if_unreferenced", { dbUrl, relativePaths });
  for (const relativePath of relativePaths) {
    projectIconAssetUrls.delete(relativePath);
  }
}
