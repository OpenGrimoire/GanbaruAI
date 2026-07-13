import { describe, expect, it } from "vitest";
import { doomscrollingObservationPlan } from "./doomscrolling-observation-policy";

describe("Doomscrolling observation policy", () => {
  it.each([
    [false, false, false, 0, 0],
    [true, false, false, 0, 0],
    [true, true, false, 0, 1],
    [true, false, true, 1, 0],
    [true, true, true, 1, 1],
  ] as const)(
    "bounds foreground and process commands for main=%s blocking=%s usage=%s",
    (main, blocking, usage, foregroundCalls, processCalls) => {
      const plan = doomscrollingObservationPlan(main, blocking, usage);
      expect(Number(plan.observeForeground)).toBe(foregroundCalls);
      expect(Number(plan.scanProcesses)).toBe(processCalls);
      expect(plan.coordinatorEnabled).toBe(foregroundCalls + processCalls > 0);
    },
  );

  it("never starts another coordinator in a detached window", () => {
    expect(doomscrollingObservationPlan(false, true, true).coordinatorEnabled).toBe(false);
    expect(doomscrollingObservationPlan(false, true, true).coordinatorEnabled).toBe(false);
  });
});
