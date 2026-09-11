import { describe, expect, it, vi } from "vitest";
import { createPomodoroClockController } from "./pomodoro-clock-controller";
import { createPomodoroExtensionController } from "./pomodoro-extension-controller";
import { createPomodoroPhaseController } from "./pomodoro-phase-controller";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";
import { createPomodoroWindowStateController } from "./pomodoro-window-state-controller";

vi.mock("./pomodoro-adaptive-decisions", async (importOriginal) => {
  const actual = await importOriginal<typeof import("./pomodoro-adaptive-decisions")>();
  return {
    ...actual,
    decideBoundaryAdaptiveForState: vi.fn(async () => null),
  };
});

function createContext() {
  const runtime = createPomodoroRuntimeFixture({ isRunning: true });
  const clock = createPomodoroClockController(runtime, () => 10_000);
  const extensions = createPomodoroExtensionController({
    runtime,
    clock,
    hasOvertime: () => false,
    activeSegment: () => null,
    persistSegment: vi.fn(),
    recordRunEvent: vi.fn(),
    scheduleBreakEndWarning: vi.fn(),
    updateTray: vi.fn(),
    nowMs: () => 10_000,
    nowIso: () => "2026-07-12T00:00:10.000Z",
  });
  return {
    runtime,
    clock,
    extensions,
    windowState: createPomodoroWindowStateController(runtime, () => true),
    segments: {
      activeSegment: () => null,
      rebuildSegments: vi.fn(async () => undefined),
      boundarySegment: () => null,
      activateBoundarySegment: vi.fn(async () => undefined),
      activateSegment: vi.fn(async (): Promise<void> => undefined),
      markSegment: vi.fn(async (): Promise<void> => undefined),
    },
    stopPausedOpportunityCountdown: vi.fn(),
    stopVisualTick: vi.fn(),
    startVisualTick: vi.fn(),
    stopOvertime: vi.fn(),
    isOvertimeActive: () => false,
    initListeners: vi.fn(),
    startIdleChecking: vi.fn(),
    closeOverlay: vi.fn(),
    clearBreakEndWarning: vi.fn(),
    clearMusicPausedByPomodoro: vi.fn(),
    resetPausedFocusNotificationState: vi.fn(),
    scheduleBreakEndWarning: vi.fn(),
    showBreakOverlay: vi.fn(),
    updateTray: vi.fn(),
    recordRunEvent: vi.fn(),
    nowMs: () => 10_000,
    nowIso: () => "2026-07-12T00:00:10.000Z",
  };
}

describe("Pomodoro phase controller", () => {
  it("starts the countdown after an explicit return from a completed break is persisted", async () => {
    const context = createContext();
    context.runtime.phase = "short_break";
    context.runtime.isRunning = false;
    context.runtime.remainingSeconds = 0;
    context.runtime.segments = [{
      id: "next-focus", runId: "run-1", eventId: "event-1", eventDate: "2026-07-12",
      phase: "focus", rhythmPosition: 2, status: "planned", pauseLog: [],
      plannedStart: "2026-07-12T00:00:10.000Z", plannedEnd: "2026-07-12T00:40:10.000Z",
      actualStart: null, actualEnd: null,
    }];
    let releaseWrite: (() => void) | undefined;
    context.segments.activateSegment.mockImplementation(() => new Promise<void>((resolve) => {
      releaseWrite = resolve;
    }));
    const advancing = createPomodoroPhaseController(context).advancePhase();
    await vi.waitFor(() => expect(context.segments.activateSegment).toHaveBeenCalledWith(0));
    expect(context.runtime.isRunning).toBe(false);
    expect(context.startVisualTick).not.toHaveBeenCalled();

    releaseWrite?.();
    await advancing;
    expect(context.runtime.phase).toBe("focus");
    expect(context.runtime.isRunning).toBe(true);
    expect(context.runtime.lastTickMs).toBe(context.nowMs());
    expect(context.stopOvertime).toHaveBeenCalledOnce();
    expect(context.startVisualTick).toHaveBeenCalledOnce();
    expect(context.startIdleChecking).toHaveBeenCalledOnce();
  });

  it("guards concurrent phase advances until persistence completes", async () => {
    const context = createContext();
    let resolveMark!: () => void;
    context.segments.markSegment.mockImplementation(
      () => new Promise<void>((resolve) => { resolveMark = resolve; }),
    );
    const controller = createPomodoroPhaseController(context);

    const first = controller.advancePhase();
    const second = controller.advancePhase();
    expect(context.segments.markSegment).toHaveBeenCalledOnce();
    resolveMark();
    await Promise.all([first, second]);
    expect(context.segments.markSegment).toHaveBeenCalledOnce();
  });

  it("stops countdown when a phase write fails", async () => {
    const context = createContext();
    context.segments.markSegment.mockRejectedValue(new Error("disk full"));
    await expect(createPomodoroPhaseController(context).advancePhase()).rejects.toThrow("disk full");
    expect(context.runtime.isRunning).toBe(false);
    expect(context.runtime.phaseEndTime).toBeNull();
    expect(context.stopVisualTick).toHaveBeenCalledOnce();
    expect(context.startVisualTick).not.toHaveBeenCalled();
  });

  it("does not start unrecorded focus when the commitment has no further interval", async () => {
    const context = createContext();
    context.runtime.phase = "short_break";
    await expect(createPomodoroPhaseController(context).startFocusSession())
      .rejects.toThrow("No further focus interval fits");
    expect(context.runtime.isRunning).toBe(false);
    expect(context.startVisualTick).not.toHaveBeenCalled();
  });
});
