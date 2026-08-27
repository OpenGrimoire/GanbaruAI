<script lang="ts">
  import { onMount } from "svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getMobileBackStack } from "$lib/stores/mobile-back-stack.svelte";
  import { getViewport } from "$lib/stores/viewport.svelte";
  import type { CalendarEvent, CalendarViewMode } from "$lib/components/calendar/types";
  import type {
    ProjectChatIntegration,
    ProjectSection,
    ProjectStatus,
    ProjectTask,
    ProjectViewId,
  } from "$lib/projects/types";
  import type { LoadFailure } from "$lib/module-load-recovery";
  import {
    projectCalendarCreateDefaults as buildProjectCalendarCreateDefaults,
    projectEventDurationMinutesInDateRange,
  } from "$lib/projects/project-scheduling";
  import {
    pickProjectTaskModalLayout,
  } from "$lib/projects/project-toolbar";
  import ProjectEmptyState from "./ProjectEmptyState.svelte";
  import ProjectWorkspaceHeader from "./ProjectWorkspaceHeader.svelte";
  import type { ProjectDesktopViewComponents } from "./project-desktop-view-components";
  import { ProjectTaskQueryController } from "./project-task-query-controller.svelte";
  import { ProjectRouteLoadController } from "./project-route-load-controller.svelte";
  import { ProjectRouteUiController } from "./project-route-ui-controller.svelte";

  type ProjectMobileListComponent = typeof import("./ProjectMobileListView.svelte").default;
  interface MobileViewLoadRecovery {
    classify(error: unknown): LoadFailure;
    recover(failure: LoadFailure, retry: () => void): void;
  }

  let {
    mobileLayout = false,
    mobileListComponent: MobileListView = null,
    mobileViewLoadRecovery = null,
    desktopViewComponents = null,
    projectChat = null,
  }: {
    mobileLayout?: boolean;
    mobileListComponent?: ProjectMobileListComponent | null;
    mobileViewLoadRecovery?: MobileViewLoadRecovery | null;
    desktopViewComponents?: ProjectDesktopViewComponents | null;
    projectChat?: ProjectChatIntegration | null;
  } = $props();

  const projects = getProjects();
  const mobileBackStack = getMobileBackStack();
  const calendar = getCalendar();
  const preferences = getPreferences();
  const viewport = getViewport();
  const { t } = getLocalization();
  let showInactiveProjects = $state(false);
  let projectCalendarViewMode = $state<CalendarViewMode>("week");
  let projectCalendarViewModeInitialized = false;
  let MobileDashboardView = $state<typeof import("./ProjectDashboardView.svelte").default | null>(null);
  let MobileKanbanView = $state<typeof import("./ProjectKanbanView.svelte").default | null>(null);
  let MobileProjectCalendarView = $state<typeof import("$lib/components/calendar/CalendarView.svelte").default | null>(null);
  let MobileGanttView = $state<typeof import("./ProjectGanttView.svelte").default | null>(null);
  let mobileViewLoadError = $state<{ view: ProjectViewId; failure: LoadFailure } | null>(null);
  const mobileViewLoads = new Map<ProjectViewId, Promise<void>>();

  $effect(() => {
    if (projectCalendarViewModeInitialized) return;
    projectCalendarViewModeInitialized = true;
    projectCalendarViewMode = mobileLayout ? "day" : "week";
  });
  const taskQuery = new ProjectTaskQueryController({ projects, calendar, translate: t });
  const routeLoad = new ProjectRouteLoadController(projects);
  const routeUi = new ProjectRouteUiController({
    setActiveView: (view) => { projects.activeView = view; },
  });

  function mobileViewLoaded(view: ProjectViewId): boolean {
    if (view === "dashboard") return MobileDashboardView !== null;
    if (view === "kanban") return MobileKanbanView !== null;
    if (view === "calendar") return MobileProjectCalendarView !== null;
    if (view === "gantt") return MobileGanttView !== null;
    return true;
  }

  function loadMobileView(view: ProjectViewId): Promise<void> {
    if (view === "list" || mobileViewLoaded(view)) return Promise.resolve();
    const existing = mobileViewLoads.get(view);
    if (existing) return existing;
    mobileViewLoadError = null;
    const request = (async () => {
      if (view === "dashboard") {
        MobileDashboardView = (await import("./ProjectDashboardView.svelte")).default;
      } else if (view === "kanban") {
        MobileKanbanView = (await import("./ProjectKanbanView.svelte")).default;
      } else if (view === "calendar") {
        MobileProjectCalendarView = (await import("$lib/components/calendar/CalendarView.svelte")).default;
      } else if (view === "gantt") {
        MobileGanttView = (await import("./ProjectGanttView.svelte")).default;
      }
    })().catch((error: unknown) => {
      const failure = mobileViewLoadRecovery?.classify(error);
      if (failure) mobileViewLoadError = { view, failure };
      throw error;
    }).finally(() => {
      mobileViewLoads.delete(view);
    });
    mobileViewLoads.set(view, request);
    return request;
  }

  $effect(() => {
    if (!mobileLayout) return;
    const view = projects.activeView;
    if (view === "list" || mobileViewLoaded(view)) return;
    void loadMobileView(view).catch((error: unknown) => {
      console.error(`load mobile Project ${view} view failed`, error);
    });
  });

  const toolbarLoadState = $derived(routeLoad.optionalState("toolbar"));
  const bulkActionsLoadState = $derived(routeLoad.optionalState("bulk-actions"));
  const taskFinderLoadState = $derived(routeLoad.optionalState("task-finder"));
  const taskDetailLoadState = $derived(routeLoad.optionalState("task-detail"));

  const selectedProject = $derived(projects.selectedProject);
  const selectedGroup = $derived(projects.selectedGroup);
  const selectedProjectId = $derived(selectedProject?.id ?? null);
  const toolbarDataReady = $derived.by(() => {
    const projectId = selectedProjectId;
    return Boolean(
      projectId
      && projects.taskViewPage?.projectId === projectId
      && projects.projectOptionalDataLoaded("saved_views", projectId),
    );
  });
  const taskDetailDataReady = $derived.by(() => {
    return Boolean(
      routeUi.selectedTaskId
      && projects.taskById(routeUi.selectedTaskId)?.detailLoaded,
    );
  });

  $effect(() => {
    if (!selectedProjectId || projects.projectDataLoaded(selectedProjectId)) return;
    void projects.ensureProjectData(selectedProjectId).catch((error) => {
      console.error("load selected project data failed", error);
    });
  });
  const inactiveSectionCount = $derived(taskQuery.inactiveSectionCount);
  const sections = $derived(taskQuery.sections);
  const statuses = $derived(taskQuery.statuses);
  const priorities = $derived(taskQuery.priorities);
  const archivedProjectTaskCount = $derived(taskQuery.archivedTaskCount);
  const allProjectTasks = $derived(taskQuery.allTasks);
  const projectTags = $derived(taskQuery.projectTags);
  const projectCustomFields = $derived(taskQuery.customFields);
  const todayDate = $derived(taskQuery.today);
  const scheduledTaskIds = $derived(taskQuery.scheduledTaskIds);
  const allProjectEvents = $derived(taskQuery.allEvents);

  function projectCalendarCreateDefaults(input: {
    start: string;
    end: string;
    allDay?: boolean;
  }): Partial<CalendarEvent> {
    return buildProjectCalendarCreateDefaults({
      project: selectedProject,
      ...input,
      globalIdleDefaults: {
        idlePauseEnabled: preferences.focusIdlePauseOnEventCreate,
        idleThresholdMinutes: preferences.focusIdleThresholdMinutes,
      },
    });
  }
  const tasks = $derived(taskQuery.tasks);
  const listTaskGroups = $derived(taskQuery.listGroups);
  const selectableTasks = $derived.by(() => tasks.filter((task) => !task.parentTaskId));
  const selectedTaskIdSet = $derived.by(() => new Set(routeUi.selectedTaskIds));
  const selectedTasks = $derived.by(() => allProjectTasks.filter((task) => selectedTaskIdSet.has(task.id)));
  const selectedArchivedTaskCount = $derived(selectedTasks.filter((task) => Boolean(task.archivedAt)).length);
  const selectedActiveTaskCount = $derived(selectedTasks.length - selectedArchivedTaskCount);
  const matchingTaskCount = $derived(taskQuery.matchingTaskCount);
  const taskFiltersActive = $derived(taskQuery.filtersActive);
  const taskFilterControlsActive = $derived(taskQuery.filterControlsActive);
  const taskGroupingActive = $derived(taskQuery.groupingActive);
  const taskCustomizeActive = $derived(taskQuery.customizeActive);
  const savedTaskViews = $derived(taskQuery.savedViews);
  const scheduledThisWeekMinutes = $derived.by(() => thisWeekScheduledMinutes());
  const selectedTask = $derived.by(() =>
    routeUi.selectedTaskId ? allProjectTasks.find((task) => task.id === routeUi.selectedTaskId) : undefined
  );
  const taskListColumnControls = $derived(taskQuery.columnControls);
  const taskDetailModalLayout = $derived(pickProjectTaskModalLayout({
    viewportWidth: viewport.width,
    viewportHeight: viewport.height,
  }));

  const taskDetailComponentReady = $derived(
    taskDetailDataReady
      && taskDetailLoadState?.status === "ready"
      && taskDetailLoadState.component.kind === "task-detail",
  );

  $effect(() => {
    if (!mobileLayout || routeUi.selectedTaskId === null || taskDetailComponentReady) return;
    return mobileBackStack.activate({
      handle: () => {
        routeUi.selectedTaskId = null;
        routeLoad.invalidateTaskDetailData();
      },
    });
  });

  $effect(() => {
    if (!mobileLayout || !routeUi.taskFinderOpen) return;
    return mobileBackStack.activate({
      handle: () => {
        routeUi.closeFinder();
      },
    });
  });

  $effect(() => {
    if (!mobileLayout || routeUi.toolbarPanel === null) return;
    return mobileBackStack.activate({
      handle: () => {
        routeUi.requestToolbarClose();
      },
    });
  });

  onMount(() => {
    void projects.ensureLoaded().catch((error) => {
      console.error("load projects failed", error);
    });
  });

  $effect(() => {
    const view = projects.activeView;
    if (!selectedProject || !selectedGroup) return;
    void projects.ensureProjectViewData(selectedProject.id, view).catch((error) => {
      console.error(`load optional Project ${view} data failed`, error);
    });
  });

  $effect(() => {
    projects.activeView;
    taskQuery.filterState;
    taskQuery.sections;
    taskQuery.allEvents;
    taskQuery.loadCurrent(routeUi.selectedTaskIds, routeUi.selectedTaskId);
  });

  $effect(() => {
    if (!routeUi.toolbarPanel) return;
    routeLoad.requestOptional("toolbar");
    const projectId = selectedProjectId;
    void routeLoad.requestToolbarData(projectId, () => (
      routeUi.toolbarPanel !== null && selectedProjectId === projectId
    ));
  });

  $effect(() => {
    if (!mobileLayout && routeUi.selectedTaskIds.length > 0) {
      routeLoad.requestOptional("bulk-actions");
    }
  });

  $effect(() => {
    if (!mobileLayout && (routeUi.taskFinderOpen || taskQuery.search.trim())) {
      routeLoad.requestOptional("task-finder");
    }
  });

  $effect(() => {
    if (!routeUi.selectedTaskId) return;
    routeLoad.requestOptional("task-detail");
    void routeLoad.requestTaskDetailData(selectedProjectId, routeUi.selectedTaskId);
  });

  $effect(() => {
    selectedProjectId;
    projects.viewPreferences;
    projectCustomFields;
    taskQuery.syncListPreferences();
  });

  $effect(() => {
    if (routeUi.selectedTaskId && !selectedTask) {
      routeUi.selectedTaskId = null;
      routeLoad.invalidateTaskDetailData();
    }
  });

  $effect(() => {
    sections;
    projectTags;
    projectCustomFields;
    taskQuery.repairDisappearingFields();
  });

  function terminalStatus(): ProjectStatus | undefined {
    return statuses.find((status) => status.terminal);
  }

  function firstOpenStatus(): ProjectStatus | undefined {
    return statuses.find((status) => status.name.toLowerCase() === "to do")
      ?? statuses.find((status) => !status.terminal);
  }

  function clearTaskFilters(): void {
    taskQuery.reset();
  }

  function revealCreatedTask(task: ProjectTask | undefined): void {
    if (!task) return;
    projects.activeView = "list";
    taskQuery.reset();
    routeUi.selectedTaskId = task.id;
    routeUi.clearTaskSelection();
    taskQuery.showArchivedTasks = false;
    taskQuery.showInactiveSections = false;
  }


  async function toggleSectionCollapsed(section: ProjectSection): Promise<void> {
    await projects.updateSection(section, { collapsed: !section.collapsed });
  }

  function thisWeekScheduledMinutes(): number {
    return projectEventDurationMinutesInDateRange(allProjectEvents, todayDate, taskQuery.weekEnd);
  }

