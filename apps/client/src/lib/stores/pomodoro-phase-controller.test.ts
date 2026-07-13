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
      activateSegment: vi.fn(async () => undefined),
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
});
