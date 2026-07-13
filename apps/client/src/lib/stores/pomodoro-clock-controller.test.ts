import { describe, expect, it } from "vitest";
import { createPomodoroClockController } from "./pomodoro-clock-controller";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";

describe("Pomodoro clock controller", () => {
  it("keeps work duration and block-limited visible time distinct", () => {
    const nowMs = 10_000;
    const runtime = createPomodoroRuntimeFixture({
      activeBlockId: "block-1",
      activeBlockEndMs: nowMs + 60_000,
    });
    const clock = createPomodoroClockController(runtime, () => nowMs);

    expect(clock.setPhaseRemainingSeconds(1_500, 120)).toBe(60);
    expect(runtime.phaseElapsedSeconds).toBe(120);
    expect(runtime.phaseWorkDurationSeconds).toBe(1_620);
    expect(runtime.phaseTotalSeconds).toBe(180);
  });

  it("expires paused opportunity time against the current deadline", () => {
    let nowMs = 10_000;
    const runtime = createPomodoroRuntimeFixture({
      activeBlockId: "block-1",
      activeBlockEndMs: nowMs + 2_000,
      remainingSeconds: 20,
      phaseWorkDurationSeconds: 20,
    });
    const clock = createPomodoroClockController(runtime, () => nowMs);

    expect(clock.refreshPausedOpportunityRemaining()).toBe(true);
    expect(runtime.remainingSeconds).toBe(2);
    nowMs += 2_000;
    expect(clock.activeBlockDeadlineReached()).toBe(true);
    expect(clock.refreshPausedOpportunityRemaining()).toBe(true);
    expect(runtime.remainingSeconds).toBe(0);
  });

  it("does not allow pause or resume during suspend or idle ownership", () => {
    const runtime = createPomodoroRuntimeFixture({
      isRunning: true,
      suspendedAway: { awaySeconds: 30 },
    });
    const clock = createPomodoroClockController(runtime);

    expect(clock.canPauseResume()).toBe(false);
    runtime.suspendedAway = null;
    expect(clock.canPauseResume()).toBe(true);
    runtime.idlePaused = {
      idleSeconds: 60,
      nativeOverlay: false,
      idleStartMs: 1,
      overlayStartedAtMs: 1,
      focusFailed: false,
      focusFailedAtMs: null,
    };
    expect(clock.canPauseResume()).toBe(false);
  });
});
