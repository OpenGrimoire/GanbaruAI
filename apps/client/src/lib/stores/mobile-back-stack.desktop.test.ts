import { describe, expect, it, vi } from "vitest";
import { getMobileBackStack } from "./mobile-back-stack.desktop";

describe("desktop mobile Back stack", () => {
  it("never retains or consumes Android Back layers", () => {
    const handle = vi.fn();
    const stack = getMobileBackStack();
    const deactivate = stack.activate({ handle });

    expect(stack.hasActiveLayer).toBe(false);
    expect(stack.consume()).toBe(false);
    expect(handle).not.toHaveBeenCalled();
    expect(getMobileBackStack()).toBe(stack);
    expect(deactivate()).toBeUndefined();
  });
});
