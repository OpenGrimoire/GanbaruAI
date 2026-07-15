// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import MusicBuilderHeader from "./MusicBuilderHeader.svelte";

describe("MusicBuilderHeader", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
    vi.useRealTimers();
  });

  it("keeps local typing immediate and sends one debounced search", async () => {
    vi.useFakeTimers();
    target = document.createElement("div");
    document.body.append(target);
    const onSearch = vi.fn();
    component = mount(MusicBuilderHeader, {
      target,
      props: {
        destination: { kind: "library" },
        search: "",
        searchAvailable: true,
        busy: false,
        reviewCount: 12,
        issueCount: 2,
        onOpenPlayer: vi.fn(),
        onNavigate: vi.fn(),
        onSearch,
        onRefresh: vi.fn(),
      },
    });
    await tick();

    expect(target.querySelector("input")).toBeNull();
    target.querySelector<HTMLButtonElement>("button[aria-label='Search music']")?.click();
    await tick();
    const input = target.querySelector<HTMLInputElement>("input[type='search']");
    expect(input).not.toBeNull();
    input!.value = "rain";
    input!.dispatchEvent(new InputEvent("input", { bubbles: true }));
    await tick();

    expect(input!.value).toBe("rain");
    expect(onSearch).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(180);
    expect(onSearch).toHaveBeenCalledOnce();
    expect(onSearch).toHaveBeenCalledWith("rain");
  });

  it("opens the media player directly", async () => {
    target = document.createElement("div");
    document.body.append(target);
    const onOpenPlayer = vi.fn();
    component = mount(MusicBuilderHeader, {
      target,
      props: {
        destination: { kind: "playlists" },
        search: "",
        searchAvailable: false,
        busy: false,
        reviewCount: 0,
        issueCount: 0,
        onOpenPlayer,
        onNavigate: vi.fn(),
        onSearch: vi.fn(),
        onRefresh: vi.fn(),
      },
    });
    await tick();

    target.querySelector<HTMLButtonElement>("[data-music-focus-key='builder:back-to-player']")?.click();

    expect(onOpenPlayer).toHaveBeenCalledOnce();
  });

  it("renders the six destinations as one roving top-bar navigation group", async () => {
    target = document.createElement("div");
    document.body.append(target);
    const onNavigate = vi.fn();
    component = mount(MusicBuilderHeader, {
      target,
      props: {
        destination: { kind: "review" },
        search: "",
        searchAvailable: true,
        busy: false,
        reviewCount: 4,
        issueCount: 1,
        onOpenPlayer: vi.fn(),
        onNavigate,
        onSearch: vi.fn(),
        onRefresh: vi.fn(),
      },
    });
    await tick();

    const items = [...target.querySelectorAll<HTMLButtonElement>("[data-builder-nav-item]")];
    expect(items).toHaveLength(6);
    expect(items.filter((item) => item.tabIndex === 0)).toEqual([items[0]]);
    expect(target.querySelector("h1")).toBeNull();
    items[0]?.focus();
    items[0]?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true, cancelable: true }));
    expect(document.activeElement).toBe(items[1]);
    items[1]?.click();
    expect(onNavigate).toHaveBeenCalledWith({ kind: "playlists" });
  });
});
