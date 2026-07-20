import { describe, expect, it } from "vitest";
import { moveMusicPlaylistOrder } from "./music-playlist-order";

const playlists = ["a", "b", "c", "d"].map((id) => ({ id }));

describe("music playlist order", () => {
  it("moves playlists in either direction without mutating the source", () => {
    expect(moveMusicPlaylistOrder(playlists, "b", 3).map(({ id }) => id)).toEqual(["a", "c", "d", "b"]);
    expect(moveMusicPlaylistOrder(playlists, "d", 1).map(({ id }) => id)).toEqual(["a", "d", "b", "c"]);
    expect(playlists.map(({ id }) => id)).toEqual(["a", "b", "c", "d"]);
  });

  it("bounds edge targets and preserves unknown ids", () => {
    expect(moveMusicPlaylistOrder(playlists, "c", -10).map(({ id }) => id)).toEqual(["c", "a", "b", "d"]);
    expect(moveMusicPlaylistOrder(playlists, "b", 99).map(({ id }) => id)).toEqual(["a", "c", "d", "b"]);
    expect(moveMusicPlaylistOrder(playlists, "missing", 1)).toEqual(playlists);
  });
});
