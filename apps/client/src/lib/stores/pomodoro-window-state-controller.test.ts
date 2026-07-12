import { describe, expect, it } from "vitest";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";
import { createPomodoroWindowStateController } from "./pomodoro-window-state-controller";

describe("Pomodoro window state controller", () => {
  it("does not apply secondary snapshots to the coordinator", () => {
    const source = createPomodoroRuntimeFixture({
      remainingSeconds: 300,
      isRunning: true,
    });
    const snapshot = createPomodoroWindowStateController(
      source,
      () => true,
    ).buildWindowSnapshot();
    const coordinator = createPomodoroRuntimeFixture({ remainingSeconds: 900 });

    createPomodoroWindowStateController(
      coordinator,
      () => true,
    ).applyWindowSnapshot(snapshot);

    expect(coordinator.remainingSeconds).toBe(900);
  });

  it("clones secondary state and restores the active segment index", () => {
    const source = createPomodoroRuntimeFixture({
      segments: [
        {
          id: "segment-1",
          phase: "focus",
          status: "active",
          plannedStart: "2026-07-12T00:00:00.000Z",
          plannedEnd: "2026-07-12T00:25:00.000Z",
          actualStart: null,
          actualEnd: null,
          pauseLog: [],
          eventId: "event-1",
          eventDate: "2026-07-12",
          runId: "run-1",
          rhythmPosition: 1,
        },
      ],
      currentSegmentIndex: 0,
    });
    const sourceController = createPomodoroWindowStateController(
      source,
      () => true,
    );
    const target = createPomodoroRuntimeFixture();
    createPomodoroWindowStateController(
      target,
      () => false,
    ).applyWindowSnapshot(sourceController.buildWindowSnapshot());

    expect(target.currentSegmentIndex).toBe(0);
    expect(target.segments).not.toBe(source.segments);
    expect(target.segments[0]?.pauseLog).not.toBe(source.segments[0]?.pauseLog);
  });
});
