import type { Translate } from "$lib/i18n/translator.svelte";
import type { MusicPlaylistSummary } from "$lib/music/library-contracts";

export const SYSTEM_MUSIC_PLAYLIST_IDS = [
  "playlist-default-reading",
  "playlist-default-exercise",
  "playlist-default-hygiene",
  "playlist-default-commute",
  "playlist-default-chores",
  "playlist-default-meditate",
  "playlist-default-start-of-day",
  "playlist-default-working",
  "playlist-default-short-breaks",
  "playlist-default-long-breaks",
] as const;

export type SystemMusicPlaylistId = (typeof SYSTEM_MUSIC_PLAYLIST_IDS)[number];

export function isSystemMusicPlaylistId(id: string): id is SystemMusicPlaylistId {
  return SYSTEM_MUSIC_PLAYLIST_IDS.includes(id as SystemMusicPlaylistId);
}

export function systemMusicPlaylistName(id: string, storedName: string, t: Translate): string {
  if (id === "playlist-default-reading") return t("music.builder.defaultPlaylist.reading");
  if (id === "playlist-default-exercise") return t("music.builder.defaultPlaylist.exercise");
  if (id === "playlist-default-hygiene") return t("music.builder.defaultPlaylist.hygiene");
  if (id === "playlist-default-commute") return t("music.builder.defaultPlaylist.commute");
  if (id === "playlist-default-chores") return t("music.builder.defaultPlaylist.chores");
  if (id === "playlist-default-meditate") return t("music.builder.defaultPlaylist.meditate");
  if (id === "playlist-default-start-of-day") return t("music.builder.defaultPlaylist.startOfDay");
  if (id === "playlist-default-working") return t("music.builder.defaultPlaylist.working");
  if (id === "playlist-default-short-breaks") return t("music.builder.defaultPlaylist.shortBreaks");
  if (id === "playlist-default-long-breaks") return t("music.builder.defaultPlaylist.longBreaks");
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
