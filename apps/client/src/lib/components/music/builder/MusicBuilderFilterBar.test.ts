// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import MusicBuilderFilterBar from "./MusicBuilderFilterBar.svelte";

describe("MusicBuilderFilterBar", () => {
  let target: HTMLDivElement | null = null;
  let component: ReturnType<typeof mount> | null = null;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = null;
    target = null;
  });

  it("removes one active filter without clearing the others", async () => {
    const onChange = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicBuilderFilterBar, {
      target,
      props: {
        sourceKind: "local-file",
        availability: "missing",
        sort: "title",
        direction: "ascending",
        resultCount: 3,
        snoozed: null,
        onChange,
      },
    });
    await tick();

    target.querySelector<HTMLButtonElement>('button[aria-haspopup="menu"]')?.click();
    await tick();
    [...target.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]')]
      .find((button) => button.textContent?.includes("All sources"))?.click();
    expect(onChange).toHaveBeenCalledWith({ sourceKind: null });
    expect([...target.querySelectorAll('[role="menuitemradio"]')]
      .some((button) => button.textContent?.includes("Missing"))).toBe(true);
  });

  it("opens a truthful keyboard menu and restores its trigger on Escape", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicBuilderFilterBar, {
      target,
      props: {
        sourceKind: "local-file",
        availability: null,
        sort: "title",
        direction: "ascending",
        resultCount: 3,
        snoozed: null,
        onChange: vi.fn(),
      },
    });
    await tick();

    const trigger = target.querySelectorAll<HTMLButtonElement>('button[aria-haspopup="menu"]')[1] ?? null;
    trigger?.click();
    await tick();
    await Promise.resolve();
    const menu = target.querySelector<HTMLElement>('[role="menu"]');
    const options = [...(menu?.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]') ?? [])];
    expect(options).toHaveLength(8);
    expect(options[0]?.getAttribute("aria-checked")).toBe("true");
    expect(document.activeElement).toBe(options[0]);

    options[0]?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true }));
    expect(document.activeElement).toBe(options[1]);
    options[1]?.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }));
    await Promise.resolve();
    expect(target.querySelector('[role="menu"]')).toBeNull();
    expect(document.activeElement).toBe(trigger);
  });
});
