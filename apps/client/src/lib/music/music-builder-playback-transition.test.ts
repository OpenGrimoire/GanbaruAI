import { describe, expect, it } from "vitest";
import { musicBuilderPlaybackDecision } from "./music-builder-playback-transition";

describe("music builder playback transitions", () => {
  it("keeps Review ownership during internal builder navigation", () => {
    expect(musicBuilderPlaybackDecision(true, "internal-navigation")).toBe("continue-review");
  });

  it("resolves ownership only when leaving or starting other playback", () => {
    expect(musicBuilderPlaybackDecision(true, "external-exit")).toBe("resolve-review-exit");
    expect(musicBuilderPlaybackDecision(true, "explicit-playback")).toBe("release-review");
    expect(musicBuilderPlaybackDecision(true, "automation-boundary")).toBe("supersede-review");
  });

  it("does nothing when Review does not own playback", () => {
    expect(musicBuilderPlaybackDecision(false, "external-exit")).toBe("no-review-action");
  });
});
