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

  it("uses the native clock and validates the response", async () => {
    invokeMock.mockResolvedValue({ kind: "none" });
    const completeWrite = vi.fn();
    const repository = createPomodoroRunRepository({
      endReasonForSegment: () => null,
      completeWrite,
    });

    await expect(repository.recoverMobileRun())
      .resolves.toEqual({ kind: "none" });
    expect(invokeMock).toHaveBeenCalledWith("pomodoro_recover_mobile_run", {
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
