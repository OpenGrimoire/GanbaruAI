import { describe, expect, it } from "vitest";
import {
  backMusicBuilderRoute,
  initialMusicBuilderRoute,
  pushMusicBuilderRoute,
  repairMusicBuilderHistory,
} from "./music-builder-routing";

const context = {
  playlistIds: new Set(["focus"]),
  itemIds: new Set(["track-1"]),
};

describe("music builder routing", () => {
  it("opens Review first only when work is waiting", () => {
    expect(initialMusicBuilderRoute(3, { kind: "library" }, context).current.destination.kind).toBe("library");
    expect(initialMusicBuilderRoute(3, null, context).current.destination.kind).toBe("review");
    expect(initialMusicBuilderRoute(0, null, context).current.destination.kind).toBe("playlists");
  });

  it("supports deep inspector routes and browser-like Back", () => {
    let history = initialMusicBuilderRoute(0, null, context);
    history = pushMusicBuilderRoute(history, {
      destination: { kind: "playlist", playlistId: "focus" },
      inspectorItemId: "track-1",
    }, context);
    expect(history.current.inspectorItemId).toBe("track-1");
    const back = backMusicBuilderRoute(history, context);
    expect(back?.current.destination.kind).toBe("playlists");
    expect(backMusicBuilderRoute(back!, context)).toBeNull();
  });

  it("repairs deleted playlists and items without corrupting history", () => {
    let history = initialMusicBuilderRoute(0, null, context);
    history = pushMusicBuilderRoute(history, {
      destination: { kind: "playlist", playlistId: "focus" },
      inspectorItemId: "track-1",
    }, context);
    const repaired = repairMusicBuilderHistory(history, {
      playlistIds: new Set(),
      itemIds: new Set(),
    });
    expect(repaired.current).toEqual({ destination: { kind: "playlists" }, inspectorItemId: null });
  });
});
