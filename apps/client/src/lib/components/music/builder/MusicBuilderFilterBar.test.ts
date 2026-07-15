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
        reviewState: null,
        sort: "title",
        direction: "ascending",
        groupBy: "none",
        resultCount: 3,
        sourceCollectionId: null,
        membershipPlaylistId: null,
        snoozed: null,
        onChange,
      },
    });
    await tick();

    target.querySelector<HTMLButtonElement>('button[aria-label="Clear Local filter"]')?.click();
    expect(onChange).toHaveBeenCalledWith({ sourceKind: null });
    expect(target.querySelector('button[aria-label="Clear Missing filter"]')).not.toBeNull();
  });

  it("opens a truthful keyboard menu and restores its trigger on Escape", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicBuilderFilterBar, {
      target,
      props: {
        sourceKind: "local-file",
        availability: null,
        reviewState: null,
        sort: "title",
        direction: "ascending",
        groupBy: "none",
        resultCount: 3,
        sourceCollectionId: null,
        membershipPlaylistId: null,
        snoozed: null,
        onChange: vi.fn(),
      },
    });
    await tick();

    const trigger = target.querySelector<HTMLButtonElement>('button[aria-haspopup="menu"]');
    trigger?.click();
    await tick();
    await Promise.resolve();
    const menu = target.querySelector<HTMLElement>('[role="menu"]');
    const options = [...(menu?.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]') ?? [])];
    expect(options).toHaveLength(3);
    expect(options[1]?.getAttribute("aria-checked")).toBe("true");
    expect(document.activeElement).toBe(options[1]);

    options[1]?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true }));
    expect(document.activeElement).toBe(options[2]);
    options[2]?.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }));
    await Promise.resolve();
    expect(target.querySelector('[role="menu"]')).toBeNull();
    expect(document.activeElement).toBe(trigger);
  });
});
