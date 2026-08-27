import {
  createLazyComponentLoader,
  type LazyComponentImporter,
} from "$lib/lazy-component-loader";
import type {
  LoadedProjectOptionalComponent,
  ProjectOptionalComponentKind,
} from "./project-component-registry-contract";

export type {
  LoadedProjectOptionalComponent,
  ProjectOptionalComponentKind,
} from "./project-component-registry-contract";

type LoadedMobileOptionalComponent = Extract<
  LoadedProjectOptionalComponent,
  { kind: "toolbar" | "task-detail" }
>;

const MOBILE_OPTIONAL_IMPORTERS = {
  toolbar: () => import("./ProjectToolbarPanels.svelte")
    .then((module) => ({ default: { kind: "toolbar" as const, component: module.default } })),
  "task-detail": () => import("./ProjectTaskDetailPanel.svelte")
    .then((module) => ({
      default: { kind: "task-detail" as const, component: module.default },
    })),
} satisfies Readonly<Record<
  "toolbar" | "task-detail",
  LazyComponentImporter<LoadedMobileOptionalComponent>
>>;

const mobileOptionalLoader = createLazyComponentLoader<
  "toolbar" | "task-detail",
  LoadedMobileOptionalComponent
>(MOBILE_OPTIONAL_IMPORTERS);

function unsupportedMobileComponent(
  kind: Exclude<ProjectOptionalComponentKind, "toolbar" | "task-detail">,
): Promise<LoadedProjectOptionalComponent> {
  return Promise.reject(new Error(`Project ${kind} is unavailable in the mobile composition`));
}

/** Load the mobile task detail without emitting desktop-only optional chunks. */
export function loadProjectOptionalComponent(
  kind: ProjectOptionalComponentKind,
): Promise<LoadedProjectOptionalComponent> {
  if (kind === "toolbar" || kind === "task-detail") return mobileOptionalLoader.load(kind);
  return unsupportedMobileComponent(kind);
}

/** Retry the only optional Project surface available in the mobile composition. */
export function retryProjectOptionalComponent(
  kind: ProjectOptionalComponentKind,
): Promise<LoadedProjectOptionalComponent> {
  if (kind === "toolbar" || kind === "task-detail") return mobileOptionalLoader.retry(kind);
  return unsupportedMobileComponent(kind);
}

/** Report whether the mobile task-detail constructor is cached. */
export function projectOptionalComponentHasLoaded(kind: ProjectOptionalComponentKind): boolean {
  return (kind === "toolbar" || kind === "task-detail") && mobileOptionalLoader.hasLoaded(kind);
}
