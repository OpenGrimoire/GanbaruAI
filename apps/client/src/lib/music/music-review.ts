import type {
  LocalRootBinding,
  MusicInspectorDetail,
  MusicPlaylistSummary,
  MusicWeight,
} from "$lib/music/library-contracts";
import { parseMusicSourceInput, type MusicSource } from "$lib/music/sources";

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
  return parseMusicSourceInput(`${root}${separator}${relative}`).source;
}

export function sortReviewPlaylists(
  playlists: readonly MusicPlaylistSummary[],
  search: string,
): MusicPlaylistSummary[] {
  const query = search.trim().toLocaleLowerCase();
  return playlists
    .filter((playlist) => !query || `${playlist.name} ${playlist.description}`.toLocaleLowerCase().includes(query))
    .toSorted((left, right) => left.name.localeCompare(right.name));
}

export function nextMusicWeight(weight: MusicWeight): MusicWeight {
  const weights: MusicWeight[] = ["rarely", "less-often", "normal", "more-often", "much-more-often"];
  return weights[(weights.indexOf(weight) + 1) % weights.length] ?? "normal";
}

export function isMusicReviewEditableTarget(target: EventTarget | null): boolean {
  return target instanceof HTMLElement
    && Boolean(target.closest("input, textarea, select, [contenteditable='true']"));
}
