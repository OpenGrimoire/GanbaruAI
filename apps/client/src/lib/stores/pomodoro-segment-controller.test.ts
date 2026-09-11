import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PersistedSegment } from "$lib/components/calendar/types";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";
import { createPomodoroRunRepository } from "./pomodoro-run-repository";
import { createPomodoroSegmentController } from "./pomodoro-segment-controller";

const invokeMock = vi.fn<(...args: unknown[]) => Promise<unknown>>();

vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invokeMock(...args) }));
vi.mock("$lib/api/db", () => ({ dbUrl: () => "sqlite:ganbaru-ai.sqlite" }));

/** Use the real repository queue so native failure propagation is part of each assertion. */
function setup() {
  const planned: PersistedSegment = {
    id: "focus", runId: "run", eventId: "event", eventDate: "2026-09-06",
    phase: "focus", rhythmPosition: 2, status: "planned", pauseLog: [],
    plannedStart: "2026-09-06T15:00:00.000Z", plannedEnd: "2026-09-06T15:40:00.000Z",
    actualStart: null, actualEnd: null,
  };
  const context = {
    ...createPomodoroRuntimeFixture({ activeRunId: "run", segments: [planned] }),
    publishWindowSnapshot: vi.fn(),
    refreshCurrentPhaseLimit: vi.fn(),
    scheduleBreakEndWarning: vi.fn(),
    updateTray: vi.fn(),
    startHeartbeat: vi.fn(),
    stopSession: vi.fn(async (): Promise<void> => undefined),
    isPausedForBridgeSegment: () => false,
    applyRunStartAdaptiveDecision: vi.fn(),
  };
  const repository = createPomodoroRunRepository({ endReasonForSegment: () => null, completeWrite: vi.fn() });
  return { planned, context, controller: createPomodoroSegmentController(context, repository) };
}

beforeEach(() => { invokeMock.mockReset(); });

describe("Pomodoro segment activation", () => {
  it("publishes an accepted segment only after the native write acknowledges it", async () => {
    const { planned, context, controller } = setup();
    let commit: (() => void) | undefined;
    invokeMock.mockImplementation(() => new Promise<void>((resolve) => { commit = resolve; }));
    const activation = controller.activateSegment(0);
    await vi.waitFor(() => expect(invokeMock).toHaveBeenCalledOnce());
    expect(context.segments).toEqual([planned]);
    expect(context.currentSegmentIndex).toBe(-1);
    expect(context.publishWindowSnapshot).not.toHaveBeenCalled();

    commit?.();
    await activation;
    expect(context.segments[0].status).toBe("active");
    expect(context.segments[0].actualStart).not.toBeNull();
    expect(context.currentSegmentIndex).toBe(0);
    expect(context.publishWindowSnapshot).toHaveBeenCalledOnce();
  });

  it.each(["planned", "boundary"] as const)("keeps %s activation unpublished after a failed native write", async (kind) => {
    const { planned, context, controller } = setup();
    invokeMock.mockRejectedValue(new Error("disk full"));
    const activation = kind === "planned"
      ? controller.activateSegment(0)
      : controller.activateBoundarySegment({
        ...planned, status: "active", actualStart: planned.plannedStart,
      }, null);
    await expect(activation).rejects.toThrow("disk full");
    expect(context.segments).toEqual([planned]);
    expect(context.currentSegmentIndex).toBe(-1);
    expect(context.publishWindowSnapshot).not.toHaveBeenCalled();
  });
});
