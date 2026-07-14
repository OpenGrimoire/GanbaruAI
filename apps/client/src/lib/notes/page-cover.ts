import { externalMediaUrlIsSupported } from "./media";
import type { NotesPageCover } from "./types";

export interface NotesPageCoverAssetMetadata {
  relativePath: string;
  originalName?: string | null;
  contentType: string;
  byteSize: number;
  sha256: string;
}

export interface NotesPageCoverPreset {
  id: "calm-lines" | "focus-dawn" | "deep-work" | "greenhouse";
  colors: readonly [string, string, string];
}

export const NOTES_PAGE_COVER_PRESETS = [
  { id: "calm-lines", colors: ["#f3f4f6", "#dbeafe", "#334155"] },
  { id: "focus-dawn", colors: ["#fff7ed", "#fed7aa", "#7c2d12"] },
  { id: "deep-work", colors: ["#eef2ff", "#c4b5fd", "#312e81"] },
  { id: "greenhouse", colors: ["#ecfdf5", "#86efac", "#14532d"] },
] as const satisfies readonly NotesPageCoverPreset[];

const NOTES_PAGE_COVER_ASSET_PATTERN = /^notes\/page-covers\/[a-f0-9]{64}\.(png|jpg|jpeg|webp)$/i;

/** Create a Notion-style external page cover payload for the Tauri update boundary. */
export function createNotesExternalPageCover(url: string): NotesPageCover {
  const trimmed = url.trim();
  if (!trimmed) throw new Error("page cover URL must not be empty");
  if (!isSupportedExternalPageCoverUrl(trimmed)) {
    throw new Error("page cover URL must be a supported HTTPS image URL");
  }
  return { type: "external", external: { url: trimmed } };
}

/** Create a Notion-style local file cover payload from a managed asset. */
export function createNotesLocalFilePageCover(asset: NotesPageCoverAssetMetadata): NotesPageCover {
  const relativePath = asset.relativePath.trim();
  if (!isNotesPageCoverAssetPath(relativePath)) {
    throw new Error("page cover asset path must stay under notes/page-covers");
  }
  if (!["image/png", "image/jpeg", "image/webp"].includes(asset.contentType)) {
    throw new Error("page cover asset must be a PNG, JPG, or WebP image");
  }
  if (!Number.isInteger(asset.byteSize) || asset.byteSize <= 0) {
    throw new Error("page cover asset size must be positive");
  }
  if (!/^[a-f0-9]{64}$/.test(asset.sha256)) {
    throw new Error("page cover asset hash must be a SHA-256 hex digest");
  }
  const originalName = asset.originalName?.trim();
  return {
    type: "file",
    file: {
      url: managedCoverAssetUrl(relativePath),
      ...(originalName ? { name: originalName } : {}),
      content_type: asset.contentType as "image/png" | "image/jpeg" | "image/webp",
      byte_size: asset.byteSize,
      sha256: asset.sha256,
      ganbaru_asset_path: relativePath,
    },
  };
}

/** Return the previewable URL for cover sources that can be rendered locally. */
export function notesPageCoverUrl(cover: NotesPageCover | null): string | null {
  if (!cover) return null;
  if (cover.type === "external") return cover.external.url;
  if (cover.type === "file" && !cover.file.ganbaru_asset_path) return cover.file.url;
  return null;
}

/** Return the managed asset path referenced by a local cover, if one exists. */
export function notesPageCoverAssetPath(cover: NotesPageCover | null): string | null {
  if (!cover || cover.type !== "file") return null;
  return cover.file.ganbaru_asset_path ?? null;
}

/** Return true when the relative path points to a managed Notes page cover asset. */
export function isNotesPageCoverAssetPath(value: string): boolean {
  return NOTES_PAGE_COVER_ASSET_PATTERN.test(value.trim());
}

/** Return true when the URL can be used as an external page cover image. */
export function isSupportedExternalPageCoverUrl(url: string): boolean {
  return externalMediaUrlIsSupported("image", url);
}

/** Return a CSS background for the generated cover preset preview. */
export function notesPageCoverPresetBackground(preset: NotesPageCoverPreset): string {
  const [start, middle, end] = preset.colors;
  return `linear-gradient(135deg, ${start} 0%, ${middle} 52%, ${end} 100%)`;
}

function managedCoverAssetUrl(relativePath: string): string {
  return `ganbaru-asset:${relativePath.trim()}`;
}
