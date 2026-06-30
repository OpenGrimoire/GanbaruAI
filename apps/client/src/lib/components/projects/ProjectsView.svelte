<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";
  import { onMount } from "svelte";
  import CalendarView from "$lib/components/calendar/CalendarView.svelte";
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
  import ProjectDashboardView from "./ProjectDashboardView.svelte";
  import ProjectKanbanView from "./ProjectKanbanView.svelte";
  import ProjectBulkActionController from "./ProjectBulkActionController.svelte";
  import ProjectEmptyState from "./ProjectEmptyState.svelte";
  import ProjectGanttView from "./ProjectGanttView.svelte";
  import ProjectListView from "./ProjectListView.svelte";
  import ProjectTaskFinder from "./ProjectTaskFinder.svelte";
  import ProjectTaskDetailPanel from "./ProjectTaskDetailPanel.svelte";
  import ProjectToolbarPanels from "./ProjectToolbarPanels.svelte";
  import ProjectWorkspaceHeader from "./ProjectWorkspaceHeader.svelte";

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
  let projectsRootElement = $state<HTMLDivElement | null>(null);

  const selectedProject = $derived(projects.selectedProject);
  const selectedGroup = $derived(projects.selectedGroup);
  const selectedProjectId = $derived(selectedProject?.id ?? null);
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

  function toggleProjectToolbarPanel(panel: ProjectToolbarPanel): void {
    projectToolbarPanel = projectToolbarPanel === panel ? null : panel;
  }

  function openTaskFinder(): void {
    taskFinderOpen = true;
    projectToolbarPanel = null;
    taskFinderFocusRequestId += 1;
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
    projectToolbarPanel = null;
    selectedTaskId = task.id;
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

<div bind:this={projectsRootElement} class="projects-view-root relative flex h-full min-h-0 overflow-hidden text-foreground" style="background-color: var(--cal-bg);">
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
            projectToolbarPanel = null;
          }}
          onToggleToolbarPanel={toggleProjectToolbarPanel}
        />
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
            projectToolbarPanel = null;
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
      </header>

      <div class="relative min-h-0 flex-1" style="background-color: var(--cal-bg);">
        {#if projects.activeView === "list"}
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
        {:else if projects.activeView === "kanban"}
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
        {:else if projects.activeView === "calendar"}
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
        {:else if projects.activeView === "gantt"}
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
        {:else if projects.activeView === "dashboard"}
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
      </div>
    {:else}
      <ProjectEmptyState
        {selectedProjectId}
        bind:showInactiveProjects
        onProjectSelected={() => {
          selectedTaskId = null;
          projectToolbarPanel = null;
        }}
      />
    {/if}
  </section>

  {#if selectedProject && (taskFinderOpen || taskSearch.trim().length > 0)}
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
  {/if}

  {#if selectedTaskId}
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
