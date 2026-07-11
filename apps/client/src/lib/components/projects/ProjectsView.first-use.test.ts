// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ProjectsSnapshot } from "$lib/projects/types";

const backend = vi.hoisted(() => {
  let calls = 0;
  const promise = new Promise<ProjectsSnapshot>(() => undefined);
  return {
    promise,
    get calls() {
      return calls;
    },
    recordCall() {
      calls += 1;
    },
  };
});

vi.mock("@tauri-apps/api/window", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@tauri-apps/api/window")>();
  return {
    ...actual,
    getCurrentWindow: () => ({ label: "main" }),
  };
});

vi.mock("@tauri-apps/api/event", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@tauri-apps/api/event")>();
  return {
    ...actual,
    listen: () => Promise.resolve(() => undefined),
  };
});

vi.mock("$lib/api/projects", async (importOriginal) => {
  const actual = await importOriginal<typeof import("$lib/api/projects")>();
  return {
    ...actual,
    loadProjectsSnapshot: () => {
      backend.recordCall();
      return backend.promise;
    },
  };
});

describe("ProjectsView first use", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
  });

  it("renders a useful loading state while the initial backend promise is unresolved", async () => {
    target = document.createElement("div");
    document.body.append(target);
    const { default: ProjectsView } = await import("./ProjectsView.svelte");

    component = mount(ProjectsView, { target });
    await tick();

    expect(backend.calls).toBe(1);
    expect(target.querySelector('[data-first-use-shell="projects"]')).not.toBeNull();
    expect(target.querySelector("[data-projects-first-use-state]")?.getAttribute("aria-busy"))
      .toBe("true");
  }, 15_000);
});
