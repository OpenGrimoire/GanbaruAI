import { describe, expect, it } from "vitest";
import {
  musicContextStateAfterAction,
  shouldResumePomodoroPausedMusic,
} from "./music-automation-ownership";

describe("music automation ownership", () => {
  it("keeps contextual ownership for automation and gives the rest of the phase to manual transport", () => {
    expect(musicContextStateAfterAction("playing", "context")).toBe("playing");
    expect(musicContextStateAfterAction("playing", "system")).toBe("playing");
    expect(musicContextStateAfterAction("playing", "pomodoro-pause")).toBe("playing");
    expect(musicContextStateAfterAction("playing", "manual")).toBe("overridden");
    expect(musicContextStateAfterAction("overridden", "context")).toBe("overridden");
  });

  it("resumes a focus-pause-owned pause when no manual action intervened", () => {
    expect(shouldResumePomodoroPausedMusic({
      pauseOwned: true,
      manualActionVersionAtPause: 4,
      currentManualActionVersion: 4,
      coordinator: true,
      preferenceEnabled: true,
      hasSource: true,
      playing: false,
      busy: false,
    })).toBe(true);
  });

  it("never auto-resumes after a user play or pause action", () => {
    expect(shouldResumePomodoroPausedMusic({
      pauseOwned: true,
      manualActionVersionAtPause: 4,
      currentManualActionVersion: 5,
      coordinator: true,
      preferenceEnabled: true,
      hasSource: true,
      playing: false,
      busy: false,
    })).toBe(false);
  });
});