</script>

<svelte:window onkeydown={(event) => routeUi.handleWindowKeydown(event)} />

<svelte:document
  onselectstart={(event) => routeUi.handleDocumentSelectStart(event)}
  onselectionchange={() => routeUi.handleDocumentSelectionChange()}
/>

<div
  bind:this={routeUi.rootElement}
  class="projects-view-root relative flex h-full min-h-0 overflow-hidden text-foreground"
  style="background-color: var(--cal-bg);"
  data-first-use-shell="projects"
>
  <section class="flex min-w-0 flex-1 flex-col">
    {#if selectedProject && selectedGroup}
      <header class="flex shrink-0 flex-col" style="background-color: var(--cal-header-bg);">
        <ProjectWorkspaceHeader
          {selectedProject}
          {selectedGroup}
          {selectedProjectId}
          {showInactiveProjects}
          projectToolbarPanel={routeUi.toolbarPanel}
          {taskGroupingActive}
          taskFiltersActive={taskFilterControlsActive}
          {taskCustomizeActive}
          {mobileLayout}
          {projectChat}
          onShowInactiveProjectsChange={(value) => {
            showInactiveProjects = value;
          }}
          onProjectSelected={() => {
            routeUi.selectedTaskId = null;
            routeUi.closeToolbarImmediately();
          }}
          onToggleToolbarPanel={(panel) => routeUi.toggleToolbarPanel(panel)}
        />
        {#if routeUi.toolbarPanel}
          {#if toolbarDataReady && toolbarLoadState?.status === "ready" && toolbarLoadState.component.kind === "toolbar"}
            {@const ProjectToolbarPanels = toolbarLoadState.component.component}
            <ProjectToolbarPanels
              {mobileLayout}
              panel={routeUi.toolbarPanel}
              projectId={selectedProjectId}
              {sections}
              {priorities}
              {projectTags}
              {projectCustomFields}
              {savedTaskViews}
              {taskListColumnControls}
              {archivedProjectTaskCount}
              {inactiveSectionCount}
              {taskFiltersActive}
              savedViewSaving={taskQuery.savedViewSaving}
              savedViewError={taskQuery.savedViewError}
              bind:taskStatusFilter={taskQuery.statusFilter}
              bind:taskSectionFilter={taskQuery.sectionFilter}
              bind:taskPriorityFilter={taskQuery.priorityFilter}
              bind:taskDueFilter={taskQuery.dueFilter}
              bind:taskDueRangeStart={taskQuery.dueRangeStart}
              bind:taskDueRangeEnd={taskQuery.dueRangeEnd}
              bind:taskScheduleFilter={taskQuery.scheduleFilter}
              bind:taskDependencyFilter={taskQuery.dependencyFilter}
              bind:taskTagFilter={taskQuery.tagFilter}
              bind:taskCustomFieldFilters={taskQuery.customFieldFilters}
              bind:taskGroupBy={taskQuery.groupBy}
              bind:taskSortMode={taskQuery.sortMode}
              bind:taskSortDirection={taskQuery.sortDirection}
              bind:showArchivedTasks={taskQuery.showArchivedTasks}
              bind:showInactiveSections={taskQuery.showInactiveSections}
              bind:savedViewNameDraft={taskQuery.savedViewNameDraft}
              onClose={() => {
                routeUi.requestToolbarClose();
              }}
              onProjectSettingsDirtyChange={(dirty) => {
                routeUi.settingsDirty = dirty;
              }}
              onRevealInactive={() => {
                showInactiveProjects = true;
              }}
              onClearTaskFilters={clearTaskFilters}
              onSaveCurrentTaskView={() => { void taskQuery.saveCurrentView(); }}
              onApplyTaskView={(view) => {
                void taskQuery.applySavedView(view).finally(() => routeUi.clearTaskSelection());
              }}
              onDeleteSavedTaskView={(view) => { void taskQuery.deleteSavedView(view); }}
              onToggleTaskListColumn={(column) => { void taskQuery.toggleColumn(column); }}
            />
          {:else if routeLoad.toolbarDataError || toolbarLoadState?.status === "failed"}
            <div class="flex min-h-9 items-center justify-center gap-2 border-t border-border px-3 text-xs text-muted-foreground" role="alert">
              <span>{t("common.viewLoadFailed", t("projects.header.projectSettings"))}</span>
              <button
                type="button"
                class="font-medium text-foreground underline-offset-2 hover:underline"
                onclick={() => {
                  if (routeLoad.toolbarDataError) {
                    const projectId = selectedProjectId;
                    void routeLoad.requestToolbarData(projectId, () => (
                      routeUi.toolbarPanel !== null && selectedProjectId === projectId
                    ));
                  } else routeLoad.requestOptional("toolbar", true);
                }}
              >
                {t("common.retry")}
              </button>
            </div>
          {:else}
            <span class="sr-only" aria-busy="true">{t("common.loading")}</span>
          {/if}
        {/if}
        {#if !mobileLayout && routeUi.selectedTaskIds.length > 0}
          {#if bulkActionsLoadState?.status === "ready" && bulkActionsLoadState.component.kind === "bulk-actions"}
            {@const ProjectBulkActionController = bulkActionsLoadState.component.component}
            <ProjectBulkActionController
              {selectedProject}
              {priorities}
              {selectedTasks}
              {selectableTasks}
              {selectedActiveTaskCount}
              {selectedArchivedTaskCount}
              terminalStatus={terminalStatus()}
              firstOpenStatus={firstOpenStatus()}
              bind:selectedTaskIds={routeUi.selectedTaskIds}
              bind:showArchivedTasks={taskQuery.showArchivedTasks}
            />
          {:else if bulkActionsLoadState?.status === "failed"}
            <div class="flex min-h-9 items-center justify-center gap-2 border-t border-border px-3 text-xs text-muted-foreground" role="alert">
              <span>{t("common.viewLoadFailed", t("projects.bulk.selected", routeUi.selectedTaskIds.length))}</span>
              <button
                type="button"
                class="font-medium text-foreground underline-offset-2 hover:underline"
                onclick={() => routeLoad.requestOptional("bulk-actions", true)}
              >
                {t("common.retry")}
              </button>
            </div>
          {:else}
            <span class="sr-only" aria-busy="true">{t("common.loading")}</span>
          {/if}
        {/if}
      </header>

    {:else}
      <header
        data-projects-shell-header
        class="flex h-11 shrink-0 items-center border-b border-border px-4 text-sm font-semibold"
        style="background-color: var(--cal-header-bg);"
      >
        {t("titleBar.tab.projects")}
      </header>
    {/if}

    <div
      data-projects-content-frame
      class="relative min-h-0 flex-1"
      style="background-color: var(--cal-bg);"
    >
      {#if selectedProject && selectedGroup}
          {#if mobileLayout && mobileViewLoadError?.view === projects.activeView}
            <div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center" role="alert">
              <p class="max-w-sm text-sm text-muted-foreground">{mobileViewLoadError.failure.message}</p>
              <button
                type="button"
                class="min-h-12 rounded-xl border border-border bg-card px-5 text-sm font-medium active:bg-accent"
                onclick={() => {
                  const failed = mobileViewLoadError;
                  if (!failed) return;
                  const retry = () => {
                    mobileViewLoadError = null;
                    void loadMobileView(failed.view).catch((error: unknown) => {
                      console.error(`retry mobile Project ${failed.view} view failed`, error);
                    });
                  };
                  if (mobileViewLoadRecovery) mobileViewLoadRecovery.recover(failed.failure, retry);
                  else retry();
                }}
              >
                {t("common.retry")}
              </button>
            </div>
          {:else if projects.activeView === "list"}
            {#if mobileLayout && MobileListView}
            <MobileListView
              {tasks}
              {statuses}
              {priorities}
              onOpenTask={(task) => routeUi.openTask(task)}
              onCreateTask={async (title) => {
                if (!selectedProjectId) return undefined;
                return projects.addTask(selectedProjectId, title);
              }}
              onNeedMore={() => taskQuery.loadNextList(routeUi.selectedTaskIds)}
            />
            {:else if !mobileLayout && desktopViewComponents}
            {@const ProjectListView = desktopViewComponents.list}
            <ProjectListView
              {selectedProjectId}
              {sections}
              {statuses}
              {priorities}
              {tasks}
              {allProjectTasks}
              {listTaskGroups}
              taskGroupBy={taskQuery.groupBy}
              taskSortMode={taskQuery.sortMode}
              taskSortDirection={taskQuery.sortDirection}
              taskListColumns={taskQuery.listColumns}
              taskListColumnWidths={taskQuery.listColumnWidths}
              {projectCustomFields}
              selectedTaskId={routeUi.selectedTaskId}
              selectedTaskIds={routeUi.selectedTaskIds}
              showArchivedTasks={taskQuery.showArchivedTasks}
              onOpenTask={(task) => routeUi.openTask(task)}
              onSelectedTaskIdsChange={(taskIds) => {
                routeUi.selectedTaskIds = taskIds;
              }}
              onRevealTask={revealCreatedTask}
              onTaskListColumnWidthsChange={(widths, options) => {
                void taskQuery.updateColumnWidths(widths, options);
              }}
              onNeedMore={() => taskQuery.loadNextList(routeUi.selectedTaskIds)}
            />
            {:else}
              <div class="flex h-full items-center justify-center text-sm text-muted-foreground" aria-busy="true">
                {t("common.loading")}
              </div>
            {/if}
          {:else if projects.activeView === "kanban" && (desktopViewComponents?.kanban || MobileKanbanView)}
            {@const ProjectKanbanView = desktopViewComponents?.kanban ?? MobileKanbanView}
            {#if ProjectKanbanView}
            <ProjectKanbanView
              {mobileLayout}
              {tasks}
              {statuses}
              {priorities}
              selectedTaskIds={routeUi.selectedTaskIds}
              taskSortMode={taskQuery.sortMode}
              taskSortDirection={taskQuery.sortDirection}
              onOpenTask={(task) => routeUi.openTask(task)}
              onToggleTaskSelection={(task) => routeUi.toggleTaskSelection(task)}
              columnCounts={projects.taskViewPage?.columnCounts ?? []}
              onNeedMore={() => taskQuery.loadNextKanban(routeUi.selectedTaskIds)}
            />
            {/if}
          {:else if projects.activeView === "calendar" && (desktopViewComponents?.calendar || MobileProjectCalendarView)}
            {@const CalendarView = desktopViewComponents?.calendar ?? MobileProjectCalendarView}
            {#if CalendarView}
            <div class="h-full min-h-112 overflow-hidden">
              <CalendarView
                eventFilter={(event) => taskQuery.eventMatches(event)}
                createDefaults={projectCalendarCreateDefaults}
                initialViewMode={projectCalendarViewMode}
                onViewModeChange={(mode) => {
                  projectCalendarViewMode = mode;
                }}
                {mobileLayout}
              />
            </div>
            {/if}
          {:else if projects.activeView === "gantt" && (desktopViewComponents?.gantt || MobileGanttView)}
            {@const ProjectGanttView = desktopViewComponents?.gantt ?? MobileGanttView}
            {#if ProjectGanttView}
            <ProjectGanttView
              tasks={tasks}
              statuses={statuses}
              sections={sections}
              todayDate={todayDate}
              onOpenTask={(task) => routeUi.openTask(task)}
              onToggleSectionCollapsed={(section) => {
                void toggleSectionCollapsed(section);
              }}
            />
            {/if}
          {:else if projects.activeView === "dashboard" && (desktopViewComponents?.dashboard || MobileDashboardView)}
            {@const ProjectDashboardView = desktopViewComponents?.dashboard ?? MobileDashboardView}
            {#if ProjectDashboardView}
            <ProjectDashboardView
              {mobileLayout}
              projectId={selectedProjectId}
              {tasks}
              {statuses}
              {priorities}
              {todayDate}
              {scheduledTaskIds}
              {scheduledThisWeekMinutes}
              aggregates={projects.taskViewPage?.aggregates}
              onOpenTask={(task) => routeUi.openTask(task)}
            />
            {/if}
          {:else}
            <div class="flex h-full items-center justify-center text-sm text-muted-foreground" aria-busy="true">
              {t("common.loading")}
            </div>
          {/if}
      {:else}
        <ProjectEmptyState
          {selectedProjectId}
          bind:showInactiveProjects
          {mobileLayout}
          onProjectSelected={() => {
            routeUi.selectedTaskId = null;
            routeUi.closeToolbarImmediately();
          }}
        />
      {/if}
    </div>
  </section>

  {#if !mobileLayout && selectedProject && (routeUi.taskFinderOpen || taskQuery.search.trim().length > 0)}
    {#if taskFinderLoadState?.status === "ready" && taskFinderLoadState.component.kind === "task-finder"}
      {@const ProjectTaskFinder = taskFinderLoadState.component.component}
      <ProjectTaskFinder
        taskSearch={taskQuery.search}
        {matchingTaskCount}
        totalTaskCount={projects.taskViewPage?.totalCount ?? allProjectTasks.length}
        focusRequestId={routeUi.taskFinderFocusRequestId}
        onTaskSearchChange={(value) => {
          taskQuery.search = value;
        }}
        onClose={() => routeUi.closeFinder()}
        onClearAndClose={() => routeUi.closeOrClearFinder(taskQuery.search, () => { taskQuery.search = ""; })}
      />
    {:else if taskFinderLoadState?.status === "failed"}
      <div class="absolute inset-x-3 top-3 z-80 flex min-h-10 items-center justify-center gap-2 rounded-md border border-border bg-card px-3 text-xs text-muted-foreground shadow-lg" role="alert">
        <span>{t("common.viewLoadFailed", t("projects.finder.label"))}</span>
        <button
          type="button"
          class="font-medium text-foreground underline-offset-2 hover:underline"
          onclick={() => routeLoad.requestOptional("task-finder", true)}
        >
          {t("common.retry")}
        </button>
      </div>
    {:else}
      <span class="sr-only" aria-busy="true">{t("common.loading")}</span>
    {/if}
  {/if}

  {#if routeUi.selectedTaskId}
    {#if taskDetailDataReady && taskDetailLoadState?.status === "ready" && taskDetailLoadState.component.kind === "task-detail"}
      {@const ProjectTaskDetailPanel = taskDetailLoadState.component.component}
      <ProjectTaskDetailPanel
        taskId={routeUi.selectedTaskId}
        layout={taskDetailModalLayout}
        showArchivedTasks={taskQuery.showArchivedTasks}
        showInactiveSections={taskQuery.showInactiveSections}
        onClose={() => {
          routeUi.selectedTaskId = null;
          routeLoad.invalidateTaskDetailData();
        }}
        onOpenTask={(taskId) => {
          routeUi.selectedTaskId = taskId;
        }}
        onShowArchivedTasks={() => {
          taskQuery.showArchivedTasks = true;
        }}
      />
    {:else if routeLoad.taskDetailDataError || taskDetailLoadState?.status === "failed"}
      <div class="absolute inset-0 z-80 flex items-center justify-center bg-black/40 p-4" role="alert">
        <div class="flex min-h-32 w-full max-w-sm flex-col items-center justify-center gap-3 rounded-lg border border-border bg-card p-4 text-center text-sm text-muted-foreground shadow-xl">
          <p>{t("common.viewLoadFailed", t("projects.detail.title"))}</p>
          <div class="flex gap-2">
            <button
              type="button"
              class="min-h-9 rounded-md border border-border bg-background px-3 font-medium text-foreground hover:bg-accent"
              onclick={() => {
                if (routeLoad.taskDetailDataError) {
                  void routeLoad.requestTaskDetailData(selectedProjectId, routeUi.selectedTaskId);
                } else routeLoad.requestOptional("task-detail", true);
              }}
            >
              {t("common.retry")}
            </button>
            <button
              type="button"
              class="min-h-9 rounded-md px-3 font-medium text-foreground hover:bg-accent"
              onclick={() => {
                routeUi.selectedTaskId = null;
                routeLoad.invalidateTaskDetailData();
              }}
            >
              {t("common.cancel")}
            </button>
          </div>
        </div>
      </div>
    {:else}
      <div class="absolute inset-0 z-80 flex items-center justify-center bg-black/30 p-4" aria-busy="true">
        <div class="rounded-md border border-border bg-card px-4 py-3 text-sm text-muted-foreground shadow-lg">
          {t("common.loading")}
        </div>
      </div>
    {/if}
  {/if}

  {#if routeUi.discardConfirmOpen}
    <ConfirmDialog
      title={t("calendar.view.discardUnsavedTitle")}
      message={t("calendar.view.changesLost")}
      confirmLabel={t("calendar.view.discard")}
      cancelLabel={t("common.cancel")}
      onConfirm={() => routeUi.confirmDiscard()}
      onCancel={() => routeUi.cancelDiscard()}
    />
  {/if}
</div>

<style>
  :global(.projects-view-root),
  :global(.projects-view-root *) {
    user-select: none;
  }

  :global(.projects-view-root input),
  :global(.projects-view-root textarea),
  :global(.projects-view-root [contenteditable="true"]),
  :global(.projects-view-root [contenteditable="true"] *) {
    user-select: text;
  }
</style>
