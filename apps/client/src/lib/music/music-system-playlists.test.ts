import { describe, expect, it } from "vitest";
import { isSystemMusicPlaylistId, systemMusicPlaylistName } from "$lib/music/music-system-playlists";

describe("system music playlists", () => {
  it("localizes protected playlist names from stable ids", () => {
    const t = ((key: string) => key === "music.builder.defaultPlaylist.reading" ? "Lectura" : key) as Parameters<typeof systemMusicPlaylistName>[2];
    expect(systemMusicPlaylistName("playlist-default-reading", "Reading", t)).toBe("Lectura");
    expect(systemMusicPlaylistName("custom", "Focus", t)).toBe("Focus");
    expect(isSystemMusicPlaylistId("playlist-default-reading")).toBe(true);
    expect(isSystemMusicPlaylistId("custom")).toBe(false);
  });
});
