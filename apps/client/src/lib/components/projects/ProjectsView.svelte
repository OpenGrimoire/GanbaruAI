<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";
  import { onMount } from "svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import {
    beginLazyComponentLoad,
    rejectLazyComponentLoad,
    resolveLazyComponentLoad,
    type LazyComponentLoadState,
  } from "$lib/lazy-component-loader";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getViewport } from "$lib/stores/viewport.svelte";
  import { formatCalendarDate } from "$lib/components/calendar/utils";
  import { cn, isAppShortcutBlockedTarget, isEditableKeyboardTarget } from "$lib/utils";
  import type {
    CalendarEvent,
    CalendarViewMode,
  } from "$lib/components/calendar/types";
  import type {
    ProjectCustomField,
    ProjectCustomFieldFilter,
    ProjectCustomFieldValue,
    ProjectPriority,
    ProjectSavedTaskView,
    ProjectSection,
    ProjectStatus,
    ProjectTask,
    ProjectTaskDependencyFilter,
    ProjectTaskDueFilter,
    ProjectTaskGroupMode,
    ProjectTaskTagFilter,
    ProjectTaskListColumn,
    ProjectTaskScheduleFilter,
    ProjectTaskSortDirection,
    ProjectTaskSortMode,
    ProjectTaskStatusFilter,
    ProjectViewId,
  } from "$lib/projects/types";
  import {
    projectCalendarCreateDefaults as buildProjectCalendarCreateDefaults,
    projectCalendarEventRootId,
    projectEventDurationMinutesInDateRange,
  } from "$lib/projects/project-scheduling";
  import {
    PROJECT_TASK_FILTER_DEFAULTS,
    projectTaskDataFiltersActive,
    selectedProjectTaskIdsInView,
    taskListColumnWidthsForProject,
    type ProjectTaskListColumnWidths,
    type ProjectTaskFilterState,
  } from "$lib/projects/project-list-view";
  import {
    PROJECT_TASK_LIST_COLUMNS,
    PROJECT_VIEW_IDS,
  } from "$lib/projects/types";
  import {
    customFieldIdFromCustomFieldReference,
    customFieldIdFromTaskListColumn,
    customTaskListColumn,
    DEFAULT_TASK_LIST_COLUMNS,
    taskListColumnsMatch,
    taskListColumnsForProject,
  } from "$lib/projects/task-list-columns";
  import {
    createProjectSavedTaskViewSnapshot,
    projectCustomFieldFilterStillExists,
    projectTaskFilterStateFromSavedTaskView,
  } from "$lib/projects/saved-task-views";
  import {
    buildProjectTaskListGroups,
    buildProjectTaskView,
    projectTaskCustomFieldKey,
  } from "$lib/projects/task-view";
  import {
    deriveProjectListColumnControls,
    pickProjectTaskModalLayout,
    toggleProjectListColumn,
    type ProjectToolbarPanel,
  } from "$lib/projects/project-toolbar";
  import ProjectEmptyState from "./ProjectEmptyState.svelte";
  import ProjectWorkspaceHeader from "./ProjectWorkspaceHeader.svelte";
  import {
    loadProjectOptionalComponent,
    loadProjectView,
    retryProjectOptionalComponent,
    retryProjectView,
    type LoadedProjectOptionalComponent,
    type LoadedProjectView,
    type ProjectOptionalComponentKind,
  } from "./project-component-registry";

  const projects = getProjects();
  const calendar = getCalendar();
  const preferences = getPreferences();
  const viewport = getViewport();
  const { t } = getLocalization();
  const PROJECT_VIEW_SHORTCUTS = new Map<string, ProjectViewId>(
    PROJECT_VIEW_IDS.map((view, index) => [String(index + 1), view]),
  );

  let showInactiveProjects = $state(false);
  let showInactiveSections = $state(false);
  let showArchivedTasks = $state(false);
  let taskSearch = $state("");
  let taskFinderOpen = $state(false);
  let taskFinderFocusRequestId = $state(0);
  let taskStatusFilter = $state<ProjectTaskStatusFilter>("all");
  let taskSectionFilter = $state<string | "all">("all");
  let taskPriorityFilter = $state<ProjectPriority | "all">("all");
  let taskDueFilter = $state<ProjectTaskDueFilter>("all");
  let taskDueRangeStart = $state("");
  let taskDueRangeEnd = $state("");
  let taskScheduleFilter = $state<ProjectTaskScheduleFilter>("all");
  let taskDependencyFilter = $state<ProjectTaskDependencyFilter>("all");
  let taskTagFilter = $state<ProjectTaskTagFilter>("all");
  let taskCustomFieldFilters = $state<ProjectCustomFieldFilter[]>([]);
  let taskGroupBy = $state<ProjectTaskGroupMode>("section");
  let taskSortMode = $state<ProjectTaskSortMode>("manual");
  let taskSortDirection = $state<ProjectTaskSortDirection>("asc");
  let taskListColumns = $state<ProjectTaskListColumn[]>([...DEFAULT_TASK_LIST_COLUMNS]);
  let taskListColumnWidths = $state<ProjectTaskListColumnWidths>({});
  let projectCalendarViewMode = $state<CalendarViewMode>("week");
  let savedViewNameDraft = $state("");
  let savedViewSaving = $state(false);
  let savedViewError = $state<string | null>(null);
  let selectedTaskId = $state<string | null>(null);
  let selectedTaskIds = $state<string[]>([]);
  let projectToolbarPanel = $state<ProjectToolbarPanel | null>(null);
  let projectSettingsDirty = $state(false);
  let projectSettingsDiscardConfirmOpen = $state(false);
  let pendingProjectSettingsAction: (() => void) | null = null;
  let projectsRootElement = $state<HTMLDivElement | null>(null);
  let toolbarDataError = $state<string | null>(null);
  let taskDetailDataError = $state<string | null>(null);
  let viewLoadState = $state<LazyComponentLoadState<
    ProjectViewId,
    LoadedProjectView
  > | null>(null);
  type ProjectOptionalLoadState = LazyComponentLoadState<
    ProjectOptionalComponentKind,
    LoadedProjectOptionalComponent
  >;
  let optionalLoadStates = $state<Partial<Record<
    ProjectOptionalComponentKind,
    ProjectOptionalLoadState
  >>>({});

  const activeViewLoadState = $derived(
    viewLoadState?.key === projects.activeView ? viewLoadState : null,
  );
  const toolbarLoadState = $derived(optionalLoadStates.toolbar ?? null);
  const bulkActionsLoadState = $derived(optionalLoadStates["bulk-actions"] ?? null);
  const taskFinderLoadState = $derived(optionalLoadStates["task-finder"] ?? null);
  const taskDetailLoadState = $derived(optionalLoadStates["task-detail"] ?? null);

  function requestProjectView(view: ProjectViewId, retry = false): void {
    if (!retry && viewLoadState?.key === view) return;
    const loadingState = beginLazyComponentLoad(viewLoadState, view);
    viewLoadState = loadingState;
    const request = retry ? retryProjectView(view) : loadProjectView(view);
    void request
      .then((component) => {
        if (!viewLoadState) return;
        viewLoadState = resolveLazyComponentLoad(
          viewLoadState,
          view,
          loadingState.requestId,
          component,
        );
      })
      .catch((error: unknown) => {
        if (!viewLoadState) return;
        viewLoadState = rejectLazyComponentLoad(
          viewLoadState,
          view,
          loadingState.requestId,
          error,
        );
        console.error(`Failed to load Project ${view} view:`, error);
      });
  }

  function requestProjectOptionalComponent(
    kind: ProjectOptionalComponentKind,
    retry = false,
  ): void {
    const current = optionalLoadStates[kind] ?? null;
    if (!retry && current) return;
    const loadingState = beginLazyComponentLoad(current, kind);
    optionalLoadStates = { ...optionalLoadStates, [kind]: loadingState };
    const request = retry
      ? retryProjectOptionalComponent(kind)
      : loadProjectOptionalComponent(kind);
    void request
      .then((component) => {
        const active = optionalLoadStates[kind];
        if (!active) return;
        const next = resolveLazyComponentLoad(
          active,
          kind,
          loadingState.requestId,
          component,
        );
        if (next !== active) optionalLoadStates = { ...optionalLoadStates, [kind]: next };
      })
      .catch((error: unknown) => {
        const active = optionalLoadStates[kind];
        if (!active) return;
        const next = rejectLazyComponentLoad(
          active,
          kind,
          loadingState.requestId,
          error,
        );
        if (next !== active) optionalLoadStates = { ...optionalLoadStates, [kind]: next };
        console.error(`Failed to load optional Project surface ${kind}:`, error);
      });
  }

  const selectedProject = $derived(projects.selectedProject);
  const selectedGroup = $derived(projects.selectedGroup);
  const selectedProjectId = $derived(selectedProject?.id ?? null);
  const toolbarDataReady = $derived.by(() => {
    const projectId = selectedProjectId;
    return Boolean(
      projectId
      && projects.projectOptionalDataLoaded("relationships", projectId)
      && projects.projectOptionalDataLoaded("custom_fields", projectId)
      && projects.projectOptionalDataLoaded("saved_views", projectId),
    );
  });
  const taskDetailDataReady = $derived.by(() => {
    const projectId = selectedProjectId;
    return Boolean(
      projectId
      && projects.projectOptionalDataLoaded("relationships", projectId)
      && projects.projectOptionalDataLoaded("custom_fields", projectId)
      && projects.projectOptionalDataLoaded("history", projectId)
      && projects.projectOptionalDataLoaded("checklist", projectId),
    );
  });

  function requestProjectToolbarData(): void {
    const projectId = selectedProjectId;
    if (!projectId) return;
    toolbarDataError = null;
    void projects.ensureProjectToolbarData(projectId).catch((error) => {
      if (selectedProjectId !== projectId || !projectToolbarPanel) return;
      toolbarDataError = error instanceof Error ? error.message : String(error);
      console.error("load optional Project toolbar data failed", error);
    });
  }

  function requestTaskDetailData(): void {
    const projectId = selectedProjectId;
    const taskId = selectedTaskId;
    if (!projectId || !taskId) return;
    taskDetailDataError = null;
    void projects.ensureTaskDetailData(projectId).catch((error) => {
      if (selectedProjectId !== projectId || selectedTaskId !== taskId) return;
      taskDetailDataError = error instanceof Error ? error.message : String(error);
      console.error("load optional Project task detail data failed", error);
    });
  }
  $effect(() => {
    if (!selectedProjectId || projects.projectDataLoaded(selectedProjectId)) return;
    void projects.ensureProjectData(selectedProjectId).catch((error) => {
      console.error("load selected project data failed", error);
    });
  });
  const allProjectSections = $derived(projects.sectionsForProjectIncludingInactive(selectedProjectId));
  const inactiveSectionCount = $derived(
    allProjectSections.filter((section) => Boolean(section.hiddenAt || section.archivedAt)).length,
  );
  const sections = $derived.by(() =>
    showInactiveSections ? allProjectSections : projects.sectionsForProject(selectedProjectId)
  );
  const visibleSectionIds = $derived.by(() => new Set(sections.map((section) => section.id)));
  const statuses = $derived(projects.statusesForProject(selectedProjectId));
  const priorities = $derived(projects.prioritiesForProject(selectedProjectId));
  const activeProjectTasks = $derived.by(() =>
    projects.tasksForProject(selectedProjectId).filter((task) => visibleSectionIds.has(task.sectionId))
  );
  const allProjectTasksWithArchived = $derived.by(() =>
    projects.tasksForProjectIncludingArchived(selectedProjectId)
      .filter((task) => visibleSectionIds.has(task.sectionId))
  );
  const archivedProjectTaskCount = $derived(
    allProjectTasksWithArchived.filter((task) => Boolean(task.archivedAt)).length,
  );
  const allProjectTasks = $derived(showArchivedTasks ? allProjectTasksWithArchived : activeProjectTasks);
  const allProjectTaskIds = $derived.by(() => new Set(allProjectTasks.map((task) => task.id)));
  const projectTags = $derived(projects.tagsForProject(selectedProjectId));
  const projectCustomFields = $derived(projects.customFieldsForProject(selectedProjectId));
  const customFieldValuesByTaskField = $derived.by(() => {
    const values = new Map<string, ProjectCustomFieldValue>();
    for (const value of projects.customFieldValues) {
      if (!allProjectTaskIds.has(value.taskId)) continue;
      values.set(projectTaskCustomFieldKey(value.taskId, value.fieldId), value);
    }
    return values;
  });
  const customFieldOptionIdsByTaskField = $derived.by(() => {
    const optionIdsByTaskField = new Map<string, Set<string>>();
    for (const value of projects.customFieldOptionValues) {
      if (!allProjectTaskIds.has(value.taskId)) continue;
      const key = projectTaskCustomFieldKey(value.taskId, value.fieldId);
      const optionIds = optionIdsByTaskField.get(key) ?? new Set<string>();
      optionIds.add(value.optionId);
      optionIdsByTaskField.set(key, optionIds);
    }
    return optionIdsByTaskField;
  });
  const taskTagIdsByTaskId = $derived.by(() => {
    const tagsByTask = new Map<string, Set<string>>();
    for (const link of projects.taskTagLinks) {
      if (!allProjectTaskIds.has(link.taskId)) continue;
      const tagIds = tagsByTask.get(link.taskId) ?? new Set<string>();
      tagIds.add(link.tagId);
      tagsByTask.set(link.taskId, tagIds);
    }
    return tagsByTask;
  });
  const todayDate = $derived(Temporal.Now.plainDateISO().toString());
  const taskFilterWeekEnd = $derived(Temporal.PlainDate.from(todayDate).add({ days: 7 }).toString());
  const normalizedTaskDueRangeStart = $derived(normalizeFilterDate(taskDueRangeStart));
  const normalizedTaskDueRangeEnd = $derived(normalizeFilterDate(taskDueRangeEnd));
  const taskFilterState = $derived.by((): ProjectTaskFilterState => ({
    search: taskSearch,
    statusFilter: taskStatusFilter,
    sectionFilter: taskSectionFilter,
    priorityFilter: taskPriorityFilter,
    dueFilter: taskDueFilter,
    dueRangeStart: normalizedTaskDueRangeStart ?? "",
    dueRangeEnd: normalizedTaskDueRangeEnd ?? "",
    scheduleFilter: taskScheduleFilter,
    dependencyFilter: taskDependencyFilter,
    tagFilter: taskTagFilter,
    customFieldFilters: taskCustomFieldFilters,
    groupBy: taskGroupBy,
    sortMode: taskSortMode,
    sortDirection: taskSortDirection,
  }));
  const scheduledTaskIds = $derived.by(() => new Set(
    projects.eventLinks
      .filter((link) => link.linkKind === "scheduled")
      .map((link) => link.taskId),
  ));
  const nowCalendarDateTime = $derived(formatCalendarDate(new Date()));
  const allProjectEvents = $derived.by(() => {
    if (!selectedProjectId) return [];
    return calendar.rawBlocks
      .filter((event) => event.projectId === selectedProjectId)
      .sort((a, b) => a.start.localeCompare(b.start));
  });
  const allProjectEventsById = $derived.by(() => new Map(allProjectEvents.map((event) => [event.id, event])));
  function projectCalendarEventFilter(event: CalendarEvent): boolean {
    if (!selectedProjectId || event.projectId !== selectedProjectId) return false;
    if (!taskDataFiltersActive) return true;
    return eventIdsForMatchedTasks.has(event.id) || eventIdsForMatchedTasks.has(projectCalendarEventRootId(event));
  }

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
  const nextScheduledStartByTaskId = $derived.by(() => {
    const startsByTaskId = new Map<string, string[]>();
    for (const link of projects.eventLinks) {
      if (link.linkKind !== "scheduled" || !allProjectTaskIds.has(link.taskId)) continue;
      const start = allProjectEventsById.get(link.eventId)?.start;
      if (!start) continue;
      const starts = startsByTaskId.get(link.taskId) ?? [];
      starts.push(start);
      startsByTaskId.set(link.taskId, starts);
    }
    const nextStarts = new Map<string, string>();
    for (const [taskId, starts] of startsByTaskId) {
      const sortedStarts = starts.sort((a, b) => a.localeCompare(b));
      nextStarts.set(taskId, sortedStarts.find((start) => start >= nowCalendarDateTime) ?? sortedStarts[0]);
    }
    return nextStarts;
  });
  const dependencyBlockedTaskIds = $derived.by(() => new Set(
    projects.dependencies
      .filter((dependency) => allProjectTaskIds.has(dependency.blockedTaskId))
      .map((dependency) => dependency.blockedTaskId),
  ));
  const dependencyBlockingTaskIds = $derived.by(() => new Set(
    projects.dependencies
      .filter((dependency) => allProjectTaskIds.has(dependency.blockingTaskId))
      .map((dependency) => dependency.blockingTaskId),
  ));
  const taskView = $derived.by(() => buildProjectTaskView({
    tasks: allProjectTasks,
    statuses,
    priorities,
    scheduledTaskIds,
    nextScheduledStartByTaskId,
    taskTagIdsByTaskId,
    dependencyBlockedTaskIds,
    dependencyBlockingTaskIds,
    today: todayDate,
    weekEnd: taskFilterWeekEnd,
    ...taskFilterState,
    customFields: projectCustomFields,
    customFieldOptions: projects.customFieldOptions,
    customFieldValuesByTaskField,
    customFieldOptionIdsByTaskField,
  }));
  const tasks = $derived.by(() => {
    return taskView.tasks;
  });
  const listTaskGroups = $derived.by(() => {
    if (taskGroupBy === "section") return [];
    return buildProjectTaskListGroups({
      tasks,
      statuses,
      priorities,
      scheduledTaskIds,
      today: todayDate,
      weekEnd: taskFilterWeekEnd,
      groupBy: taskGroupBy,
    });
  });
  const selectableTasks = $derived.by(() => tasks.filter((task) => !task.parentTaskId));
  const selectedTaskIdSet = $derived.by(() => new Set(selectedTaskIds));
  const selectedTasks = $derived.by(() => allProjectTasks.filter((task) => selectedTaskIdSet.has(task.id)));
  const selectedArchivedTaskCount = $derived(selectedTasks.filter((task) => Boolean(task.archivedAt)).length);
  const selectedActiveTaskCount = $derived(selectedTasks.length - selectedArchivedTaskCount);
  const matchingTaskCount = $derived(taskView.matchedTaskIds.size);
  const activeTaskFilterCount = $derived(taskView.activeFilterCount);
  const taskFiltersActive = $derived(activeTaskFilterCount > 0);
  const taskFilterControlsActive = $derived(taskFiltersActive || showArchivedTasks || showInactiveSections);
  const taskGroupingActive = $derived(taskGroupBy !== "section");
  const taskCustomizeActive = $derived(!taskListColumnsMatch(taskListColumns, DEFAULT_TASK_LIST_COLUMNS));
  const taskDataFiltersActive = $derived(projectTaskDataFiltersActive(taskFilterState));
  const savedTaskViews = $derived.by(() => projects.savedTaskViewsForProject(selectedProjectId));
  const eventIdsForMatchedTasks = $derived.by(() => new Set(
    projects.eventLinks
      .filter((link) => taskView.matchedTaskIds.has(link.taskId))
      .map((link) => link.eventId),
  ));
  const projectEvents = $derived.by(() => {
    return allProjectEvents.filter((event) => !taskDataFiltersActive || eventIdsForMatchedTasks.has(event.id));
  });
  const scheduledThisWeekMinutes = $derived.by(() => thisWeekScheduledMinutes());
  const selectedTask = $derived.by(() =>
    selectedTaskId ? allProjectTasks.find((task) => task.id === selectedTaskId) : undefined
  );
  const availableTaskListColumns = $derived.by(() => [
    ...PROJECT_TASK_LIST_COLUMNS,
    ...projectCustomFields.map((field) => customTaskListColumn(field.id)),
  ]);
  const taskListColumnControls = $derived.by(() =>
    deriveProjectListColumnControls(availableTaskListColumns, taskListColumns, taskListColumnLabel)
  );
  const taskDetailModalLayout = $derived(pickProjectTaskModalLayout({
    viewportWidth: viewport.width,
    viewportHeight: viewport.height,
  }));


  onMount(() => {
    void projects.ensureLoaded().catch((error) => {
      console.error("load projects failed", error);
    });
  });

  $effect(() => {
    const view = projects.activeView;
    if (!selectedProject || !selectedGroup) return;
    requestProjectView(view);
    void projects.ensureProjectViewData(selectedProject.id, view).catch((error) => {
      console.error(`load optional Project ${view} data failed`, error);
    });
  });

  $effect(() => {
    if (!projectToolbarPanel) return;
    requestProjectOptionalComponent("toolbar");
    requestProjectToolbarData();
  });

  $effect(() => {
    if (selectedTaskIds.length > 0) requestProjectOptionalComponent("bulk-actions");
  });

  $effect(() => {
    if (taskFinderOpen || taskSearch.trim()) requestProjectOptionalComponent("task-finder");
  });

  $effect(() => {
    if (!selectedTaskId) return;
    requestProjectOptionalComponent("task-detail");
    requestTaskDetailData();
  });

  $effect(() => {
    const customFieldIds = new Set(projectCustomFields.map((field) => field.id));
    taskListColumns = taskListColumnsForProject(
      projects.viewPreferences,
      selectedProjectId,
      customFieldIds,
    );
    taskListColumnWidths = taskListColumnWidthsForProject(
      projects.viewPreferences,
      selectedProjectId,
      customFieldIds,
    );
  });

  $effect(() => {
    if (selectedTaskId && !selectedTask) {
      selectedTaskId = null;
    }
  });

  $effect(() => {
    const nextSelectedTaskIds = selectedProjectTaskIdsInView(selectedTaskIds, allProjectTasks);
    if (nextSelectedTaskIds.length !== selectedTaskIds.length) {
      selectedTaskIds = nextSelectedTaskIds;
    }
  });

  $effect(() => {
    if (taskSectionFilter !== "all" && !sections.some((section) => section.id === taskSectionFilter)) {
      taskSectionFilter = "all";
    }
    if (
      taskTagFilter !== "all"
      && taskTagFilter !== "none"
      && !projectTags.some((tag) => tag.id === taskTagFilter)
    ) {
      taskTagFilter = "all";
    }
    const fieldIds = new Set(projectCustomFields.map((field) => field.id));
    const optionIds = new Set(
      projectCustomFields.flatMap((field) => projects.customFieldOptionsForField(field.id).map((option) => option.id)),
    );
    const nextCustomFieldFilters = taskCustomFieldFilters.filter((filter) =>
      projectCustomFieldFilterStillExists(filter, fieldIds, optionIds)
    );
    if (nextCustomFieldFilters.length !== taskCustomFieldFilters.length) {
      taskCustomFieldFilters = nextCustomFieldFilters;
    }
    const customSortFieldId = customFieldIdFromCustomFieldReference(taskSortMode);
    if (customSortFieldId && !fieldIds.has(customSortFieldId)) {
      taskSortMode = "manual";
      taskSortDirection = "asc";
    }
  });

  function taskListColumnLabel(column: ProjectTaskListColumn): string {
    const customFieldId = customFieldIdFromTaskListColumn(column);
    if (customFieldId) {
      return projectCustomFields.find((field) => field.id === customFieldId)?.name
        ?? t("projects.columns.customField");
    }
    if (column === "priority") return t("projects.columns.priority");
    if (column === "estimate") return t("projects.columns.estimate");
    if (column === "start") return t("projects.columns.start");
    if (column === "due") return t("projects.columns.due");
    if (column === "scheduled") return t("projects.columns.scheduled");
    if (column === "dependencies") return t("projects.columns.dependencies");
    if (column === "assignee") return t("projects.columns.assignee");
    if (column === "reviewer") return t("projects.columns.reviewer");
    return t("projects.columns.status");
  }

  function projectViewLabel(view: ProjectViewId): string {
    if (view === "dashboard") return t("projects.tabs.dashboard");
    if (view === "list") return t("projects.tabs.list");
    if (view === "kanban") return t("projects.tabs.kanban");
    if (view === "calendar") return t("projects.tabs.calendar");
    return t("projects.tabs.gantt");
  }

  async function toggleTaskListColumn(column: ProjectTaskListColumn): Promise<void> {
    if (!selectedProjectId) return;
    const nextColumns = toggleProjectListColumn(taskListColumns, column);
    taskListColumns = nextColumns;
    await projects.saveTaskListColumns(selectedProjectId, nextColumns);
  }

  async function updateTaskListColumnWidths(
    widths: ProjectTaskListColumnWidths,
    options: { persist?: boolean } = {},
  ): Promise<void> {
    taskListColumnWidths = widths;
    if (!options.persist || !selectedProjectId) return;
    await projects.saveTaskListColumnWidths(selectedProjectId, widths);
  }

  function estimateLabel(minutes: number): string {
    return t("projects.list.estimateMinutes", minutes);
  }

  function terminalStatus(): ProjectStatus | undefined {
    return statuses.find((status) => status.terminal);
  }

  function firstOpenStatus(): ProjectStatus | undefined {
    return statuses.find((status) => status.name.toLowerCase() === "to do")
      ?? statuses.find((status) => !status.terminal);
  }

  function runAfterProjectSettingsClose(action: () => void): void {
    if (projectToolbarPanel === "settings" && projectSettingsDirty) {
      pendingProjectSettingsAction = action;
      projectSettingsDiscardConfirmOpen = true;
      return;
    }
    action();
  }

  function setProjectToolbarPanel(panel: ProjectToolbarPanel | null): void {
    projectToolbarPanel = panel;
    if (panel !== "settings") projectSettingsDirty = false;
  }

  function toggleProjectToolbarPanel(panel: ProjectToolbarPanel): void {
    const nextPanel = projectToolbarPanel === panel ? null : panel;
    runAfterProjectSettingsClose(() => {
      setProjectToolbarPanel(nextPanel);
    });
  }

  function requestProjectToolbarPanelClose(): void {
    runAfterProjectSettingsClose(() => {
      setProjectToolbarPanel(null);
    });
  }

  function confirmDiscardProjectSettings(): void {
    const action = pendingProjectSettingsAction;
    pendingProjectSettingsAction = null;
    projectSettingsDiscardConfirmOpen = false;
    projectSettingsDirty = false;
    action?.();
  }

  function cancelDiscardProjectSettings(): void {
    pendingProjectSettingsAction = null;
    projectSettingsDiscardConfirmOpen = false;
  }

  function closeProjectToolbarPanelImmediately(): void {
    pendingProjectSettingsAction = null;
    projectSettingsDiscardConfirmOpen = false;
    setProjectToolbarPanel(null);
  }

  function openTaskFinder(): void {
    runAfterProjectSettingsClose(() => {
      taskFinderOpen = true;
      setProjectToolbarPanel(null);
      taskFinderFocusRequestId += 1;
    });
  }

  function clearAndCloseTaskFinder(): void {
    taskSearch = "";
    taskFinderOpen = false;
  }

  function closeTaskFinder(): void {
    taskFinderOpen = false;
  }

  function closeOrClearTaskFinder(): void {
    if (taskSearch.trim()) {
      clearAndCloseTaskFinder();
      return;
    }
    closeTaskFinder();
  }

  function projectsKeyboardTargetBlocked(target: EventTarget | null): boolean {
    return isEditableKeyboardTarget(target)
      || isAppShortcutBlockedTarget(target)
      || (target instanceof Element && target.closest("[role='dialog']") !== null);
  }

  function projectsViewShortcutBlocked(event: KeyboardEvent): boolean {
    return selectedTaskId !== null
      || taskFinderOpen
      || projectToolbarPanel !== null
      || projectsKeyboardTargetBlocked(event.target)
      || projectsKeyboardTargetBlocked(document.activeElement)
      || projectsRootElement?.querySelector("[role='dialog']") !== null;
  }

  function handleProjectViewShortcut(event: KeyboardEvent): boolean {
    if (event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return false;
    const view = PROJECT_VIEW_SHORTCUTS.get(event.key);
    if (!view || projectsViewShortcutBlocked(event)) return false;
    event.preventDefault();
    projects.activeView = view;
    return true;
  }


  function projectsEditableSelectionTarget(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return false;
    return Boolean(target.closest("input, textarea, [contenteditable='true'], [role='textbox']"));
  }

  function projectsSelectionNodeInside(node: Node | null): boolean {
    return Boolean(projectsRootElement && node && projectsRootElement.contains(node));
  }

  function handleProjectWindowKeydown(event: KeyboardEvent): void {
    if (event.defaultPrevented) return;
    if (selectedTaskId) return;
    if (handleProjectViewShortcut(event)) return;
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "f") {
      event.preventDefault();
      openTaskFinder();
      return;
    }
    if (event.key === "Escape" && taskFinderOpen) {
      event.preventDefault();
      closeTaskFinder();
    }
  }


  function handleProjectDocumentSelectStart(event: Event): void {
    const target = event.target;
    if (!(target instanceof Node) || !projectsRootElement?.contains(target)) return;
    if (projectsEditableSelectionTarget(target)) return;
    event.preventDefault();
  }

  function handleProjectDocumentSelectionChange(): void {
    const selection = document.getSelection();
    if (!selection || selection.isCollapsed) return;

    const activeElement = document.activeElement;
    if (activeElement && projectsEditableSelectionTarget(activeElement)) return;
    if (!projectsSelectionNodeInside(selection.anchorNode) && !projectsSelectionNodeInside(selection.focusNode)) return;

    selection.removeAllRanges();
  }

  function clearTaskCustomFieldFilter(fieldId: string): void {
    taskCustomFieldFilters = taskCustomFieldFilters.filter((filter) => filter.fieldId !== fieldId);
  }

  function applyTaskFilterState(state: ProjectTaskFilterState): void {
    taskSearch = state.search;
    taskStatusFilter = state.statusFilter;
    taskSectionFilter = state.sectionFilter;
    taskPriorityFilter = state.priorityFilter;
    taskDueFilter = state.dueFilter;
    taskDueRangeStart = state.dueRangeStart;
    taskDueRangeEnd = state.dueRangeEnd;
    taskScheduleFilter = state.scheduleFilter;
    taskDependencyFilter = state.dependencyFilter;
    taskTagFilter = state.tagFilter;
    taskCustomFieldFilters = [...state.customFieldFilters];
    taskGroupBy = state.groupBy;
    taskSortMode = state.sortMode;
    taskSortDirection = state.sortDirection;
  }

  function resetTaskFilterState(): void {
    applyTaskFilterState(PROJECT_TASK_FILTER_DEFAULTS);
  }

  function clearTaskFilters(): void {
    resetTaskFilterState();
  }

  function taskViewSnapshot(name: string, viewId: string): ProjectSavedTaskView | undefined {
    if (!selectedProjectId) return undefined;
    return createProjectSavedTaskViewSnapshot({
      ...taskFilterState,
      projectId: selectedProjectId,
      id: viewId,
      name,
      viewId: projects.activeView,
      collapsedSectionIds: sections
        .filter((section) => section.collapsed)
        .map((section) => section.id),
      showArchivedTasks,
      visibleColumns: taskListColumns,
    });
  }

  async function saveCurrentTaskView(): Promise<void> {
    const name = savedViewNameDraft.trim();
    if (!name) {
      savedViewError = t("projects.savedViews.nameRequired");
      return;
    }
    const existingView = savedTaskViews.find((view) => view.name.toLowerCase() === name.toLowerCase());
    const view = taskViewSnapshot(name, existingView?.id ?? crypto.randomUUID());
    if (!view) return;
    savedViewSaving = true;
    savedViewError = null;
    try {
      await projects.saveTaskView(view);
      savedViewNameDraft = "";
    } catch (error) {
      savedViewError = t(
        "projects.savedViews.saveFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      savedViewSaving = false;
    }
  }

  async function applyTaskView(view: ProjectSavedTaskView): Promise<void> {
    projects.activeView = view.viewId;
    applyTaskFilterState(projectTaskFilterStateFromSavedTaskView(view));
    showArchivedTasks = view.showArchivedTasks;
    taskListColumns = view.visibleColumns;
    if (selectedProjectId) {
      void projects.saveTaskListColumns(selectedProjectId, view.visibleColumns);
    }
    savedViewError = null;
    try {
      if (view.groupBy === "section") {
        await projects.applySectionCollapseState(view.projectId, view.collapsedSectionIds);
      }
    } catch (error) {
      savedViewError = t(
        "projects.savedViews.applyFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
    clearTaskSelection();
  }

  async function deleteSavedTaskView(view: ProjectSavedTaskView): Promise<void> {
    savedViewSaving = true;
    savedViewError = null;
    try {
      await projects.deleteTaskView(view);
    } catch (error) {
      savedViewError = t(
        "projects.savedViews.deleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      savedViewSaving = false;
    }
  }

  function toggleTaskSelection(task: ProjectTask): void {
    selectedTaskIds = selectedTaskIdSet.has(task.id)
      ? selectedTaskIds.filter((taskId) => taskId !== task.id)
      : [...selectedTaskIds, task.id];
  }

  function clearTaskSelection(): void {
    selectedTaskIds = [];
  }

  function revealCreatedTask(task: ProjectTask | undefined): void {
    if (!task) return;
    projects.activeView = "list";
    resetTaskFilterState();
    selectedTaskId = task.id;
    selectedTaskIds = [];
    showArchivedTasks = false;
    showInactiveSections = false;
  }


  async function toggleSectionCollapsed(section: ProjectSection): Promise<void> {
    await projects.updateSection(section, { collapsed: !section.collapsed });
  }

  function openTaskDetail(task: ProjectTask): void {
    runAfterProjectSettingsClose(() => {
      setProjectToolbarPanel(null);
      selectedTaskId = task.id;
    });
  }

  function normalizeFilterDate(value: string): string | undefined {
    const trimmed = value.trim();
    if (!trimmed) return undefined;
    try {
      return Temporal.PlainDate.from(trimmed).toString();
    } catch {
      return undefined;
    }
  }


  function thisWeekScheduledMinutes(): number {
    const weekEnd = Temporal.PlainDate.from(todayDate).add({ days: 7 }).toString();
    return projectEventDurationMinutesInDateRange(allProjectEvents, todayDate, weekEnd);
  }

</script>

<svelte:window onkeydown={handleProjectWindowKeydown} />

<svelte:document
  onselectstart={handleProjectDocumentSelectStart}
  onselectionchange={handleProjectDocumentSelectionChange}
/>

<div
  bind:this={projectsRootElement}
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
          {projectToolbarPanel}
          {taskGroupingActive}
          taskFiltersActive={taskFilterControlsActive}
          {taskCustomizeActive}
          onShowInactiveProjectsChange={(value) => {
            showInactiveProjects = value;
          }}
          onProjectSelected={() => {
            selectedTaskId = null;
            closeProjectToolbarPanelImmediately();
          }}
          onToggleToolbarPanel={toggleProjectToolbarPanel}
        />
        {#if projectToolbarPanel}
          {#if toolbarDataReady && toolbarLoadState?.status === "ready" && toolbarLoadState.component.kind === "toolbar"}
            {@const ProjectToolbarPanels = toolbarLoadState.component.component}
            <ProjectToolbarPanels
              panel={projectToolbarPanel}
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
              {savedViewSaving}
              {savedViewError}
              bind:taskStatusFilter
              bind:taskSectionFilter
              bind:taskPriorityFilter
              bind:taskDueFilter
              bind:taskDueRangeStart
              bind:taskDueRangeEnd
              bind:taskScheduleFilter
              bind:taskDependencyFilter
              bind:taskTagFilter
              bind:taskCustomFieldFilters
              bind:taskGroupBy
              bind:taskSortMode
              bind:taskSortDirection
              bind:showArchivedTasks
              bind:showInactiveSections
              bind:savedViewNameDraft
              onClose={() => {
                requestProjectToolbarPanelClose();
              }}
              onProjectSettingsDirtyChange={(dirty) => {
                projectSettingsDirty = dirty;
              }}
              onRevealInactive={() => {
                showInactiveProjects = true;
              }}
              onClearTaskFilters={clearTaskFilters}
              onSaveCurrentTaskView={() => { void saveCurrentTaskView(); }}
              onApplyTaskView={(view) => { void applyTaskView(view); }}
              onDeleteSavedTaskView={(view) => { void deleteSavedTaskView(view); }}
              onToggleTaskListColumn={(column) => { void toggleTaskListColumn(column); }}
            />
          {:else if toolbarDataError || toolbarLoadState?.status === "failed"}
            <div class="flex min-h-9 items-center justify-center gap-2 border-t border-border px-3 text-xs text-muted-foreground" role="alert">
              <span>{t("common.viewLoadFailed", t("projects.header.projectSettings"))}</span>
              <button
                type="button"
                class="font-medium text-foreground underline-offset-2 hover:underline"
                onclick={() => {
                  if (toolbarDataError) requestProjectToolbarData();
                  else requestProjectOptionalComponent("toolbar", true);
                }}
              >
                {t("common.retry")}
              </button>
            </div>
          {:else}
            <span class="sr-only" aria-busy="true">{t("common.loading")}</span>
          {/if}
        {/if}
        {#if selectedTaskIds.length > 0}
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
              bind:selectedTaskIds
              bind:showArchivedTasks
            />
          {:else if bulkActionsLoadState?.status === "failed"}
            <div class="flex min-h-9 items-center justify-center gap-2 border-t border-border px-3 text-xs text-muted-foreground" role="alert">
              <span>{t("common.viewLoadFailed", t("projects.bulk.selected", selectedTaskIds.length))}</span>
              <button
                type="button"
                class="font-medium text-foreground underline-offset-2 hover:underline"
                onclick={() => requestProjectOptionalComponent("bulk-actions", true)}
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
        {#if activeViewLoadState?.status === "ready"}
          {@const loadedView = activeViewLoadState.component}
          {#if loadedView.view === "list"}
            {@const ProjectListView = loadedView.component}
            <ProjectListView
              {selectedProjectId}
              {sections}
              {statuses}
              {priorities}
              {tasks}
              {allProjectTasks}
              {listTaskGroups}
              {taskGroupBy}
              {taskSortMode}
              {taskSortDirection}
              {taskListColumns}
              {taskListColumnWidths}
              {projectCustomFields}
              {selectedTaskId}
              {selectedTaskIds}
              {showArchivedTasks}
              onOpenTask={openTaskDetail}
              onSelectedTaskIdsChange={(taskIds) => {
                selectedTaskIds = taskIds;
              }}
              onRevealTask={revealCreatedTask}
              onTaskListColumnWidthsChange={(widths, options) => {
                void updateTaskListColumnWidths(widths, options);
              }}
            />
          {:else if loadedView.view === "kanban"}
            {@const ProjectKanbanView = loadedView.component}
            <ProjectKanbanView
              {tasks}
              {statuses}
              {priorities}
              {selectedTaskIds}
              {taskSortMode}
              {taskSortDirection}
              onOpenTask={openTaskDetail}
              onToggleTaskSelection={toggleTaskSelection}
            />
          {:else if loadedView.view === "calendar"}
            {@const CalendarView = loadedView.component}
            <div class="h-full min-h-112 overflow-hidden">
              <CalendarView
                eventFilter={projectCalendarEventFilter}
                createDefaults={projectCalendarCreateDefaults}
                initialViewMode={projectCalendarViewMode}
                onViewModeChange={(mode) => {
                  projectCalendarViewMode = mode;
                }}
              />
            </div>
          {:else if loadedView.view === "gantt"}
            {@const ProjectGanttView = loadedView.component}
            <ProjectGanttView
              tasks={tasks}
              statuses={statuses}
              sections={sections}
              todayDate={todayDate}
              onOpenTask={openTaskDetail}
              onToggleSectionCollapsed={(section) => {
                void toggleSectionCollapsed(section);
              }}
            />
          {:else if loadedView.view === "dashboard"}
            {@const ProjectDashboardView = loadedView.component}
            <ProjectDashboardView
              projectId={selectedProjectId}
              {tasks}
              {statuses}
              {priorities}
              {todayDate}
              {scheduledTaskIds}
              {scheduledThisWeekMinutes}
              onOpenTask={openTaskDetail}
            />
          {/if}
        {:else if activeViewLoadState?.status === "failed"}
          <div
            class="flex h-full min-h-40 flex-col items-center justify-center gap-3 p-4 text-center text-sm text-muted-foreground"
            role="alert"
          >
            <p>{t("common.viewLoadFailed", projectViewLabel(projects.activeView))}</p>
            <button
              type="button"
              class="min-h-9 rounded-md border border-border bg-background px-3 font-medium text-foreground hover:bg-accent"
              onclick={() => requestProjectView(projects.activeView, true)}
            >
              {t("common.retry")}
            </button>
          </div>
        {:else}
          <div
            data-projects-view-loading
            class="flex h-full min-h-40 items-center justify-center p-4 text-sm text-muted-foreground"
            aria-busy="true"
          >
            {t("common.loading")}
          </div>
        {/if}
      {:else}
        <ProjectEmptyState
          {selectedProjectId}
          bind:showInactiveProjects
          onProjectSelected={() => {
            selectedTaskId = null;
            closeProjectToolbarPanelImmediately();
          }}
        />
      {/if}
    </div>
  </section>

  {#if selectedProject && (taskFinderOpen || taskSearch.trim().length > 0)}
    {#if taskFinderLoadState?.status === "ready" && taskFinderLoadState.component.kind === "task-finder"}
      {@const ProjectTaskFinder = taskFinderLoadState.component.component}
      <ProjectTaskFinder
        {taskSearch}
        {matchingTaskCount}
        totalTaskCount={allProjectTasks.length}
        focusRequestId={taskFinderFocusRequestId}
        onTaskSearchChange={(value) => {
          taskSearch = value;
        }}
        onClose={closeTaskFinder}
        onClearAndClose={closeOrClearTaskFinder}
      />
    {:else if taskFinderLoadState?.status === "failed"}
      <div class="absolute inset-x-3 top-3 z-80 flex min-h-10 items-center justify-center gap-2 rounded-md border border-border bg-card px-3 text-xs text-muted-foreground shadow-lg" role="alert">
        <span>{t("common.viewLoadFailed", t("projects.finder.label"))}</span>
        <button
          type="button"
          class="font-medium text-foreground underline-offset-2 hover:underline"
          onclick={() => requestProjectOptionalComponent("task-finder", true)}
        >
          {t("common.retry")}
        </button>
      </div>
    {:else}
      <span class="sr-only" aria-busy="true">{t("common.loading")}</span>
    {/if}
  {/if}

  {#if selectedTaskId}
    {#if taskDetailDataReady && taskDetailLoadState?.status === "ready" && taskDetailLoadState.component.kind === "task-detail"}
      {@const ProjectTaskDetailPanel = taskDetailLoadState.component.component}
      <ProjectTaskDetailPanel
        taskId={selectedTaskId}
        layout={taskDetailModalLayout}
        showArchivedTasks={showArchivedTasks}
        showInactiveSections={showInactiveSections}
        onClose={() => {
          selectedTaskId = null;
        }}
        onOpenTask={(taskId) => {
          selectedTaskId = taskId;
        }}
        onShowArchivedTasks={() => {
          showArchivedTasks = true;
        }}
      />
    {:else if taskDetailDataError || taskDetailLoadState?.status === "failed"}
      <div class="absolute inset-0 z-80 flex items-center justify-center bg-black/40 p-4" role="alert">
        <div class="flex min-h-32 w-full max-w-sm flex-col items-center justify-center gap-3 rounded-lg border border-border bg-card p-4 text-center text-sm text-muted-foreground shadow-xl">
          <p>{t("common.viewLoadFailed", t("projects.detail.title"))}</p>
          <div class="flex gap-2">
            <button
              type="button"
              class="min-h-9 rounded-md border border-border bg-background px-3 font-medium text-foreground hover:bg-accent"
              onclick={() => {
                if (taskDetailDataError) requestTaskDetailData();
                else requestProjectOptionalComponent("task-detail", true);
              }}
            >
              {t("common.retry")}
            </button>
            <button
              type="button"
              class="min-h-9 rounded-md px-3 font-medium text-foreground hover:bg-accent"
              onclick={() => { selectedTaskId = null; }}
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

  {#if projectSettingsDiscardConfirmOpen}
    <ConfirmDialog
      title={t("calendar.view.discardUnsavedTitle")}
      message={t("calendar.view.changesLost")}
      confirmLabel={t("calendar.view.discard")}
      cancelLabel={t("common.cancelShortcut")}
      onConfirm={confirmDiscardProjectSettings}
      onCancel={cancelDiscardProjectSettings}
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
