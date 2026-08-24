// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import ConfirmDialog from "./ConfirmDialog.svelte";

describe("ConfirmDialog", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
  });

  it("adds the standard keyboard hints to custom button labels", () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ConfirmDialog, {
      target,
      props: {
        message: "Delete this item?",
        confirmLabel: "Delete",
        cancelLabel: "Keep",
        onConfirm: () => undefined,
        onCancel: () => undefined,
      },
    });

    expect(target.textContent).toContain("Keep (Esc)");
    expect(target.textContent).toContain("Delete (Enter)");
  });

  it("adds the standard keyboard hints to the default labels", () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ConfirmDialog, {
      target,
      props: {
        message: "Continue?",
        onConfirm: () => undefined,
        onCancel: () => undefined,
      },
    });

    expect(target.textContent).toContain("No (Esc)");
    expect(target.textContent).toContain("Yes (Enter)");
  });

  it("separates dismissing the dialog from its explicit cancel action", async () => {
    target = document.createElement("div");
    document.body.append(target);
    const onCancel = vi.fn();
    const onDismiss = vi.fn();
    component = mount(ConfirmDialog, {
      target,
      props: {
        message: "Resume the session?",
        onConfirm: vi.fn(),
        onCancel,
        onDismiss,
      },
    });
    await tick();

    target.querySelector<HTMLElement>(".confirm-dialog")?.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }),
    );
    expect(onDismiss).toHaveBeenCalledOnce();
    expect(onCancel).not.toHaveBeenCalled();

    target.querySelectorAll<HTMLButtonElement>(".confirm-dialog button").item(0).click();
    expect(onCancel).toHaveBeenCalledOnce();
  });
});
