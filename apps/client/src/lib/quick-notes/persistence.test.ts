import { describe, expect, it, vi } from "vitest";
import {
  flushQuickNoteEditors,
  registerQuickNotesFlusher,
} from "./persistence";

describe("Quick note editor persistence", () => {
  it("flushes every mounted editor and stops after unregistration", async () => {
    const first = vi.fn(async () => undefined);
    const second = vi.fn(async () => undefined);
    const unregisterFirst = registerQuickNotesFlusher(first);
    const unregisterSecond = registerQuickNotesFlusher(second);

    await flushQuickNoteEditors();
    unregisterFirst();
    await flushQuickNoteEditors();
    unregisterSecond();

    expect(first).toHaveBeenCalledOnce();
    expect(second).toHaveBeenCalledTimes(2);
  });
});
