import type { DetachableTabView, View } from "$lib/navigation";
import {
  beginLazyComponentLoad,
  createLazyComponentLoader,
  rejectLazyComponentLoad,
  resolveLazyComponentLoad,
  type LazyComponentImporter,
  type LazyComponentLoader,
  type LazyComponentLoadState,
  type LoadingLazyComponentState,
} from "$lib/lazy-component-loader";

export type TabViewImporter<Component> = LazyComponentImporter<Component>;
export type TabViewLoader<Component> = LazyComponentLoader<View, Component>;
export type TabViewLoadState<Component> = LazyComponentLoadState<View, Component>;
export type LoadingTabViewState = LoadingLazyComponentState<View>;

/** Creates the keyed loader for detachable views and the global Music view. */
export function createTabViewLoader<Component>(
  detachableImporters: Readonly<Record<DetachableTabView, TabViewImporter<Component>>>,
  musicImporter: TabViewImporter<Component>,
): TabViewLoader<Component> {
  return createLazyComponentLoader<View, Component>({
    ...detachableImporters,
    music: musicImporter,
  });
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
  return beginLazyComponentLoad(current, view);
}

/** Applies a loaded view only when it belongs to the current request. */
export function resolveTabViewLoad<Component>(
  current: TabViewLoadState<Component>,
  view: View,
  requestId: number,
  component: Component,
): TabViewLoadState<Component> {
  return resolveLazyComponentLoad(current, view, requestId, component);
}

/** Applies a view import failure only when it belongs to the current request. */
export function rejectTabViewLoad<Component>(
  current: TabViewLoadState<Component>,
  view: View,
  requestId: number,
  error: unknown,
): TabViewLoadState<Component> {
  return rejectLazyComponentLoad(current, view, requestId, error);
}
