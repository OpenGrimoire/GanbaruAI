import { describe, expect, it } from "vitest";

import { IDLE_CHECK_MAX_INTERVAL_MS } from "./pomodoro-machine";
import { nextIdleStatusCheckDelayMs } from "./pomodoro-idle-controller";

describe("nextIdleStatusCheckDelayMs", () => {
  it("retries an unavailable idle source at the maximum polling interval", () => {
    expect(nextIdleStatusCheckDelayMs(300_000, null, false)).toBe(
      IDLE_CHECK_MAX_INTERVAL_MS,
    );
  });

  it("keeps threshold-aware scheduling for an available idle source", () => {
    expect(nextIdleStatusCheckDelayMs(60_000, 59_750, false)).toBe(1_000);
  });
});
