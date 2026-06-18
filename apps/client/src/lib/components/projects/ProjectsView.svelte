<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";
  import { onMount } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import ArrowUpDown from "@lucide/svelte/icons/arrow-up-down";
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import CircleGauge from "@lucide/svelte/icons/circle-gauge";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import Funnel from "@lucide/svelte/icons/funnel";
  import Folder from "@lucide/svelte/icons/folder";
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import List from "@lucide/svelte/icons/list";
  import MoreHorizontal from "@lucide/svelte/icons/more-horizontal";
  import Plus from "@lucide/svelte/icons/plus";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import Save from "@lucide/svelte/icons/save";
  import Search from "@lucide/svelte/icons/search";
  import Timer from "@lucide/svelte/icons/timer";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import CalendarView from "$lib/components/calendar/CalendarView.svelte";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { formatCalendarDate, getEventColor } from "$lib/components/calendar/utils";
  import { createPresetPomodoroConfig } from "$lib/pomodoro/rhythm";
  import { cn } from "$lib/utils";
  import type {
    CalendarEvent,
    CalendarViewMode,
    EventColor,
  } from "$lib/components/calendar/types";
  import type {
    Project,
    ProjectCustomField,
    ProjectCustomFieldFilter,
    ProjectCustomFieldOption,
    ProjectCustomFieldValue,
    ProjectCoreTaskSortMode,
    ProjectLabel,
    ProjectLifecycleStatus,
    ProjectPriority,
    ProjectSavedTaskView,
    ProjectSection,
    ProjectStatus,
    ProjectTask,
    ProjectTaskDependencyFilter,
    ProjectTaskDueFilter,
    ProjectTaskGroupMode,
    ProjectTaskLabelFilter,
    ProjectTaskListColumn,
    ProjectTaskScheduleFilter,
    ProjectTaskSortDirection,
    ProjectTaskSortMode,
    ProjectTaskStatusFilter,
    ProjectViewId,
  } from "$lib/projects/types";
  import {
    PROJECT_CUSTOM_FIELD_TYPES,
    PROJECT_LIFECYCLE_STATUSES,
    PROJECT_PRIORITIES,
    PROJECT_TASK_GROUP_MODES,
    PROJECT_TASK_LIST_COLUMNS,
    PROJECT_TASK_SORT_MODES,
    PROJECT_TASK_TYPES,
    PROJECT_VIEW_IDS,
  } from "$lib/projects/types";
  import {
    customFieldIdFromCustomFieldReference,
    customFieldIdFromTaskListColumn,
    customFieldReference,
    customTaskListColumn,
    DEFAULT_TASK_LIST_COLUMNS,
    taskListColumnsForProject,
  } from "$lib/projects/task-list-columns";
  import {
    buildProjectTaskListGroups,
    buildProjectTaskView,
    projectTaskCustomFieldKey,
  } from "$lib/projects/task-view";
  import {
    projectListDropSortOrder,
    type ProjectListDropPosition,
  } from "$lib/projects/list-drag";
  import ProjectBoardView from "./ProjectBoardView.svelte";
  import ProjectIcon from "./ProjectIcon.svelte";
  import ProjectNavigator from "./ProjectNavigator.svelte";
  import ProjectGanttView from "./ProjectGanttView.svelte";
  import ProjectSettingsPanel from "./ProjectSettingsPanel.svelte";
  import ProjectSummaryView from "./ProjectSummaryView.svelte";
  import ProjectTaskDetailPanel from "./ProjectTaskDetailPanel.svelte";

  const projects = getProjects();
  const calendar = getCalendar();
  const theme = getTheme();
  const { t } = getLocalization();

  const TASK_STATUS_FILTERS: ProjectTaskStatusFilter[] = ["all", "open", "blocked", "done"];
  const TASK_DUE_FILTERS: ProjectTaskDueFilter[] = ["all", "overdue", "today", "week", "none", "range"];
  const TASK_SCHEDULE_FILTERS: ProjectTaskScheduleFilter[] = ["all", "scheduled", "unscheduled"];
  const TASK_DEPENDENCY_FILTERS: ProjectTaskDependencyFilter[] = ["all", "linked", "blocked_by", "blocking", "none"];
  const TASK_SORT_MODES: ProjectCoreTaskSortMode[] = [...PROJECT_TASK_SORT_MODES];
  const PROJECT_LIST_DRAG_MIME = "application/x-ganbaru-project-list-task";
  type TaskCreateTarget = "quick" | `section:${string}`;

  let showInactiveProjects = $state(false);
  let showInactiveSections = $state(false);
  let showArchivedTasks = $state(false);
  let taskSearch = $state("");
  let taskStatusFilter = $state<ProjectTaskStatusFilter>("all");
  let taskSectionFilter = $state<string | "all">("all");
  let taskPriorityFilter = $state<ProjectPriority | "all">("all");
  let taskDueFilter = $state<ProjectTaskDueFilter>("all");
  let taskDueRangeStart = $state("");
  let taskDueRangeEnd = $state("");
  let taskScheduleFilter = $state<ProjectTaskScheduleFilter>("all");
  let taskDependencyFilter = $state<ProjectTaskDependencyFilter>("all");
  let taskLabelFilter = $state<ProjectTaskLabelFilter>("all");
  let taskCustomFieldFilters = $state<ProjectCustomFieldFilter[]>([]);
  let taskGroupBy = $state<ProjectTaskGroupMode>("section");
  let taskSortMode = $state<ProjectTaskSortMode>("manual");
  let taskSortDirection = $state<ProjectTaskSortDirection>("asc");
  let taskListColumns = $state<ProjectTaskListColumn[]>([...DEFAULT_TASK_LIST_COLUMNS]);
  let projectCalendarViewMode = $state<CalendarViewMode>("week");
  let savedViewNameDraft = $state("");
  let savedViewSaving = $state(false);
  let savedViewError = $state<string | null>(null);
  let quickTaskTitle = $state("");
  let taskCreatePendingTarget = $state<TaskCreateTarget | null>(null);
  let taskCreateErrorTarget = $state<TaskCreateTarget | null>(null);
  let taskCreateErrorMessage = $state<string | null>(null);
  let sectionDraft = $state("");
  let sectionTaskDrafts = $state<Record<string, string>>({});
  let schedulingTaskId = $state<string | null>(null);
  let scheduleDate = $state("");
  let scheduleStartTime = $state("");
  let scheduleDurationMinutes = $state(60);
  let schedulePending = $state(false);
  let scheduleError = $state<string | null>(null);
  let bulkScheduleOpen = $state(false);
  let bulkScheduleDate = $state("");
  let bulkScheduleStartTime = $state("");
  let bulkScheduleDurationMinutes = $state(60);
  let listDraggingTaskId = $state<string | null>(null);
  let listDragOverSectionId = $state<string | null>(null);
  let listDragOverTaskId = $state<string | null>(null);
  let listDragOverPosition = $state<ProjectListDropPosition | "section" | null>(null);
  let listDropPendingTaskId = $state<string | null>(null);
  let selectedTaskId = $state<string | null>(null);
  let selectedTaskIds = $state<string[]>([]);
  let bulkTaskActionPending = $state(false);
  let bulkTaskError = $state<string | null>(null);
  let projectSettingsOpen = $state(false);
  let sectionNameDrafts = $state<Record<string, string>>({});

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
  const projectLabels = $derived(projects.labelsForProject(selectedProjectId));
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
  const visibleCustomFieldListFields = $derived.by(() => {
    const fieldsById = new Map(projectCustomFields.map((field) => [field.id, field]));
    return taskListColumns
      .map(customFieldIdFromTaskListColumn)
      .filter((fieldId): fieldId is string => fieldId !== undefined)
      .map((fieldId) => fieldsById.get(fieldId))
      .filter((field): field is ProjectCustomField => field !== undefined);
  });
  const taskLabelIdsByTaskId = $derived.by(() => {
    const labelsByTask = new Map<string, Set<string>>();
    for (const link of projects.taskLabelLinks) {
      if (!allProjectTaskIds.has(link.taskId)) continue;
      const labelIds = labelsByTask.get(link.taskId) ?? new Set<string>();
      labelIds.add(link.labelId);
      labelsByTask.set(link.taskId, labelIds);
    }
    return labelsByTask;
  });
  const todayDate = $derived(Temporal.Now.plainDateISO().toString());
  const taskFilterWeekEnd = $derived(Temporal.PlainDate.from(todayDate).add({ days: 7 }).toString());
  const normalizedTaskDueRangeStart = $derived(normalizeFilterDate(taskDueRangeStart));
  const normalizedTaskDueRangeEnd = $derived(normalizeFilterDate(taskDueRangeEnd));
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
  function calendarEventRootId(event: CalendarEvent): string {
    return event.recurringParentId ?? event.id.split("::")[0] ?? event.id;
  }

  function projectCalendarEventFilter(event: CalendarEvent): boolean {
    if (!selectedProjectId || event.projectId !== selectedProjectId) return false;
    if (!taskDataFiltersActive) return true;
    return eventIdsForMatchedTasks.has(event.id) || eventIdsForMatchedTasks.has(calendarEventRootId(event));
  }

  function projectCalendarCreateDefaults(input: {
    start: string;
    end: string;
    allDay?: boolean;
  }): Partial<CalendarEvent> {
    const project = selectedProject;
    if (!project) return {};
    let end = input.end;
    if (!input.allDay) {
      const startDate = input.start.split(" ")[0] ?? "";
      const startTime = input.start.split(" ")[1] ?? "";
      const nextWindow = scheduleWindowFor(
        startDate,
        startTime,
        project.defaultEventDurationMinutes,
      );
      if (nextWindow) end = nextWindow.end;
    }
    return {
      title: project.name,
      start: input.start,
      end,
      projectId: project.id,
      color: project.color,
      environmentId: project.workEnvironmentId,
      playlistId: project.focusPlaylistId,
      pomodoroConfig: !input.allDay && project.defaultPomodoroPresetKey
        ? createPresetPomodoroConfig(
            project.defaultPomodoroPresetKey,
            project.defaultIdleTimeoutMinutes ?? null,
          )
        : undefined,
    };
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
    scheduledTaskIds,
    nextScheduledStartByTaskId,
    taskLabelIdsByTaskId,
    dependencyBlockedTaskIds,
    dependencyBlockingTaskIds,
    today: todayDate,
    weekEnd: taskFilterWeekEnd,
    search: taskSearch,
    statusFilter: taskStatusFilter,
    sectionFilter: taskSectionFilter,
    priorityFilter: taskPriorityFilter,
    dueFilter: taskDueFilter,
    dueRangeStart: normalizedTaskDueRangeStart,
    dueRangeEnd: normalizedTaskDueRangeEnd,
    scheduleFilter: taskScheduleFilter,
    dependencyFilter: taskDependencyFilter,
    labelFilter: taskLabelFilter,
    customFields: projectCustomFields,
    customFieldOptions: projects.customFieldOptions,
    customFieldValuesByTaskField,
    customFieldOptionIdsByTaskField,
    customFieldFilters: taskCustomFieldFilters,
    groupBy: taskGroupBy,
    sortMode: taskSortMode,
    sortDirection: taskSortDirection,
  }));
  const tasks = $derived.by(() => {
    return taskView.tasks;
  });
  const listTaskGroups = $derived.by(() => {
    if (taskGroupBy === "section") return [];
    return buildProjectTaskListGroups({
      tasks,
      statuses,
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
  const taskDataFiltersActive = $derived.by(() =>
    taskSearch.trim().length > 0
    || taskStatusFilter !== "all"
    || taskSectionFilter !== "all"
    || taskPriorityFilter !== "all"
    || taskDueFilter !== "all"
    || taskScheduleFilter !== "all"
    || taskDependencyFilter !== "all"
    || taskLabelFilter !== "all"
    || taskCustomFieldFilters.length > 0
  );
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
  const nextUpcomingProjectEvent = $derived.by(() =>
    allProjectEvents.find((event) => event.start.slice(0, 10) >= todayDate)
  );
  const openTaskCount = $derived(tasks.filter((task) => {
    const status = projects.statusById(task.statusId);
    return !status?.terminal;
  }).length);
  const blockedTaskCount = $derived(tasks.filter((task) =>
    projects.statusById(task.statusId)?.category === "blocked"
  ).length);
  const completedTaskCount = $derived(tasks.filter((task) =>
    projects.statusById(task.statusId)?.terminal
  ).length);
  const selectedTask = $derived.by(() =>
    selectedTaskId ? allProjectTasks.find((task) => task.id === selectedTaskId) : undefined
  );


  onMount(() => {
    void projects.ensureLoaded().catch((error) => {
      console.error("load projects failed", error);
    });
  });

  $effect(() => {
    taskListColumns = taskListColumnsForProject(
      projects.viewPreferences,
      selectedProjectId,
      new Set(projectCustomFields.map((field) => field.id)),
    );
  });

  $effect(() => {
    if (selectedTaskId && !selectedTask) {
      selectedTaskId = null;
    }
  });

  $effect(() => {
    const visibleTaskIds = new Set(allProjectTasks.map((task) => task.id));
    const nextSelectedTaskIds = selectedTaskIds.filter((taskId) => visibleTaskIds.has(taskId));
    if (nextSelectedTaskIds.length !== selectedTaskIds.length) {
      selectedTaskIds = nextSelectedTaskIds;
    }
  });

  $effect(() => {
    if (taskSectionFilter !== "all" && !sections.some((section) => section.id === taskSectionFilter)) {
      taskSectionFilter = "all";
    }
    if (
      taskLabelFilter !== "all"
      && taskLabelFilter !== "none"
      && !projectLabels.some((label) => label.id === taskLabelFilter)
    ) {
      taskLabelFilter = "all";
    }
    const fieldIds = new Set(projectCustomFields.map((field) => field.id));
    const optionIds = new Set(
      projectCustomFields.flatMap((field) => projects.customFieldOptionsForField(field.id).map((option) => option.id)),
    );
    const nextCustomFieldFilters = taskCustomFieldFilters.filter((filter) =>
      customFieldFilterStillExists(filter, fieldIds, optionIds)
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

  function priorityLabel(priority: ProjectPriority): string {
    if (priority === "low") return t("projects.priority.low");
    if (priority === "high") return t("projects.priority.high");
    if (priority === "urgent") return t("projects.priority.urgent");
    return t("projects.priority.normal");
  }

  function taskListColumnLabel(column: ProjectTaskListColumn): string {
    const customFieldId = customFieldIdFromTaskListColumn(column);
    if (customFieldId) {
      return projectCustomFields.find((field) => field.id === customFieldId)?.name
        ?? t("projects.columns.customField");
    }
    if (column === "priority") return t("projects.columns.priority");
    if (column === "estimate") return t("projects.columns.estimate");
    if (column === "due") return t("projects.columns.due");
    if (column === "scheduled") return t("projects.columns.scheduled");
    if (column === "dependencies") return t("projects.columns.dependencies");
    return t("projects.columns.status");
  }

  function taskListColumnVisible(column: ProjectTaskListColumn): boolean {
    return taskListColumns.includes(column);
  }

  async function toggleTaskListColumn(column: ProjectTaskListColumn): Promise<void> {
    if (!selectedProjectId) return;
    const nextColumns = taskListColumnVisible(column)
      ? taskListColumns.filter((entry) => entry !== column)
      : [...taskListColumns, column];
    taskListColumns = nextColumns;
    await projects.saveTaskListColumns(selectedProjectId, nextColumns);
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

  function projectLifecycleLabel(status: ProjectLifecycleStatus): string {
    if (status === "hidden") return t("projects.lifecycle.hidden");
    if (status === "archived") return t("projects.lifecycle.archived");
    return t("projects.lifecycle.active");
  }

  function projectLifecycleBadgeClass(status: ProjectLifecycleStatus): string {
    if (status === "hidden") return "border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-300";
    if (status === "archived") return "border-muted-foreground/30 bg-muted/50 text-muted-foreground";
    return "border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
  }

  function taskStatusFilterLabel(filter: ProjectTaskStatusFilter): string {
    if (filter === "open") return t("projects.filters.open");
    if (filter === "blocked") return t("projects.filters.blocked");
    if (filter === "done") return t("projects.filters.done");
    return t("projects.filters.allStatuses");
  }

  function taskDueFilterLabel(filter: ProjectTaskDueFilter): string {
    if (filter === "overdue") return t("projects.filters.overdue");
    if (filter === "today") return t("projects.filters.today");
    if (filter === "week") return t("projects.filters.thisWeek");
    if (filter === "none") return t("projects.filters.noDueDate");
    if (filter === "range") return t("projects.filters.dueRange");
    return t("projects.filters.allDueDates");
  }

  function taskScheduleFilterLabel(filter: ProjectTaskScheduleFilter): string {
    if (filter === "scheduled") return t("projects.filters.scheduled");
    if (filter === "unscheduled") return t("projects.filters.unscheduled");
    return t("projects.filters.allSchedule");
  }

  function taskDependencyFilterLabel(filter: ProjectTaskDependencyFilter): string {
    if (filter === "linked") return t("projects.filters.hasDependencies");
    if (filter === "blocked_by") return t("projects.filters.blockedByDependencies");
    if (filter === "blocking") return t("projects.filters.blockingDependencies");
    if (filter === "none") return t("projects.filters.noDependencies");
    return t("projects.filters.allDependencies");
  }

  function taskLabelFilterLabel(filter: ProjectTaskLabelFilter): string {
    if (filter === "all") return t("projects.filters.allLabels");
    if (filter === "none") return t("projects.filters.noLabels");
    return projectLabels.find((label) => label.id === filter)?.name ?? t("projects.filters.allLabels");
  }

  function taskGroupModeLabel(mode: ProjectTaskGroupMode): string {
    if (mode === "status") return t("projects.grouping.status");
    if (mode === "priority") return t("projects.grouping.priority");
    if (mode === "due") return t("projects.grouping.due");
    if (mode === "scheduled") return t("projects.grouping.scheduled");
    return t("projects.grouping.section");
  }

  function taskListGroupTitle(value: string): string {
    if (taskGroupBy === "status") {
      return statuses.find((status) => status.id === value)?.name ?? t("projects.grouping.missingStatus");
    }
    if (taskGroupBy === "priority" && PROJECT_PRIORITIES.includes(value as ProjectPriority)) {
      return priorityLabel(value as ProjectPriority);
    }
    if (taskGroupBy === "due") {
      if (value === "overdue") return t("projects.filters.overdue");
      if (value === "today") return t("projects.filters.today");
      if (value === "week") return t("projects.filters.thisWeek");
      if (value === "later") return t("projects.grouping.laterDue");
      if (value === "earlier") return t("projects.grouping.earlierDue");
      return t("projects.filters.noDueDate");
    }
    if (value === "scheduled") return t("projects.filters.scheduled");
    if (value === "unscheduled") return t("projects.filters.unscheduled");
    return value;
  }

  function taskCustomFieldFilterFor(fieldId: string): ProjectCustomFieldFilter | undefined {
    return taskCustomFieldFilters.find((filter) => filter.fieldId === fieldId);
  }

  function setTaskCustomFieldFilter(filter: ProjectCustomFieldFilter | undefined): void {
    if (!filter) return;
    taskCustomFieldFilters = [
      ...taskCustomFieldFilters.filter((entry) => entry.fieldId !== filter.fieldId),
      filter,
    ];
  }

  function clearTaskCustomFieldFilter(fieldId: string): void {
    taskCustomFieldFilters = taskCustomFieldFilters.filter((filter) => filter.fieldId !== fieldId);
  }

  function customFieldFilterStillExists(
    filter: ProjectCustomFieldFilter,
    fieldIds: ReadonlySet<string>,
    optionIds: ReadonlySet<string>,
  ): boolean {
    if (!fieldIds.has(filter.fieldId)) return false;
    return filter.mode !== "option" || optionIds.has(filter.optionId);
  }

  function customFieldFilterButtonClass(active: boolean): string {
    return active
      ? "bg-background text-foreground shadow-sm"
      : "text-muted-foreground hover:bg-background/60 hover:text-foreground";
  }

  function customFieldSortMode(field: ProjectCustomField): ProjectTaskSortMode {
    return customFieldReference(field.id);
  }

  function taskSortModeLabel(mode: ProjectTaskSortMode): string {
    const customFieldId = customFieldIdFromCustomFieldReference(mode);
    if (customFieldId) {
      return projectCustomFields.find((field) => field.id === customFieldId)?.name
        ?? t("projects.columns.customField");
    }
    if (mode === "status") return t("projects.sort.status");
    if (mode === "section") return t("projects.sort.section");
    if (mode === "priority") return t("projects.sort.priority");
    if (mode === "due") return t("projects.sort.due");
    if (mode === "scheduled") return t("projects.sort.scheduled");
    if (mode === "created") return t("projects.sort.created");
    if (mode === "updated") return t("projects.sort.updated");
    if (mode === "estimate") return t("projects.sort.estimate");
    return t("projects.sort.manual");
  }

  function taskSortDirectionLabel(direction: ProjectTaskSortDirection): string {
    return direction === "asc" ? t("projects.sort.ascending") : t("projects.sort.descending");
  }

  function taskArchivedBadgeClass(task: ProjectTask): string {
    return task.archivedAt
      ? "border-muted-foreground/30 bg-muted/50 text-muted-foreground"
      : "border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
  }

  function clearTaskFilters(): void {
    taskSearch = "";
    taskStatusFilter = "all";
    taskSectionFilter = "all";
    taskPriorityFilter = "all";
    taskDueFilter = "all";
    taskDueRangeStart = "";
    taskDueRangeEnd = "";
    taskScheduleFilter = "all";
    taskDependencyFilter = "all";
    taskLabelFilter = "all";
    taskCustomFieldFilters = [];
    taskGroupBy = "section";
    taskSortMode = "manual";
    taskSortDirection = "asc";
  }

  function taskViewSnapshot(name: string, viewId: string): ProjectSavedTaskView | undefined {
    if (!selectedProjectId) return undefined;
    return {
      id: viewId,
      projectId: selectedProjectId,
      name,
      viewId: projects.activeView,
      search: taskSearch,
      statusFilter: taskStatusFilter,
      sectionFilter: taskSectionFilter,
      priorityFilter: taskPriorityFilter,
      dueFilter: taskDueFilter,
      dueRangeStart: normalizedTaskDueRangeStart ?? "",
      dueRangeEnd: normalizedTaskDueRangeEnd ?? "",
      scheduleFilter: taskScheduleFilter,
      dependencyFilter: taskDependencyFilter,
      labelFilter: taskLabelFilter,
      customFieldFilters: taskCustomFieldFilters,
      sortMode: taskSortMode,
      sortDirection: taskSortDirection,
      groupBy: taskGroupBy,
      collapsedSectionIds: sections
        .filter((section) => section.collapsed)
        .map((section) => section.id),
      showArchivedTasks,
      visibleColumns: taskListColumns,
      updatedAt: new Date().toISOString(),
    };
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
    taskSearch = view.search;
    taskStatusFilter = view.statusFilter;
    taskSectionFilter = view.sectionFilter;
    taskPriorityFilter = view.priorityFilter;
    taskDueFilter = view.dueFilter;
    taskDueRangeStart = view.dueRangeStart;
    taskDueRangeEnd = view.dueRangeEnd;
    taskScheduleFilter = view.scheduleFilter;
    taskDependencyFilter = view.dependencyFilter;
    taskLabelFilter = view.labelFilter;
    taskCustomFieldFilters = view.customFieldFilters;
    taskGroupBy = view.groupBy;
    taskSortMode = view.sortMode;
    taskSortDirection = view.sortDirection;
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



  function nextPriority(priority: ProjectPriority): ProjectPriority {
    if (priority === "low") return "normal";
    if (priority === "normal") return "high";
    if (priority === "high") return "urgent";
    return "low";
  }

  function tasksForSection(section: ProjectSection): ProjectTask[] {
    return tasks.filter((task) => task.sectionId === section.id && !task.parentTaskId);
  }

  function listDragEnabled(): boolean {
    return taskGroupBy === "section" && taskSortMode === "manual";
  }

  function subtasksForTask(parent: ProjectTask): ProjectTask[] {
    return showArchivedTasks
      ? projects.subtasksForTaskIncludingArchived(parent.id)
      : projects.subtasksForTask(parent.id);
  }

  function taskById(taskId: string): ProjectTask | undefined {
    return allProjectTasks.find((task) => task.id === taskId);
  }

  function resetListDragTarget(): void {
    listDragOverSectionId = null;
    listDragOverTaskId = null;
    listDragOverPosition = null;
  }

  function listDragTaskId(event: DragEvent): string | null {
    return event.dataTransfer?.getData(PROJECT_LIST_DRAG_MIME) || listDraggingTaskId;
  }

  function canDropListTask(task: ProjectTask | undefined, section: ProjectSection): task is ProjectTask {
    return listDragEnabled()
      && !!task
      && !task.archivedAt
      && !task.parentTaskId
      && !section.archivedAt
      && !section.hiddenAt
      && task.projectId === section.projectId;
  }

  function handleListTaskDragStart(event: DragEvent, task: ProjectTask): void {
    if (!listDragEnabled() || task.archivedAt || task.parentTaskId) {
      event.preventDefault();
      return;
    }
    listDraggingTaskId = task.id;
    event.dataTransfer?.setData(PROJECT_LIST_DRAG_MIME, task.id);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function handleListTaskDragEnd(): void {
    listDraggingTaskId = null;
    listDropPendingTaskId = null;
    resetListDragTarget();
  }

  function listRowDropPosition(event: DragEvent): ProjectListDropPosition {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return event.clientY >= rect.top + rect.height / 2 ? "after" : "before";
  }

  function handleListRowDragOver(event: DragEvent, section: ProjectSection, task: ProjectTask): void {
    const dragged = taskById(listDragTaskId(event) ?? "");
    if (!canDropListTask(dragged, section) || dragged.id === task.id) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    listDragOverSectionId = section.id;
    listDragOverTaskId = task.id;
    listDragOverPosition = listRowDropPosition(event);
  }

  function handleListSectionDragOver(event: DragEvent, section: ProjectSection): void {
    const dragged = taskById(listDragTaskId(event) ?? "");
    if (!canDropListTask(dragged, section)) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    listDragOverSectionId = section.id;
    listDragOverTaskId = null;
    listDragOverPosition = "section";
  }

  async function dropListTask(
    event: DragEvent,
    section: ProjectSection,
    targetTask?: ProjectTask,
    position?: ProjectListDropPosition,
  ): Promise<void> {
    event.preventDefault();
    event.stopPropagation();
    const dragged = taskById(listDragTaskId(event) ?? "");
    if (!canDropListTask(dragged, section) || dragged.id === targetTask?.id) {
      resetListDragTarget();
      return;
    }

    const orderedTasks = tasksForSection(section);
    const nextSectionSortOrder = projectListDropSortOrder({
      orderedTasks,
      draggedTaskId: dragged.id,
      overTaskId: targetTask?.id,
      position,
      sortDirection: taskSortDirection,
    });

    if (
      dragged.sectionId === section.id
      && dragged.sectionSortOrder === nextSectionSortOrder
    ) {
      resetListDragTarget();
      return;
    }

    listDropPendingTaskId = dragged.id;
    resetListDragTarget();
    try {
      await projects.updateTask(dragged, {
        sectionId: section.id,
        sectionSortOrder: nextSectionSortOrder,
      });
    } finally {
      listDropPendingTaskId = null;
      listDraggingTaskId = null;
    }
  }

  function listDropMarkerVisible(
    section: ProjectSection,
    task: ProjectTask,
    position: ProjectListDropPosition,
  ): boolean {
    return listDragOverSectionId === section.id
      && listDragOverTaskId === task.id
      && listDragOverPosition === position;
  }

  function statusForTask(task: ProjectTask): ProjectStatus | undefined {
    return projects.statusById(task.statusId);
  }

  function taskSelected(task: ProjectTask): boolean {
    return selectedTaskIdSet.has(task.id);
  }

  function toggleTaskSelection(task: ProjectTask): void {
    selectedTaskIds = taskSelected(task)
      ? selectedTaskIds.filter((taskId) => taskId !== task.id)
      : [...selectedTaskIds, task.id];
  }

  function selectFilteredTasks(): void {
    const nextIds = new Set(selectedTaskIds);
    for (const task of selectableTasks) {
      nextIds.add(task.id);
    }
    selectedTaskIds = Array.from(nextIds);
  }

  function clearTaskSelection(): void {
    selectedTaskIds = [];
    bulkTaskError = null;
    bulkScheduleOpen = false;
  }

  function blockedByDependencies(task: ProjectTask) {
    return projects.dependenciesBlockingTask(task.id);
  }

  function blocksDependencies(task: ProjectTask) {
    return projects.dependenciesBlockedByTask(task.id);
  }

  function labelsForTask(task: ProjectTask): ProjectLabel[] {
    return projects.labelsForTask(task.id);
  }

  function visibleTaskLabels(task: ProjectTask): ProjectLabel[] {
    return labelsForTask(task).slice(0, 3);
  }

  function hiddenTaskLabelCount(task: ProjectTask): number {
    return Math.max(0, labelsForTask(task).length - visibleTaskLabels(task).length);
  }

  function customFieldOptions(field: ProjectCustomField): ProjectCustomFieldOption[] {
    return projects.customFieldOptionsForField(field.id);
  }

  function customFieldDisplayValue(task: ProjectTask, field: ProjectCustomField): string | undefined {
    const value = projects.customFieldValueForTask(task.id, field.id);
    if (field.fieldType === "text" || field.fieldType === "url") {
      const text = value?.textValue?.trim();
      return text || undefined;
    }
    if (field.fieldType === "number") {
      const number = value?.numberValue;
      if (number === undefined) return undefined;
      return Number.isInteger(number) ? number.toFixed(0) : number.toString();
    }
    if (field.fieldType === "date") {
      return value?.dateValue || undefined;
    }
    if (field.fieldType === "checkbox") {
      if (value?.checkboxValue === undefined) return undefined;
      return value.checkboxValue
        ? t("projects.customFields.checked")
        : t("projects.customFields.unchecked");
    }
    const optionNames = projects.customFieldOptionValuesForTask(task.id, field.id)
      .map((option) => option.name);
    return optionNames.length > 0 ? optionNames.join(", ") : undefined;
  }

  function labelColorDotStyle(color: EventColor | undefined): string {
    if (color === undefined) return "";
    return `background-color: ${getEventColor(color, theme.current).bg};`;
  }

  function labelColorSwatchClass(color: EventColor | undefined): string {
    return color === undefined ? "border-border bg-muted/50" : "border-transparent";
  }

  function sectionForTask(task: ProjectTask): ProjectSection | undefined {
    return sections.find((section) => section.id === task.sectionId);
  }

  function adjacentSection(section: ProjectSection, direction: -1 | 1): ProjectSection | undefined {
    const index = sections.findIndex((entry) => entry.id === section.id);
    if (index < 0) return undefined;
    return sections[index + direction];
  }

  function adjacentTaskInSection(task: ProjectTask, direction: -1 | 1): ProjectTask | undefined {
    const ordered = projects.topLevelTasksForSection(task.projectId, task.sectionId);
    const index = ordered.findIndex((entry) => entry.id === task.id);
    if (index < 0) return undefined;
    return ordered[index + direction];
  }

  function statusBadgeClass(status: ProjectStatus | undefined): string {
    if (status?.category === "done") return "border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
    if (status?.category === "blocked") return "border-destructive/40 bg-destructive/10 text-destructive";
    if (status?.category === "active") return "border-sky-500/40 bg-sky-500/10 text-sky-700 dark:text-sky-300";
    return "border-border bg-muted/50 text-muted-foreground";
  }

  function priorityClass(priority: ProjectPriority): string {
    if (priority === "urgent") return "border-destructive/40 bg-destructive/10 text-destructive";
    if (priority === "high") return "border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-300";
    if (priority === "low") return "border-muted-foreground/20 bg-muted/30 text-muted-foreground";
    return "border-border bg-background/70 text-foreground";
  }

  function revealCreatedTask(task: ProjectTask | undefined): void {
    if (!task) return;
    projects.activeView = "list";
    taskSearch = "";
    taskStatusFilter = "all";
    taskSectionFilter = "all";
    taskPriorityFilter = "all";
    taskDueFilter = "all";
    taskDueRangeStart = "";
    taskDueRangeEnd = "";
    taskScheduleFilter = "all";
    taskDependencyFilter = "all";
    taskLabelFilter = "all";
    taskCustomFieldFilters = [];
    selectedTaskId = task.id;
    selectedTaskIds = [];
    showArchivedTasks = false;
    showInactiveSections = false;
  }

  function sectionTaskCreateTarget(sectionId: string): TaskCreateTarget {
    return `section:${sectionId}`;
  }

  function clearTaskCreateError(target: TaskCreateTarget): void {
    if (taskCreateErrorTarget !== target) return;
    taskCreateErrorTarget = null;
    taskCreateErrorMessage = null;
  }

  function setTaskCreateError(target: TaskCreateTarget, message: string): void {
    taskCreateErrorTarget = target;
    taskCreateErrorMessage = message;
  }

  function taskCreateErrorFor(target: TaskCreateTarget): string | null {
    return taskCreateErrorTarget === target ? taskCreateErrorMessage : null;
  }

  function taskCreateFailedMessage(error: unknown): string {
    const message = error instanceof Error ? error.message : String(error);
    return t("projects.tasks.createFailed", message);
  }

  async function createTaskFromDraft(target: TaskCreateTarget, title: string, sectionId?: string): Promise<ProjectTask | undefined> {
    const projectId = selectedProjectId;
    if (!projectId) {
      setTaskCreateError(target, t("projects.tasks.selectProjectFirst"));
      return undefined;
    }
    const displayTitle = title.trim();
    if (!displayTitle) return undefined;
    taskCreatePendingTarget = target;
    clearTaskCreateError(target);
    try {
      return await projects.addTask(projectId, displayTitle, sectionId);
    } catch (error) {
      console.error("create project task failed", error);
      setTaskCreateError(target, taskCreateFailedMessage(error));
      return undefined;
    } finally {
      if (taskCreatePendingTarget === target) {
        taskCreatePendingTarget = null;
      }
    }
  }

  async function submitQuickTask(): Promise<void> {
    const title = quickTaskTitle.trim();
    if (!title) return;
    const createdTask = await createTaskFromDraft("quick", title);
    if (!createdTask) return;
    quickTaskTitle = "";
    revealCreatedTask(createdTask);
  }

  async function submitSectionTask(sectionId: string): Promise<void> {
    const target = sectionTaskCreateTarget(sectionId);
    const title = (sectionTaskDrafts[sectionId] ?? "").trim();
    if (!title) return;
    const createdTask = await createTaskFromDraft(target, title, sectionId);
    if (!createdTask) return;
    sectionTaskDrafts = { ...sectionTaskDrafts, [sectionId]: "" };
    revealCreatedTask(createdTask);
  }

  async function submitSection(): Promise<void> {
    if (!selectedProjectId) return;
    await projects.addSection(selectedProjectId, sectionDraft);
    sectionDraft = "";
  }

  function sectionDraftDirty(section: ProjectSection): boolean {
    return (sectionNameDrafts[section.id] ?? section.name) !== section.name;
  }

  async function saveSection(section: ProjectSection): Promise<void> {
    const name = (sectionNameDrafts[section.id] ?? section.name).trim();
    if (!name) return;
    await projects.updateSection(section, { name });
  }

  async function hideSection(section: ProjectSection): Promise<void> {
    await projects.hideSection(section);
  }

  async function archiveSection(section: ProjectSection): Promise<void> {
    await projects.archiveSection(section);
  }

  async function restoreSection(section: ProjectSection): Promise<void> {
    await projects.restoreSection(section);
  }

  async function moveSection(section: ProjectSection, direction: -1 | 1): Promise<void> {
    await projects.moveSection(section, direction);
  }

  async function toggleSectionCollapsed(section: ProjectSection): Promise<void> {
    await projects.updateSection(section, { collapsed: !section.collapsed });
  }

  function openTaskDetail(task: ProjectTask): void {
    projectSettingsOpen = false;
    selectedTaskId = task.id;
  }

  function openProjectSettings(): void {
    selectedTaskId = null;
    projectSettingsOpen = true;
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

  async function moveTaskWithinSection(task: ProjectTask, direction: -1 | 1): Promise<void> {
    if (taskSortMode !== "manual") return;
    await projects.moveTaskInSection(task, direction);
  }



  async function runBulkTaskAction(action: () => Promise<void>): Promise<void> {
    if (selectedTasks.length === 0) return;
    bulkTaskActionPending = true;
    bulkTaskError = null;
    try {
      await action();
      selectedTaskIds = [];
    } catch (error) {
      bulkTaskError = t(
        "projects.bulk.actionFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      bulkTaskActionPending = false;
    }
  }

  async function bulkSetSelectedStatus(status: ProjectStatus | undefined): Promise<void> {
    if (!status) return;
    await runBulkTaskAction(() => projects.setTasksStatus(selectedTasks, status.id));
  }

  async function bulkSetSelectedPriority(priority: ProjectPriority): Promise<void> {
    await runBulkTaskAction(() => projects.setTasksPriority(selectedTasks, priority));
  }

  async function bulkArchiveSelectedTasks(): Promise<void> {
    await runBulkTaskAction(() => projects.archiveTasks(selectedTasks));
  }

  async function bulkRestoreSelectedTasks(): Promise<void> {
    showArchivedTasks = true;
    await runBulkTaskAction(() => projects.restoreTasks(selectedTasks));
  }

  function eventDurationMinutes(event: CalendarEvent): number {
    try {
      const start = Temporal.PlainDateTime.from(event.start.replace(" ", "T"));
      const end = Temporal.PlainDateTime.from(event.end.replace(" ", "T"));
      return Math.max(0, Math.round(start.until(end).total({ unit: "minutes" })));
    } catch {
      return 0;
    }
  }

  function thisWeekScheduledMinutes(): number {
    const weekEnd = Temporal.PlainDate.from(todayDate).add({ days: 7 }).toString();
    return allProjectEvents
      .filter((event) => {
        const eventDate = event.start.slice(0, 10);
        return eventDate >= todayDate && eventDate <= weekEnd;
      })
      .reduce((total, event) => total + eventDurationMinutes(event), 0);
  }

  function formatMinutesAsHours(minutes: number): string {
    if (minutes < 60) return t("projects.summary.minutes", minutes);
    const hours = minutes / 60;
    return t("projects.summary.hours", Number.isInteger(hours) ? hours.toFixed(0) : hours.toFixed(1));
  }

  function projectHeaderEventLabel(event: CalendarEvent): string {
    const title = event.title.trim() || t("calendar.event.noTitle");
    return t("projects.header.nextEvent", title, event.start);
  }

  function defaultScheduleStart(): { date: string; time: string } {
    const now = Temporal.Now.plainDateTimeISO();
    const dayStart = now.with({ hour: 0, minute: 0, second: 0, millisecond: 0, microsecond: 0, nanosecond: 0 });
    const totalMinutes = now.hour * 60 + now.minute;
    const roundedMinutes = Math.ceil(totalMinutes / 15) * 15;
    const start = dayStart.add({ minutes: roundedMinutes });
    return {
      date: start.toPlainDate().toString(),
      time: `${String(start.hour).padStart(2, "0")}:${String(start.minute).padStart(2, "0")}`,
    };
  }

  function openScheduleForm(task: ProjectTask): void {
    const start = defaultScheduleStart();
    const project = projects.projectById(task.projectId);
    schedulingTaskId = task.id;
    scheduleDate = task.startDate ?? task.dueDate ?? start.date;
    scheduleStartTime = start.time;
    scheduleDurationMinutes = project?.defaultEventDurationMinutes ?? 60;
    scheduleError = null;
  }

  function closeScheduleForm(): void {
    schedulingTaskId = null;
    scheduleError = null;
  }

  function selectedSchedulableTasks(): ProjectTask[] {
    return selectedTasks.filter((task) => !task.archivedAt);
  }

  function openBulkScheduleForm(): void {
    const start = defaultScheduleStart();
    bulkScheduleOpen = true;
    bulkScheduleDate = start.date;
    bulkScheduleStartTime = start.time;
    bulkScheduleDurationMinutes = selectedProject?.defaultEventDurationMinutes ?? 60;
    bulkTaskError = null;
  }

  function closeBulkScheduleForm(): void {
    bulkScheduleOpen = false;
    bulkTaskError = null;
  }

  function scheduledLinksForTask(taskId: string) {
    return projects.eventLinksForTask(taskId).filter((link) => link.linkKind === "scheduled");
  }

  function scheduledLabel(taskId: string): string | null {
    const count = scheduledLinksForTask(taskId).length;
    const nextStart = nextScheduledStartByTaskId.get(taskId);
    if (nextStart) return t("projects.schedule.nextScheduled", nextStart.slice(0, 16), count);
    return count > 0 ? t("projects.schedule.scheduledCount", count) : null;
  }

  function formatScheduleWindowStart(start: Temporal.PlainDateTime, durationMinutes: number): { start: string; end: string } {
    const end = start.add({ minutes: durationMinutes });
    const startTime = `${String(start.hour).padStart(2, "0")}:${String(start.minute).padStart(2, "0")}`;
    const endTime = `${String(end.hour).padStart(2, "0")}:${String(end.minute).padStart(2, "0")}`;
    return {
      start: `${start.toPlainDate().toString()} ${startTime}`,
      end: `${end.toPlainDate().toString()} ${endTime}`,
    };
  }

  function scheduleWindowFor(
    date: string,
    startTime: string,
    durationMinutes: number,
  ): { start: string; end: string } | null {
    const duration = Math.round(Number(durationMinutes));
    if (duration <= 0 || !date || !startTime) return null;
    try {
      return formatScheduleWindowStart(Temporal.PlainDateTime.from(`${date}T${startTime}`), duration);
    } catch {
      return null;
    }
  }

  function scheduleWindow(): { start: string; end: string } | null {
    return scheduleWindowFor(scheduleDate, scheduleStartTime, scheduleDurationMinutes);
  }

  async function createScheduledTaskBlock(
    task: ProjectTask,
    project: Project,
    scheduledWindow: { start: string; end: string },
  ): Promise<CalendarEvent> {
    let createdEventId: string | null = null;
    try {
      const event = await calendar.addBlock({
        title: task.title,
        start: scheduledWindow.start,
        end: scheduledWindow.end,
        projectId: project.id,
        color: project.color,
        pomodoroConfig: project.defaultPomodoroPresetKey
          ? createPresetPomodoroConfig(
              project.defaultPomodoroPresetKey,
              project.defaultIdleTimeoutMinutes ?? null,
            )
          : undefined,
      });
      createdEventId = event.id;
      const scheduledDate = scheduledWindow.start.slice(0, 10);
      await projects.linkTaskEvent(task.id, event.id, "scheduled");
      await projects.updateTask(task, {
        startDate: scheduledDate,
        targetEndDate: scheduledDate,
        dueDate: task.dueDate ?? scheduledDate,
      });
      return event;
    } catch (error) {
      if (createdEventId) {
        await calendar.deleteBlock(createdEventId).catch((deleteError) => {
          console.error("delete failed scheduled event after task link error", deleteError);
        });
      }
      throw error;
    }
  }

  async function scheduleTask(task: ProjectTask): Promise<void> {
    const project = projects.projectById(task.projectId);
    const scheduledWindow = scheduleWindow();
    if (!project || !scheduledWindow) {
      scheduleError = t("projects.schedule.invalid");
      return;
    }
    schedulePending = true;
    scheduleError = null;
    try {
      await createScheduledTaskBlock(task, project, scheduledWindow);
      closeScheduleForm();
      projects.activeView = "calendar";
    } catch (error) {
      scheduleError = t("projects.schedule.failed", error instanceof Error ? error.message : String(error));
    } finally {
      schedulePending = false;
    }
  }

  async function bulkScheduleSelectedTasks(): Promise<void> {
    const project = selectedProject;
    const schedulableTasks = selectedSchedulableTasks();
    const duration = Math.round(Number(bulkScheduleDurationMinutes));
    if (!project || schedulableTasks.length === 0 || duration <= 0 || !bulkScheduleDate || !bulkScheduleStartTime) {
      bulkTaskError = t("projects.schedule.invalid");
      return;
    }
    let cursor: Temporal.PlainDateTime;
    try {
      cursor = Temporal.PlainDateTime.from(`${bulkScheduleDate}T${bulkScheduleStartTime}`);
    } catch {
      bulkTaskError = t("projects.schedule.invalid");
      return;
    }

    bulkTaskActionPending = true;
    bulkTaskError = null;
    let scheduledCount = 0;
    try {
      for (const task of schedulableTasks) {
        const scheduledWindow = formatScheduleWindowStart(cursor, duration);
        await createScheduledTaskBlock(task, project, scheduledWindow);
        scheduledCount += 1;
        cursor = cursor.add({ minutes: duration });
      }
      bulkScheduleOpen = false;
      clearTaskSelection();
      projects.activeView = "calendar";
    } catch (error) {
      bulkTaskError = t(
        "projects.bulk.scheduleFailed",
        scheduledCount,
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      bulkTaskActionPending = false;
    }
  }
</script>

<div class="flex h-full min-h-0 bg-background text-foreground">
  <ProjectNavigator
    selectedProjectId={selectedProjectId}
    showInactiveProjects={showInactiveProjects}
    onShowInactiveProjectsChange={(value) => {
      showInactiveProjects = value;
    }}
    onProjectSelected={() => {
      selectedTaskId = null;
      projectSettingsOpen = false;
    }}
  />

  <section class="flex min-w-0 flex-1 flex-col">
    {#if selectedProject && selectedGroup}
      <header class="flex shrink-0 flex-col gap-2 border-b border-border bg-card/40 px-3 py-2">
        <div class="flex min-w-0 items-center gap-2">
          <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-accent text-accent-foreground">
            <ProjectIcon name={selectedProject.icon} size={16} />
          </span>
          <div class="min-w-0 flex-1">
            <div class="truncate text-[0.933333rem] font-semibold leading-5">{selectedProject.name}</div>
            <div class="flex min-w-0 items-center gap-1.5">
              <span class="truncate text-[0.733333rem] text-muted-foreground">{selectedGroup.name}</span>
              {#if selectedProject.status !== "active"}
                <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(selectedProject.status))}>
                  {projectLifecycleLabel(selectedProject.status)}
                </span>
              {/if}
            </div>
          </div>
          <div class="hidden shrink-0 items-center gap-1 text-[0.733333rem] text-muted-foreground min-[760px]:flex">
            <span>{t("projects.header.openTasks", openTaskCount)}</span>
            <span class="h-3 w-px bg-border"></span>
            <span>{t("projects.header.blockedTasks", blockedTaskCount)}</span>
            <span class="h-3 w-px bg-border"></span>
            <span>{t("projects.header.scheduledEvents", projectEvents.length)}</span>
            <span class="h-3 w-px bg-border"></span>
            <span>{t("projects.header.completedTasks", completedTaskCount)}</span>
            {#if scheduledThisWeekMinutes > 0}
              <span class="h-3 w-px bg-border"></span>
              <span>{t("projects.header.scheduledThisWeek", formatMinutesAsHours(scheduledThisWeekMinutes))}</span>
            {/if}
            {#if nextUpcomingProjectEvent}
              {@const nextEventLabel = projectHeaderEventLabel(nextUpcomingProjectEvent)}
              <span class="hidden h-3 w-px bg-border min-[1180px]:block"></span>
              <span class="hidden max-w-48 truncate min-[1180px]:block" title={nextEventLabel}>{nextEventLabel}</span>
            {/if}
          </div>
          <button
            type="button"
            class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
            aria-label={t("projects.header.projectSettings")}
            onclick={openProjectSettings}
          >
            <MoreHorizontal size={15} strokeWidth={1.75} />
          </button>
        </div>
        <div class="flex min-w-0 flex-wrap items-center gap-2">
          <nav class="flex min-w-0 gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
            {#each PROJECT_VIEW_IDS as view}
              {@const Icon = viewIcon(view)}
              <button
                type="button"
                class={cn(
                  "flex h-8 shrink-0 items-center gap-1.5 rounded px-2 text-[0.8rem] font-medium",
                  projects.activeView === view
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
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
          <div class="flex min-w-44 flex-1 items-center gap-2 rounded-md border border-border bg-background px-2">
            <Search size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
            <input
              bind:value={taskSearch}
              placeholder={t("projects.header.searchPlaceholder")}
              class="min-h-8 min-w-0 flex-1 bg-transparent text-[0.8rem] placeholder:text-muted-foreground"
            />
          </div>
          <div class="flex min-w-48 flex-1 flex-col gap-1">
            <form class="flex gap-1" onsubmit={(event) => { event.preventDefault(); void submitQuickTask(); }}>
              <input
                bind:value={quickTaskTitle}
                placeholder={t("projects.header.quickAddPlaceholder")}
                class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem] placeholder:text-muted-foreground"
              />
              <button
                type="button"
                disabled={taskCreatePendingTarget !== null}
                onclick={(event) => { event.preventDefault(); void submitQuickTask(); }}
                class="flex min-h-8 items-center gap-1.5 rounded-md bg-primary px-2 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
              >
                <Plus size={14} strokeWidth={1.75} />
                <span>{t("projects.header.addTask")}</span>
              </button>
            </form>
            {#if taskCreateErrorFor("quick")}
              <div class="rounded border border-destructive/30 bg-destructive/10 px-2 py-1 text-[0.733333rem] text-destructive">
                {taskCreateErrorFor("quick")}
              </div>
            {/if}
          </div>
        </div>
        <div class="flex min-w-0 flex-wrap items-center gap-2 text-[0.766667rem]">
          <div class="flex h-8 shrink-0 items-center gap-1.5 text-muted-foreground">
            <Funnel size={14} strokeWidth={1.75} />
            <span>{t("projects.filters.title")}</span>
          </div>
          <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
            {#each TASK_STATUS_FILTERS as filter}
              <button
                type="button"
                class={cn(
                  "h-7 shrink-0 rounded px-2 font-medium",
                  taskStatusFilter === filter
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                )}
                onclick={() => {
                  taskStatusFilter = filter;
                }}
              >
                {taskStatusFilterLabel(filter)}
              </button>
            {/each}
          </div>
          <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
            <button
              type="button"
              class={cn(
                "h-7 shrink-0 rounded px-2 font-medium",
                taskSectionFilter === "all"
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
              )}
              onclick={() => {
                taskSectionFilter = "all";
              }}
            >
              {t("projects.filters.allSections")}
            </button>
            {#each sections as section (section.id)}
              <button
                type="button"
                class={cn(
                  "h-7 max-w-36 shrink-0 truncate rounded px-2 font-medium",
                  taskSectionFilter === section.id
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                )}
                title={section.name}
                onclick={() => {
                  taskSectionFilter = section.id;
                }}
              >
                {section.name}
              </button>
            {/each}
          </div>
          <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
            <button
              type="button"
              class={cn(
                "h-7 shrink-0 rounded px-2 font-medium",
                taskPriorityFilter === "all"
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
              )}
              onclick={() => {
                taskPriorityFilter = "all";
              }}
            >
              {t("projects.filters.allPriorities")}
            </button>
            {#each PROJECT_PRIORITIES as priority}
              <button
                type="button"
                class={cn(
                  "h-7 shrink-0 rounded px-2 font-medium",
                  taskPriorityFilter === priority
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                )}
                onclick={() => {
                  taskPriorityFilter = priority;
                }}
              >
                {priorityLabel(priority)}
              </button>
            {/each}
          </div>
          <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
            {#each TASK_DUE_FILTERS as filter}
              <button
                type="button"
                class={cn(
                  "h-7 shrink-0 rounded px-2 font-medium",
                  taskDueFilter === filter
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                )}
                onclick={() => {
                  taskDueFilter = filter;
                }}
              >
                {taskDueFilterLabel(filter)}
              </button>
            {/each}
          </div>
          {#if taskDueFilter === "range"}
            <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md border border-border bg-background px-1 py-0.5">
              <input
                value={taskDueRangeStart}
                placeholder={t("projects.filters.dueRangeStart")}
                aria-label={t("projects.filters.dueRangeStart")}
                class="h-7 w-28 shrink-0 rounded bg-transparent px-2 text-[0.766667rem] text-foreground placeholder:text-muted-foreground"
                oninput={(event) => {
                  taskDueRangeStart = event.currentTarget.value;
                }}
              />
              <span class="shrink-0 text-muted-foreground">{t("projects.filters.dueRangeTo")}</span>
              <input
                value={taskDueRangeEnd}
                placeholder={t("projects.filters.dueRangeEnd")}
                aria-label={t("projects.filters.dueRangeEnd")}
                class="h-7 w-28 shrink-0 rounded bg-transparent px-2 text-[0.766667rem] text-foreground placeholder:text-muted-foreground"
                oninput={(event) => {
                  taskDueRangeEnd = event.currentTarget.value;
                }}
              />
            </div>
          {/if}
          <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
            {#each TASK_SCHEDULE_FILTERS as filter}
              <button
                type="button"
                class={cn(
                  "h-7 shrink-0 rounded px-2 font-medium",
                  taskScheduleFilter === filter
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                )}
                onclick={() => {
                  taskScheduleFilter = filter;
                }}
              >
                {taskScheduleFilterLabel(filter)}
              </button>
            {/each}
          </div>
          <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
            {#each TASK_DEPENDENCY_FILTERS as filter}
              <button
                type="button"
                class={cn(
                  "h-7 shrink-0 rounded px-2 font-medium",
                  taskDependencyFilter === filter
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                )}
                onclick={() => {
                  taskDependencyFilter = filter;
                }}
              >
                {taskDependencyFilterLabel(filter)}
              </button>
            {/each}
          </div>
          <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
            <button
              type="button"
              class={cn(
                "h-7 shrink-0 rounded px-2 font-medium",
                taskLabelFilter === "all"
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
              )}
              onclick={() => {
                taskLabelFilter = "all";
              }}
            >
              {taskLabelFilterLabel("all")}
            </button>
            <button
              type="button"
              class={cn(
                "h-7 shrink-0 rounded px-2 font-medium",
                taskLabelFilter === "none"
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
              )}
              onclick={() => {
                taskLabelFilter = "none";
              }}
            >
              {taskLabelFilterLabel("none")}
            </button>
            {#each projectLabels as label (label.id)}
              <button
                type="button"
                class={cn(
                  "flex h-7 max-w-36 shrink-0 items-center gap-1 rounded px-2 font-medium",
                  taskLabelFilter === label.id
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                )}
                title={label.name}
                onclick={() => {
                  taskLabelFilter = label.id;
                }}
              >
                <span
                  class={cn("h-2 w-2 shrink-0 rounded-full border", labelColorSwatchClass(label.color))}
                  style={labelColorDotStyle(label.color)}
                ></span>
                <span class="truncate">{label.name}</span>
              </button>
            {/each}
          </div>
          {#if projectCustomFields.length > 0}
            <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
              <span class="flex h-7 shrink-0 items-center px-1 text-muted-foreground">
                {t("projects.customFields.title")}
              </span>
              {#each projectCustomFields as field (field.id)}
                {@const currentCustomFieldFilter = taskCustomFieldFilterFor(field.id)}
                <div class="flex h-7 shrink-0 items-center overflow-hidden rounded border border-border bg-background/70">
                  <span class="max-w-28 truncate px-2 font-medium text-muted-foreground" title={field.name}>
                    {field.name}
                  </span>
                  <button
                    type="button"
                    class={cn("h-full shrink-0 border-l border-border px-2 font-medium", customFieldFilterButtonClass(currentCustomFieldFilter === undefined))}
                    onclick={() => clearTaskCustomFieldFilter(field.id)}
                  >
                    {t("projects.filters.allValues")}
                  </button>
                  {#if field.fieldType === "select" || field.fieldType === "multi_select"}
                    <button
                      type="button"
                      class={cn("h-full shrink-0 border-l border-border px-2 font-medium", customFieldFilterButtonClass(currentCustomFieldFilter?.mode === "empty"))}
                      onclick={() => setTaskCustomFieldFilter({ fieldId: field.id, mode: "empty" })}
                    >
                      {t("projects.filters.empty")}
                    </button>
                    {#each customFieldOptions(field) as option (option.id)}
                      <button
                        type="button"
                        class={cn(
                          "h-full max-w-32 shrink-0 truncate border-l border-border px-2 font-medium",
                          customFieldFilterButtonClass(
                            currentCustomFieldFilter?.mode === "option"
                              && currentCustomFieldFilter.optionId === option.id,
                          ),
                        )}
                        title={option.name}
                        onclick={() => setTaskCustomFieldFilter({ fieldId: field.id, mode: "option", optionId: option.id })}
                      >
                        {option.name}
                      </button>
                    {/each}
                  {:else if field.fieldType === "checkbox"}
                    <button
                      type="button"
                      class={cn(
                        "h-full shrink-0 border-l border-border px-2 font-medium",
                        customFieldFilterButtonClass(
                          currentCustomFieldFilter?.mode === "checkbox" && currentCustomFieldFilter.checked,
                        ),
                      )}
                      onclick={() => setTaskCustomFieldFilter({ fieldId: field.id, mode: "checkbox", checked: true })}
                    >
                      {t("projects.customFields.checked")}
                    </button>
                    <button
                      type="button"
                      class={cn(
                        "h-full shrink-0 border-l border-border px-2 font-medium",
                        customFieldFilterButtonClass(
                          currentCustomFieldFilter?.mode === "checkbox" && !currentCustomFieldFilter.checked,
                        ),
                      )}
                      onclick={() => setTaskCustomFieldFilter({ fieldId: field.id, mode: "checkbox", checked: false })}
                    >
                      {t("projects.customFields.unchecked")}
                    </button>
                    <button
                      type="button"
                      class={cn("h-full shrink-0 border-l border-border px-2 font-medium", customFieldFilterButtonClass(currentCustomFieldFilter?.mode === "empty"))}
                      onclick={() => setTaskCustomFieldFilter({ fieldId: field.id, mode: "empty" })}
                    >
                      {t("projects.filters.empty")}
                    </button>
                  {:else}
                    <button
                      type="button"
                      class={cn("h-full shrink-0 border-l border-border px-2 font-medium", customFieldFilterButtonClass(currentCustomFieldFilter?.mode === "filled"))}
                      onclick={() => setTaskCustomFieldFilter({ fieldId: field.id, mode: "filled" })}
                    >
                      {t("projects.filters.filled")}
                    </button>
                    <button
                      type="button"
                      class={cn("h-full shrink-0 border-l border-border px-2 font-medium", customFieldFilterButtonClass(currentCustomFieldFilter?.mode === "empty"))}
                      onclick={() => setTaskCustomFieldFilter({ fieldId: field.id, mode: "empty" })}
                    >
                      {t("projects.filters.empty")}
                    </button>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
          <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
            <span class="flex h-7 shrink-0 items-center px-1 text-muted-foreground">
              <ArrowUpDown size={13} strokeWidth={1.75} />
            </span>
            {#each TASK_SORT_MODES as mode}
              <button
                type="button"
                class={cn(
                  "h-7 shrink-0 rounded px-2 font-medium",
                  taskSortMode === mode
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                )}
                onclick={() => {
                  taskSortMode = mode;
                }}
              >
                {taskSortModeLabel(mode)}
              </button>
            {/each}
            {#if projectCustomFields.length > 0}
              <span class="flex h-7 shrink-0 items-center px-1 text-muted-foreground">
                {t("projects.customFields.title")}
              </span>
              {#each projectCustomFields as field (field.id)}
                {@const mode = customFieldSortMode(field)}
                <button
                  type="button"
                  class={cn(
                    "h-7 max-w-40 shrink-0 truncate rounded px-2 font-medium",
                    taskSortMode === mode
                      ? "bg-background text-foreground shadow-sm"
                      : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                  )}
                  title={field.name}
                  onclick={() => {
                    taskSortMode = mode;
                  }}
                >
                  {field.name}
                </button>
              {/each}
            {/if}
            <button
              type="button"
              class="flex h-7 shrink-0 items-center gap-1 rounded px-2 font-medium text-muted-foreground hover:bg-background/60 hover:text-foreground"
              aria-label={t("projects.sort.toggleDirection")}
              title={t("projects.sort.toggleDirection")}
              onclick={() => {
                taskSortDirection = taskSortDirection === "asc" ? "desc" : "asc";
              }}
            >
              {#if taskSortDirection === "asc"}
                <ArrowUp size={13} strokeWidth={1.75} />
              {:else}
                <ArrowDown size={13} strokeWidth={1.75} />
              {/if}
              <span>{taskSortDirectionLabel(taskSortDirection)}</span>
            </button>
          </div>
          <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
            <span class="flex h-7 shrink-0 items-center px-1 text-muted-foreground">
              {t("projects.grouping.title")}
            </span>
            {#each PROJECT_TASK_GROUP_MODES as mode}
              <button
                type="button"
                class={cn(
                  "h-7 shrink-0 rounded px-2 font-medium",
                  taskGroupBy === mode
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                )}
                onclick={() => {
                  taskGroupBy = mode;
                }}
              >
                {taskGroupModeLabel(mode)}
              </button>
            {/each}
          </div>
          <span class="h-7 shrink-0 rounded-md border border-border bg-background px-2 py-1 text-muted-foreground">
            {t("projects.filters.matchingTasks", matchingTaskCount, allProjectTasks.length)}
          </span>
          <button
            type="button"
            class={cn(
              "flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-background px-2 font-medium hover:bg-accent hover:text-foreground",
              showArchivedTasks ? "text-foreground" : "text-muted-foreground",
            )}
            aria-label={showArchivedTasks ? t("projects.filters.hideArchivedTasks") : t("projects.filters.showArchivedTasks")}
            title={showArchivedTasks ? t("projects.filters.hideArchivedTasks") : t("projects.filters.showArchivedTasks")}
            onclick={() => {
              showArchivedTasks = !showArchivedTasks;
            }}
          >
            {#if showArchivedTasks}
              <ArchiveRestore size={13} strokeWidth={1.75} />
            {:else}
              <Archive size={13} strokeWidth={1.75} />
            {/if}
            <span>
              {showArchivedTasks
                ? t("projects.filters.hideArchived")
                : t("projects.filters.showArchived", archivedProjectTaskCount)}
            </span>
          </button>
          {#if taskFiltersActive}
            <button
              type="button"
              class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-background px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
              onclick={clearTaskFilters}
            >
              <RotateCcw size={13} strokeWidth={1.75} />
              <span>{t("projects.filters.reset")}</span>
            </button>
          {/if}
        </div>
        <div class="flex min-w-0 flex-wrap items-center gap-1 text-[0.766667rem]">
          <form
            class="flex min-w-52 max-w-full flex-1 gap-1 min-[820px]:max-w-sm"
            onsubmit={(event) => { event.preventDefault(); void saveCurrentTaskView(); }}
          >
            <input
              bind:value={savedViewNameDraft}
              placeholder={t("projects.savedViews.namePlaceholder")}
              class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem] placeholder:text-muted-foreground"
            />
            <button
              type="submit"
              class="flex min-h-8 shrink-0 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] font-medium hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
              disabled={savedViewSaving}
            >
              <Save size={13} strokeWidth={1.75} />
              <span>{savedViewSaving ? t("common.loading") : t("projects.savedViews.save")}</span>
            </button>
          </form>
          {#if savedTaskViews.length > 0}
            <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
              {#each savedTaskViews as view (view.id)}
                <div class="flex h-7 shrink-0 items-center overflow-hidden rounded border border-border bg-background">
                  <button
                    type="button"
                    class="flex h-full max-w-40 items-center gap-1.5 px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
                    title={view.name}
                    onclick={() => { void applyTaskView(view); }}
                  >
                    <span class="truncate">{view.name}</span>
                  </button>
                  <button
                    type="button"
                    class="flex h-full w-7 shrink-0 items-center justify-center border-l border-border text-muted-foreground hover:bg-destructive/10 hover:text-destructive disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={savedViewSaving}
                    aria-label={t("projects.savedViews.delete", view.name)}
                    title={t("projects.savedViews.delete", view.name)}
                    onclick={() => { void deleteSavedTaskView(view); }}
                  >
                    <Trash2 size={12} strokeWidth={1.75} />
                  </button>
                </div>
              {/each}
            </div>
          {/if}
          <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md border border-border bg-background px-1 py-0.5">
            <span class="shrink-0 px-1 text-[0.733333rem] font-medium text-muted-foreground">
              {t("projects.columns.title")}
            </span>
            {#each PROJECT_TASK_LIST_COLUMNS as column}
              <button
                type="button"
                class={cn(
                  "h-7 shrink-0 rounded px-2 font-medium",
                  taskListColumnVisible(column)
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
                )}
                aria-pressed={taskListColumnVisible(column)}
                onclick={() => { void toggleTaskListColumn(column); }}
              >
                {taskListColumnLabel(column)}
              </button>
            {/each}
            {#if projectCustomFields.length > 0}
              <span class="shrink-0 px-1 text-[0.733333rem] font-medium text-muted-foreground">
                {t("projects.customFields.title")}
              </span>
              {#each projectCustomFields as field (field.id)}
                {@const column = customTaskListColumn(field.id)}
                <button
                  type="button"
                  class={cn(
                    "h-7 max-w-40 shrink-0 rounded px-2 font-medium",
                    taskListColumnVisible(column)
                      ? "bg-accent text-accent-foreground"
                      : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
                  )}
                  aria-pressed={taskListColumnVisible(column)}
                  title={field.name}
                  onclick={() => { void toggleTaskListColumn(column); }}
                >
                  <span class="block truncate">{field.name}</span>
                </button>
              {/each}
            {/if}
          </div>
          {#if savedViewError}
            <div class="basis-full rounded border border-destructive/30 bg-destructive/10 px-2 py-1 text-destructive">
              {savedViewError}
            </div>
          {/if}
        </div>
        {#if selectedTasks.length > 0}
          {@const bulkSchedulableCount = selectedSchedulableTasks().length}
          <div class="flex min-w-0 flex-wrap items-center gap-1 rounded-md border border-border bg-background px-2 py-1 text-[0.766667rem]">
            <span class="mr-1 shrink-0 font-medium">{t("projects.bulk.selected", selectedTasks.length)}</span>
            <button
              type="button"
              class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
              disabled={bulkTaskActionPending || selectableTasks.length === selectedTasks.length}
              onclick={selectFilteredTasks}
            >
              <Check size={13} strokeWidth={1.75} />
              <span>{t("projects.bulk.selectFiltered", selectableTasks.length)}</span>
            </button>
            <button
              type="button"
              class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
              disabled={bulkTaskActionPending || !terminalStatus()}
              onclick={() => { void bulkSetSelectedStatus(terminalStatus()); }}
            >
              <Check size={13} strokeWidth={1.75} />
              <span>{t("projects.bulk.markDone")}</span>
            </button>
            <button
              type="button"
              class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
              disabled={bulkTaskActionPending || !firstOpenStatus()}
              onclick={() => { void bulkSetSelectedStatus(firstOpenStatus()); }}
            >
              <RotateCcw size={13} strokeWidth={1.75} />
              <span>{t("projects.bulk.reopen")}</span>
            </button>
            <button
              type="button"
              class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
              disabled={bulkTaskActionPending || bulkSchedulableCount === 0}
              onclick={() => {
                if (bulkScheduleOpen) {
                  closeBulkScheduleForm();
                } else {
                  openBulkScheduleForm();
                }
              }}
            >
              <CalendarDays size={13} strokeWidth={1.75} />
              <span>{t("projects.bulk.schedule")}</span>
            </button>
            <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
              {#each PROJECT_PRIORITIES as priority}
                <button
                  type="button"
                  class={cn("h-6 shrink-0 rounded px-2 font-medium", priorityClass(priority))}
                  disabled={bulkTaskActionPending}
                  onclick={() => { void bulkSetSelectedPriority(priority); }}
                >
                  {priorityLabel(priority)}
                </button>
              {/each}
            </div>
            {#if selectedActiveTaskCount > 0}
              <button
                type="button"
                class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-destructive/30 bg-destructive/10 px-2 font-medium text-destructive hover:bg-destructive/15 disabled:cursor-not-allowed disabled:opacity-50"
                disabled={bulkTaskActionPending}
                onclick={() => { void bulkArchiveSelectedTasks(); }}
              >
                <Archive size={13} strokeWidth={1.75} />
                <span>{t("projects.bulk.archive")}</span>
              </button>
            {/if}
            {#if selectedArchivedTaskCount > 0}
              <button
                type="button"
                class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                disabled={bulkTaskActionPending}
                onclick={() => { void bulkRestoreSelectedTasks(); }}
              >
                <ArchiveRestore size={13} strokeWidth={1.75} />
                <span>{t("projects.bulk.restore")}</span>
              </button>
            {/if}
            <button
              type="button"
              class="ml-auto flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
              onclick={clearTaskSelection}
            >
              <X size={13} strokeWidth={1.75} />
              <span>{t("projects.bulk.clear")}</span>
            </button>
            {#if bulkScheduleOpen}
              <form
                class="basis-full grid gap-2 border-t border-border/70 pt-2 min-[860px]:grid-cols-[minmax(0,1fr)_8rem_7rem_6rem_auto_auto]"
                onsubmit={(event) => { event.preventDefault(); void bulkScheduleSelectedTasks(); }}
              >
                <div class="self-end text-[0.733333rem] text-muted-foreground">
                  {t("projects.bulk.scheduleHint", bulkSchedulableCount)}
                </div>
                <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
                  <span>{t("projects.schedule.date")}</span>
                  <input
                    bind:value={bulkScheduleDate}
                    placeholder="YYYY-MM-DD"
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
                  <span>{t("projects.schedule.start")}</span>
                  <input
                    bind:value={bulkScheduleStartTime}
                    placeholder="HH:MM"
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
                  <span>{t("projects.schedule.duration")}</span>
                  <input
                    type="number"
                    min="1"
                    step="5"
                    bind:value={bulkScheduleDurationMinutes}
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <button
                  type="submit"
                  class="self-end rounded-md bg-primary px-2 py-1.5 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
                  disabled={bulkTaskActionPending || bulkSchedulableCount === 0}
                >
                  {t("projects.bulk.scheduleSelected")}
                </button>
                <button
                  type="button"
                  class="self-end rounded-md border border-border bg-card px-2 py-1.5 text-[0.8rem] hover:bg-accent"
                  disabled={bulkTaskActionPending}
                  onclick={closeBulkScheduleForm}
                >
                  {t("common.cancel")}
                </button>
              </form>
            {/if}
            {#if bulkTaskError}
              <div class="basis-full rounded border border-destructive/30 bg-destructive/10 px-2 py-1 text-destructive">
                {bulkTaskError}
              </div>
            {/if}
          </div>
        {/if}
      </header>

      <div class={cn("min-h-0 flex-1", projects.activeView === "calendar" ? "overflow-hidden" : "overflow-auto")}>
        {#if projects.activeView === "list"}
          <div class="flex min-h-full flex-col gap-3 p-3">
            <div class="flex flex-wrap items-center gap-2">
              <form class="flex min-w-52 max-w-md flex-1 gap-1" onsubmit={(event) => { event.preventDefault(); void submitSection(); }}>
                <input
                  bind:value={sectionDraft}
                  placeholder={t("projects.header.addSection")}
                  class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-card px-2 text-[0.8rem]"
                />
                <button type="submit" class="flex min-h-8 items-center gap-1 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent">
                  <Plus size={14} strokeWidth={1.75} />
                  <span>{t("common.save")}</span>
                </button>
              </form>
              {#if inactiveSectionCount > 0}
                <button
                  type="button"
                  class={cn(
                    "flex min-h-8 shrink-0 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] font-medium hover:bg-accent",
                    showInactiveSections ? "text-foreground" : "text-muted-foreground",
                  )}
                  aria-label={showInactiveSections ? t("projects.filters.hideInactiveSections") : t("projects.filters.showInactiveSections")}
                  title={showInactiveSections ? t("projects.filters.hideInactiveSections") : t("projects.filters.showInactiveSections")}
                  onclick={() => {
                    showInactiveSections = !showInactiveSections;
                  }}
                >
                  {#if showInactiveSections}
                    <EyeOff size={14} strokeWidth={1.75} />
                  {:else}
                    <Eye size={14} strokeWidth={1.75} />
                  {/if}
                  <span>
                    {showInactiveSections
                      ? t("projects.filters.hideInactiveSectionsShort")
                      : t("projects.filters.showInactiveSectionsShort", inactiveSectionCount)}
                  </span>
                </button>
              {/if}
            </div>
            {#if taskGroupBy === "section"}
            {#each sections as section (section.id)}
              {@const sectionTasks = tasksForSection(section)}
              {@const previousSection = adjacentSection(section, -1)}
              {@const nextSection = adjacentSection(section, 1)}
              <section
                class={cn(
                  "flex flex-col gap-1 rounded-lg border border-transparent p-1",
                  listDragOverSectionId === section.id && "border-primary/40 bg-primary/5",
                )}
                role="list"
                aria-label={section.name}
                ondragover={(event) => handleListSectionDragOver(event, section)}
                ondrop={(event) => { void dropListTask(event, section); }}
              >
                <div class="flex min-h-9 items-center gap-1 border-b border-border/70 px-1">
                  <button
                    type="button"
                    class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
                    aria-label={section.collapsed ? t("projects.actions.expandSection", section.name) : t("projects.actions.collapseSection", section.name)}
                    title={section.collapsed ? t("projects.actions.expandSection", section.name) : t("projects.actions.collapseSection", section.name)}
                    onclick={() => { void toggleSectionCollapsed(section); }}
                  >
                    {#if section.collapsed}
                      <ChevronRight size={14} strokeWidth={1.75} />
                    {:else}
                      <ChevronDown size={14} strokeWidth={1.75} />
                    {/if}
                  </button>
                  <input
                    value={sectionNameDrafts[section.id] ?? section.name}
                    class="min-h-8 min-w-0 flex-1 bg-transparent text-[0.866667rem] font-semibold"
                    aria-label={t("projects.list.sectionName")}
                    oninput={(event) => {
                      sectionNameDrafts = {
                        ...sectionNameDrafts,
                        [section.id]: event.currentTarget.value,
                      };
                    }}
                    onkeydown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        void saveSection(section);
                      }
                    }}
                  />
                  <span class="text-[0.733333rem] text-muted-foreground">{sectionTasks.length}</span>
                  {#if section.hiddenAt}
                    <span class="rounded border border-sky-500/40 bg-sky-500/10 px-1.5 py-0.5 text-[0.733333rem] text-sky-700 dark:text-sky-300">
                      {t("projects.list.hiddenSection")}
                    </span>
                  {/if}
                  {#if section.archivedAt}
                    <span class="rounded border border-muted-foreground/30 bg-muted/70 px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground">
                      {t("projects.list.archivedSection")}
                    </span>
                  {/if}
                  <button
                    type="button"
                    class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!previousSection || Boolean(section.hiddenAt || section.archivedAt)}
                    aria-label={t("projects.actions.moveSectionUp", section.name)}
                    title={t("projects.actions.moveSectionUp", section.name)}
                    onclick={() => { void moveSection(section, -1); }}
                  >
                    <ArrowUp size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!nextSection || Boolean(section.hiddenAt || section.archivedAt)}
                    aria-label={t("projects.actions.moveSectionDown", section.name)}
                    title={t("projects.actions.moveSectionDown", section.name)}
                    onclick={() => { void moveSection(section, 1); }}
                  >
                    <ArrowDown size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class="flex h-7 items-center gap-1 rounded-md border border-border bg-card px-2 text-[0.733333rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={!sectionDraftDirty(section) || Boolean(section.hiddenAt || section.archivedAt)}
                    onclick={() => { void saveSection(section); }}
                  >
                    <Save size={13} strokeWidth={1.75} />
                    <span>{t("projects.list.saveSection")}</span>
                  </button>
                  {#if section.hiddenAt || section.archivedAt}
                    <button
                      type="button"
                      class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
                      aria-label={t("projects.actions.restoreSection", section.name)}
                      title={t("projects.actions.restoreSection", section.name)}
                      onclick={() => { void restoreSection(section); }}
                    >
                      <ArchiveRestore size={13} strokeWidth={1.75} />
                    </button>
                  {:else}
                    <button
                      type="button"
                      class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
                      aria-label={t("projects.actions.hideSection", section.name)}
                      title={t("projects.actions.hideSection", section.name)}
                      onclick={() => { void hideSection(section); }}
                    >
                      <EyeOff size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
                      aria-label={t("projects.actions.archiveSection", section.name)}
                      title={t("projects.actions.archiveSection", section.name)}
                      onclick={() => { void archiveSection(section); }}
                    >
                      <Archive size={13} strokeWidth={1.75} />
                    </button>
                  {/if}
                </div>
                {#if !section.collapsed && !section.archivedAt && !section.hiddenAt}
                  <div class="grid gap-1">
                    {#each sectionTasks as task (task.id)}
                      {@const status = statusForTask(task)}
                      {@const subtasks = subtasksForTask(task)}
                      {@const scheduled = scheduledLabel(task.id)}
                      {@const taskLabels = visibleTaskLabels(task)}
                      {@const hiddenLabels = hiddenTaskLabelCount(task)}
                      {@const previousSectionTask = adjacentTaskInSection(task, -1)}
                      {@const nextSectionTask = adjacentTaskInSection(task, 1)}
                      {@const blockedByCount = blockedByDependencies(task).length}
                      {@const blocksCount = blocksDependencies(task).length}
                      {#if listDropMarkerVisible(section, task, "before")}
                        <div class="h-1 rounded-full bg-primary"></div>
                      {/if}
                      <div
                        role="listitem"
                        class={cn(
                          "rounded-md border bg-card px-2 py-1.5",
                          selectedTaskId === task.id ? "border-primary/60 ring-1 ring-primary/20" : "border-border",
                          task.archivedAt && "opacity-70",
                          listDraggingTaskId === task.id && "opacity-50",
                          listDropPendingTaskId === task.id && "opacity-60",
                        )}
                        ondragover={(event) => handleListRowDragOver(event, section, task)}
                        ondrop={(event) => { void dropListTask(event, section, task, listRowDropPosition(event)); }}
                      >
                        <div class="grid min-h-8 grid-cols-[auto_auto_auto_minmax(0,1fr)_auto] items-center gap-2">
                          <button
                            type="button"
                            class="flex h-5 w-5 shrink-0 cursor-grab items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground active:cursor-grabbing disabled:cursor-not-allowed disabled:opacity-40"
                            draggable={listDragEnabled() && !task.archivedAt && listDropPendingTaskId === null}
                            disabled={!listDragEnabled() || Boolean(task.archivedAt) || listDropPendingTaskId !== null}
                            aria-label={t("projects.actions.dragTask", task.title)}
                            title={t("projects.actions.dragTask", task.title)}
                            ondragstart={(event) => handleListTaskDragStart(event, task)}
                            ondragend={handleListTaskDragEnd}
                          >
                            <GripVertical size={13} strokeWidth={1.75} />
                          </button>
                          <button
                            type="button"
                            class={cn(
                              "flex h-5 w-5 shrink-0 items-center justify-center rounded border",
                              taskSelected(task) ? "border-primary bg-primary text-primary-foreground" : "border-border hover:bg-accent",
                            )}
                            aria-label={taskSelected(task) ? t("projects.actions.unselectTask", task.title) : t("projects.actions.selectTask", task.title)}
                            onclick={() => toggleTaskSelection(task)}
                          >
                            {#if taskSelected(task)}
                              <Check size={13} strokeWidth={2} />
                            {/if}
                          </button>
                          <button
                            type="button"
                            class={cn(
                              "flex h-5 w-5 shrink-0 items-center justify-center rounded border disabled:cursor-not-allowed disabled:opacity-40",
                              status?.terminal ? "border-emerald-500 bg-emerald-500 text-white" : "border-border hover:bg-accent",
                            )}
                            aria-label={t("projects.actions.toggleComplete")}
                            disabled={Boolean(task.archivedAt)}
                            onclick={() => { void projects.toggleTaskDone(task); }}
                          >
                            {#if status?.terminal}
                              <Check size={13} strokeWidth={2} />
                            {/if}
                          </button>
                          <button
                            type="button"
                            class="min-w-0 text-left"
                            aria-label={t("projects.actions.openTaskDetails", task.title)}
                            onclick={() => openTaskDetail(task)}
                          >
                            <div class="truncate text-[0.866667rem]">{task.title}</div>
                            {#if taskLabels.length > 0}
                              <div class="mt-1 flex min-w-0 flex-wrap gap-1">
                                {#each taskLabels as label (label.id)}
                                  <span class="inline-flex max-w-full items-center gap-1 rounded border border-border bg-muted/50 px-1.5 py-0.5 text-[0.666667rem] text-muted-foreground">
                                    <span
                                      class={cn("h-1.5 w-1.5 shrink-0 rounded-full border", labelColorSwatchClass(label.color))}
                                      style={labelColorDotStyle(label.color)}
                                    ></span>
                                    <span class="truncate">{label.name}</span>
                                  </span>
                                {/each}
                                {#if hiddenLabels > 0}
                                  <span class="rounded border border-border bg-muted/50 px-1.5 py-0.5 text-[0.666667rem] text-muted-foreground">
                                    {t("projects.list.moreLabels", hiddenLabels)}
                                  </span>
                                {/if}
                              </div>
                            {/if}
                            {#if subtasks.length > 0}
                              <div class="truncate text-[0.733333rem] text-muted-foreground">
                                {t("projects.list.subtasks", subtasks.length)}
                              </div>
                            {/if}
                          </button>
                          <div class="flex min-w-0 items-center justify-end gap-1">
                            {#if taskListColumnVisible("status")}
                              <button
                                type="button"
                                class={cn("rounded border px-1.5 py-0.5 text-[0.733333rem]", statusBadgeClass(status))}
                              >
                                {status?.name ?? t("projects.list.status")}
                              </button>
                            {/if}
                            {#if taskListColumnVisible("priority")}
                              <button
                                type="button"
                                class={cn(
                                  "rounded border px-1.5 py-0.5 text-[0.733333rem] disabled:cursor-not-allowed disabled:opacity-60",
                                  priorityClass(task.priority),
                                )}
                                disabled={Boolean(task.archivedAt)}
                                onclick={() => { void projects.setTaskPriority(task, nextPriority(task.priority)); }}
                              >
                                {priorityLabel(task.priority)}
                              </button>
                            {/if}
                            {#if taskListColumnVisible("estimate") && task.estimateMinutes !== undefined}
                              <span class="rounded border border-border bg-background px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground">
                                {estimateLabel(task.estimateMinutes)}
                              </span>
                            {/if}
                            {#if taskListColumnVisible("due") && task.dueDate}
                              <span class="rounded border border-border bg-background px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground">
                                {task.dueDate}
                              </span>
                            {/if}
                            {#each visibleCustomFieldListFields as field (field.id)}
                              {@const customValue = customFieldDisplayValue(task, field)}
                              {#if customValue}
                                <span
                                  class="inline-flex min-w-0 max-w-48 items-center gap-1 rounded border border-border bg-background px-1.5 py-0.5 text-[0.733333rem]"
                                  title={`${field.name}: ${customValue}`}
                                >
                                  <span class="max-w-20 truncate text-muted-foreground">{field.name}</span>
                                  <span class="max-w-28 truncate text-foreground">{customValue}</span>
                                </span>
                              {/if}
                            {/each}
                            {#if taskListColumnVisible("scheduled") && scheduled}
                              <span class="rounded border border-sky-500/40 bg-sky-500/10 px-1.5 py-0.5 text-[0.733333rem] text-sky-700 dark:text-sky-300">
                                {scheduled}
                              </span>
                            {/if}
                            {#if task.archivedAt}
                              <span class={cn("rounded border px-1.5 py-0.5 text-[0.733333rem]", taskArchivedBadgeClass(task))}>
                                {t("projects.taskLifecycle.archived")}
                              </span>
                            {/if}
                            {#if taskListColumnVisible("dependencies")}
                              {#if blockedByCount > 0}
                                <span class="rounded border border-destructive/40 bg-destructive/10 px-1.5 py-0.5 text-[0.733333rem] text-destructive">
                                  {t("projects.list.blockedBy", blockedByCount)}
                                </span>
                              {/if}
                              {#if blocksCount > 0}
                                <span class="rounded border border-amber-500/40 bg-amber-500/10 px-1.5 py-0.5 text-[0.733333rem] text-amber-700 dark:text-amber-300">
                                  {t("projects.list.blocks", blocksCount)}
                                </span>
                              {/if}
                            {/if}
                            <button
                              type="button"
                              class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                              disabled={Boolean(task.archivedAt) || taskSortMode !== "manual" || !previousSectionTask}
                              aria-label={previousSectionTask ? t("projects.actions.moveTaskUp", task.title) : t("projects.actions.noPreviousTask")}
                              title={previousSectionTask ? t("projects.actions.moveTaskUp", task.title) : t("projects.actions.noPreviousTask")}
                              onclick={() => { void moveTaskWithinSection(task, -1); }}
                            >
                              <ArrowUp size={13} strokeWidth={1.75} />
                            </button>
                            <button
                              type="button"
                              class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                              disabled={Boolean(task.archivedAt) || taskSortMode !== "manual" || !nextSectionTask}
                              aria-label={nextSectionTask ? t("projects.actions.moveTaskDown", task.title) : t("projects.actions.noNextTask")}
                              title={nextSectionTask ? t("projects.actions.moveTaskDown", task.title) : t("projects.actions.noNextTask")}
                              onclick={() => { void moveTaskWithinSection(task, 1); }}
                            >
                              <ArrowDown size={13} strokeWidth={1.75} />
                            </button>
                            <button
                              type="button"
                              class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                              disabled={Boolean(task.archivedAt)}
                              aria-label={t("projects.actions.scheduleTask")}
                              title={t("projects.actions.scheduleTask")}
                              onclick={() => openScheduleForm(task)}
                            >
                              <CalendarDays size={13} strokeWidth={1.75} />
                            </button>
                          </div>
                        </div>
                        {#if schedulingTaskId === task.id && !task.archivedAt}
                          <form
                            class="mt-2 grid gap-2 border-t border-border/70 pt-2 min-[720px]:grid-cols-[minmax(0,1fr)_7rem_6rem_auto_auto]"
                            onsubmit={(event) => { event.preventDefault(); void scheduleTask(task); }}
                          >
                            <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
                              <span>{t("projects.schedule.date")}</span>
                              <input
                                bind:value={scheduleDate}
                                placeholder="YYYY-MM-DD"
                                class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                              />
                            </label>
                            <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
                              <span>{t("projects.schedule.start")}</span>
                              <input
                                bind:value={scheduleStartTime}
                                placeholder="HH:MM"
                                class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                              />
                            </label>
                            <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
                              <span>{t("projects.schedule.duration")}</span>
                              <input
                                type="number"
                                min="1"
                                step="5"
                                bind:value={scheduleDurationMinutes}
                                class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                              />
                            </label>
                            <button
                              type="submit"
                              disabled={schedulePending}
                              class="self-end rounded-md bg-primary px-2 py-1.5 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
                            >
                              {t("projects.schedule.schedule")}
                            </button>
                            <button
                              type="button"
                              class="self-end rounded-md border border-border bg-card px-2 py-1.5 text-[0.8rem] hover:bg-accent"
                              onclick={closeScheduleForm}
                            >
                              {t("common.cancel")}
                            </button>
                            {#if scheduleError}
                              <div class="text-[0.733333rem] text-destructive min-[720px]:col-span-5">
                                {scheduleError}
                              </div>
                            {/if}
                          </form>
                        {/if}
                        {#if subtasks.length > 0}
                          <div class="mt-1 grid gap-1 border-t border-border/70 pt-1">
                            {#each subtasks as subtask (subtask.id)}
                              {@const subtaskStatus = statusForTask(subtask)}
                              <div
                                class={cn(
                                  "grid min-h-7 grid-cols-[1.75rem_auto_minmax(0,1fr)_auto_auto] items-center gap-2 rounded px-1",
                                  selectedTaskId === subtask.id ? "bg-accent/80" : "bg-transparent",
                                  subtask.archivedAt && "opacity-70",
                                )}
                              >
                                <span class="h-px w-4 justify-self-center bg-border"></span>
                                <button
                                  type="button"
                                  class={cn(
                                    "flex h-5 w-5 shrink-0 items-center justify-center rounded border",
                                    subtaskStatus?.terminal ? "border-emerald-500 bg-emerald-500 text-white" : "border-border hover:bg-accent",
                                  )}
                                  aria-label={t("projects.actions.toggleComplete")}
                                  onclick={() => { void projects.toggleTaskDone(subtask); }}
                                >
                                  {#if subtaskStatus?.terminal}
                                    <Check size={13} strokeWidth={2} />
                                  {/if}
                                </button>
                                <button
                                  type="button"
                                  class="min-w-0 text-left"
                                  aria-label={t("projects.actions.openTaskDetails", subtask.title)}
                                  onclick={() => openTaskDetail(subtask)}
                                >
                                  <span class="block truncate text-[0.8rem]">{subtask.title}</span>
                                </button>
                                <span class={cn("rounded border px-1.5 py-0.5 text-[0.733333rem]", statusBadgeClass(subtaskStatus))}>
                                  {subtaskStatus?.name ?? t("projects.list.status")}
                                </span>
                                {#if subtask.archivedAt}
                                  <span class={cn("rounded border px-1.5 py-0.5 text-[0.733333rem]", taskArchivedBadgeClass(subtask))}>
                                    {t("projects.taskLifecycle.archived")}
                                  </span>
                                {/if}
                              </div>
                            {/each}
                          </div>
                        {/if}
                      </div>
                      {#if listDropMarkerVisible(section, task, "after")}
                        <div class="h-1 rounded-full bg-primary"></div>
                      {/if}
                    {/each}
                    {#if listDragOverSectionId === section.id && listDragOverPosition === "section"}
                      <div class="h-1 rounded-full bg-primary"></div>
                    {/if}
                    {#if sectionTasks.length === 0}
                      <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                        {t("projects.list.emptySection")}
                      </div>
                    {/if}
                  </div>
                  <div class="grid gap-1 pl-7">
                    <form class="flex gap-1" onsubmit={(event) => { event.preventDefault(); void submitSectionTask(section.id); }}>
                      <input
                        value={sectionTaskDrafts[section.id] ?? ""}
                        oninput={(event) => {
                          sectionTaskDrafts = {
                            ...sectionTaskDrafts,
                            [section.id]: event.currentTarget.value,
                          };
                        }}
                        placeholder={t("projects.list.addTaskInSection", section.name)}
                        class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-card px-2 text-[0.8rem]"
                      />
                      <button
                        type="button"
                        disabled={taskCreatePendingTarget !== null}
                        onclick={(event) => { event.preventDefault(); void submitSectionTask(section.id); }}
                        class="flex min-h-8 items-center justify-center rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-60"
                      >
                        <Plus size={14} strokeWidth={1.75} />
                      </button>
                    </form>
                    {#if taskCreateErrorFor(sectionTaskCreateTarget(section.id))}
                      <div class="rounded border border-destructive/30 bg-destructive/10 px-2 py-1 text-[0.733333rem] text-destructive">
                        {taskCreateErrorFor(sectionTaskCreateTarget(section.id))}
                      </div>
                    {/if}
                  </div>
                {/if}
              </section>
            {/each}
            {:else}
              {#each listTaskGroups as group (group.id)}
                <section class="flex flex-col gap-1">
                  <div class="flex min-h-9 items-center gap-2 border-b border-border/70 px-1">
                    <span class="min-w-0 flex-1 truncate text-[0.866667rem] font-semibold">
                      {taskListGroupTitle(group.value)}
                    </span>
                    <span class="rounded border border-border bg-background px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground">
                      {group.tasks.length}
                    </span>
                  </div>
                  <div class="grid gap-1">
                    {#each group.tasks as task (task.id)}
                      {@const status = statusForTask(task)}
                      {@const section = sectionForTask(task)}
                      {@const subtasks = subtasksForTask(task)}
                      {@const scheduled = scheduledLabel(task.id)}
                      {@const taskLabels = visibleTaskLabels(task)}
                      {@const hiddenLabels = hiddenTaskLabelCount(task)}
                      {@const blockedByCount = blockedByDependencies(task).length}
                      {@const blocksCount = blocksDependencies(task).length}
                      <div
                        class={cn(
                          "rounded-md border bg-card px-2 py-1.5",
                          selectedTaskId === task.id ? "border-primary/60 ring-1 ring-primary/20" : "border-border",
                          task.archivedAt && "opacity-70",
                        )}
                      >
                        <div class="grid min-h-8 grid-cols-[auto_auto_minmax(0,1fr)_auto] items-center gap-2">
                          <button
                            type="button"
                            class={cn(
                              "flex h-5 w-5 shrink-0 items-center justify-center rounded border",
                              taskSelected(task) ? "border-primary bg-primary text-primary-foreground" : "border-border hover:bg-accent",
                            )}
                            aria-label={taskSelected(task) ? t("projects.actions.unselectTask", task.title) : t("projects.actions.selectTask", task.title)}
                            onclick={() => toggleTaskSelection(task)}
                          >
                            {#if taskSelected(task)}
                              <Check size={13} strokeWidth={2} />
                            {/if}
                          </button>
                          <button
                            type="button"
                            class={cn(
                              "flex h-5 w-5 shrink-0 items-center justify-center rounded border disabled:cursor-not-allowed disabled:opacity-40",
                              status?.terminal ? "border-emerald-500 bg-emerald-500 text-white" : "border-border hover:bg-accent",
                            )}
                            aria-label={t("projects.actions.toggleComplete")}
                            disabled={Boolean(task.archivedAt)}
                            onclick={() => { void projects.toggleTaskDone(task); }}
                          >
                            {#if status?.terminal}
                              <Check size={13} strokeWidth={2} />
                            {/if}
                          </button>
                          <button
                            type="button"
                            class="min-w-0 text-left"
                            aria-label={t("projects.actions.openTaskDetails", task.title)}
                            onclick={() => openTaskDetail(task)}
                          >
                            <div class="truncate text-[0.866667rem]">{task.title}</div>
                            <div class="mt-0.5 flex min-w-0 flex-wrap items-center gap-1 text-[0.733333rem] text-muted-foreground">
                              {#if section}
                                <span class="truncate">{section.name}</span>
                              {/if}
                              {#if subtasks.length > 0}
                                <span>{t("projects.list.subtasks", subtasks.length)}</span>
                              {/if}
                            </div>
                            {#if taskLabels.length > 0}
                              <div class="mt-1 flex min-w-0 flex-wrap gap-1">
                                {#each taskLabels as label (label.id)}
                                  <span class="inline-flex max-w-full items-center gap-1 rounded border border-border bg-muted/50 px-1.5 py-0.5 text-[0.666667rem] text-muted-foreground">
                                    <span
                                      class={cn("h-1.5 w-1.5 shrink-0 rounded-full border", labelColorSwatchClass(label.color))}
                                      style={labelColorDotStyle(label.color)}
                                    ></span>
                                    <span class="truncate">{label.name}</span>
                                  </span>
                                {/each}
                                {#if hiddenLabels > 0}
                                  <span class="rounded border border-border bg-muted/50 px-1.5 py-0.5 text-[0.666667rem] text-muted-foreground">
                                    {t("projects.list.moreLabels", hiddenLabels)}
                                  </span>
                                {/if}
                              </div>
                            {/if}
                          </button>
                          <div class="flex min-w-0 items-center justify-end gap-1">
                            {#if taskListColumnVisible("status")}
                              <button
                                type="button"
                                class={cn("rounded border px-1.5 py-0.5 text-[0.733333rem]", statusBadgeClass(status))}
                              >
                                {status?.name ?? t("projects.list.status")}
                              </button>
                            {/if}
                            {#if taskListColumnVisible("priority")}
                              <button
                                type="button"
                                class={cn(
                                  "rounded border px-1.5 py-0.5 text-[0.733333rem] disabled:cursor-not-allowed disabled:opacity-60",
                                  priorityClass(task.priority),
                                )}
                                disabled={Boolean(task.archivedAt)}
                                onclick={() => { void projects.setTaskPriority(task, nextPriority(task.priority)); }}
                              >
                                {priorityLabel(task.priority)}
                              </button>
                            {/if}
                            {#if taskListColumnVisible("estimate") && task.estimateMinutes !== undefined}
                              <span class="rounded border border-border bg-background px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground">
                                {estimateLabel(task.estimateMinutes)}
                              </span>
                            {/if}
                            {#if taskListColumnVisible("due") && task.dueDate}
                              <span class="rounded border border-border bg-background px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground">
                                {task.dueDate}
                              </span>
                            {/if}
                            {#each visibleCustomFieldListFields as field (field.id)}
                              {@const customValue = customFieldDisplayValue(task, field)}
                              {#if customValue}
                                <span
                                  class="inline-flex min-w-0 max-w-48 items-center gap-1 rounded border border-border bg-background px-1.5 py-0.5 text-[0.733333rem]"
                                  title={`${field.name}: ${customValue}`}
                                >
                                  <span class="max-w-20 truncate text-muted-foreground">{field.name}</span>
                                  <span class="max-w-28 truncate text-foreground">{customValue}</span>
                                </span>
                              {/if}
                            {/each}
                            {#if taskListColumnVisible("scheduled") && scheduled}
                              <span class="rounded border border-sky-500/40 bg-sky-500/10 px-1.5 py-0.5 text-[0.733333rem] text-sky-700 dark:text-sky-300">
                                {scheduled}
                              </span>
                            {/if}
                            {#if task.archivedAt}
                              <span class={cn("rounded border px-1.5 py-0.5 text-[0.733333rem]", taskArchivedBadgeClass(task))}>
                                {t("projects.taskLifecycle.archived")}
                              </span>
                            {/if}
                            {#if taskListColumnVisible("dependencies")}
                              {#if blockedByCount > 0}
                                <span class="rounded border border-destructive/40 bg-destructive/10 px-1.5 py-0.5 text-[0.733333rem] text-destructive">
                                  {t("projects.list.blockedBy", blockedByCount)}
                                </span>
                              {/if}
                              {#if blocksCount > 0}
                                <span class="rounded border border-amber-500/40 bg-amber-500/10 px-1.5 py-0.5 text-[0.733333rem] text-amber-700 dark:text-amber-300">
                                  {t("projects.list.blocks", blocksCount)}
                                </span>
                              {/if}
                            {/if}
                            <button
                              type="button"
                              class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                              disabled={Boolean(task.archivedAt)}
                              aria-label={t("projects.actions.scheduleTask")}
                              title={t("projects.actions.scheduleTask")}
                              onclick={() => openScheduleForm(task)}
                            >
                              <CalendarDays size={13} strokeWidth={1.75} />
                            </button>
                          </div>
                        </div>
                        {#if schedulingTaskId === task.id && !task.archivedAt}
                          <form
                            class="mt-2 grid gap-2 border-t border-border/70 pt-2 min-[720px]:grid-cols-[minmax(0,1fr)_7rem_6rem_auto_auto]"
                            onsubmit={(event) => { event.preventDefault(); void scheduleTask(task); }}
                          >
                            <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
                              <span>{t("projects.schedule.date")}</span>
                              <input
                                bind:value={scheduleDate}
                                placeholder="YYYY-MM-DD"
                                class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                              />
                            </label>
                            <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
                              <span>{t("projects.schedule.start")}</span>
                              <input
                                bind:value={scheduleStartTime}
                                placeholder="HH:MM"
                                class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                              />
                            </label>
                            <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
                              <span>{t("projects.schedule.duration")}</span>
                              <input
                                type="number"
                                min="1"
                                step="5"
                                bind:value={scheduleDurationMinutes}
                                class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                              />
                            </label>
                            <button
                              type="submit"
                              disabled={schedulePending}
                              class="self-end rounded-md bg-primary px-2 py-1.5 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
                            >
                              {t("projects.schedule.schedule")}
                            </button>
                            <button
                              type="button"
                              class="self-end rounded-md border border-border bg-card px-2 py-1.5 text-[0.8rem] hover:bg-accent"
                              onclick={closeScheduleForm}
                            >
                              {t("common.cancel")}
                            </button>
                            {#if scheduleError}
                              <div class="text-[0.733333rem] text-destructive min-[720px]:col-span-5">
                                {scheduleError}
                              </div>
                            {/if}
                          </form>
                        {/if}
                      </div>
                    {/each}
                  </div>
                </section>
              {/each}
            {/if}
            {#if tasks.length === 0 && allProjectTasks.length > 0}
              <div class="rounded-md border border-dashed border-border px-2 py-3 text-[0.8rem] text-muted-foreground">
                {t("projects.filters.noMatchingTasks")}
              </div>
            {/if}
          </div>
        {:else if projects.activeView === "board"}
          <ProjectBoardView
            {tasks}
            {statuses}
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
        {:else}
          <ProjectSummaryView
            projectId={selectedProjectId}
            {tasks}
            {statuses}
            {todayDate}
            {scheduledTaskIds}
            {scheduledThisWeekMinutes}
            onOpenTask={openTaskDetail}
          />
        {/if}
      </div>
    {:else}
      <div class="flex h-full items-center justify-center p-4 text-[0.866667rem] text-muted-foreground">
        {projects.loading ? t("projects.loading") : t("projects.navigator.empty")}
      </div>
    {/if}
  </section>

  {#if projectSettingsOpen && selectedProjectId}
    <ProjectSettingsPanel
      projectId={selectedProjectId}
      onClose={() => {
        projectSettingsOpen = false;
      }}
      onRevealInactive={() => {
        showInactiveProjects = true;
      }}
    />
  {:else if selectedTaskId}
    <ProjectTaskDetailPanel
      taskId={selectedTaskId}
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
