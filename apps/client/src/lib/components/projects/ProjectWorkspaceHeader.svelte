<script lang="ts">
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import CircleGauge from "@lucide/svelte/icons/circle-gauge";
  import Folder from "@lucide/svelte/icons/folder";
  import Funnel from "@lucide/svelte/icons/funnel";
  import List from "@lucide/svelte/icons/list";
  import MoreHorizontal from "@lucide/svelte/icons/more-horizontal";
  import Timer from "@lucide/svelte/icons/timer";
  import X from "@lucide/svelte/icons/x";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectLifecycleBadgeClass,
    projectLifecycleLabel,
  } from "$lib/projects/project-display";
  import {
    projectNavigatorPanelGeometry,
    type ProjectFilterChip,
    type ProjectToolbarPanel,
  } from "$lib/projects/project-toolbar";
  import { PROJECT_VIEW_IDS, type Project, type ProjectGroup, type ProjectViewId } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getViewport } from "$lib/stores/viewport.svelte";
  import { cn } from "$lib/utils";
  import ProjectIcon from "./ProjectIcon.svelte";
  import ProjectNavigator from "./ProjectNavigator.svelte";

  let {
    selectedProject,
    selectedGroup,
    selectedProjectId,
    showInactiveProjects,
    projectToolbarPanel,
    activeFilterChips,
    matchingTaskCount,
    totalTaskCount,
    onShowInactiveProjectsChange,
    onProjectSelected,
    onToggleToolbarPanel,
    onClearFilterChip,
  }: {
    selectedProject: Project;
    selectedGroup: ProjectGroup;
    selectedProjectId: string | null;
    showInactiveProjects: boolean;
    projectToolbarPanel: ProjectToolbarPanel | null;
    activeFilterChips: ProjectFilterChip[];
    matchingTaskCount: number;
    totalTaskCount: number;
    onShowInactiveProjectsChange: (value: boolean) => void;
    onProjectSelected: () => void;
    onToggleToolbarPanel: (panel: ProjectToolbarPanel) => void;
    onClearFilterChip: (chip: ProjectFilterChip) => void;
  } = $props();

  const projects = getProjects();
  const viewport = getViewport();
  const { t } = getLocalization();

  let projectNavigatorOpen = $state(false);
  let projectNavigatorTriggerElement = $state<HTMLButtonElement | null>(null);
  let projectNavigatorPanelElement = $state<HTMLDivElement | null>(null);
  let projectNavigatorPanelStyle = $state("");

  function viewIcon(view: ProjectViewId) {
    if (view === "list") return List;
    if (view === "board") return Folder;
    if (view === "calendar") return CalendarDays;
    if (view === "gantt") return Timer;
    return CircleGauge;
  }

  function viewLabel(view: ProjectViewId): string {
    if (view === "list") return t("projects.tabs.list");
    if (view === "board") return t("projects.tabs.board");
    if (view === "calendar") return t("projects.tabs.calendar");
    if (view === "gantt") return t("projects.tabs.gantt");
    return t("projects.tabs.summary");
  }

  function refreshProjectNavigatorPanelGeometry(): void {
    if (!projectNavigatorOpen || !projectNavigatorTriggerElement) return;
    const rect = projectNavigatorTriggerElement.getBoundingClientRect();
    const geometry = projectNavigatorPanelGeometry({
      anchorLeft: rect.left,
      anchorBottom: rect.bottom,
      viewportWidth: viewport.width,
      viewportHeight: viewport.height,
    });
    projectNavigatorPanelStyle = [
      `left: ${Math.round(geometry.left)}px`,
      `top: ${Math.round(geometry.top)}px`,
      `width: ${Math.round(geometry.width)}px`,
      `height: ${Math.round(geometry.height)}px`,
    ].join("; ");
  }

  function openProjectNavigator(): void {
    projectNavigatorOpen = true;
    refreshProjectNavigatorPanelGeometry();
    requestAnimationFrame(refreshProjectNavigatorPanelGeometry);
  }

  function toggleProjectNavigator(): void {
    if (projectNavigatorOpen) {
      projectNavigatorOpen = false;
      return;
    }
    openProjectNavigator();
  }

  function handleProjectWindowPointerDown(event: PointerEvent): void {
    const target = event.target;
    if (!(target instanceof Node)) return;
    if (
      projectNavigatorOpen
      && !projectNavigatorTriggerElement?.contains(target)
      && !projectNavigatorPanelElement?.contains(target)
    ) {
      projectNavigatorOpen = false;
    }
  }

  $effect(() => {
    if (!projectNavigatorOpen) return;
    const viewportWidth = viewport.width;
    const viewportHeight = viewport.height;
    void viewportWidth;
    void viewportHeight;
    requestAnimationFrame(refreshProjectNavigatorPanelGeometry);
  });
</script>

<svelte:window onpointerdown={handleProjectWindowPointerDown} />

<div
  class="flex shrink-0 items-center gap-1 overflow-x-auto px-3"
  style="height: var(--cal-header-row-h); background-color: var(--cal-header-bg); border-bottom: 1px solid var(--sidebar);"
  onscroll={refreshProjectNavigatorPanelGeometry}
