import { describe, expect, it, vi } from "vitest";
import { APP_VIEWS, DETACHABLE_TAB_VIEWS, type DetachableTabView } from "$lib/navigation";
import {
  beginTabViewLoad,
  createTabViewLoader,
  initialTabView,
  rejectTabViewLoad,
  resolveTabViewLoad,
  type TabViewImporter,
} from "$lib/tab-view-loader";

function deferred<Component>() {
  let resolve!: (value: { default: Component }) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<{ default: Component }>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

describe("tab view loader", () => {
  it("imports only the requested view and deduplicates concurrent loads", async () => {
    const pending = deferred<string>();
    const calls: string[] = [];
    const importer = (view: DetachableTabView): TabViewImporter<string> => () => {
      calls.push(view);
      return pending.promise;
    };
    const loader = createTabViewLoader(
      {
        calendar: importer("calendar"),
        projects: importer("projects"),
        notes: importer("notes"),
      },
      vi.fn(async () => ({ default: "music component" })),
    );

    const first = loader.load("projects");
    const second = loader.load("projects");

    expect(second).toBe(first);
    expect(calls).toEqual(["projects"]);
    expect(loader.hasLoaded("projects")).toBe(false);

    pending.resolve({ default: "projects component" });
    await expect(first).resolves.toBe("projects component");
    await expect(loader.load("projects")).resolves.toBe("projects component");
    expect(calls).toEqual(["projects"]);
    expect(loader.hasLoaded("projects")).toBe(true);
  });

  it("clears failed imports so a retry starts a fresh request", async () => {
    let attempt = 0;
    const calendarImporter = vi.fn(() => {
      attempt += 1;
      if (attempt === 1) throw new Error("offline");
      return Promise.resolve({ default: "calendar component" });
    });
    const neverCalled = vi.fn(async () => ({ default: "unused component" }));
    const loader = createTabViewLoader(
      {
        calendar: calendarImporter,
        projects: neverCalled,
        notes: neverCalled,
      },
      neverCalled,
    );

    await expect(loader.load("calendar")).rejects.toThrow("offline");
    await expect(loader.retry("calendar")).resolves.toBe("calendar component");
    expect(calendarImporter).toHaveBeenCalledTimes(2);
    expect(neverCalled).not.toHaveBeenCalled();
  });

  it("ignores stale success and failure results after navigation changes", () => {
    const calendarRequest = beginTabViewLoad<string>(null, "calendar");
    const projectsRequest = beginTabViewLoad(calendarRequest, "projects");

    expect(
      resolveTabViewLoad(
        projectsRequest,
        "calendar",
        calendarRequest.requestId,
        "calendar component",
      ),
    ).toBe(projectsRequest);
    expect(
      rejectTabViewLoad(
        projectsRequest,
        "calendar",
        calendarRequest.requestId,
        new Error("stale"),
      ),
    ).toBe(projectsRequest);
    expect(
      resolveTabViewLoad(
        projectsRequest,
        "projects",
        projectsRequest.requestId,
        "projects component",
      ),
    ).toEqual({
      status: "ready",
      view: "projects",
      requestId: projectsRequest.requestId,
      component: "projects component",
    });

    const error = new Error("chunk unavailable");
    expect(
      rejectTabViewLoad(
        projectsRequest,
        "projects",
        projectsRequest.requestId,
        error,
      ),
    ).toEqual({
      status: "failed",
      view: "projects",
      requestId: projectsRequest.requestId,
      error,
    });
  });

  it("selects every direct main launch and gives detached routing precedence", () => {
    for (const view of APP_VIEWS) {
      expect(initialTabView(view, undefined)).toBe(view);
    }
    for (const detachedView of DETACHABLE_TAB_VIEWS) {
      expect(initialTabView("music", detachedView)).toBe(detachedView);
    }
  });
});
