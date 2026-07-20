import { describe, expect, it } from "vitest";
import { parseMusicSoundscapeSnapshot, parseMusicSoundscapes, parseMusicSoundscapeState } from "./soundscape-contracts";

describe("soundscape boundary contracts", () => {
  it("parses generated and device-local definitions", () => {
    expect(parseMusicSoundscapes([{
      id: "generated-pink-noise",
      sourceKind: "generated-noise",
      generatedKind: "pink",
      bundledIdentity: null,
      name: "Pink noise",
      availability: "available",
      localPath: null,
      createdAt: 1,
      updatedAt: 1,
      version: 1,
    }])).toHaveLength(1);
  });

  it("rejects unknown sources and nonfinite volume", () => {
    const definition = { id: "x", sourceKind: "youtube", generatedKind: null, bundledIdentity: null, name: "X", availability: "available", localPath: null, createdAt: 1, updatedAt: 1, version: 1 };
    expect(() => parseMusicSoundscapes([definition])).toThrow("sourceKind");
    expect(() => parseMusicSoundscapeState({ activeSoundscapeId: null, desiredPlaying: false, volume: Number.NaN, updatedAt: 1, version: 1 })).toThrow("volume");
  });

  it("parses native snapshots without exposing local paths", () => {
    expect(parseMusicSoundscapeSnapshot({ status: "playing", sourceId: "rain", volume: 0.4, errorCode: null })).toEqual({ status: "playing", sourceId: "rain", volume: 0.4, errorCode: null });
  });
});
