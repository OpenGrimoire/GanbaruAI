import { describe, expect, it } from "vitest";
import type { MusicSource } from "$lib/music/sources";
import { activeMusicSnapshotBackend } from "./music-snapshot-policy";

const base = { originalInput: "a", title: "a", startMs: null, endMs: null };
const local: MusicSource = { ...base, kind: "local-file", path: "/music/a.mp3", artworkPath: null, identity: "local:a" };
const youtube: MusicSource = { ...base, kind: "youtube-video", videoId: "abc", playlistId: null, identity: "youtube:abc" };

describe("Music snapshot policy", () => {
  it("polls no backend while idle", () => {
    expect(activeMusicSnapshotBackend(null, "idle", false)).toBeNull();
    expect(activeMusicSnapshotBackend(local, "idle", true)).toBeNull();
    expect(activeMusicSnapshotBackend(youtube, "ended", false)).toBeNull();
  });

  it("selects exactly the active source backend", () => {
    expect(activeMusicSnapshotBackend(local, "loading", true)).toBe("native");
    expect(activeMusicSnapshotBackend(local, "playing", true)).toBe("native");
    expect(activeMusicSnapshotBackend(local, "playing", false)).toBeNull();
    expect(activeMusicSnapshotBackend(youtube, "loading", true)).toBeNull();
    expect(activeMusicSnapshotBackend(youtube, "playing", true)).toBe("youtube");
    expect(activeMusicSnapshotBackend(youtube, "paused", false)).toBe("youtube");
  });
});
