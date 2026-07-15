import type { LocalRootBinding, MusicPlaylistPlaybackEntry } from "$lib/music/library-contracts";
import { localFileSourceFromPath, youtubeVideoSourceFromId, type MusicSource } from "$lib/music/sources";

export type MusicPlaylistSkipReason = "disabled" | "snoozed" | "unavailable" | "unbound-root" | "invalid-source";

export interface MusicPlaylistPlaybackProjection {
  sources: MusicSource[];
  itemIds: string[];
  skipped: Record<MusicPlaylistSkipReason, number>;
}

const emptySkipped = (): Record<MusicPlaylistSkipReason, number> => ({
  disabled: 0,
  snoozed: 0,
  unavailable: 0,
  "unbound-root": 0,
  "invalid-source": 0,
});

export function projectMusicPlaylistPlayback(
  entries: readonly MusicPlaylistPlaybackEntry[],
  bindings: readonly LocalRootBinding[],
  explicitItemId: string | null = null,
): MusicPlaylistPlaybackProjection {
  const sources: MusicSource[] = [];
  const itemIds: string[] = [];
  const skipped = emptySkipped();
  const bindingPaths = new Map(bindings.map((binding) => [binding.rootId, binding.folderPath]));
  for (const entry of entries) {
    const reason = ineligibleReason(entry, bindingPaths, explicitItemId);
    if (reason) { skipped[reason] += 1; continue; }
    const source = playbackSource(entry, bindingPaths);
    if (!source) { skipped["invalid-source"] += 1; continue; }
    sources.push(source);
    itemIds.push(entry.itemId);
  }
  return { sources, itemIds, skipped };
}

function ineligibleReason(
  entry: MusicPlaylistPlaybackEntry,
  bindingPaths: ReadonlyMap<string, string | null>,
  explicitItemId: string | null,
): MusicPlaylistSkipReason | null {
  if (!entry.enabled) return "disabled";
  if (entry.snoozed && entry.itemId !== explicitItemId) return "snoozed";
  if (entry.availability !== "available") return "unavailable";
  if (entry.sourceKind === "local-file" && (!entry.rootId || !bindingPaths.get(entry.rootId))) return "unbound-root";
  return null;
}

function playbackSource(
  entry: MusicPlaylistPlaybackEntry,
  bindingPaths: ReadonlyMap<string, string | null>,
): MusicSource | null {
  if (entry.sourceKind === "youtube-video") {
    if (!entry.youtubeVideoId) return null;
    return {
      ...youtubeVideoSourceFromId(entry.youtubeVideoId, { startMs: entry.startMs, endMs: entry.endMs }),
      title: entry.title,
    };
  }
  if (!entry.rootId || !entry.relativePath) return null;
  const folder = bindingPaths.get(entry.rootId);
  if (!folder) return null;
  const separator = folder.includes("\\") && !folder.includes("/") ? "\\" : "/";
  const path = `${folder.replace(/[\\/]+$/, "")}${separator}${entry.relativePath.replace(/[\\/]+/g, separator)}`;
  return {
    ...localFileSourceFromPath(path, entry.title),
    startMs: entry.startMs,
    endMs: entry.endMs,
  };
}
