import { describe, expect, it } from "vitest";
import { createMusicPlaybackWriter } from "./music-playback-writer";

describe("Music playback writer", () => {
  it("coalesces queued checkpoints and rejects a late old generation", async () => {
    const writes: string[] = [];
    let releaseFirst: (() => void) | undefined;
    const firstBlocked = new Promise<void>((resolve) => { releaseFirst = resolve; });
    const writer = createMusicPlaybackWriter(async (state: { sourceIdentity: string; updatedAt: number }) => {
      writes.push(`${state.sourceIdentity}:${state.updatedAt}`);
      if (writes.length === 1) await firstBlocked;
    });
    let generation = 1;
    const current = (candidate: number, source: string) => candidate === generation && source === "new";

    const old = writer.save({ sourceIdentity: "old", updatedAt: 1 }, 1, () => true);
    const skipped = writer.save({ sourceIdentity: "old", updatedAt: 2 }, 1, current);
    generation = 2;
    const newest = writer.save({ sourceIdentity: "new", updatedAt: 3 }, 2, current);
    releaseFirst!();

    expect(await old).toBe(true);
    expect(await skipped).toBe(false);
    expect(await newest).toBe(true);
    expect(writes).toEqual(["old:1", "new:3"]);
  });
});
