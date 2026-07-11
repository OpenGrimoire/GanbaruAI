<script lang="ts">
  import {
    beginLazyComponentLoad,
    rejectLazyComponentLoad,
    resolveLazyComponentLoad,
    type LazyComponentLoadState,
  } from "$lib/lazy-component-loader";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { ProjectNavigatorPanelMode } from "$lib/projects/project-toolbar";
  import {
    loadProjectOptionalComponent,
    retryProjectOptionalComponent,
    type LoadedProjectOptionalComponent,
  } from "./project-component-registry";

  let {
    selectedProjectId,
    selectedGroupId = null,
    showInactiveProjects,
    onShowInactiveProjectsChange,
    onProjectSelected,
    panelMode = "groups",
    panelMaxHeight = null,
  }: {
    selectedProjectId: string | null;
    selectedGroupId?: string | null;
    showInactiveProjects: boolean;
    onShowInactiveProjectsChange: (value: boolean) => void;
    onProjectSelected: () => void;
    panelMode?: ProjectNavigatorPanelMode;
    panelMaxHeight?: number | null;
  } = $props();

  const { t } = getLocalization();
  let loadState = $state<LazyComponentLoadState<
    "project-navigator",
    LoadedProjectOptionalComponent
  > | null>(null);

  function requestNavigator(retry = false): void {
    if (!retry && loadState) return;
    const loadingState = beginLazyComponentLoad(loadState, "project-navigator");
    loadState = loadingState;
    const request = retry
      ? retryProjectOptionalComponent("project-navigator")
      : loadProjectOptionalComponent("project-navigator");
    void request
      .then((component) => {
        if (!loadState) return;
        loadState = resolveLazyComponentLoad(
          loadState,
          "project-navigator",
          loadingState.requestId,
          component,
        );
      })
      .catch((error: unknown) => {
        if (!loadState) return;
        loadState = rejectLazyComponentLoad(
          loadState,
          "project-navigator",
          loadingState.requestId,
          error,
        );
        console.error("Failed to load Project navigator:", error);
      });
  }

  $effect(() => requestNavigator());
</script>

{#if loadState?.status === "ready" && loadState.component.kind === "project-navigator"}
  {@const ProjectNavigator = loadState.component.component}
  <ProjectNavigator
    {selectedProjectId}
    {selectedGroupId}
    {showInactiveProjects}
    {onShowInactiveProjectsChange}
    {onProjectSelected}
    {panelMode}
    {panelMaxHeight}
  />
{:else if loadState?.status === "failed"}
  <div class="flex min-h-24 flex-col items-center justify-center gap-2 rounded-md border border-border bg-card p-3 text-center text-xs text-muted-foreground" role="alert">
    <span>{t("common.viewLoadFailed", t("projects.navigator.pickerLabel"))}</span>
    <button
      type="button"
      class="min-h-8 rounded-md border border-border bg-background px-3 font-medium text-foreground hover:bg-accent"
      onclick={() => requestNavigator(true)}
    >
      {t("common.retry")}
    </button>
  </div>
{:else}
  <div class="flex min-h-24 items-center justify-center rounded-md border border-border bg-card p-3 text-xs text-muted-foreground" aria-busy="true">
    {t("common.loading")}
  </div>
{/if}
