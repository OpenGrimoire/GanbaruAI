import {
  availableAppViews,
  isViewAvailable,
  parseInitialViewSearch,
  type View,
} from "$lib/navigation";

export type { View } from "$lib/navigation";

function initialView(): View {
  const fallback = availableAppViews()[0] ?? "calendar";
  if (typeof window === "undefined") return fallback;
  return parseInitialViewSearch(window.location.search) ?? fallback;
}

let currentView = $state<View>(initialView());

function setCurrentView(view: View): boolean {
  if (!isViewAvailable(view)) return false;
  currentView = view;
  return true;
}

export function getNavigation() {
  return {
    get current() {
      return currentView;
    },
    set current(view: View) {
      setCurrentView(view);
    },
    navigate(view: View) {
      return setCurrentView(view);
    },
  };
}
