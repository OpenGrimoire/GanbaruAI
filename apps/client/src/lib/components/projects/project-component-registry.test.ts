import { describe, expect, it, vi } from "vitest";
import {
  beginLazyComponentLoad,
  rejectLazyComponentLoad,
  resolveLazyComponentLoad,
} from "$lib/lazy-component-loader";
import type { ProjectViewId } from "$lib/projects/types";
import {
  loadProjectOptionalComponent,
  loadProjectView,
  projectOptionalComponentHasLoaded,
  projectViewHasLoaded,
} from "./project-component-registry";

vi.mock("@tauri-apps/api/window", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@tauri-apps/api/window")>();
  return {
    ...actual,
    getCurrentWindow: () => ({ label: "main" }),
  };
});

describe("Project component registry", () => {
  it("loads only the requested view constructor and caches it", async () => {
    const first = loadProjectView("kanban");
    const second = loadProjectView("kanban");

    expect(second).toBe(first);
    await expect(first).resolves.toMatchObject({ view: "kanban" });
    expect(projectViewHasLoaded("kanban")).toBe(true);
    expect(projectViewHasLoaded("list")).toBe(false);
    expect(projectViewHasLoaded("calendar")).toBe(false);
    expect(projectViewHasLoaded("gantt")).toBe(false);
    expect(projectViewHasLoaded("dashboard")).toBe(false);
  });

  it("keeps optional surfaces separate and single-flight", async () => {
    const first = loadProjectOptionalComponent("task-finder");
    const second = loadProjectOptionalComponent("task-finder");

    expect(second).toBe(first);
    await expect(first).resolves.toMatchObject({ kind: "task-finder" });
    expect(projectOptionalComponentHasLoaded("task-finder")).toBe(true);
    expect(projectOptionalComponentHasLoaded("toolbar")).toBe(false);
    expect(projectOptionalComponentHasLoaded("toolbar-settings")).toBe(false);
    expect(projectOptionalComponentHasLoaded("bulk-actions")).toBe(false);
    expect(projectOptionalComponentHasLoaded("task-detail")).toBe(false);
    expect(projectOptionalComponentHasLoaded("project-navigator")).toBe(false);
  });

  it("does not let a slow previous view replace the current view", () => {
    const list = beginLazyComponentLoad<ProjectViewId, string>(null, "list");
    const gantt = beginLazyComponentLoad<ProjectViewId, string>(list, "gantt");

    expect(resolveLazyComponentLoad(list, "list", list.requestId, "list component"))
      .not.toBe(gantt);
    expect(resolveLazyComponentLoad(gantt, "list", list.requestId, "list component"))
      .toBe(gantt);
    expect(rejectLazyComponentLoad(gantt, "list", list.requestId, new Error("stale")))
      .toBe(gantt);
    expect(resolveLazyComponentLoad(gantt, "gantt", gantt.requestId, "gantt component"))
      .toEqual({
        status: "ready",
        key: "gantt",
        requestId: gantt.requestId,
        component: "gantt component",
      });
  });
});
