import { describe, expect, it } from "vitest";
import type { MusicPlaylistSummary } from "$lib/music/library-contracts";
import {
  isSystemMusicPlaylistId,
  orderMusicPlaylists,
  SYSTEM_MUSIC_PLAYLIST_IDS,
  systemMusicPlaylistName,
} from "$lib/music/music-system-playlists";

describe("system music playlists", () => {
  it("localizes protected playlist names from stable ids", () => {
    const t = ((key: string) => key === "music.builder.defaultPlaylist.workFocus" ? "Trabajo (concentración)" : key) as Parameters<typeof systemMusicPlaylistName>[2];
    expect(systemMusicPlaylistName("playlist-default-work-focus", "Work (focus)", t)).toBe("Trabajo (concentración)");
    expect(systemMusicPlaylistName("custom", "Focus", t)).toBe("Focus");
    expect(isSystemMusicPlaylistId("playlist-default-work-focus")).toBe(true);
    expect(isSystemMusicPlaylistId("custom")).toBe(false);
  });

  it("keeps built-in playlists in their intended review order", () => {
    expect(SYSTEM_MUSIC_PLAYLIST_IDS).toEqual([
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
    ]);
  });

  it("orders built-in playlists before alphabetical custom playlists", () => {
    const playlist = (id: string, name: string) => ({ id, name }) as MusicPlaylistSummary;
    const ordered = orderMusicPlaylists([
      playlist("custom-z", "Zen"),
      playlist("playlist-default-commute", "Commute"),
      playlist("custom-a", "Ambient"),
      playlist("playlist-default-start-of-day", "Start of the day!"),
      playlist("playlist-default-work-focus", "Work (focus)"),
    ]);

    expect(ordered.map(({ id }) => id)).toEqual([
      "playlist-default-start-of-day",
      "playlist-default-work-focus",
      "playlist-default-commute",
      "custom-a",
      "custom-z",
    ]);
  });
});
