import { describe, expect, it } from "vitest";
import type { LoadedProjectView } from "./project-component-registry";
import {
  ProjectRouteLoadController,
  type ProjectRouteLoaders,
} from "./project-route-load-controller.svelte";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((onResolve, onReject) => {
    resolve = onResolve;
    reject = onReject;
  });
  return { promise, resolve, reject };
}

type ProjectsStore = ConstructorParameters<typeof ProjectRouteLoadController>[0];

describe("Project route load controller", () => {
  it("does not let a stale view import replace the active request", async () => {
    const list = deferred<LoadedProjectView>();
    const gantt = deferred<LoadedProjectView>();
    const loaders = {
      loadView: (view) => view === "list" ? list.promise : gantt.promise,
      retryView: (view) => view === "list" ? list.promise : gantt.promise,
      loadOptional: () => new Promise(() => undefined),
      retryOptional: () => new Promise(() => undefined),
    } satisfies ProjectRouteLoaders;
    const controller = new ProjectRouteLoadController({} as ProjectsStore, loaders);

    controller.requestView("list");
    controller.requestView("gantt");
    list.resolve({ view: "list", component: {} } as LoadedProjectView);
    await list.promise;
    await Promise.resolve();

    expect(controller.viewState).toMatchObject({ key: "gantt", status: "loading" });
    gantt.resolve({ view: "gantt", component: {} } as LoadedProjectView);
    await gantt.promise;
    await Promise.resolve();
    expect(controller.viewState).toMatchObject({ key: "gantt", status: "ready" });
  });

  it("isolates stale task-detail failures by project and task identity", async () => {
    const first = deferred<void>();
    const second = deferred<void>();
    let call = 0;
    const store = {
      selectedProject: { id: "project-1" },
      ensureTaskDetailData: () => (call++ === 0 ? first.promise : second.promise),
    } as unknown as ProjectsStore;
    const controller = new ProjectRouteLoadController(store);

    const firstRequest = controller.requestTaskDetailData("project-1", "task-1");
    const secondRequest = controller.requestTaskDetailData("project-1", "task-2");
    first.reject(new Error("stale failure"));
    await firstRequest;
    expect(controller.taskDetailDataError).toBeNull();

    second.reject(new Error("current failure"));
    await secondRequest;
    expect(controller.taskDetailDataError).toBe("current failure");
  });
});
