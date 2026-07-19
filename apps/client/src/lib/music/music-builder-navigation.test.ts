import { describe, expect, it } from "vitest";
import { projectMusicBuilderDockItems } from "./music-builder-navigation";

describe("music builder navigation dock", () => {
  it("projects all destinations with playlist details activating Playlists", () => {
    const items = projectMusicBuilderDockItems({ kind: "playlist", playlistId: "focus" }, 12, 3);
    expect(items.map((item) => item.kind)).toEqual([
      "review", "playlists", "library", "sources", "issues", "soundscapes",
    ]);
    expect(items.find((item) => item.active)?.kind).toBe("playlists");
    expect(items.map((item) => item.shortcut)).toEqual(["1", "2", "3", "4", "5", "6"]);
  });

  it("shows only positive review and issue badges", () => {
    const items = projectMusicBuilderDockItems({ kind: "library" }, 0, 5);
    expect(items.find((item) => item.kind === "review")?.badge).toBeNull();
    expect(items.find((item) => item.kind === "issues")?.badge).toBe(5);
  });
});
