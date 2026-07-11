import type { DetachableTabView, View } from "$lib/navigation";

export interface TabViewModule<Component> {
  default: Component;
}

export type TabViewImporter<Component> = () => Promise<TabViewModule<Component>>;

export interface TabViewLoader<Component> {
  load: (view: View) => Promise<Component>;
  retry: (view: View) => Promise<Component>;
  hasLoaded: (view: View) => boolean;
}

interface TabViewLoadBase {
  view: View;
  requestId: number;
}

export interface LoadingTabViewState extends TabViewLoadBase {
  status: "loading";
}

export interface ReadyTabViewState<Component> extends TabViewLoadBase {
  status: "ready";
  component: Component;
}

export interface FailedTabViewState extends TabViewLoadBase {
  status: "failed";
  error: unknown;
}

export type TabViewLoadState<Component> =
  | LoadingTabViewState
  | ReadyTabViewState<Component>
  | FailedTabViewState;

/**
 * Creates a process-local loader that caches successful view modules and
 * deduplicates concurrent imports for the same view.
 */
export function createTabViewLoader<Component>(
  detachableImporters: Readonly<Record<DetachableTabView, TabViewImporter<Component>>>,
  musicImporter: TabViewImporter<Component>,
): TabViewLoader<Component> {
  const loaded = new Map<View, Component>();
  const inFlight = new Map<View, Promise<Component>>();

  function importerFor(view: View): TabViewImporter<Component> {
    return view === "music" ? musicImporter : detachableImporters[view];
  }

  function load(view: View): Promise<Component> {
    if (loaded.has(view)) return Promise.resolve(loaded.get(view) as Component);
    const existing = inFlight.get(view);
    if (existing) return existing;

    let imported: Promise<TabViewModule<Component>>;
    try {
      imported = importerFor(view)();
    } catch (error) {
      imported = Promise.reject(error);
    }
    let request: Promise<Component>;
    request = imported
      .then((module) => {
        loaded.set(view, module.default);
        return module.default;
      })
      .finally(() => {
        if (inFlight.get(view) === request) inFlight.delete(view);
      });
    inFlight.set(view, request);
    return request;
  }

  return {
    load,
    retry: load,
    hasLoaded: (view) => loaded.has(view),
  };
}

/** Selects the detached view when present, otherwise the navigation view. */
export function initialTabView(
  navigationView: View,
  detachedView: DetachableTabView | undefined,
): View {
  return detachedView ?? navigationView;
}

/** Starts a new active-view request and invalidates the previous render state. */
export function beginTabViewLoad<Component>(
  current: TabViewLoadState<Component> | null,
  view: View,
): LoadingTabViewState {
  return {
    status: "loading",
    view,
    requestId: (current?.requestId ?? 0) + 1,
  };
}

/** Applies a loaded component only when it belongs to the current request. */
export function resolveTabViewLoad<Component>(
  current: TabViewLoadState<Component>,
  view: View,
  requestId: number,
  component: Component,
): TabViewLoadState<Component> {
  if (current.view !== view || current.requestId !== requestId) return current;
  return { status: "ready", view, requestId, component };
}

/** Applies an import failure only when it belongs to the current request. */
export function rejectTabViewLoad<Component>(
  current: TabViewLoadState<Component>,
  view: View,
  requestId: number,
  error: unknown,
): TabViewLoadState<Component> {
  if (current.view !== view || current.requestId !== requestId) return current;
  return { status: "failed", view, requestId, error };
}
