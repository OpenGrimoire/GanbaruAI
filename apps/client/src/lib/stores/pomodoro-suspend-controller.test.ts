import { describe, expect, it, vi } from "vitest";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";
import { createPomodoroSuspendController } from "./pomodoro-suspend-controller";

function createContext() {
  const runtime = createPomodoroRuntimeFixture({
    suspendedAway: { awaySeconds: 60 },
    remainingSeconds: 120,
    activeBlockId: "block-1",
    activeBlockEndMs: 50_000,
  });
  return {
    runtime,
    stopPausedOpportunityCountdown: vi.fn(),
    startVisualTick: vi.fn(),
    scheduleBreakEndWarning: vi.fn(),
    startIdleChecking: vi.fn(),
    stopIdleChecking: vi.fn(),
    updateTray: vi.fn(),
    clearBreakEndWarning: vi.fn(),
    stopOvertime: vi.fn(),
    resetPhaseProgress: vi.fn(),
    resetFocusNotificationState: vi.fn(),
    clearSegments: vi.fn(),
    activeSegmentPauseStart: () => null,
    interruptCurrentSegment: vi.fn(async () => undefined),
    skipRemainingSegments: vi.fn(async () => undefined),
    closeActiveRun: vi.fn(async (): Promise<void> => undefined),
    defaultFocusSeconds: 1_500,
    nowMs: () => 10_000,
    nowIso: () => "2026-07-12T00:00:10.000Z",
  };
}

describe("Pomodoro suspend controller", () => {
  it("resumes with a fresh block lookup boundary and one timer owner", async () => {
    const context = createContext();
    await createPomodoroSuspendController(context).dismiss(true);

    expect(context.runtime.suspendedAway).toBeNull();
    expect(context.runtime.activeBlockEndMs).toBeNull();
    expect(context.runtime.phaseEndTime).toBe(130_000);
    expect(context.startVisualTick).toHaveBeenCalledOnce();
    expect(context.startIdleChecking).toHaveBeenCalledOnce();
  });

  it("closes persistence before clearing a stopped suspended session", async () => {
    const context = createContext();
    let resolveClose!: () => void;
    context.closeActiveRun.mockImplementation(
      () => new Promise<void>((resolve) => { resolveClose = resolve; }),
    );
    const dismissal = createPomodoroSuspendController(context).dismiss(false);

    expect(context.runtime.activeBlockId).toBe("block-1");
    resolveClose();
    await dismissal;
    expect(context.runtime.activeBlockId).toBeNull();
    expect(context.runtime.dismissedBlockId).toBe("block-1");
  });
});
