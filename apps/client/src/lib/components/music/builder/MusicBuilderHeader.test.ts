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
        title: "Library",
        search: "",
        busy: false,
        resultCount: 12,
        onBack: vi.fn(),
        onSearch,
        onRefresh: vi.fn(),
      },
    });
    await tick();

    const input = target.querySelector<HTMLInputElement>("input");
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
});
