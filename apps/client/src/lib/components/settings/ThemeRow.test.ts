// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import { lightTheme } from "$lib/stores/themes";
import ThemeRow from "./ThemeRow.svelte";

describe("ThemeRow mobile actions", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
  });

  it("keeps editor actions while omitting the unavailable file export", async () => {
    const onOpen = vi.fn();
    const onDuplicate = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ThemeRow, {
      target,
      props: {
        theme: lightTheme,
        isActive: false,
        isBuiltin: true,
        onApply: vi.fn(),
        onOpen,
        onDuplicate,
        onExport: vi.fn(),
        onDelete: vi.fn(),
        showFileActions: false,
        mobileLayout: true,
      },
    });
    await tick();

    const duplicate = target.querySelector<HTMLButtonElement>(
      '[aria-label="Duplicate and edit theme"]',
    );
    const open = target.querySelector<HTMLButtonElement>('[aria-label="View theme"]');
    expect(duplicate).not.toBeNull();
    expect(open).not.toBeNull();
    expect(target.querySelector('[aria-label="Export theme JSON"]')).toBeNull();
    expect(open?.classList.contains("size-12")).toBe(true);

    duplicate?.click();
    open?.click();
    expect(onDuplicate).toHaveBeenCalledOnce();
    expect(onOpen).toHaveBeenCalledOnce();
  });
});
