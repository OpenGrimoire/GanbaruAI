import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPomodoroRunRepository } from "./pomodoro-run-repository";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("$lib/api/db", () => ({
  dbUrl: () => "sqlite:ganbaru-ai.sqlite",
}));

describe("Pomodoro run repository mobile recovery", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("recovers only committed state using the native clock without querying notification projections", async () => {
    invokeMock.mockResolvedValue({ kind: "none" });
    const completeWrite = vi.fn();
    const repository = createPomodoroRunRepository({
      endReasonForSegment: () => null,
      completeWrite,
    });

    await expect(repository.recoverMobileRun())
      .resolves.toEqual({ kind: "none" });
    expect(invokeMock).toHaveBeenCalledExactlyOnceWith("pomodoro_recover_mobile_run", {
      dbUrl: "sqlite:ganbaru-ai.sqlite",
    });
    expect(completeWrite).not.toHaveBeenCalled();
  });

  it("bumps persisted segment readers after recovery closes a run", async () => {
    invokeMock.mockResolvedValue({
      kind: "closed",
      reason: "invalid_state",
      closedRunIds: ["run-1"],
    });
    const completeWrite = vi.fn();
    const repository = createPomodoroRunRepository({
      endReasonForSegment: () => null,
      completeWrite,
    });

    await expect(repository.recoverMobileRun())
      .resolves.toMatchObject({ kind: "closed", reason: "invalid_state" });
    expect(completeWrite).toHaveBeenCalledOnce();
  });

  it("waits for pending local writes before recovering their phase window", async () => {
    let releaseWrite: (() => void) | undefined;
    const write = new Promise<void>((resolve) => { releaseWrite = resolve; });
    invokeMock.mockImplementation((command: string) => command === "pomodoro_update_run_window"
      ? write
      : Promise.resolve({ kind: "none" }));
    const repository = createPomodoroRunRepository({
      endReasonForSegment: () => null,
      completeWrite: vi.fn(),
    });
    repository.updateRunWindow({ runId: "run-1", plannedEnd: "2026-05-29T11:00:00.000Z" });
    const recovery = repository.recoverMobileRun();
    await vi.waitFor(() => expect(invokeMock).toHaveBeenCalledOnce());
    expect(invokeMock.mock.calls[0][0]).toBe("pomodoro_update_run_window");

    releaseWrite?.();
    await expect(recovery).resolves.toEqual({ kind: "none" });
    expect(invokeMock.mock.calls[1][0]).toBe("pomodoro_recover_mobile_run");
  });

  it("rejects malformed native recovery data before state can consume it", async () => {
    invokeMock.mockResolvedValue({ kind: "resumed", run: { runId: 42 } });
    const repository = createPomodoroRunRepository({
      endReasonForSegment: () => null,
      completeWrite: vi.fn(),
    });

    await expect(repository.recoverMobileRun())
      .rejects.toThrow("Invalid resumed mobile pomodoro recovery response");
  });
});
