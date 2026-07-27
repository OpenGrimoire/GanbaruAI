<script lang="ts">
  import ArrowUpDown from "@lucide/svelte/icons/arrow-up-down";
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import Columns3 from "@lucide/svelte/icons/columns-3";
  import FileChartColumnIncreasing from "@lucide/svelte/icons/file-chart-column-increasing";
  import Layers from "@lucide/svelte/icons/layers";
  import ListFilter from "@lucide/svelte/icons/list-filter";
  import Route from "@lucide/svelte/icons/route";
  import ListCollapse from "@lucide/svelte/icons/list-collapse";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import Settings2 from "@lucide/svelte/icons/settings-2";
  import SquareKanban from "@lucide/svelte/icons/square-kanban";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    COMPACT_IDENTITY_EMOJI_SCALE,
    COMPACT_IDENTITY_ICON_SIZE,
    COMPACT_IDENTITY_ICON_STROKE_WIDTH,
  } from "$lib/icon-sizing";
  import {
    projectLifecycleBadgeClass,
    projectLifecycleLabel,
  } from "$lib/projects/project-display";
  import {
    projectNavigatorPanelGeometry,
    type ProjectToolbarPanel,
    type ProjectNavigatorPanelMode,
  } from "$lib/projects/project-toolbar";
  import { PROJECT_VIEW_IDS, type Project, type ProjectGroup, type ProjectViewId } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getNavigation } from "$lib/stores/navigation.svelte";
  import { getViewport } from "$lib/stores/viewport.svelte";
  import { cn } from "$lib/utils";
  import ProjectIcon from "./ProjectIcon.svelte";
  import WorkspaceBreadcrumbTerminalIcon from "$lib/components/WorkspaceBreadcrumbTerminalIcon.svelte";
  import ProjectNavigator from "./ProjectNavigator.svelte";

  let {
    selectedProject,
    selectedGroup,
    selectedProjectId,
    showInactiveProjects,
    projectToolbarPanel,
    taskGroupingActive,
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
    taskGroupingActive: boolean;
    taskFiltersActive: boolean;
    taskCustomizeActive: boolean;
    onShowInactiveProjectsChange: (value: boolean) => void;
    onProjectSelected: () => void;
    onToggleToolbarPanel: (panel: ProjectToolbarPanel) => void;
  } = $props();

  const projects = getProjects();
  const chat = getChat();
  const navigation = getNavigation();
  const viewport = getViewport();
  const { t } = getLocalization();
  const projectIdentityIconSize = COMPACT_IDENTITY_ICON_SIZE;
  const projectIdentityIconStrokeWidth = COMPACT_IDENTITY_ICON_STROKE_WIDTH;
  const projectIdentityEmojiScale = COMPACT_IDENTITY_EMOJI_SCALE;

  let projectNavigatorOpen = $state(false);
  let projectNavigatorMode = $state<ProjectNavigatorPanelMode>("groups");
  let projectHeaderElement = $state<HTMLDivElement | null>(null);
  let projectIdentityElement = $state<HTMLDivElement | null>(null);
  let projectNavigatorAnchorElement = $state<HTMLButtonElement | null>(null);
  let projectGroupTriggerElement = $state<HTMLButtonElement | null>(null);
  let projectProjectTriggerElement = $state<HTMLButtonElement | null>(null);
  let projectNavigatorPanelElement = $state<HTMLDivElement | null>(null);
  let expandedViewTabsMeasureElement = $state<HTMLElement | null>(null);
  let toolbarActionsElement = $state<HTMLDivElement | null>(null);
  let viewLabelsCollapsed = $state(false);
  let projectNavigatorPanelStyle = $state("");
  let projectNavigatorPanelMaxHeight = $state(0);
  let viewTabDensityFrame: number | null = null;
  const projectWorkingFolders = $derived(chat.workingFolders.filter((entry) => (
    entry.workingFolder.projectId === selectedProject.id
      && entry.workingFolder.archivedAt === null
  )));

  $effect(() => {
    void chat.ensureLoaded().catch((error) => {
      console.error("load project working folders failed", error);
    });
  });

  async function openProjectChat(workingFolderId?: string): Promise<void> {
    await chat.ensureLoaded();
    if (workingFolderId) chat.selectWorkingFolder(workingFolderId);
    else await chat.syncProjectSelection(selectedProject.id);
    navigation.navigate("chat");
  }

  interface ProjectNavigatorBounds {
    left: number;
    right: number;
    top: number;
    bottom: number;
  }

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

  function projectNavigatorBounds(): ProjectNavigatorBounds {
    const boundsElement = projectHeaderElement?.closest(".projects-view-root");
    const rect = boundsElement?.getBoundingClientRect();
    if (rect) {
      return {
        left: rect.left,
        right: rect.right,
        top: rect.top,
        bottom: rect.bottom,
      };
    }

    return {
      left: 0,
      right: viewport.width,
      top: 0,
      bottom: viewport.height,
    };
  }

  function refreshProjectNavigatorPanelGeometry(): void {
    if (!projectNavigatorOpen || !projectNavigatorAnchorElement) return;
    const rect = projectNavigatorAnchorElement.getBoundingClientRect();
    const bounds = projectNavigatorBounds();
    const geometry = projectNavigatorPanelGeometry({
      anchorLeft: rect.left,
      anchorBottom: rect.bottom,
      viewportWidth: viewport.width,
      viewportHeight: viewport.height,
      boundsLeft: bounds.left,
      boundsRight: bounds.right,
      boundsTop: bounds.top,
      boundsBottom: bounds.bottom,
    });
    projectNavigatorPanelStyle = [
      `left: ${Math.round(geometry.left)}px`,
      `top: ${Math.round(geometry.top)}px`,
      `width: ${Math.round(geometry.width)}px`,
    ].join("; ");
    projectNavigatorPanelMaxHeight = geometry.height;
  }

  function openProjectNavigator(mode: ProjectNavigatorPanelMode): void {
    projectNavigatorMode = mode;
    projectNavigatorAnchorElement = mode === "groups"
      ? projectGroupTriggerElement
      : projectProjectTriggerElement;
    projectNavigatorOpen = true;
    refreshProjectNavigatorPanelGeometry();
    requestAnimationFrame(refreshProjectNavigatorPanelGeometry);
  }

  function toggleProjectNavigator(mode: ProjectNavigatorPanelMode): void {
    if (projectNavigatorOpen && projectNavigatorMode === mode) {
      projectNavigatorOpen = false;
      return;
    }
    openProjectNavigator(mode);
  }

  function handleProjectWindowPointerDown(event: PointerEvent): void {
    const target = event.target;
    if (!(target instanceof Node)) return;
    if (
      projectNavigatorOpen
      && !projectGroupTriggerElement?.contains(target)
      && !projectProjectTriggerElement?.contains(target)
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
    <div class="flex h-7 min-w-0 max-w-full items-center gap-0.5 text-sm">
      <button
        bind:this={projectGroupTriggerElement}
        type="button"
        class={cn(
          "flex h-7 min-w-0 items-center gap-1.5 rounded-md px-1.5 text-left hover:bg-accent hover:text-accent-foreground",
          projectNavigatorOpen && projectNavigatorMode === "groups" && "bg-accent text-accent-foreground",
        )}
        aria-label={t("projects.navigator.open")}
        aria-expanded={projectNavigatorOpen && projectNavigatorMode === "groups"}
        onpointerenter={() => openProjectNavigator("groups")}
        onclick={() => toggleProjectNavigator("groups")}
      >
        <ProjectIcon
          name={selectedGroup.icon}
          size={projectIdentityIconSize}
          strokeWidth={projectIdentityIconStrokeWidth}
          emojiScale={projectIdentityEmojiScale}
          class="shrink-0"
        />
        <span class="min-w-0 truncate font-semibold text-foreground">{selectedGroup.name}</span>
      </button>
      <span class="shrink-0 px-0.5 font-semibold text-muted-foreground">/</span>
      <button
        bind:this={projectProjectTriggerElement}
        type="button"
        class={cn(
          "flex h-7 min-w-0 items-center gap-1.5 rounded-md px-1.5 text-left hover:bg-accent hover:text-accent-foreground",
          projectNavigatorOpen && projectNavigatorMode === "projects" && "bg-accent text-accent-foreground",
        )}
        aria-label={t("projects.navigator.open")}
        aria-expanded={projectNavigatorOpen && projectNavigatorMode === "projects"}
        onpointerenter={() => openProjectNavigator("projects")}
        onclick={() => toggleProjectNavigator("projects")}
      >
        <ProjectIcon
          name={selectedProject.icon}
          size={projectIdentityIconSize}
          strokeWidth={projectIdentityIconStrokeWidth}
          emojiScale={projectIdentityEmojiScale}
          class="shrink-0"
        />
        <span class="min-w-0 truncate font-semibold text-foreground">{selectedProject.name}</span>
        <WorkspaceBreadcrumbTerminalIcon kind="chevron" class="shrink-0 text-muted-foreground" />
        {#if selectedProject.status !== "active"}
          <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(selectedProject.status))}>
            {projectLifecycleLabel(selectedProject.status, t)}
          </span>
        {/if}
      </button>
    </div>
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
          selectedGroupId={selectedGroup.id}
          {showInactiveProjects}
          panelMode={projectNavigatorMode}
          panelMaxHeight={projectNavigatorPanelMaxHeight}
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
      class={toolbarIconButtonClass(false)}
      aria-label={t("projects.header.openChat")}
      title={t("projects.header.openChat")}
      onclick={() => { void openProjectChat(); }}
    >
      <MessageSquare size={14} strokeWidth={1.75} />
    </button>
    {#if projectWorkingFolders.length > 1}
      <select
        class="h-7 max-w-28 rounded-md border border-border bg-background px-1 text-[0.68rem] text-muted-foreground"
        aria-label={t("projects.header.chatFolder")}
        value=""
        onchange={(event) => {
          const workingFolderId = event.currentTarget.value;
          event.currentTarget.value = "";
          if (workingFolderId) void openProjectChat(workingFolderId);
        }}
      >
        <option value="">{t("projects.header.chatFolder")}</option>
        {#each projectWorkingFolders as folder (folder.workingFolder.id)}
          <option value={folder.workingFolder.id}>{folder.workingFolder.displayName}</option>
        {/each}
      </select>
    {/if}
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
      data-project-toolbar-trigger="group"
      class={toolbarIconButtonClass(taskGroupingActive, projectToolbarPanel === "group")}
      aria-label={t("projects.toolbar.group")}
      title={t("projects.toolbar.group")}
      aria-expanded={projectToolbarPanel === "group"}
      onclick={() => onToggleToolbarPanel("group")}
    >
      <Layers size={14} strokeWidth={1.75} />
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
