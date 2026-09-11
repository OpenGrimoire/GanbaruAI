export type ProjectOptionalComponentKind =
  | "toolbar"
  | "bulk-actions"
  | "task-finder"
  | "task-detail";

export type LoadedProjectOptionalComponent =
  | { kind: "toolbar"; component: typeof import("./ProjectToolbarPanels.svelte").default }
  | { kind: "bulk-actions"; component: typeof import("./ProjectBulkActionController.svelte").default }
  | { kind: "task-finder"; component: typeof import("./ProjectTaskFinder.svelte").default }
  | { kind: "task-detail"; component: typeof import("./ProjectTaskDetailPanel.svelte").default };