>
  <div class="relative min-w-36 shrink-0 min-[760px]:max-w-md">
    <button
      bind:this={projectNavigatorTriggerElement}
      type="button"
      class={cn(
        "flex h-7 min-w-0 max-w-full items-center gap-1.5 rounded-md px-1.5 text-left text-sm hover:bg-accent hover:text-accent-foreground",
        projectNavigatorOpen && "bg-accent text-accent-foreground",
      )}
      aria-label={t("projects.navigator.open")}
      aria-expanded={projectNavigatorOpen}
      onclick={toggleProjectNavigator}
    >
      <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md bg-accent text-accent-foreground">
        <ProjectIcon name={selectedGroup.icon} size={14} />
      </span>
      <span class="min-w-0 truncate font-semibold text-foreground">{selectedGroup.name}</span>
      <span class="shrink-0 font-semibold text-foreground">/</span>
      <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md bg-accent text-accent-foreground">
        <ProjectIcon name={selectedProject.icon} size={14} />
      </span>
      <span class="min-w-0 truncate font-semibold text-foreground">{selectedProject.name}</span>
      <ChevronDown size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
      {#if selectedProject.status !== "active"}
        <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(selectedProject.status))}>
          {projectLifecycleLabel(selectedProject.status, t)}
        </span>
      {/if}
    </button>
    {#if projectNavigatorOpen}
      <div
        bind:this={projectNavigatorPanelElement}
        class="fixed z-80"
        style={projectNavigatorPanelStyle}
        role="dialog"
        tabindex="-1"
        aria-label={t("projects.navigator.pickerLabel")}
      >
        <ProjectNavigator
          {selectedProjectId}
          {showInactiveProjects}
          presentation="panel"
          onShowInactiveProjectsChange={onShowInactiveProjectsChange}
          onProjectSelected={() => {
            projectNavigatorOpen = false;
            onProjectSelected();
          }}
        />
      </div>
    {/if}
  </div>
  <div class="flex-1"></div>
  <nav class="flex min-w-0 shrink-0 items-center gap-0.5 overflow-x-auto">
    {#each PROJECT_VIEW_IDS as view}
      {@const Icon = viewIcon(view)}
      <button
        type="button"
        class={cn(
          "flex h-7 shrink-0 items-center gap-1 rounded-md px-2 text-xs font-medium transition-colors",
          projects.activeView === view
            ? "bg-card text-card-foreground"
            : "text-muted-foreground hover:bg-accent hover:text-foreground",
        )}
        onclick={() => {
          projects.activeView = view;
        }}
      >
        <Icon size={14} strokeWidth={1.75} />
        <span>{viewLabel(view)}</span>
      </button>
    {/each}
  </nav>
  <button
    type="button"
    class={cn(
      "flex h-7 shrink-0 items-center gap-1.5 rounded-md px-2 text-xs font-medium transition-colors hover:bg-accent hover:text-foreground",
      projectToolbarPanel === "filters" ? "bg-accent text-foreground" : "text-muted-foreground",
    )}
    aria-expanded={projectToolbarPanel === "filters"}
    onclick={() => onToggleToolbarPanel("filters")}
  >
    <Funnel size={13} strokeWidth={1.75} />
    <span>{t("projects.filters.title")}</span>
    {#if activeFilterChips.length > 0}
      <span class="rounded bg-primary/10 px-1 text-[0.666667rem] text-primary">
        {activeFilterChips.length}
      </span>
    {/if}
  </button>
  <button
    type="button"
    class={cn(
      "flex h-7 shrink-0 items-center gap-1.5 rounded-md px-2 text-xs font-medium transition-colors hover:bg-accent hover:text-foreground",
      projectToolbarPanel === "customize" ? "bg-accent text-foreground" : "text-muted-foreground",
    )}
    aria-expanded={projectToolbarPanel === "customize"}
    onclick={() => onToggleToolbarPanel("customize")}
  >
    <CircleGauge size={13} strokeWidth={1.75} />
    <span>{t("projects.toolbar.customize")}</span>
  </button>
  <button
    type="button"
    class={cn(
      "flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground",
      projectToolbarPanel === "more" && "bg-accent text-foreground",
    )}
    aria-label={t("projects.toolbar.more")}
    aria-expanded={projectToolbarPanel === "more"}
    onclick={() => onToggleToolbarPanel("more")}
  >
    <MoreHorizontal size={14} strokeWidth={1.75} />
  </button>
</div>
{#if activeFilterChips.length > 0}
  <div class="flex min-w-0 flex-wrap items-center gap-1 px-3 py-1 text-[0.733333rem]">
    {#each activeFilterChips as chip (chip.id)}
      <button
        type="button"
        class="flex max-w-52 items-center gap-1 rounded-full border border-border bg-card px-2 py-1 text-muted-foreground hover:bg-accent hover:text-foreground"
        title={chip.label}
        onclick={() => onClearFilterChip(chip)}
      >
        <span class="truncate">{chip.label}</span>
        <X size={12} strokeWidth={1.75} />
      </button>
    {/each}
    <span class="rounded-full border border-border bg-muted/60 px-2 py-1 text-muted-foreground">
      {t("projects.filters.matchingTasks", matchingTaskCount, totalTaskCount)}
    </span>
  </div>
{/if}
