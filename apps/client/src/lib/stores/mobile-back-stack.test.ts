import { describe, expect, it } from "vitest";
import { getMobileBackStack } from "./mobile-back-stack.svelte";

describe("mobile Back stack", () => {
  it("consumes the most recently activated layer first", () => {
    const stack = getMobileBackStack();
    const handled: string[] = [];
    const removeDetail = stack.activate({
      handle: () => handled.push("detail"),
    });
    const removePicker = stack.activate({
      handle: () => handled.push("picker"),
    });

    expect(stack.hasActiveLayer).toBe(true);
    expect(stack.consume()).toBe(true);
    expect(handled).toEqual(["picker"]);

    removePicker();
    expect(stack.consume()).toBe(true);
    expect(handled).toEqual(["picker", "detail"]);

    removeDetail();
    expect(stack.hasActiveLayer).toBe(false);
    expect(stack.consume()).toBe(false);
  });

  it("allows cleanup to run more than once", () => {
    const stack = getMobileBackStack();
    const remove = stack.activate({ handle: () => undefined });

    remove();
    remove();

    expect(stack.hasActiveLayer).toBe(false);
  });

  it("moves a reactivated layer to the top", () => {
    const stack = getMobileBackStack();
    const handled: string[] = [];
    const removeDetail = stack.activate({
      handle: () => handled.push("detail"),
    });
    const removePicker = stack.activate({
      handle: () => handled.push("picker-old"),
    });
    removePicker();
    const removeReopenedPicker = stack.activate({
      handle: () => handled.push("picker-new"),
    });

    expect(stack.consume()).toBe(true);
    expect(handled).toEqual(["picker-new"]);

    removeReopenedPicker();
    removeDetail();
  });

  it("keeps the top layer active when a lower layer is removed", () => {
    const stack = getMobileBackStack();
    const handled: string[] = [];
    const removeDetail = stack.activate({
      handle: () => handled.push("detail"),
    });
    const removePicker = stack.activate({
      handle: () => handled.push("picker"),
    });

    removeDetail();
    expect(stack.consume()).toBe(true);
    expect(handled).toEqual(["picker"]);

    removePicker();
  });
});
