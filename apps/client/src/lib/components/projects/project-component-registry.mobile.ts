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

type LoadedTaskDetailComponent = Extract<
  LoadedProjectOptionalComponent,
  { kind: "task-detail" }
>;

const TASK_DETAIL_IMPORTERS = {
  "task-detail": () => import("./ProjectTaskDetailPanel.svelte")
    .then((module) => ({
      default: { kind: "task-detail" as const, component: module.default },
    })),
} satisfies Readonly<Record<
  "task-detail",
  LazyComponentImporter<LoadedTaskDetailComponent>
>>;

const taskDetailLoader = createLazyComponentLoader<
  "task-detail",
  LoadedTaskDetailComponent
>(TASK_DETAIL_IMPORTERS);

function unsupportedMobileComponent(
  kind: Exclude<ProjectOptionalComponentKind, "task-detail">,
): Promise<LoadedProjectOptionalComponent> {
  return Promise.reject(new Error(`Project ${kind} is unavailable in the mobile composition`));
}

/** Load the mobile task detail without emitting desktop-only optional chunks. */
export function loadProjectOptionalComponent(
  kind: ProjectOptionalComponentKind,
): Promise<LoadedProjectOptionalComponent> {
  return kind === "task-detail"
    ? taskDetailLoader.load(kind)
    : unsupportedMobileComponent(kind);
}

/** Retry the only optional Project surface available in the mobile composition. */
export function retryProjectOptionalComponent(
  kind: ProjectOptionalComponentKind,
): Promise<LoadedProjectOptionalComponent> {
  return kind === "task-detail"
    ? taskDetailLoader.retry(kind)
    : unsupportedMobileComponent(kind);
}

/** Report whether the mobile task-detail constructor is cached. */
export function projectOptionalComponentHasLoaded(kind: ProjectOptionalComponentKind): boolean {
  return kind === "task-detail" && taskDetailLoader.hasLoaded(kind);
}
