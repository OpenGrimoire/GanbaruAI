import { describe, expect, it } from "vitest";
import { createDoomscrollingUsageBatch } from "./doomscrolling-usage-batch";

describe("Doomscrolling usage batch", () => {
  it("coalesces concurrent interval and shutdown flushes into one write", async () => {
    const writes: number[][] = [];
    const batch = createDoomscrollingUsageBatch<number>(async (samples) => { writes.push(samples); });
    batch.enqueue(1);
    batch.enqueue(2);
    await Promise.all([batch.flush(), batch.flush()]);
    expect(writes).toEqual([[1, 2]]);
  });

  it("keeps samples after a write failure for the next bounded flush", async () => {
    let fail = true;
    const writes: number[][] = [];
    const batch = createDoomscrollingUsageBatch<number>(async (samples) => {
      if (fail) throw new Error("process list failed");
      writes.push(samples);
    });
    batch.enqueue(1);
    await expect(batch.flush()).rejects.toThrow("process list failed");
    expect(batch.size).toBe(1);
    fail = false;
    await batch.flush();
    expect(writes).toEqual([[1]]);
  });
});
