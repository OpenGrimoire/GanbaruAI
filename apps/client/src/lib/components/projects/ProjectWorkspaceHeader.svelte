<script lang="ts">
  import ArrowUpDown from "@lucide/svelte/icons/arrow-up-down";
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Columns3 from "@lucide/svelte/icons/columns-3";
  import FileChartColumnIncreasing from "@lucide/svelte/icons/file-chart-column-increasing";
  import ListFilter from "@lucide/svelte/icons/list-filter";
  import Route from "@lucide/svelte/icons/route";
  import ListCollapse from "@lucide/svelte/icons/list-collapse";
  import Settings2 from "@lucide/svelte/icons/settings-2";
  import SquareKanban from "@lucide/svelte/icons/square-kanban";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectLifecycleBadgeClass,
    projectLifecycleLabel,
  } from "$lib/projects/project-display";
  import {
    projectNavigatorPanelGeometry,
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
    taskFiltersActive,
    taskCustomizeActive,
    onShowInactiveProjectsChange,
    onProjectSelected,
    onToggleToolbarPanel,
  }: {
    selectedProject: Project;
    selectedGroup: ProjectGroup;
    selectedProjectId: string | null;
    showInactiveProjects: boolean;
    projectToolbarPanel: ProjectToolbarPanel | null;
    taskFiltersActive: boolean;
    taskCustomizeActive: boolean;
    onShowInactiveProjectsChange: (value: boolean) => void;
    onProjectSelected: () => void;
    onToggleToolbarPanel: (panel: ProjectToolbarPanel) => void;
  } = $props();

  const projects = getProjects();
  const viewport = getViewport();
  const { t } = getLocalization();
  const projectIdentityIconStrokeWidth = 1.5;
  const projectIdentityEmojiScale = 0.94;

  let projectNavigatorOpen = $state(false);
  let projectHeaderElement = $state<HTMLDivElement | null>(null);
  let projectIdentityElement = $state<HTMLDivElement | null>(null);
  let projectNavigatorTriggerElement = $state<HTMLButtonElement | null>(null);
  let projectNavigatorPanelElement = $state<HTMLDivElement | null>(null);
  let expandedViewTabsMeasureElement = $state<HTMLElement | null>(null);
  let toolbarActionsElement = $state<HTMLDivElement | null>(null);
  let viewLabelsCollapsed = $state(false);
  let projectNavigatorPanelStyle = $state("");
  let viewTabDensityFrame: number | null = null;

  function viewIcon(view: ProjectViewId) {
    if (view === "dashboard") return FileChartColumnIncreasing;
    if (view === "list") return ListCollapse;
    if (view === "kanban") return SquareKanban;
    if (view === "calendar") return CalendarDays;
    if (view === "gantt") return Route;
    return ListCollapse;
  }

  function viewLabel(view: ProjectViewId): string {
    if (view === "dashboard") return t("projects.tabs.dashboard");
    if (view === "list") return t("projects.tabs.list");
    if (view === "kanban") return t("projects.tabs.kanban");
    if (view === "calendar") return t("projects.tabs.calendar");
    if (view === "gantt") return t("projects.tabs.gantt");
    return t("projects.tabs.list");
  }

  function viewShortcutTitle(view: ProjectViewId): string {
    const shortcut = String(PROJECT_VIEW_IDS.indexOf(view) + 1);
    return `${viewLabel(view)} (${t("calendar.toolbar.shortcutKey", shortcut)})`;
  }

  function toolbarIconButtonClass(active: boolean, open = false): string {
    return cn(
      "flex h-7 w-7 shrink-0 items-center justify-center rounded-md transition-colors hover:bg-accent",
      active ? "text-foreground" : "text-muted-foreground",
      open && "bg-accent",
    );
  }

  function projectHeaderGap(): number {
    if (!projectHeaderElement) return 0;
    const style = getComputedStyle(projectHeaderElement);
    return Number.parseFloat(style.columnGap || style.gap) || 0;
  }

  function projectHeaderHorizontalPadding(): number {
    if (!projectHeaderElement) return 0;
    const style = getComputedStyle(projectHeaderElement);
    return (Number.parseFloat(style.paddingLeft) || 0) + (Number.parseFloat(style.paddingRight) || 0);
  }

  function syncViewTabDensity(): void {
    viewTabDensityFrame = null;
    if (!projectHeaderElement || !projectIdentityElement || !expandedViewTabsMeasureElement || !toolbarActionsElement) {
      return;
    }

    const requiredExpandedWidth = projectHeaderHorizontalPadding()
      + projectIdentityElement.getBoundingClientRect().width
      + expandedViewTabsMeasureElement.getBoundingClientRect().width
      + toolbarActionsElement.getBoundingClientRect().width
      + projectHeaderGap() * 3;
    viewLabelsCollapsed = requiredExpandedWidth > projectHeaderElement.clientWidth;
  }

  function requestViewTabDensitySync(): void {
    if (viewTabDensityFrame !== null) cancelAnimationFrame(viewTabDensityFrame);
    viewTabDensityFrame = requestAnimationFrame(syncViewTabDensity);
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

  $effect(() => {
    const elements = [
      projectHeaderElement,
      projectIdentityElement,
      expandedViewTabsMeasureElement,
      toolbarActionsElement,
    ];
    if (elements.some((element) => element === null)) return;

    requestViewTabDensitySync();
    if (typeof ResizeObserver === "undefined") return;

    const observer = new ResizeObserver(requestViewTabDensitySync);
    for (const element of elements) {
      if (element) observer.observe(element);
    }
    return () => {
      observer.disconnect();
      if (viewTabDensityFrame !== null) {
        cancelAnimationFrame(viewTabDensityFrame);
        viewTabDensityFrame = null;
      }
    };
  });
</script>

<svelte:window onpointerdown={handleProjectWindowPointerDown} />

<div
  bind:this={projectHeaderElement}
  class="flex shrink-0 items-center gap-1 overflow-x-auto px-3"
  style="height: var(--cal-header-row-h); background-color: var(--cal-header-bg); border-bottom: 1px solid var(--sidebar);"
  onscroll={refreshProjectNavigatorPanelGeometry}
>
  <div bind:this={projectIdentityElement} class="relative min-w-36 shrink-0 min-[760px]:max-w-md">
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
      <ProjectIcon
        name={selectedGroup.icon}
        size={14}
        strokeWidth={projectIdentityIconStrokeWidth}
        ignoreColor
        emojiScale={projectIdentityEmojiScale}
        class="shrink-0"
      />
      <span class="min-w-0 truncate font-semibold text-foreground">{selectedGroup.name}</span>
      <span class="shrink-0 font-semibold text-foreground">/</span>
      <ProjectIcon
        name={selectedProject.icon}
        size={14}
        strokeWidth={projectIdentityIconStrokeWidth}
        ignoreColor
        emojiScale={projectIdentityEmojiScale}
        class="shrink-0"
      />
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
  <nav
    bind:this={expandedViewTabsMeasureElement}
    class="pointer-events-none fixed left-0 top-0 flex h-7 items-center gap-0.5 overflow-visible whitespace-nowrap opacity-0"
    aria-hidden="true"
    inert
  >
    {#each PROJECT_VIEW_IDS as view}
      {@const Icon = viewIcon(view)}
      {@const label = viewLabel(view)}
      <span class="flex h-7 shrink-0 items-center gap-1 rounded-md px-2 text-xs font-medium">
        <Icon size={14} strokeWidth={1.75} class={view === "gantt" ? "-scale-x-100" : undefined} />
        <span>{label}</span>
      </span>
    {/each}
  </nav>
  <nav class="flex min-w-0 shrink-0 items-center gap-0.5 overflow-x-auto">
    {#each PROJECT_VIEW_IDS as view}
      {@const Icon = viewIcon(view)}
      {@const label = viewLabel(view)}
      {@const title = viewShortcutTitle(view)}
      <button
        type="button"
        class={cn(
          "flex h-7 shrink-0 items-center gap-1 rounded-md text-xs font-medium transition-colors hover:bg-accent",
          viewLabelsCollapsed ? "w-7 justify-center px-0" : "px-2",
          projects.activeView === view
            ? "text-foreground"
            : "text-muted-foreground",
        )}
        aria-label={label}
        title={title}
        onclick={() => {
          projects.activeView = view;
        }}
      >
        <Icon size={14} strokeWidth={1.75} class={view === "gantt" ? "-scale-x-100" : undefined} />
        {#if !viewLabelsCollapsed}
          <span>{label}</span>
        {/if}
      </button>
    {/each}
  </nav>
  <div bind:this={toolbarActionsElement} class="flex shrink-0 items-center gap-1">
    <button
      type="button"
      data-project-toolbar-trigger="filters"
      class={toolbarIconButtonClass(taskFiltersActive, projectToolbarPanel === "filters")}
      aria-label={t("projects.filters.title")}
      title={t("projects.filters.title")}
      aria-expanded={projectToolbarPanel === "filters"}
      onclick={() => onToggleToolbarPanel("filters")}
    >
      <ListFilter size={14} strokeWidth={1.75} />
    </button>
    <button
      type="button"
      data-project-toolbar-trigger="sort"
      class={toolbarIconButtonClass(false)}
      aria-label={t("projects.toolbar.sort")}
      title={t("projects.toolbar.sort")}
    >
      <ArrowUpDown size={14} strokeWidth={1.75} />
    </button>
    <button
      type="button"
      data-project-toolbar-trigger="customize"
      class={toolbarIconButtonClass(taskCustomizeActive, projectToolbarPanel === "customize")}
      aria-label={t("projects.toolbar.customize")}
      title={t("projects.toolbar.customize")}
      aria-expanded={projectToolbarPanel === "customize"}
      onclick={() => onToggleToolbarPanel("customize")}
    >
      <Columns3 size={14} strokeWidth={1.75} />
    </button>
    <button
      type="button"
      data-project-toolbar-trigger="settings"
      class={toolbarIconButtonClass(false, projectToolbarPanel === "settings")}
      aria-label={t("projects.header.projectSettings")}
      title={t("projects.header.projectSettings")}
      aria-expanded={projectToolbarPanel === "settings"}
      onclick={() => onToggleToolbarPanel("settings")}
    >
      <Settings2 size={14} strokeWidth={1.75} />
    </button>
  </div>
</div>
