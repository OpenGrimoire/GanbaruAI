import type {
  LocalRootBinding,
  MusicInspectorDetail,
  MusicPlaylistSummary,
  MusicWeight,
} from "$lib/music/library-contracts";
import { musicArtworkDataUrl, musicEmbeddedArtworkDataUrl } from "$lib/music/music-artwork-cache";
import { orderMusicPlaylists } from "$lib/music/music-system-playlists";
import { localFileSourceFromPath, parseMusicSourceInput, type MusicSource } from "$lib/music/sources";

export type MusicReviewExitPreference = "ask" | "restore" | "keep";

export function parseMusicReviewExitPreference(value: unknown): MusicReviewExitPreference {
  return value === "restore" || value === "keep" ? value : "ask";
}

export function parseMusicReviewAutoplay(value: unknown): boolean {
  return value === true;
}

export function musicReviewSource(
  detail: MusicInspectorDetail,
  bindings: readonly LocalRootBinding[],
): MusicSource | null {
  const item = detail.item;
  if (item.sourceKind === "youtube-video" && item.youtubeVideoId) {
    return parseMusicSourceInput(`https://www.youtube.com/watch?v=${item.youtubeVideoId}`).source;
  }
  const available = detail.locations.find((location) => location.availability === "available");
  if (!available) return null;
  const folder = bindings.find((binding) => binding.rootId === available.rootId)?.folderPath;
  if (!folder) return null;
  const separator = folder.includes("\\") && !folder.includes("/") ? "\\" : "/";
  const root = folder.replace(/[\\/]+$/, "");
  const relative = available.relativePath.replace(/[\\/]+/g, separator);
  const originalSidecar = item.originalArtworkIdentity?.startsWith("sidecar:")
    ? item.originalArtworkIdentity.slice("sidecar:".length).replace(/[\\/]+/g, separator)
    : null;
  const artworkPath = item.artworkOverride
    ?? (originalSidecar ? `${root}${separator}${originalSidecar}` : null);
  return localFileSourceFromPath(
    `${root}${separator}${relative}`,
    item.titleOverride ?? item.originalTitle,
    artworkPath,
  );
}

/** Loads builder-only review artwork without starting another media decoder. */
export function musicReviewArtworkDataUrl(
  detail: MusicInspectorDetail,
  bindings: readonly LocalRootBinding[],
): Promise<string | null> {
  const source = musicReviewSource(detail, bindings);
  if (!source || source.kind !== "local-file") return Promise.resolve(null);
  if (source.artworkPath) return musicArtworkDataUrl(source.artworkPath);
  if (detail.item.originalArtworkIdentity?.startsWith("embedded:")) {
    return musicEmbeddedArtworkDataUrl(source.path);
  }
  return Promise.resolve(null);
}

export function sortReviewPlaylists(
  playlists: readonly MusicPlaylistSummary[],
  search: string,
): MusicPlaylistSummary[] {
  const query = search.trim().toLocaleLowerCase();
  return orderMusicPlaylists(
    playlists.filter((playlist) => !query || playlist.name.toLocaleLowerCase().includes(query)),
  );
}

export function nextMusicWeight(weight: MusicWeight): MusicWeight {
  const weights: MusicWeight[] = ["rarely", "less-often", "normal", "more-often", "much-more-often"];
  return weights[(weights.indexOf(weight) + 1) % weights.length] ?? "normal";
}

export function isMusicReviewEditableTarget(target: EventTarget | null): boolean {
  return target instanceof HTMLElement
    && Boolean(target.closest("input, textarea, select, [contenteditable='true']"));
}
