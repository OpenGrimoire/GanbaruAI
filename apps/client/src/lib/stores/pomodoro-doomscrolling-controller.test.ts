import { describe, expect, it, vi } from "vitest";
import { createPomodoroClockController } from "./pomodoro-clock-controller";
import { createPomodoroDoomscrollingController } from "./pomodoro-doomscrolling-controller";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";

describe("Pomodoro doomscrolling controller", () => {
  it("deduplicates equivalent projections and permits forced heartbeats", () => {
    const runtime = createPomodoroRuntimeFixture({
      isRunning: true,
      activeRunId: "run-1",
      remainingSeconds: 120,
    });
    const writeState = vi.fn(() => Promise.resolve());
    const controller = createPomodoroDoomscrollingController(
      runtime,
      createPomodoroClockController(runtime),
      {
        isCoordinator: () => true,
        writeState,
        now: () => "2026-07-12T00:00:00.000Z",
      },
    );

    controller.writeCurrentState();
    controller.writeCurrentState();
    controller.writeCurrentState(true);

    expect(writeState).toHaveBeenCalledTimes(2);
    expect(writeState).toHaveBeenLastCalledWith(
      expect.objectContaining({
        active: true,
        paused: false,
        activeRunId: "run-1",
        remainingSeconds: 120,
      }),
    );
  });

  it("projects idle and suspend pause reasons without writing in secondary windows", () => {
    const runtime = createPomodoroRuntimeFixture({
      isRunning: false,
      remainingSeconds: 100,
      phaseTotalSeconds: 1_500,
      idlePaused: {
        idleSeconds: 60,
        nativeOverlay: false,
        idleStartMs: 1,
        overlayStartedAtMs: 1,
        focusFailed: false,
        focusFailedAtMs: null,
      },
    });
    const writeState = vi.fn(() => Promise.resolve());
    const controller = createPomodoroDoomscrollingController(
      runtime,
      createPomodoroClockController(runtime),
      { isCoordinator: () => true, writeState },
    );

    controller.writeCurrentState();
    expect(writeState).toHaveBeenCalledWith(
      expect.objectContaining({ paused: true, pauseReason: "idle" }),
    );

    const secondaryWrite = vi.fn(() => Promise.resolve());
    createPomodoroDoomscrollingController(
      runtime,
      createPomodoroClockController(runtime),
      { isCoordinator: () => false, writeState: secondaryWrite },
    ).writeCurrentState(true);
    expect(secondaryWrite).not.toHaveBeenCalled();
  });
});
