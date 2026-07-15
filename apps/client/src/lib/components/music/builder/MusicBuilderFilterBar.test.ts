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
});
