// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import MusicBuilderNavigation from "./MusicBuilderNavigation.svelte";

describe("MusicBuilderNavigation", () => {
  let target: HTMLDivElement | null = null;
  let component: ReturnType<typeof mount> | null = null;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = null;
    target = null;
  });

  it("keeps one tab stop and moves it with arrow navigation", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicBuilderNavigation, {
      target,
      props: {
        destination: { kind: "review" },
        playlists: [],
        reviewCount: 4,
        issueCount: 1,
        onNavigate: vi.fn(),
      },
    });
    await tick();
    await Promise.resolve();

    const items = [...target.querySelectorAll<HTMLButtonElement>("[data-builder-nav-item]")];
    expect(items.filter((item) => item.tabIndex === 0)).toEqual([items[0]]);
    items[0]?.focus();
    items[0]?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true }));
    expect(document.activeElement).toBe(items[1]);
    expect(items.filter((item) => item.tabIndex === 0)).toEqual([items[1]]);
  });
});
