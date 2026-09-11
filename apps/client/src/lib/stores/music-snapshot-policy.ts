import type { PlaybackStatus } from "$lib/music/playback";
import type { MusicSource } from "$lib/music/sources";
import { isYouTubeSource } from "$lib/music/sources";

export type MusicSnapshotBackend = "native" | "youtube";

/** Select the only backend allowed to poll for the current active source. */
export function activeMusicSnapshotBackend(
  source: MusicSource | null,
  status: PlaybackStatus,
  usesNativeLocalBackend: boolean,
): MusicSnapshotBackend | null {
  if (!source) return null;
  if (isYouTubeSource(source)) {
    return status === "playing" || status === "paused" ? "youtube" : null;
  }
  return usesNativeLocalBackend
    && (status === "loading" || status === "playing" || status === "paused")
    ? "native"
    : null;
}
