// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import ChatTitleEditor from "./ChatTitleEditor.svelte";

describe("ChatTitleEditor", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
  });

  function setup(onCommit = vi.fn(async () => undefined), onCancel = vi.fn()): HTMLInputElement {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatTitleEditor, {
      target,
      props: { title: "Existing title", onCommit, onCancel },
    });
    const input = target.querySelector("input");
    if (!(input instanceof HTMLInputElement)) throw new Error("Title input did not render");
    return input;
  }

  it("commits a trimmed title with Enter", async () => {
    const onCommit = vi.fn(async () => undefined);
    const input = setup(onCommit);
    input.value = "  Updated title  ";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await tick();
    expect(onCommit).toHaveBeenCalledWith("Updated title");
  });

  it("preserves editing and announces an empty title", async () => {
    const onCommit = vi.fn(async () => undefined);
    const input = setup(onCommit);
    input.value = "   ";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await tick();
    expect(onCommit).not.toHaveBeenCalled();
    expect(target?.querySelector("[role='alert']")?.textContent).toContain("cannot be empty");
  });

  it("cancels without committing on Escape", async () => {
    const onCommit = vi.fn(async () => undefined);
    const onCancel = vi.fn();
    const input = setup(onCommit, onCancel);
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await tick();
    expect(onCancel).toHaveBeenCalledOnce();
    expect(onCommit).not.toHaveBeenCalled();
  });

  it("commits on blur", async () => {
    const onCommit = vi.fn(async () => undefined);
    const input = setup(onCommit);
    input.value = "Blurred title";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new FocusEvent("blur", { bubbles: true }));
    await tick();
    expect(onCommit).toHaveBeenCalledWith("Blurred title");
  });
});
