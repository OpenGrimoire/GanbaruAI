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
  import { getViewport } from "$lib/stores/viewport.svelte";
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
    deriveProjectFilterChips,
    deriveProjectListColumnControls,
    pickProjectTaskModalLayout,
    projectNavigatorPanelGeometry,
    toggleProjectListColumn,
    type ProjectFilterChip,
    type ProjectToolbarPanel,
  } from "$lib/projects/project-toolbar";
  import {
    projectListDropSortOrder,
    projectListSectionDropSortOrder,
    type ProjectListDropPosition,
  } from "$lib/projects/list-drag";
  import ProjectBoardView from "./ProjectBoardView.svelte";
  import ProjectIcon from "./ProjectIcon.svelte";
  import ProjectNavigator from "./ProjectNavigator.svelte";
  import ProjectGanttView from "./ProjectGanttView.svelte";
  import ProjectListScrollbars from "./ProjectListScrollbars.svelte";
  import ProjectSettingsPanel from "./ProjectSettingsPanel.svelte";
  import ProjectSummaryView from "./ProjectSummaryView.svelte";
  import ProjectTaskDetailPanel from "./ProjectTaskDetailPanel.svelte";

  const projects = getProjects();
  const calendar = getCalendar();
  const theme = getTheme();
  const viewport = getViewport();
  const { t } = getLocalization();

  const TASK_STATUS_FILTERS: ProjectTaskStatusFilter[] = ["all", "open", "blocked", "done"];
  const TASK_DUE_FILTERS: ProjectTaskDueFilter[] = ["all", "overdue", "today", "week", "none", "range"];
  const TASK_SCHEDULE_FILTERS: ProjectTaskScheduleFilter[] = ["all", "scheduled", "unscheduled"];
  const TASK_DEPENDENCY_FILTERS: ProjectTaskDependencyFilter[] = ["all", "linked", "blocked_by", "blocking", "none"];
  const TASK_SORT_MODES: ProjectCoreTaskSortMode[] = [...PROJECT_TASK_SORT_MODES];
  const PROJECT_LIST_DRAG_MIME = "application/x-ganbaru-project-list-task";
  const PROJECT_LIST_SECTION_DRAG_MIME = "application/x-ganbaru-project-list-section";
  const LIST_ROW_DRAG_THRESHOLD_PX = 4;
  const LIST_ROW_DRAG_HOLD_MS = 120;
  const PROJECT_LIST_KEYBOARD_SCROLL_PX = 48;
  type TaskCreateTarget = `section:${string}`;
  interface ListRowDragGesture {
    taskId: string;
    pointerId: number;
    startX: number;
    startY: number;
    startedAt: number;
  }
  interface ListSectionDragGesture {
    sectionId: string;
    pointerId: number;
    startX: number;
    startY: number;
    startedAt: number;
  }

  let showInactiveProjects = $state(false);
  let projectNavigatorOpen = $state(false);
  let showInactiveSections = $state(false);
  let showArchivedTasks = $state(false);
  let taskSearch = $state("");
  let taskFinderOpen = $state(false);
  let taskFinderInputElement = $state<HTMLInputElement | null>(null);
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
  let listRowDragGesture = $state<ListRowDragGesture | null>(null);
  let listDraggingSectionId = $state<string | null>(null);
  let listSectionDragOverId = $state<string | null>(null);
  let listSectionDragOverPosition = $state<ProjectListDropPosition | null>(null);
  let listSectionDropPendingId = $state<string | null>(null);
  let listSectionDragGesture = $state<ListSectionDragGesture | null>(null);
  let suppressedTaskOpenTaskId = $state<string | null>(null);
  let selectedTaskId = $state<string | null>(null);
  let selectedTaskIds = $state<string[]>([]);
  let bulkTaskActionPending = $state(false);
  let bulkTaskError = $state<string | null>(null);
  let projectSettingsOpen = $state(false);
  let sectionNameDrafts = $state<Record<string, string>>({});
  let projectToolbarPanel = $state<ProjectToolbarPanel | null>(null);
  let projectNavigatorTriggerElement = $state<HTMLButtonElement | null>(null);
  let projectNavigatorPanelElement = $state<HTMLDivElement | null>(null);
  let projectNavigatorPanelStyle = $state("");
  let sectionOptionsMenuId = $state<string | null>(null);
  let statusMenuTaskId = $state<string | null>(null);
  let projectsRootElement = $state<HTMLDivElement | null>(null);
  let projectViewScrollContainer = $state<HTMLDivElement | null>(null);

  function cssPixelValue(value: string): number {
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) ? parsed : 0;
  }

  function projectListMaxHorizontalScrollLeft(): number {
    const el = projectViewScrollContainer;
    if (!el) return 0;

    const content = el.firstElementChild;
    if (!(content instanceof HTMLElement)) {
      return Math.max(0, el.scrollWidth - el.clientWidth);
    }

    const contentStyle = getComputedStyle(content);
    const paddingRight = cssPixelValue(contentStyle.paddingRight);
    let contentRight = 0;

    for (const child of Array.from(content.children)) {
      if (!(child instanceof HTMLElement)) continue;
      contentRight = Math.max(contentRight, child.offsetLeft + child.offsetWidth);
    }

    if (contentRight <= 0) return Math.max(0, el.scrollWidth - el.clientWidth);
    return Math.max(0, contentRight + paddingRight - el.clientWidth);
  }

  function setProjectListCounterScroll(scrollLeft: number): number {
    const el = projectViewScrollContainer;
    if (!el) return scrollLeft;
    const maxScrollLeft = projectListMaxHorizontalScrollLeft();
    const nextScrollLeft = Math.max(0, Math.min(maxScrollLeft, scrollLeft));
    el.style.setProperty("--project-list-scroll-left", `${nextScrollLeft}px`);
    el.style.setProperty("--project-list-scroll-left-negative", `${-nextScrollLeft}px`);
    return nextScrollLeft;
  }

  function setProjectListHorizontalScroll(scrollLeft: number): number {
    const el = projectViewScrollContainer;
    const nextScrollLeft = setProjectListCounterScroll(scrollLeft);
    if (el && el.scrollLeft !== nextScrollLeft) {
      el.scrollLeft = nextScrollLeft;
    }
    return nextScrollLeft;
  }

  function syncProjectListCounterScroll(): void {
    const el = projectViewScrollContainer;
    if (!el) return;
    setProjectListHorizontalScroll(el.scrollLeft);
  }

  $effect(() => {
    const el = projectViewScrollContainer;
    if (!el || projects.activeView !== "list") return;
    setProjectListCounterScroll(el.scrollLeft);
  });

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
  const taskListGridTemplate = $derived.by(() => [
    "1.5rem",
    "1.75rem",
    "minmax(16rem, 2fr)",
    ...taskListColumns.map(taskListColumnTrack),
    "2.25rem",
  ].join(" "));
  const taskListGridMinWidth = $derived.by(() => {
    const remWidth = Math.max(47, 25 + taskListColumns.length * 8.5);
    return `${remWidth}rem`;
  });
  const activeFilterChips = $derived.by(() => deriveProjectFilterChips({
    search: taskSearch,
    statusLabel: taskStatusFilter === "all" ? undefined : taskStatusFilterLabel(taskStatusFilter),
    sectionLabel: taskSectionFilter === "all" ? undefined : taskSectionFilterLabel(),
    priorityLabel: taskPriorityFilter === "all" ? undefined : priorityLabel(taskPriorityFilter),
    dueLabel: taskDueFilter === "all" ? undefined : taskDueFilterChipLabel(),
    scheduleLabel: taskScheduleFilter === "all" ? undefined : taskScheduleFilterLabel(taskScheduleFilter),
    dependencyLabel: taskDependencyFilter === "all" ? undefined : taskDependencyFilterLabel(taskDependencyFilter),
    labelFilterLabel: taskLabelFilter === "all" ? undefined : taskLabelFilterLabel(taskLabelFilter),
    customFieldFilters: taskCustomFieldFilters,
    customFieldFilterLabel,
  }));
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

  function taskListColumnTrack(column: ProjectTaskListColumn): string {
    if (column === "priority" || column === "estimate" || column === "due") return "minmax(7rem, 0.7fr)";
    if (column === "status" || column === "scheduled" || column === "dependencies") return "minmax(8rem, 0.8fr)";
    return "minmax(9rem, 0.85fr)";
  }

  async function toggleTaskListColumn(column: ProjectTaskListColumn): Promise<void> {
    if (!selectedProjectId) return;
    const nextColumns = toggleProjectListColumn(taskListColumns, column);
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

  function taskSectionFilterLabel(): string {
    if (taskSectionFilter === "all") return t("projects.filters.allSections");
    return sections.find((section) => section.id === taskSectionFilter)?.name
      ?? t("projects.filters.allSections");
  }

  function taskDueFilterChipLabel(): string {
    if (taskDueFilter !== "range") return taskDueFilterLabel(taskDueFilter);
    const start = normalizedTaskDueRangeStart ?? t("projects.filters.dueRangeStart");
    const end = normalizedTaskDueRangeEnd ?? t("projects.filters.dueRangeEnd");
    return t("projects.filters.dueRangeChip", start, end);
  }

  function customFieldFilterLabel(filter: ProjectCustomFieldFilter): string {
    const field = projectCustomFields.find((entry) => entry.id === filter.fieldId);
    const fieldName = field?.name ?? t("projects.columns.customField");
    if (filter.mode === "filled") return t("projects.filters.customFieldChip", fieldName, t("projects.filters.filled"));
    if (filter.mode === "empty") return t("projects.filters.customFieldChip", fieldName, t("projects.filters.empty"));
    if (filter.mode === "checkbox") {
      return t(
        "projects.filters.customFieldChip",
        fieldName,
        filter.checked ? t("projects.customFields.checked") : t("projects.customFields.unchecked"),
      );
    }
    if (filter.mode === "option") {
      const optionName = projectCustomFields
        .flatMap((entry) => projects.customFieldOptionsForField(entry.id))
        .find((option) => option.id === filter.optionId)?.name ?? t("projects.filters.allValues");
      return t("projects.filters.customFieldChip", fieldName, optionName);
    }
    return fieldName;
  }

  function toggleProjectToolbarPanel(panel: ProjectToolbarPanel): void {
    projectToolbarPanel = projectToolbarPanel === panel ? null : panel;
  }

  function focusTaskFinderInput(): void {
    requestAnimationFrame(() => {
      taskFinderInputElement?.focus();
      taskFinderInputElement?.select();
    });
  }

  function openTaskFinder(): void {
    taskFinderOpen = true;
    projectToolbarPanel = null;
    focusTaskFinderInput();
  }

  function clearAndCloseTaskFinder(): void {
    taskSearch = "";
    taskFinderOpen = false;
  }

  function closeTaskFinder(): void {
    taskFinderOpen = false;
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

  function closeOrClearTaskFinder(): void {
    if (taskSearch.trim()) {
      clearAndCloseTaskFinder();
      return;
    }
    closeTaskFinder();
  }

  function projectListKeyboardScrollAllowed(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return true;
    return !target.closest("input, textarea, select, [contenteditable='true'], [role='textbox']");
  }

  function projectsEditableSelectionTarget(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return false;
    return Boolean(target.closest("input, textarea, [contenteditable='true'], [role='textbox']"));
  }

  function projectsSelectionNodeInside(node: Node | null): boolean {
    return Boolean(projectsRootElement && node && projectsRootElement.contains(node));
  }

  function handleProjectListHorizontalKeydown(event: KeyboardEvent): boolean {
    if (projects.activeView !== "list") return false;
    if (event.altKey || event.ctrlKey || event.metaKey) return false;
    if (!projectListKeyboardScrollAllowed(event.target)) return false;
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return false;

    const el = projectViewScrollContainer;
    if (!el) return false;
    const maxScrollLeft = projectListMaxHorizontalScrollLeft();
    if (maxScrollLeft <= 0) return false;

    const delta = event.key === "ArrowRight"
      ? PROJECT_LIST_KEYBOARD_SCROLL_PX
      : -PROJECT_LIST_KEYBOARD_SCROLL_PX;
    const nextScrollLeft = Math.max(0, Math.min(maxScrollLeft, el.scrollLeft + delta));
    if (nextScrollLeft === el.scrollLeft) return false;

    event.preventDefault();
    setProjectListHorizontalScroll(nextScrollLeft);
    return true;
  }

  function handleProjectWindowKeydown(event: KeyboardEvent): void {
    if (event.defaultPrevented) return;
    if (projectSettingsOpen || selectedTaskId) return;
    if (handleProjectListHorizontalKeydown(event)) return;
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

  function handleTaskFinderKeydown(event: KeyboardEvent): void {
    if (event.key !== "Escape") return;
    event.preventDefault();
    if (taskSearch.trim()) {
      clearAndCloseTaskFinder();
      return;
    }
    closeTaskFinder();
  }

  $effect(() => {
    if (!projectNavigatorOpen) return;
    const viewportWidth = viewport.width;
    const viewportHeight = viewport.height;
    void viewportWidth;
    void viewportHeight;
    requestAnimationFrame(refreshProjectNavigatorPanelGeometry);
  });

  function handleProjectWindowPointerDown(event: PointerEvent): void {
    const target = event.target;
    if (!(target instanceof Node)) return;
    if (projectNavigatorOpen
      && !projectNavigatorTriggerElement?.contains(target)
      && !projectNavigatorPanelElement?.contains(target)
    ) {
      projectNavigatorOpen = false;
    }
    if (
      sectionOptionsMenuId
      && target instanceof Element
      && !target.closest("[data-section-options-root='true']")
    ) {
      sectionOptionsMenuId = null;
    }
    if (
      statusMenuTaskId
      && target instanceof Element
      && !target.closest("[data-list-status-menu-root='true']")
    ) {
      statusMenuTaskId = null;
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

  function clearProjectFilterChip(chip: ProjectFilterChip): void {
    if (chip.clearTarget === "search") {
      taskSearch = "";
      return;
    }
    if (chip.clearTarget === "status") {
      taskStatusFilter = "all";
      return;
    }
    if (chip.clearTarget === "section") {
      taskSectionFilter = "all";
      return;
    }
    if (chip.clearTarget === "priority") {
      taskPriorityFilter = "all";
      return;
    }
    if (chip.clearTarget === "due") {
      taskDueFilter = "all";
      taskDueRangeStart = "";
      taskDueRangeEnd = "";
      return;
    }
    if (chip.clearTarget === "schedule") {
      taskScheduleFilter = "all";
      return;
    }
    if (chip.clearTarget === "dependency") {
      taskDependencyFilter = "all";
      return;
    }
    if (chip.clearTarget === "label") {
      taskLabelFilter = "all";
      return;
    }
    const fieldId = chip.clearTarget.slice("custom:".length);
    clearTaskCustomFieldFilter(fieldId);
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

  function listSectionDragEnabled(): boolean {
    return taskGroupBy === "section";
  }

  function subtasksForTask(parent: ProjectTask): ProjectTask[] {
    return showArchivedTasks
      ? projects.subtasksForTaskIncludingArchived(parent.id)
      : projects.subtasksForTask(parent.id);
  }

  function taskById(taskId: string): ProjectTask | undefined {
    return allProjectTasks.find((task) => task.id === taskId);
  }

  function sectionById(sectionId: string): ProjectSection | undefined {
    return sections.find((section) => section.id === sectionId);
  }

  function canStartListTaskDrag(task: ProjectTask): boolean {
    return listDragEnabled()
      && !task.archivedAt
      && !task.parentTaskId
      && listDropPendingTaskId === null;
  }

  function canStartListSectionDrag(section: ProjectSection): boolean {
    return listSectionDragEnabled()
      && !section.archivedAt
      && !section.hiddenAt
      && listSectionDropPendingId === null;
  }

  function listRowDragTargetAllowed(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return true;
    if (target.closest("[data-list-row-drag-source='true']")) return true;
    return !target.closest("button, input, textarea, select, a, [role='button']");
  }

  function listSectionDragTargetAllowed(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return true;
    if (target.closest("[data-list-section-drag-source='true']")) return true;
    return !target.closest("button, textarea, select, a, [role='button']");
  }

  function handleListRowPointerDown(event: PointerEvent, task: ProjectTask): void {
    if (event.button !== 0 || !canStartListTaskDrag(task) || !listRowDragTargetAllowed(event.target)) {
      listRowDragGesture = null;
      return;
    }
    listRowDragGesture = {
      taskId: task.id,
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      startedAt: Date.now(),
    };
  }

  function clearListRowDragGesture(event?: PointerEvent): void {
    if (event && listRowDragGesture && event.pointerId !== listRowDragGesture.pointerId) return;
    listRowDragGesture = null;
  }

  function handleListSectionPointerDown(event: PointerEvent, section: ProjectSection): void {
    if (
      event.button !== 0
      || !canStartListSectionDrag(section)
      || !listSectionDragTargetAllowed(event.target)
    ) {
      listSectionDragGesture = null;
      return;
    }
    listSectionDragGesture = {
      sectionId: section.id,
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      startedAt: Date.now(),
    };
  }

  function clearListSectionDragGesture(event?: PointerEvent): void {
    if (event && listSectionDragGesture && event.pointerId !== listSectionDragGesture.pointerId) return;
    listSectionDragGesture = null;
  }

  function listRowDragGestureReady(event: DragEvent, task: ProjectTask): boolean {
    const gesture = listRowDragGesture;
    if (!gesture || gesture.taskId !== task.id) return false;
    const distance = Math.hypot(event.clientX - gesture.startX, event.clientY - gesture.startY);
    const elapsed = Date.now() - gesture.startedAt;
    return distance >= LIST_ROW_DRAG_THRESHOLD_PX
      || (elapsed >= LIST_ROW_DRAG_HOLD_MS && distance >= 1);
  }

  function listSectionDragGestureReady(event: DragEvent, section: ProjectSection): boolean {
    const gesture = listSectionDragGesture;
    if (!gesture || gesture.sectionId !== section.id) return false;
    const distance = Math.hypot(event.clientX - gesture.startX, event.clientY - gesture.startY);
    const elapsed = Date.now() - gesture.startedAt;
    return distance >= LIST_ROW_DRAG_THRESHOLD_PX
      || (elapsed >= LIST_ROW_DRAG_HOLD_MS && distance >= 1);
  }

  function suppressNextTaskOpen(taskId: string): void {
    suppressedTaskOpenTaskId = taskId;
    window.setTimeout(() => {
      if (suppressedTaskOpenTaskId === taskId) suppressedTaskOpenTaskId = null;
    }, 0);
  }

  function resetListDragTarget(): void {
    listDragOverSectionId = null;
    listDragOverTaskId = null;
    listDragOverPosition = null;
  }

  function resetListSectionDragTarget(): void {
    listSectionDragOverId = null;
    listSectionDragOverPosition = null;
  }

  function listDragTaskId(event: DragEvent): string | null {
    return event.dataTransfer?.getData(PROJECT_LIST_DRAG_MIME) || listDraggingTaskId;
  }

  function listDragSectionId(event: DragEvent): string | null {
    return event.dataTransfer?.getData(PROJECT_LIST_SECTION_DRAG_MIME) || listDraggingSectionId;
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

  function canDropListSection(
    draggedSection: ProjectSection | undefined,
    targetSection: ProjectSection,
  ): draggedSection is ProjectSection {
    return listSectionDragEnabled()
      && !!draggedSection
      && !draggedSection.archivedAt
      && !draggedSection.hiddenAt
      && !targetSection.archivedAt
      && !targetSection.hiddenAt
      && draggedSection.projectId === targetSection.projectId;
  }

  function handleListTaskDragStart(event: DragEvent, task: ProjectTask): void {
    if (!canStartListTaskDrag(task) || !listRowDragGestureReady(event, task)) {
      event.preventDefault();
      return;
    }
    event.stopPropagation();
    listDraggingTaskId = task.id;
    listRowDragGesture = null;
    suppressNextTaskOpen(task.id);
    event.dataTransfer?.setData(PROJECT_LIST_DRAG_MIME, task.id);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function handleListSectionDragStart(event: DragEvent, section: ProjectSection): void {
    if (!canStartListSectionDrag(section) || !listSectionDragGestureReady(event, section)) {
      event.preventDefault();
      return;
    }
    event.stopPropagation();
    listDraggingSectionId = section.id;
    listSectionDragGesture = null;
    resetListDragTarget();
    event.dataTransfer?.setData(PROJECT_LIST_SECTION_DRAG_MIME, section.id);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function handleListTaskDragEnd(): void {
    listDraggingTaskId = null;
    listDropPendingTaskId = null;
    listRowDragGesture = null;
    resetListDragTarget();
  }

  function handleListSectionDragEnd(): void {
    listDraggingSectionId = null;
    listSectionDropPendingId = null;
    listSectionDragGesture = null;
    resetListSectionDragTarget();
  }

  function listRowDropPosition(event: DragEvent): ProjectListDropPosition {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return event.clientY >= rect.top + rect.height / 2 ? "after" : "before";
  }

  function listSectionDropPosition(event: DragEvent): ProjectListDropPosition {
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

  function handleListSectionGroupDragOver(event: DragEvent, section: ProjectSection): void {
    const draggedSection = sectionById(listDragSectionId(event) ?? "");
    if (canDropListSection(draggedSection, section) && draggedSection.id !== section.id) {
      event.preventDefault();
      event.stopPropagation();
      if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
      resetListDragTarget();
      listSectionDragOverId = section.id;
      listSectionDragOverPosition = listSectionDropPosition(event);
      return;
    }
    handleListSectionDragOver(event, section);
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

  async function dropListSection(event: DragEvent, targetSection: ProjectSection): Promise<void> {
    event.preventDefault();
    event.stopPropagation();
    const draggedSection = sectionById(listDragSectionId(event) ?? "");
    if (!canDropListSection(draggedSection, targetSection) || draggedSection.id === targetSection.id) {
      resetListSectionDragTarget();
      return;
    }

    const orderedSections = sections.filter((section) => !section.archivedAt && !section.hiddenAt);
    const nextSortOrder = projectListSectionDropSortOrder({
      orderedSections,
      draggedSectionId: draggedSection.id,
      overSectionId: targetSection.id,
      position: listSectionDropPosition(event),
    });

    if (draggedSection.sortOrder === nextSortOrder) {
      resetListSectionDragTarget();
      return;
    }

    listSectionDropPendingId = draggedSection.id;
    resetListSectionDragTarget();
    try {
      await projects.updateSection(draggedSection, { sortOrder: nextSortOrder });
    } finally {
      listSectionDropPendingId = null;
      listDraggingSectionId = null;
    }
  }

  async function dropListSectionOrTask(event: DragEvent, section: ProjectSection): Promise<void> {
    const draggedSection = sectionById(listDragSectionId(event) ?? "");
    if (canDropListSection(draggedSection, section)) {
      await dropListSection(event, section);
      return;
    }
    await dropListTask(event, section);
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

  function listSectionDropMarkerVisible(
    section: ProjectSection,
    position: ProjectListDropPosition,
  ): boolean {
    return listSectionDragOverId === section.id
      && listSectionDragOverPosition === position;
  }

  function statusForTask(task: ProjectTask): ProjectStatus | undefined {
    return projects.statusById(task.statusId);
  }

  function taskSelected(task: ProjectTask): boolean {
    return selectedTaskIdSet.has(task.id);
  }

  function allTasksSelected(groupTasks: ProjectTask[]): boolean {
    return groupTasks.length > 0 && groupTasks.every((task) => selectedTaskIdSet.has(task.id));
  }

  function someTasksSelected(groupTasks: ProjectTask[]): boolean {
    return groupTasks.some((task) => selectedTaskIdSet.has(task.id));
  }

  function toggleTaskSelection(task: ProjectTask): void {
    selectedTaskIds = taskSelected(task)
      ? selectedTaskIds.filter((taskId) => taskId !== task.id)
      : [...selectedTaskIds, task.id];
  }

  function toggleTaskGroupSelection(groupTasks: ProjectTask[]): void {
    if (groupTasks.length === 0) return;
    const nextIds = new Set(selectedTaskIds);
    if (allTasksSelected(groupTasks)) {
      for (const task of groupTasks) nextIds.delete(task.id);
    } else {
      for (const task of groupTasks) nextIds.add(task.id);
    }
    selectedTaskIds = Array.from(nextIds);
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

  function statusBadgeClass(status: ProjectStatus | undefined): string {
    if (status?.category === "done") return "border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
    if (status?.category === "blocked") return "border-destructive/40 bg-destructive/10 text-destructive";
    if (status?.category === "active") return "border-sky-500/40 bg-sky-500/10 text-sky-700 dark:text-sky-300";
    return "border-border bg-muted/50 text-muted-foreground";
  }

  async function setTaskStatusFromList(task: ProjectTask, status: ProjectStatus): Promise<void> {
    if (task.archivedAt || task.statusId === status.id) {
      statusMenuTaskId = null;
      return;
    }
    await projects.setTasksStatus([task], status.id);
    statusMenuTaskId = null;
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
    const name = sectionDraft.trim();
    if (!selectedProjectId || !name) return;
    await projects.addSection(selectedProjectId, name);
    sectionDraft = "";
  }

  function sectionNameDraft(section: ProjectSection): string {
    return sectionNameDrafts[section.id] ?? section.name;
  }

  function sectionDraftDirty(section: ProjectSection): boolean {
    return sectionNameDraft(section) !== section.name;
  }

  function sectionDraftSaveable(section: ProjectSection): boolean {
    return sectionDraftDirty(section)
      && sectionNameDraft(section).trim().length > 0
      && !section.hiddenAt
      && !section.archivedAt;
  }

  async function saveSection(section: ProjectSection): Promise<void> {
    const name = sectionNameDraft(section).trim();
    if (!name) return;
    await projects.updateSection(section, { name });
    sectionNameDrafts = {
      ...sectionNameDrafts,
      [section.id]: name,
    };
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

  async function toggleSectionCollapsed(section: ProjectSection): Promise<void> {
    await projects.updateSection(section, { collapsed: !section.collapsed });
  }

  function openTaskDetail(task: ProjectTask): void {
    if (suppressedTaskOpenTaskId === task.id) {
      suppressedTaskOpenTaskId = null;
      return;
    }
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

<svelte:window
  onkeydown={handleProjectWindowKeydown}
  onpointerdown={handleProjectWindowPointerDown}
/>

<svelte:document
  onselectstart={handleProjectDocumentSelectStart}
  onselectionchange={handleProjectDocumentSelectionChange}
/>

<div bind:this={projectsRootElement} class="projects-view-root relative flex h-full min-h-0 overflow-hidden text-foreground" style="background-color: var(--cal-bg);">
  <section class="flex min-w-0 flex-1 flex-col">
    {#if selectedProject && selectedGroup}
      <header class="flex shrink-0 flex-col" style="background-color: var(--cal-header-bg);">
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
                  {projectLifecycleLabel(selectedProject.status)}
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
                  selectedProjectId={selectedProjectId}
                  showInactiveProjects={showInactiveProjects}
                  presentation="panel"
                  onShowInactiveProjectsChange={(value) => {
                    showInactiveProjects = value;
                  }}
                  onProjectSelected={() => {
                    selectedTaskId = null;
                    projectSettingsOpen = false;
                    projectNavigatorOpen = false;
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
            onclick={() => toggleProjectToolbarPanel("filters")}
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
            onclick={() => toggleProjectToolbarPanel("customize")}
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
            onclick={() => toggleProjectToolbarPanel("more")}
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
                onclick={() => clearProjectFilterChip(chip)}
              >
                <span class="truncate">{chip.label}</span>
                <X size={12} strokeWidth={1.75} />
              </button>
            {/each}
            <span class="rounded-full border border-border bg-muted/60 px-2 py-1 text-muted-foreground">
              {t("projects.filters.matchingTasks", matchingTaskCount, allProjectTasks.length)}
            </span>
          </div>
        {/if}
        {#if projectToolbarPanel === "filters"}
        <div class="mx-3 my-2 flex min-w-0 flex-wrap items-center gap-2 rounded-md border border-border bg-card p-2 text-[0.766667rem]">
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
          {#if inactiveSectionCount > 0}
            <button
              type="button"
              class={cn(
                "flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-background px-2 font-medium hover:bg-accent hover:text-foreground",
                showInactiveSections ? "text-foreground" : "text-muted-foreground",
              )}
              aria-label={showInactiveSections ? t("projects.filters.hideInactiveSections") : t("projects.filters.showInactiveSections")}
              title={showInactiveSections ? t("projects.filters.hideInactiveSections") : t("projects.filters.showInactiveSections")}
              onclick={() => {
                showInactiveSections = !showInactiveSections;
              }}
            >
              {#if showInactiveSections}
                <EyeOff size={13} strokeWidth={1.75} />
              {:else}
                <Eye size={13} strokeWidth={1.75} />
              {/if}
              <span>
                {showInactiveSections
                  ? t("projects.filters.hideInactiveSectionsShort")
                  : t("projects.filters.showInactiveSectionsShort", inactiveSectionCount)}
              </span>
            </button>
          {/if}
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
        {/if}
        {#if projectToolbarPanel === "customize"}
        <div class="mx-3 my-2 flex min-w-0 flex-wrap items-center gap-2 rounded-md border border-border bg-card p-2 text-[0.766667rem]">
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
            {#each taskListColumnControls as control (control.column)}
              <button
                type="button"
                class={cn(
                  "h-7 max-w-40 shrink-0 rounded px-2 font-medium",
                  control.visible
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
                )}
                aria-pressed={control.visible}
                title={control.label}
                onclick={() => { void toggleTaskListColumn(control.column); }}
              >
                <span class="block truncate">{control.label}</span>
              </button>
            {/each}
          </div>
          {#if savedViewError}
            <div class="basis-full rounded border border-destructive/30 bg-destructive/10 px-2 py-1 text-destructive">
              {savedViewError}
            </div>
          {/if}
        </div>
        {/if}
        {#if projectToolbarPanel === "more"}
          <div class="mx-3 my-2 flex min-w-0 flex-wrap items-center gap-2 rounded-md border border-border bg-card p-2 text-[0.766667rem]">
            <button
              type="button"
              class="flex h-8 shrink-0 items-center gap-1.5 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
              onclick={openProjectSettings}
            >
              <MoreHorizontal size={13} strokeWidth={1.75} />
              <span>{t("projects.header.projectSettings")}</span>
            </button>
            <button
              type="button"
              class="flex h-8 shrink-0 items-center gap-1.5 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
              onclick={openTaskFinder}
            >
              <Search size={13} strokeWidth={1.75} />
              <span>{t("projects.finder.open")}</span>
            </button>
            {#if taskFiltersActive}
              <button
                type="button"
                class="flex h-8 shrink-0 items-center gap-1.5 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
                onclick={clearTaskFilters}
              >
                <RotateCcw size={13} strokeWidth={1.75} />
                <span>{t("projects.filters.reset")}</span>
              </button>
            {/if}
            <span class="h-8 shrink-0 rounded-md border border-border bg-muted/60 px-2 py-1.5 text-muted-foreground">
              {t("projects.filters.matchingTasks", matchingTaskCount, allProjectTasks.length)}
            </span>
          </div>
        {/if}
        {#if selectedTasks.length > 0}
          {@const bulkSchedulableCount = selectedSchedulableTasks().length}
          <div class="mx-3 my-2 flex min-w-0 flex-wrap items-center gap-1 rounded-md border border-border bg-card px-2 py-1 text-[0.766667rem]">
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

      <div class="relative min-h-0 flex-1" style="background-color: var(--cal-bg);">
        <div
          bind:this={projectViewScrollContainer}
          class={cn(
            "h-full min-h-0",
            projects.activeView === "calendar" && "overflow-hidden",
            projects.activeView === "list" && "overflow-auto project-list-scroll",
            projects.activeView !== "calendar" && projects.activeView !== "list" && "overflow-auto",
          )}
          onscroll={syncProjectListCounterScroll}
        >
        {#if projects.activeView === "list"}
          <div class="flex min-h-full flex-col gap-5 p-3">
            {#if taskGroupBy === "section"}
            {#each sections as section (section.id)}
              {@const sectionTasks = tasksForSection(section)}
              {#if listSectionDropMarkerVisible(section, "before")}
                <div class="h-1 rounded-full bg-primary"></div>
              {/if}
              <section
                class={cn(
                  "flex flex-col gap-0 border border-transparent",
                  listDragOverSectionId === section.id && "border-primary/40 bg-primary/5",
                  listDraggingSectionId === section.id && "opacity-50",
                  listSectionDropPendingId === section.id && "opacity-60",
                )}
                style={`min-width: max(100%, ${taskListGridMinWidth});`}
                role="list"
                aria-label={section.name}
                ondragover={(event) => handleListSectionGroupDragOver(event, section)}
                ondrop={(event) => { void dropListSectionOrTask(event, section); }}
              >
                <div
                  class={cn(
                    "project-list-divider project-list-sticky-row group/section-header grid min-h-11 items-center px-1",
                    canStartListSectionDrag(section) && "cursor-grab active:cursor-grabbing",
                  )}
                  style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
                  role="group"
                  aria-label={section.name}
                  draggable={canStartListSectionDrag(section)}
                  onpointerdown={(event) => handleListSectionPointerDown(event, section)}
                  onpointerup={clearListSectionDragGesture}
                  onpointercancel={clearListSectionDragGesture}
                  ondragstart={(event) => handleListSectionDragStart(event, section)}
                  ondragend={handleListSectionDragEnd}
                >
                  <div class="flex h-7 items-center justify-center">
                    <button
                      type="button"
                      class={cn(
                        "flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded border transition-opacity",
                        allTasksSelected(sectionTasks)
                          ? "border-primary bg-primary text-primary-foreground opacity-100"
                          : "border-border bg-background opacity-0 hover:bg-accent group-hover/section-header:opacity-100 group-focus-within/section-header:opacity-100",
                        someTasksSelected(sectionTasks) && !allTasksSelected(sectionTasks) && "border-primary/70 bg-primary/10 text-primary opacity-100",
                      )}
                      aria-label={allTasksSelected(sectionTasks) ? t("projects.actions.unselectTaskGroup", section.name) : t("projects.actions.selectTaskGroup", section.name)}
                      disabled={sectionTasks.length === 0}
                      onclick={() => toggleTaskGroupSelection(sectionTasks)}
                    >
                      {#if allTasksSelected(sectionTasks)}
                        <Check size={13} strokeWidth={2} />
                      {:else if someTasksSelected(sectionTasks)}
                        <span class="h-0.5 w-2.5 rounded-full bg-current"></span>
                      {/if}
                    </button>
                  </div>
                  <button
                    type="button"
                    class="flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
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
                  <div
                    class={cn(
                      "flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded-md border px-2 transition-colors focus-within:border-foreground/50 focus-within:bg-card",
                      sectionDraftDirty(section)
                        ? "border-border bg-card shadow-sm"
                        : "border-transparent bg-transparent hover:bg-card/60",
                    )}
                  >
                    <input
                      value={sectionNameDraft(section)}
                      data-list-section-drag-source="true"
                      class="min-h-7 min-w-0 flex-1 bg-transparent text-[0.866667rem] font-semibold disabled:text-muted-foreground"
                      aria-label={t("projects.list.sectionName")}
                      disabled={Boolean(section.hiddenAt || section.archivedAt)}
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
                    {#if sectionDraftDirty(section)}
                      <button
                        type="button"
                        class="flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded-md bg-muted text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                        disabled={!sectionDraftSaveable(section)}
                        aria-label={t("projects.list.saveSection")}
                        title={t("projects.list.saveSection")}
                        onclick={() => { void saveSection(section); }}
                      >
                        <Check size={14} strokeWidth={2} />
                      </button>
                    {/if}
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
                  <div class="relative shrink-0" data-section-options-root="true">
                    <button
                      type="button"
                      class={cn(
                        "flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground opacity-0 transition-opacity hover:bg-accent hover:text-foreground group-hover/section-header:opacity-100 group-focus-within/section-header:opacity-100",
                        sectionOptionsMenuId === section.id && "bg-accent text-foreground opacity-100",
                      )}
                      aria-label={t("projects.actions.sectionOptions", section.name)}
                      title={t("projects.actions.sectionOptions", section.name)}
                      aria-expanded={sectionOptionsMenuId === section.id}
                      onclick={() => {
                        sectionOptionsMenuId = sectionOptionsMenuId === section.id ? null : section.id;
                      }}
                    >
                      <MoreHorizontal size={14} strokeWidth={1.75} />
                    </button>
                    {#if sectionOptionsMenuId === section.id}
                      <div
                        class="absolute right-0 top-8 z-30 w-52 rounded-lg border border-border bg-popover p-1 text-[0.866667rem] text-popover-foreground shadow-sm"
                        role="menu"
                        aria-label={t("projects.actions.sectionOptions", section.name)}
                      >
                        {#if section.hiddenAt || section.archivedAt}
                          <button
                            type="button"
                            class="flex min-h-9 w-full cursor-pointer items-center gap-2 rounded-md px-2 text-left hover:bg-accent hover:text-foreground"
                            role="menuitem"
                            onclick={() => {
                              sectionOptionsMenuId = null;
                              void restoreSection(section);
                            }}
                          >
                            <ArchiveRestore size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                            <span>{t("projects.actions.restoreSection", section.name)}</span>
                          </button>
                        {:else}
                          <button
                            type="button"
                            class="flex min-h-9 w-full cursor-pointer items-center gap-2 rounded-md px-2 text-left hover:bg-accent hover:text-foreground"
                            role="menuitem"
                            onclick={() => {
                              sectionOptionsMenuId = null;
                              void hideSection(section);
                            }}
                          >
                            <EyeOff size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                            <span>{t("projects.actions.hideSection", section.name)}</span>
                          </button>
                          <button
                            type="button"
                            class="flex min-h-9 w-full cursor-pointer items-center gap-2 rounded-md px-2 text-left hover:bg-accent hover:text-foreground"
                            role="menuitem"
                            onclick={() => {
                              sectionOptionsMenuId = null;
                              void archiveSection(section);
                            }}
                          >
                            <Archive size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                            <span>{t("projects.actions.archiveSection", section.name)}</span>
                          </button>
                        {/if}
                      </div>
                    {/if}
                  </div>
                  </div>
                </div>
                {#if !section.collapsed && !section.archivedAt && !section.hiddenAt}
                  <div
                    class="project-list-divider group/column-header grid min-h-11 items-center px-1 text-[0.866667rem] font-semibold text-foreground"
                    style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
                  >
                    <div></div>
                    <div></div>
                    <div class="truncate px-2">{t("projects.list.name")}</div>
                    {#each taskListColumns as column (column)}
                      <div class="truncate px-2">{taskListColumnLabel(column)}</div>
                    {/each}
                    <div></div>
                  </div>
                  <div class="grid">
                    {#each sectionTasks as task (task.id)}
                      {@const status = statusForTask(task)}
                      {@const subtasks = subtasksForTask(task)}
                      {@const scheduled = scheduledLabel(task.id)}
                      {@const taskLabels = visibleTaskLabels(task)}
                      {@const hiddenLabels = hiddenTaskLabelCount(task)}
                      {@const blockedByCount = blockedByDependencies(task).length}
                      {@const blocksCount = blocksDependencies(task).length}
                      {#if listDropMarkerVisible(section, task, "before")}
                        <div class="h-1 rounded-full bg-primary"></div>
                      {/if}
                      <div
                        role="listitem"
                        class={cn(
                          "project-list-divider group/row relative grid min-h-11 items-center px-1 transition-colors hover:bg-accent/35",
                          selectedTaskId === task.id && "bg-accent/40 ring-1 ring-inset ring-primary/20",
                          task.archivedAt && "opacity-70",
                          listDraggingTaskId === task.id && "opacity-50",
                          listDropPendingTaskId === task.id && "opacity-60",
                        )}
                        style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
                        draggable={canStartListTaskDrag(task)}
                        onpointerdown={(event) => handleListRowPointerDown(event, task)}
                        onpointerup={clearListRowDragGesture}
                        onpointercancel={clearListRowDragGesture}
                        ondragstart={(event) => handleListTaskDragStart(event, task)}
                        ondragend={handleListTaskDragEnd}
                        ondragover={(event) => handleListRowDragOver(event, section, task)}
                        ondrop={(event) => { void dropListTask(event, section, task, listRowDropPosition(event)); }}
                      >
                        <div class="flex h-full items-center justify-center">
                          <button
                            type="button"
                            class={cn(
                              "flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded border transition-opacity",
                              taskSelected(task)
                                ? "border-primary bg-primary text-primary-foreground opacity-100"
                                : "border-border bg-background opacity-0 hover:bg-accent group-hover/row:opacity-100 group-focus-within/row:opacity-100",
                            )}
                            aria-label={taskSelected(task) ? t("projects.actions.unselectTask", task.title) : t("projects.actions.selectTask", task.title)}
                            onclick={() => toggleTaskSelection(task)}
                          >
                            {#if taskSelected(task)}
                              <Check size={13} strokeWidth={2} />
                            {/if}
                          </button>
                        </div>
                        <div class="flex h-full items-center justify-center">
                          <button
                            type="button"
                            class="flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground opacity-0 transition-opacity hover:bg-accent hover:text-foreground group-hover/row:opacity-100 group-focus-within/row:opacity-100"
                            aria-label={t("projects.actions.openTaskDetails", task.title)}
                            onclick={() => openTaskDetail(task)}
                          >
                            <ChevronRight size={14} strokeWidth={1.75} />
                          </button>
                        </div>
                        <button
                          type="button"
                          data-list-row-drag-source="true"
                          class="min-w-0 cursor-pointer px-2 text-left"
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
                          {#if task.archivedAt}
                            <span class={cn("mt-1 inline-flex w-fit rounded border px-1.5 py-0.5 text-[0.733333rem]", taskArchivedBadgeClass(task))}>
                              {t("projects.taskLifecycle.archived")}
                            </span>
                          {/if}
                        </button>
                        {#each taskListColumns as column (column)}
                          <div class="flex min-w-0 items-center px-2">
                            {#if column === "status"}
                              <div class="relative max-w-full" data-list-status-menu-root="true">
                                <button
                                  type="button"
                                  class={cn(
                                    "max-w-full cursor-pointer truncate rounded border px-1.5 py-0.5 text-[0.733333rem] disabled:cursor-not-allowed disabled:opacity-60",
                                    statusBadgeClass(status),
                                  )}
                                  disabled={Boolean(task.archivedAt)}
                                  aria-haspopup="menu"
                                  aria-expanded={statusMenuTaskId === task.id}
                                  onclick={() => {
                                    statusMenuTaskId = statusMenuTaskId === task.id ? null : task.id;
                                  }}
                                >
                                  {status?.name ?? t("projects.list.status")}
                                </button>
                                {#if statusMenuTaskId === task.id}
                                  <div
                                    class="absolute left-0 top-7 z-30 w-44 rounded-lg border border-border bg-popover p-1 text-[0.8rem] text-popover-foreground shadow-sm"
                                    role="menu"
                                  >
                                    {#each statuses as nextStatus (nextStatus.id)}
                                      <button
                                        type="button"
                                        class="flex min-h-8 w-full cursor-pointer items-center justify-between gap-2 rounded-md px-2 text-left hover:bg-accent hover:text-foreground"
                                        role="menuitemradio"
                                        aria-checked={task.statusId === nextStatus.id}
                                        onclick={() => { void setTaskStatusFromList(task, nextStatus); }}
                                      >
                                        <span class={cn("min-w-0 truncate rounded border px-1.5 py-0.5 text-[0.733333rem]", statusBadgeClass(nextStatus))}>
                                          {nextStatus.name}
                                        </span>
                                        {#if task.statusId === nextStatus.id}
                                          <Check size={13} strokeWidth={2} class="shrink-0 text-muted-foreground" />
                                        {/if}
                                      </button>
                                    {/each}
                                  </div>
                                {/if}
                              </div>
                            {:else if column === "priority"}
                              <button
                                type="button"
                                class={cn(
                                  "max-w-full cursor-pointer truncate rounded border px-1.5 py-0.5 text-[0.733333rem] disabled:cursor-not-allowed disabled:opacity-60",
                                  priorityClass(task.priority),
                                )}
                                disabled={Boolean(task.archivedAt)}
                                onclick={() => { void projects.setTaskPriority(task, nextPriority(task.priority)); }}
                              >
                                {priorityLabel(task.priority)}
                              </button>
                            {:else if column === "estimate"}
                              {#if task.estimateMinutes !== undefined}
                                <span class="truncate rounded border border-border bg-background px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground">
                                  {estimateLabel(task.estimateMinutes)}
                                </span>
                              {/if}
                            {:else if column === "due"}
                              {#if task.dueDate}
                                <span class="truncate text-[0.8rem] text-muted-foreground">{task.dueDate}</span>
                              {/if}
                            {:else if column === "scheduled"}
                              {#if scheduled}
                                <span class="truncate rounded border border-sky-500/40 bg-sky-500/10 px-1.5 py-0.5 text-[0.733333rem] text-sky-700 dark:text-sky-300">
                                  {scheduled}
                                </span>
                              {/if}
                            {:else if column === "dependencies"}
                              <div class="flex min-w-0 flex-wrap gap-1">
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
                              </div>
                            {:else}
                              {@const customFieldId = customFieldIdFromTaskListColumn(column)}
                              {@const customField = customFieldId ? projectCustomFields.find((field) => field.id === customFieldId) : undefined}
                              {#if customField}
                                {@const customValue = customFieldDisplayValue(task, customField)}
                                {#if customValue}
                                  <span class="min-w-0 truncate text-[0.8rem] text-muted-foreground" title={`${customField.name}: ${customValue}`}>
                                    {customValue}
                                  </span>
                                {/if}
                              {/if}
                            {/if}
                          </div>
                        {/each}
                        <div class="flex items-center justify-end">
                          <button
                            type="button"
                            class="flex h-6 w-6 shrink-0 cursor-pointer items-center justify-center rounded text-muted-foreground opacity-0 transition-opacity hover:bg-accent hover:text-foreground group-hover/row:opacity-100 group-focus-within/row:opacity-100 disabled:cursor-not-allowed disabled:opacity-40"
                            disabled={Boolean(task.archivedAt)}
                            aria-label={t("projects.actions.scheduleTask")}
                            title={t("projects.actions.scheduleTask")}
                            onclick={() => openScheduleForm(task)}
                          >
                            <CalendarDays size={13} strokeWidth={1.75} />
                          </button>
                        </div>
                        {#if schedulingTaskId === task.id && !task.archivedAt}
                          <form
                            class="project-list-inline-divider col-span-full mt-2 grid gap-2 pt-2 min-[720px]:grid-cols-[minmax(0,1fr)_7rem_6rem_auto_auto]"
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
                              class="self-end cursor-pointer rounded-md bg-primary px-2 py-1.5 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
                            >
                              {t("projects.schedule.schedule")}
                            </button>
                            <button
                              type="button"
                              class="self-end cursor-pointer rounded-md border border-border bg-card px-2 py-1.5 text-[0.8rem] hover:bg-accent"
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
                          <div class="project-list-inline-divider col-span-full mt-1 grid gap-1 pt-1">
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
                                    "flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded border",
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
                                  class="min-w-0 cursor-pointer text-left"
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
                  </div>
                  <div
                    class="project-list-divider grid items-center px-1 py-1.5"
                    style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
                  >
                    <div></div>
                    <div></div>
                    <form class="group flex min-w-0 items-center gap-2 px-2" onsubmit={(event) => { event.preventDefault(); void submitSectionTask(section.id); }}>
                      <span
                        class={cn(
                          "flex h-5 w-5 shrink-0 items-center justify-center text-muted-foreground transition-opacity",
                          (sectionTaskDrafts[section.id] ?? "").trim() ? "opacity-0" : "opacity-100",
                        )}
                        aria-hidden="true"
                      >
                        <Plus size={15} strokeWidth={1.75} />
                      </span>
                      <input
                        value={sectionTaskDrafts[section.id] ?? ""}
                        oninput={(event) => {
                          sectionTaskDrafts = {
                            ...sectionTaskDrafts,
                            [section.id]: event.currentTarget.value,
                          };
                        }}
                        placeholder={t("projects.list.addTaskInSection", section.name)}
                        class="min-h-8 min-w-0 flex-1 bg-transparent text-[0.866667rem] text-foreground placeholder:text-muted-foreground"
                      />
                      <button
                        type="submit"
                        disabled={taskCreatePendingTarget !== null}
                        class="flex h-7 shrink-0 cursor-pointer items-center justify-center rounded-md border border-border bg-card px-2 text-[0.733333rem] text-muted-foreground opacity-0 transition-opacity hover:bg-accent hover:text-foreground group-hover:opacity-100 group-focus-within:opacity-100 disabled:cursor-not-allowed disabled:opacity-60"
                      >
                        {t("common.save")}
                      </button>
                    </form>
                    {#each taskListColumns as column (column)}
                      <div></div>
                    {/each}
                    <div></div>
                    {#if taskCreateErrorFor(sectionTaskCreateTarget(section.id))}
                      <div class="col-span-full rounded border border-destructive/30 bg-destructive/10 px-2 py-1 text-[0.733333rem] text-destructive">
                        {taskCreateErrorFor(sectionTaskCreateTarget(section.id))}
                      </div>
                    {/if}
                  </div>
                {/if}
              </section>
              {#if listSectionDropMarkerVisible(section, "after")}
                <div class="h-1 rounded-full bg-primary"></div>
              {/if}
            {/each}
            <div
              class="project-list-sticky-row grid items-center px-1 py-1.5"
              style={`grid-template-columns: ${taskListGridTemplate}; min-width: max(100%, ${taskListGridMinWidth});`}
            >
              <div></div>
              <div></div>
              <form class="group flex min-w-0 items-center gap-2 px-2" onsubmit={(event) => { event.preventDefault(); void submitSection(); }}>
                <span
                  class={cn(
                    "flex h-5 w-5 shrink-0 items-center justify-center text-muted-foreground transition-opacity",
                    sectionDraft.trim() ? "opacity-0" : "opacity-100",
                  )}
                  aria-hidden="true"
                >
                  <Plus size={15} strokeWidth={1.75} />
                </span>
                <input
                  bind:value={sectionDraft}
                  placeholder={t("projects.header.addSection")}
                  class="min-h-8 min-w-0 flex-1 bg-transparent text-[0.866667rem] text-foreground placeholder:text-muted-foreground"
                />
                <button
                  type="submit"
                  class="flex h-7 shrink-0 cursor-pointer items-center justify-center rounded-md border border-border bg-card px-2 text-[0.733333rem] text-muted-foreground opacity-0 transition-opacity hover:bg-accent hover:text-foreground group-hover:opacity-100 group-focus-within:opacity-100"
                >
                  {t("common.save")}
                </button>
              </form>
              {#each taskListColumns as column (column)}
                <div></div>
              {/each}
              <div></div>
            </div>
            {:else}
              {#each listTaskGroups as group (group.id)}
                <section
                  class="flex flex-col gap-0"
                  style={`min-width: max(100%, ${taskListGridMinWidth});`}
                >
                  <div
                    class="project-list-divider project-list-sticky-row group/list-group-header grid min-h-11 items-center px-1"
                    style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
                  >
                    <div class="flex h-7 items-center justify-center">
                      <button
                        type="button"
                        class={cn(
                          "flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded border transition-opacity",
                          allTasksSelected(group.tasks)
                            ? "border-primary bg-primary text-primary-foreground opacity-100"
                            : "border-border bg-background opacity-0 hover:bg-accent group-hover/list-group-header:opacity-100 group-focus-within/list-group-header:opacity-100",
                          someTasksSelected(group.tasks) && !allTasksSelected(group.tasks) && "border-primary/70 bg-primary/10 text-primary opacity-100",
                        )}
                        aria-label={allTasksSelected(group.tasks) ? t("projects.actions.unselectTaskGroup", taskListGroupTitle(group.value)) : t("projects.actions.selectTaskGroup", taskListGroupTitle(group.value))}
                        disabled={group.tasks.length === 0}
                        onclick={() => toggleTaskGroupSelection(group.tasks)}
                      >
                        {#if allTasksSelected(group.tasks)}
                          <Check size={13} strokeWidth={2} />
                        {:else if someTasksSelected(group.tasks)}
                          <span class="h-0.5 w-2.5 rounded-full bg-current"></span>
                        {/if}
                      </button>
                    </div>
                    <div></div>
                    <span class="min-w-0 truncate px-2 text-[0.866667rem] font-semibold">
                      {taskListGroupTitle(group.value)}
                    </span>
                  </div>
                  <div
                    class="project-list-divider group/list-column-header grid min-h-11 items-center px-1 text-[0.866667rem] font-semibold text-foreground"
                    style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
                  >
                    <div></div>
                    <div></div>
                    <div class="truncate px-2">{t("projects.list.name")}</div>
                    {#each taskListColumns as column (column)}
                      <div class="truncate px-2">{taskListColumnLabel(column)}</div>
                    {/each}
                    <div></div>
                  </div>
                  <div class="grid">
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
                          "project-list-divider group/row relative grid min-h-11 items-center px-1 transition-colors hover:bg-accent/35",
                          selectedTaskId === task.id && "bg-accent/40 ring-1 ring-inset ring-primary/20",
                          task.archivedAt && "opacity-70",
                        )}
                        style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
                      >
                        <div class="flex h-full items-center justify-center">
                          <button
                            type="button"
                            class={cn(
                              "flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded border transition-opacity",
                              taskSelected(task)
                                ? "border-primary bg-primary text-primary-foreground opacity-100"
                                : "border-border bg-background opacity-0 hover:bg-accent group-hover/row:opacity-100 group-focus-within/row:opacity-100",
                            )}
                            aria-label={taskSelected(task) ? t("projects.actions.unselectTask", task.title) : t("projects.actions.selectTask", task.title)}
                            onclick={() => toggleTaskSelection(task)}
                          >
                            {#if taskSelected(task)}
                              <Check size={13} strokeWidth={2} />
                            {/if}
                          </button>
                        </div>
                        <div class="flex h-full items-center justify-center">
                          <button
                            type="button"
                            class="flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground opacity-0 transition-opacity hover:bg-accent hover:text-foreground group-hover/row:opacity-100 group-focus-within/row:opacity-100"
                            aria-label={t("projects.actions.openTaskDetails", task.title)}
                            onclick={() => openTaskDetail(task)}
                          >
                            <ChevronRight size={14} strokeWidth={1.75} />
                          </button>
                        </div>
                        <button
                          type="button"
                          class="min-w-0 cursor-pointer px-2 text-left"
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
                          {#if task.archivedAt}
                            <span class={cn("mt-1 inline-flex w-fit rounded border px-1.5 py-0.5 text-[0.733333rem]", taskArchivedBadgeClass(task))}>
                              {t("projects.taskLifecycle.archived")}
                            </span>
                          {/if}
                        </button>
                        {#each taskListColumns as column (column)}
                          <div class="flex min-w-0 items-center px-2">
                            {#if column === "status"}
                              <div class="relative max-w-full" data-list-status-menu-root="true">
                                <button
                                  type="button"
                                  class={cn(
                                    "max-w-full cursor-pointer truncate rounded border px-1.5 py-0.5 text-[0.733333rem] disabled:cursor-not-allowed disabled:opacity-60",
                                    statusBadgeClass(status),
                                  )}
                                  disabled={Boolean(task.archivedAt)}
                                  aria-haspopup="menu"
                                  aria-expanded={statusMenuTaskId === task.id}
                                  onclick={() => {
                                    statusMenuTaskId = statusMenuTaskId === task.id ? null : task.id;
                                  }}
                                >
                                  {status?.name ?? t("projects.list.status")}
                                </button>
                                {#if statusMenuTaskId === task.id}
                                  <div
                                    class="absolute left-0 top-7 z-30 w-44 rounded-lg border border-border bg-popover p-1 text-[0.8rem] text-popover-foreground shadow-sm"
                                    role="menu"
                                  >
                                    {#each statuses as nextStatus (nextStatus.id)}
                                      <button
                                        type="button"
                                        class="flex min-h-8 w-full cursor-pointer items-center justify-between gap-2 rounded-md px-2 text-left hover:bg-accent hover:text-foreground"
                                        role="menuitemradio"
                                        aria-checked={task.statusId === nextStatus.id}
                                        onclick={() => { void setTaskStatusFromList(task, nextStatus); }}
                                      >
                                        <span class={cn("min-w-0 truncate rounded border px-1.5 py-0.5 text-[0.733333rem]", statusBadgeClass(nextStatus))}>
                                          {nextStatus.name}
                                        </span>
                                        {#if task.statusId === nextStatus.id}
                                          <Check size={13} strokeWidth={2} class="shrink-0 text-muted-foreground" />
                                        {/if}
                                      </button>
                                    {/each}
                                  </div>
                                {/if}
                              </div>
                            {:else if column === "priority"}
                              <button
                                type="button"
                                class={cn(
                                  "max-w-full cursor-pointer truncate rounded border px-1.5 py-0.5 text-[0.733333rem] disabled:cursor-not-allowed disabled:opacity-60",
                                  priorityClass(task.priority),
                                )}
                                disabled={Boolean(task.archivedAt)}
                                onclick={() => { void projects.setTaskPriority(task, nextPriority(task.priority)); }}
                              >
                                {priorityLabel(task.priority)}
                              </button>
                            {:else if column === "estimate"}
                              {#if task.estimateMinutes !== undefined}
                                <span class="truncate rounded border border-border bg-background px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground">
                                  {estimateLabel(task.estimateMinutes)}
                                </span>
                              {/if}
                            {:else if column === "due"}
                              {#if task.dueDate}
                                <span class="truncate text-[0.8rem] text-muted-foreground">{task.dueDate}</span>
                              {/if}
                            {:else if column === "scheduled"}
                              {#if scheduled}
                                <span class="truncate rounded border border-sky-500/40 bg-sky-500/10 px-1.5 py-0.5 text-[0.733333rem] text-sky-700 dark:text-sky-300">
                                  {scheduled}
                                </span>
                              {/if}
                            {:else if column === "dependencies"}
                              <div class="flex min-w-0 flex-wrap gap-1">
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
                              </div>
                            {:else}
                              {@const customFieldId = customFieldIdFromTaskListColumn(column)}
                              {@const customField = customFieldId ? projectCustomFields.find((field) => field.id === customFieldId) : undefined}
                              {#if customField}
                                {@const customValue = customFieldDisplayValue(task, customField)}
                                {#if customValue}
                                  <span class="min-w-0 truncate text-[0.8rem] text-muted-foreground" title={`${customField.name}: ${customValue}`}>
                                    {customValue}
                                  </span>
                                {/if}
                              {/if}
                            {/if}
                          </div>
                        {/each}
                        <div class="flex items-center justify-end">
                          <button
                            type="button"
                            class="flex h-6 w-6 shrink-0 cursor-pointer items-center justify-center rounded text-muted-foreground opacity-0 transition-opacity hover:bg-accent hover:text-foreground group-hover/row:opacity-100 group-focus-within/row:opacity-100 disabled:cursor-not-allowed disabled:opacity-40"
                            disabled={Boolean(task.archivedAt)}
                            aria-label={t("projects.actions.scheduleTask")}
                            title={t("projects.actions.scheduleTask")}
                            onclick={() => openScheduleForm(task)}
                          >
                            <CalendarDays size={13} strokeWidth={1.75} />
                          </button>
                        </div>
                        {#if schedulingTaskId === task.id && !task.archivedAt}
                          <form
                            class="project-list-inline-divider col-span-full mt-2 grid gap-2 pt-2 min-[720px]:grid-cols-[minmax(0,1fr)_7rem_6rem_auto_auto]"
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
                              class="self-end cursor-pointer rounded-md bg-primary px-2 py-1.5 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
                            >
                              {t("projects.schedule.schedule")}
                            </button>
                            <button
                              type="button"
                              class="self-end cursor-pointer rounded-md border border-border bg-card px-2 py-1.5 text-[0.8rem] hover:bg-accent"
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
        {#if projects.activeView === "list"}
          <ProjectListScrollbars
            scrollContainer={projectViewScrollContainer}
            getMaxScrollLeft={projectListMaxHorizontalScrollLeft}
            onScrollPositionChange={setProjectListHorizontalScroll}
          />
        {/if}
      </div>
    {:else}
      <div class="flex h-full flex-col items-center justify-center gap-3 p-4 text-center text-[0.866667rem] text-muted-foreground">
        <div>{projects.loading ? t("projects.loading") : t("projects.navigator.empty")}</div>
        {#if !projects.loading}
          <button
            type="button"
            class="flex min-h-9 items-center gap-1.5 rounded-md border border-border bg-background px-3 text-[0.8rem] font-medium text-foreground hover:bg-accent"
            onclick={openProjectNavigator}
          >
            <Folder size={14} strokeWidth={1.75} />
            <span>{t("projects.navigator.open")}</span>
          </button>
        {/if}
      </div>
    {/if}
  </section>

  {#if selectedProject && (taskFinderOpen || taskSearch.trim().length > 0)}
    <div class="pointer-events-none absolute inset-x-0 bottom-4 z-40 flex justify-center px-3">
      <div class="pointer-events-auto flex min-h-10 w-[min(32rem,100%)] items-center gap-2 rounded-lg border border-border bg-card px-2">
        <Search size={15} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
        <input
          bind:this={taskFinderInputElement}
          bind:value={taskSearch}
          aria-label={t("projects.finder.label")}
          placeholder={t("projects.header.searchPlaceholder")}
          class="min-w-0 flex-1 bg-transparent text-[0.866667rem] placeholder:text-muted-foreground"
          onkeydown={handleTaskFinderKeydown}
        />
        <span class="hidden shrink-0 rounded-md bg-muted/70 px-2 py-1 text-[0.733333rem] text-muted-foreground min-[520px]:inline">
          {t("projects.filters.matchingTasks", matchingTaskCount, allProjectTasks.length)}
        </span>
        <span class="hidden shrink-0 rounded-md border border-border px-2 py-1 text-[0.733333rem] text-muted-foreground min-[420px]:inline">
          {t("projects.finder.shortcut")}
        </span>
        <button
          type="button"
          class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
          aria-label={taskSearch.trim() ? t("projects.finder.clear") : t("common.close")}
          onclick={closeOrClearTaskFinder}
        >
          <X size={14} strokeWidth={1.75} />
        </button>
      </div>
    </div>
  {/if}

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

  .project-list-divider {
    position: relative;
    --project-list-divider-left: 3rem;
    --project-list-divider-right: 0.25rem;
  }

  .project-list-divider::after {
    position: absolute;
    right: var(--project-list-divider-right);
    bottom: 0;
    left: var(--project-list-divider-left);
    height: 0;
    border-bottom: 1px solid var(--cal-gridline);
    content: "";
    pointer-events: none;
  }

  .project-list-sticky-row.project-list-divider::after {
    transform: translateX(var(--project-list-scroll-left-negative, 0px));
    will-change: transform;
  }

  .project-list-inline-divider {
    position: relative;
    --project-list-divider-left: 3rem;
    --project-list-divider-right: 0.25rem;
  }

  .project-list-inline-divider::before {
    position: absolute;
    top: 0;
    right: var(--project-list-divider-right);
    left: var(--project-list-divider-left);
    height: 0;
    border-bottom: 1px solid var(--cal-gridline);
    content: "";
    pointer-events: none;
  }

  .project-list-sticky-row {
    position: relative;
    z-index: 1;
    transform: translateX(var(--project-list-scroll-left, 0px));
    background-color: var(--cal-bg);
    will-change: transform;
  }

  .project-list-scroll {
    overflow-x: hidden;
    overflow-y: auto;
    padding-right: 0.5rem;
    padding-bottom: 0.5rem;
    scrollbar-width: none;
  }

  .project-list-scroll::-webkit-scrollbar {
    display: none;
  }
</style>
