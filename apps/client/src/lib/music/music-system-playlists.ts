import type { Translate } from "$lib/i18n/translator.svelte";
import type { MusicPlaylistSummary } from "$lib/music/library-contracts";

export const SYSTEM_MUSIC_PLAYLIST_IDS = [
  "playlist-default-start-of-day",
  "playlist-default-work-focus",
  "playlist-default-work-ganbare",
  "playlist-default-break-calm",
  "playlist-default-break-active",
  "playlist-default-meditate",
  "playlist-default-exercise",
  "playlist-default-hygiene",
  "playlist-default-chores",
  "playlist-default-cooking",
  "playlist-default-commute",
] as const;

export type SystemMusicPlaylistId = (typeof SYSTEM_MUSIC_PLAYLIST_IDS)[number];

export function isSystemMusicPlaylistId(id: string): id is SystemMusicPlaylistId {
  return SYSTEM_MUSIC_PLAYLIST_IDS.includes(id as SystemMusicPlaylistId);
}

const SYSTEM_MUSIC_PLAYLIST_ORDER = new Map<string, number>(
  SYSTEM_MUSIC_PLAYLIST_IDS.map((id, index) => [id, index]),
);

export function orderMusicPlaylists(playlists: readonly MusicPlaylistSummary[]): MusicPlaylistSummary[] {
  return playlists.toSorted((left, right) => {
    const leftOrder = SYSTEM_MUSIC_PLAYLIST_ORDER.get(left.id);
    const rightOrder = SYSTEM_MUSIC_PLAYLIST_ORDER.get(right.id);
    if (leftOrder !== undefined && rightOrder !== undefined) return leftOrder - rightOrder;
    if (leftOrder !== undefined) return -1;
    if (rightOrder !== undefined) return 1;
    return left.name.localeCompare(right.name);
  });
}

export function systemMusicPlaylistName(id: string, storedName: string, t: Translate): string {
  if (id === "playlist-default-start-of-day") return t("music.builder.defaultPlaylist.startOfDay");
  if (id === "playlist-default-work-focus") return t("music.builder.defaultPlaylist.workFocus");
  if (id === "playlist-default-work-ganbare") return t("music.builder.defaultPlaylist.workGanbare");
  if (id === "playlist-default-break-calm") return t("music.builder.defaultPlaylist.breakCalm");
  if (id === "playlist-default-break-active") return t("music.builder.defaultPlaylist.breakActive");
  if (id === "playlist-default-meditate") return t("music.builder.defaultPlaylist.meditate");
  if (id === "playlist-default-exercise") return t("music.builder.defaultPlaylist.exercise");
  if (id === "playlist-default-hygiene") return t("music.builder.defaultPlaylist.hygiene");
  if (id === "playlist-default-chores") return t("music.builder.defaultPlaylist.chores");
  if (id === "playlist-default-cooking") return t("music.builder.defaultPlaylist.cooking");
  if (id === "playlist-default-commute") return t("music.builder.defaultPlaylist.commute");
  return storedName;
}

export function partitionMusicPlaylists(playlists: MusicPlaylistSummary[]): {
  defaults: MusicPlaylistSummary[];
  custom: MusicPlaylistSummary[];
} {
  const byId = new Map(playlists.map((playlist) => [playlist.id, playlist]));
  return {
    defaults: SYSTEM_MUSIC_PLAYLIST_IDS.flatMap((id) => {
      const playlist = byId.get(id);
      return playlist ? [playlist] : [];
    }),
    custom: playlists.filter((playlist) => !isSystemMusicPlaylistId(playlist.id)),
  };
}
