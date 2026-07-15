import type { MusicItemSignal, MusicPlaylistSummary } from "./library-contracts";

const disruptiveSignals = new Set<MusicItemSignal>(["lyrics", "sudden-changes", "high-intensity"]);

export interface MusicFocusAdvisory {
  playlistIds: string[];
  signals: MusicItemSignal[];
}

/**
 * Returns contextual, explainable guidance for selected focus-oriented playlists.
 */
export function getMusicFocusAdvisory(
  signals: readonly MusicItemSignal[],
  selectedPlaylistIds: ReadonlySet<string>,
  playlists: readonly MusicPlaylistSummary[],
): MusicFocusAdvisory | null {
  const relevantSignals = signals.filter((signal) => disruptiveSignals.has(signal));
  if (relevantSignals.length === 0) return null;
  const playlistIds = playlists
    .filter((playlist) => selectedPlaylistIds.has(playlist.id)
      && playlist.intendedUses.some((use) => use === "focus" || use === "reading"))
    .map((playlist) => playlist.id);
  return playlistIds.length > 0 ? { playlistIds, signals: relevantSignals } : null;
}
