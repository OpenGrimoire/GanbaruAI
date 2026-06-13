<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";
  import { onMount } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import ArrowUpDown from "@lucide/svelte/icons/arrow-up-down";
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
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
  import ColorPicker from "$lib/components/calendar/ColorPicker.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import CalendarView from "$lib/components/calendar/CalendarView.svelte";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { formatCalendarDate, getEventColor } from "$lib/components/calendar/utils";
  import {
    COUNT_PRESET_RHYTHMS,
    createPresetPomodoroConfig,
    type PomodoroPresetKey,
  } from "$lib/pomodoro/rhythm";
  import { cn } from "$lib/utils";
  import type {
    CalendarEvent,
    CalendarViewMode,
    EventColor,
  } from "$lib/components/calendar/types";
  import type {
    Project,
    ProjectChecklistItem,
    ProjectCustomField,
    ProjectCustomFieldFilter,
    ProjectCustomFieldOption,
    ProjectCustomFieldType,
    ProjectCustomFieldValue,
    ProjectCoreTaskSortMode,
    ProjectGroup,
    ProjectLabel,
    ProjectLifecycleStatus,
    ProjectLinkableEvent,
    ProjectPriority,
    ProjectSavedTaskView,
    ProjectSection,
    ProjectStatus,
    ProjectStatusCategory,
    ProjectTask,
    ProjectTaskChangeEvent,
    ProjectTaskDependencyFilter,
    ProjectTaskDueFilter,
    ProjectTaskGroupMode,
    ProjectTaskLabelFilter,
    ProjectTaskListColumn,
    ProjectTaskScheduleFilter,
    ProjectTaskSortDirection,
    ProjectTaskSortMode,
    ProjectTaskStatusFilter,
    ProjectTaskType,
    ProjectTemplateId,
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
    PROJECT_TEMPLATE_IDS,
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
    manualStatusCompare,
    projectTaskCustomFieldKey,
  } from "$lib/projects/task-view";
  import {
    projectBoardDropSortOrder,
    type ProjectBoardDropPosition,
  } from "$lib/projects/board-drag";
  import {
    projectListDropSortOrder,
    type ProjectListDropPosition,
  } from "$lib/projects/list-drag";
  import ProjectIcon from "./ProjectIcon.svelte";
  import {
    buildProjectDependencyCascadeProposal,
    buildProjectGanttDatePatch,
    buildProjectGanttTimeline,
    type ProjectGanttDateInteraction,
    type ProjectDependencyCascadeItem,
    type ProjectGanttDependencyEdge,
    type ProjectGanttRow,
    type ProjectGanttTick,
  } from "$lib/projects/gantt";

  const projects = getProjects();
  const calendar = getCalendar();
  const theme = getTheme();
  const { t } = getLocalization();

  const PROJECT_ICON_OPTIONS = [
    "folder",
    "repeat",
    "apple",
    "graduation-cap",
    "book-open",
    "dumbbell",
    "bath",
    "heart",
    "sparkles",
    "clapperboard",
    "smile",
    "bed",
  ] as const;
  const PROJECT_POMODORO_OPTIONS = Object.keys(COUNT_PRESET_RHYTHMS) as PomodoroPresetKey[];
  const PROJECT_STATUS_CATEGORIES: ProjectStatusCategory[] = ["not_started", "active", "blocked", "done"];
  const TASK_STATUS_FILTERS: ProjectTaskStatusFilter[] = ["all", "open", "blocked", "done"];
  const TASK_DUE_FILTERS: ProjectTaskDueFilter[] = ["all", "overdue", "today", "week", "none", "range"];
  const TASK_SCHEDULE_FILTERS: ProjectTaskScheduleFilter[] = ["all", "scheduled", "unscheduled"];
  const TASK_DEPENDENCY_FILTERS: ProjectTaskDependencyFilter[] = ["all", "linked", "blocked_by", "blocking", "none"];
  const TASK_SORT_MODES: ProjectCoreTaskSortMode[] = [...PROJECT_TASK_SORT_MODES];
  type ProjectLabelColorDraft = EventColor | "none";
  const PROJECT_BOARD_DRAG_MIME = "application/x-ganbaru-project-task";
  const PROJECT_LIST_DRAG_MIME = "application/x-ganbaru-project-list-task";

  let projectSearch = $state("");
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
  let sectionDraft = $state("");
  let groupDraft = $state("");
  let createGroupOpen = $state(false);
  let createProjectGroupId = $state<string | null>(null);
  let projectDraftByGroup = $state<Record<string, string>>({});
  let projectTemplateDraftByGroup = $state<Record<string, ProjectTemplateId>>({});
  let editingGroupId = $state<string | null>(null);
  let groupEditorName = $state("");
  let groupEditorIcon = $state("folder");
  let groupEditorColor = $state<EventColor | undefined>(undefined);
  let groupEditorSaving = $state(false);
  let groupEditorError = $state<string | null>(null);
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
  let dependencyCascadeOpen = $state(false);
  let dependencyCascadeApplying = $state(false);
  let dependencyCascadeError = $state<string | null>(null);
  let boardDraggingTaskId = $state<string | null>(null);
  let boardDragOverStatusId = $state<string | null>(null);
  let boardDragOverTaskId = $state<string | null>(null);
  let boardDragOverPosition = $state<ProjectBoardDropPosition | "column" | null>(null);
  let boardDropPendingTaskId = $state<string | null>(null);
  let listDraggingTaskId = $state<string | null>(null);
  let listDragOverSectionId = $state<string | null>(null);
  let listDragOverTaskId = $state<string | null>(null);
  let listDragOverPosition = $state<ProjectListDropPosition | "section" | null>(null);
  let listDropPendingTaskId = $state<string | null>(null);
  let ganttDateDrag = $state<{
    taskId: string;
    interaction: ProjectGanttDateInteraction;
    startClientX: number;
    dayWidthPx: number;
    pointerId: number;
    dayDelta: number;
  } | null>(null);
  let ganttDatePendingTaskId = $state<string | null>(null);
  let ganttSuppressClickTaskId = $state<string | null>(null);
  let selectedTaskId = $state<string | null>(null);
  let selectedTaskIds = $state<string[]>([]);
  let detailDraftTaskId = $state<string | null>(null);
  let detailDraftUpdatedAt = $state<string | null>(null);
  let detailTitle = $state("");
  let detailDescription = $state("");
  let detailSectionId = $state("");
  let detailStatusId = $state("");
  let detailPriority = $state<ProjectPriority>("normal");
  let detailTaskType = $state<ProjectTaskType>("task");
  let detailEstimateMinutes = $state("");
  let detailStartDate = $state("");
  let detailDueDate = $state("");
  let detailTargetEndDate = $state("");
  let detailBlockerReason = $state("");
  let detailChangeReason = $state("");
  let detailMilestone = $state(false);
  let detailSaving = $state(false);
  let detailError = $state<string | null>(null);
  let bulkTaskActionPending = $state(false);
  let bulkTaskError = $state<string | null>(null);
  let subtaskDraft = $state("");
  let checklistDraft = $state("");
  let labelDraft = $state("");
  let checklistTitleDrafts = $state<Record<string, string>>({});
  let customFieldTextDrafts = $state<Record<string, string>>({});
  let customFieldNumberDrafts = $state<Record<string, string>>({});
  let customFieldDateDrafts = $state<Record<string, string>>({});
  let customFieldCheckboxDrafts = $state<Record<string, boolean>>({});
  let customFieldSelectDrafts = $state<Record<string, string>>({});
  let customFieldMultiDrafts = $state<Record<string, string[]>>({});
  let dependencySearch = $state("");
  let parentTaskSearch = $state("");
  let eventLinkSearch = $state("");
  let eventLinkStartDate = $state("");
  let eventLinkEndDate = $state("");
  let eventLinkResults = $state<ProjectLinkableEvent[]>([]);
  let eventLinkSearchPending = $state(false);
  let eventLinkSearchError = $state<string | null>(null);
  let projectSettingsOpen = $state(false);
  let projectDraftId = $state<string | null>(null);
  let projectDraftUpdatedAt = $state<string | null>(null);
  let projectGroupDraft = $state("");
  let projectNameDraft = $state("");
  let projectIconDraft = $state("folder");
  let projectStatusDraft = $state<ProjectLifecycleStatus>("active");
  let projectColorDraft = $state<EventColor | undefined>(undefined);
  let projectDurationDraft = $state("60");
  let projectPomodoroDraft = $state<PomodoroPresetKey | "none">("none");
  let projectIdleTimeoutDraft = $state("");
  let projectFocusPlaylistDraft = $state("");
  let projectBreakPlaylistDraft = $state("");
  let projectWorkEnvironmentDraft = $state("");
  let projectBlockerRulesetDraft = $state("");
  let projectSettingsSaving = $state(false);
  let projectSettingsError = $state<string | null>(null);
  let statusNameDrafts = $state<Record<string, string>>({});
  let statusCategoryDrafts = $state<Record<string, ProjectStatusCategory>>({});
  let newStatusName = $state("");
  let newStatusCategory = $state<ProjectStatusCategory>("active");
  let labelNameDrafts = $state<Record<string, string>>({});
  let labelColorDrafts = $state<Record<string, ProjectLabelColorDraft>>({});
  let newLabelName = $state("");
  let newLabelColor = $state<ProjectLabelColorDraft>("none");
  let pendingDeleteLabelId = $state<string | null>(null);
  let customFieldNameDrafts = $state<Record<string, string>>({});
  let customFieldOptionNameDrafts = $state<Record<string, string>>({});
  let newCustomFieldName = $state("");
  let newCustomFieldType = $state<ProjectCustomFieldType>("text");
  let newCustomFieldOptionDrafts = $state<Record<string, string>>({});
  let pendingDeleteCustomFieldId = $state<string | null>(null);
  let pendingDeleteCustomFieldOptionId = $state<string | null>(null);
  let sectionNameDrafts = $state<Record<string, string>>({});
  let eventLinkSearchRunId = 0;

  const selectedProject = $derived(projects.selectedProject);
  const selectedGroup = $derived(projects.selectedGroup);
  const selectedProjectId = $derived(selectedProject?.id ?? null);
  $effect(() => {
    if (!selectedProjectId || projects.projectDataLoaded(selectedProjectId)) return;
    void projects.ensureProjectData(selectedProjectId).catch((error) => {
      console.error("load selected project data failed", error);
    });
  });
  const visibleProjectGroups = $derived.by(() => projects.visibleGroups());
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
  const pendingDeleteLabel = $derived.by(() =>
    pendingDeleteLabelId ? projectLabels.find((label) => label.id === pendingDeleteLabelId) : undefined
  );
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
  const pendingDeleteCustomField = $derived.by(() =>
    pendingDeleteCustomFieldId
      ? projectCustomFields.find((field) => field.id === pendingDeleteCustomFieldId)
      : undefined
  );
  const pendingDeleteCustomFieldOption = $derived.by(() => {
    if (!pendingDeleteCustomFieldOptionId) return undefined;
    for (const field of projectCustomFields) {
      const option = projects.customFieldOptionsForField(field.id)
        .find((entry) => entry.id === pendingDeleteCustomFieldOptionId);
      if (option) return option;
    }
    return undefined;
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
  const normalizedProjectSearch = $derived(projectSearch.trim().toLowerCase());
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
  const ganttTimeline = $derived.by(() => buildProjectGanttTimeline({
    tasks,
    statuses,
    dependencies: projects.dependencies,
    dependencyBlockedTaskIds,
    today: todayDate,
  }));
  const dependencyCascadeProposal = $derived.by(() => buildProjectDependencyCascadeProposal({
    tasks,
    dependencies: projects.dependencies,
  }));
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
  const detailDirty = $derived.by(() => {
    if (!selectedTask) return false;
    return detailTitle !== selectedTask.title
      || detailDescription !== selectedTask.description
      || detailSectionId !== selectedTask.sectionId
      || detailStatusId !== selectedTask.statusId
      || detailPriority !== selectedTask.priority
      || detailTaskType !== selectedTask.taskType
      || detailEstimateMinutes !== String(selectedTask.estimateMinutes ?? "")
      || detailStartDate !== (selectedTask.startDate ?? "")
      || detailDueDate !== (selectedTask.dueDate ?? "")
      || detailTargetEndDate !== (selectedTask.targetEndDate ?? "")
      || detailBlockerReason !== (selectedTask.blockerReason ?? "")
      || detailMilestone !== selectedTask.milestone;
  });
  const projectSettingsDirty = $derived.by(() => {
    if (!selectedProject) return false;
    return projectNameDraft !== selectedProject.name
      || projectGroupDraft !== selectedProject.groupId
      || projectIconDraft !== selectedProject.icon
      || projectStatusDraft !== selectedProject.status
      || projectColorDraft !== selectedProject.color
      || projectDurationDraft !== String(selectedProject.defaultEventDurationMinutes)
      || projectPomodoroDraft !== (selectedProject.defaultPomodoroPresetKey ?? "none")
      || projectIdleTimeoutDraft !== String(selectedProject.defaultIdleTimeoutMinutes ?? "")
      || projectFocusPlaylistDraft !== (selectedProject.focusPlaylistId ?? "")
      || projectBreakPlaylistDraft !== (selectedProject.breakPlaylistId ?? "")
      || projectWorkEnvironmentDraft !== (selectedProject.workEnvironmentId ?? "")
      || projectBlockerRulesetDraft !== (selectedProject.blockerRulesetId ?? "");
  });

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
      detailDraftTaskId = null;
      detailDraftUpdatedAt = null;
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
    if (!selectedTask) return;
    if (
      detailDraftTaskId !== selectedTask.id
      || (!detailDirty && detailDraftUpdatedAt !== selectedTask.updatedAt)
    ) {
      loadTaskDetailDraft(selectedTask);
    }
  });

  $effect(() => {
    if (!projectSettingsOpen || !selectedProject) return;
    if (
      projectDraftId !== selectedProject.id
      || (!projectSettingsDirty && projectDraftUpdatedAt !== selectedProject.updatedAt)
    ) {
      loadProjectSettingsDraft(selectedProject);
    }
  });

  $effect(() => {
    const task = selectedTask;
    const projectId = selectedProjectId;
    const query = eventLinkSearch;
    const startDateDraft = eventLinkStartDate;
    const endDateDraft = eventLinkEndDate;
    if (!task || !projectId) {
      eventLinkResults = [];
      eventLinkSearchPending = false;
      eventLinkSearchError = null;
      return;
    }

    let startDate: string | undefined;
    let endDate: string | undefined;
    try {
      startDate = normalizeOptionalDate(startDateDraft);
      endDate = normalizeOptionalDate(endDateDraft);
      if (startDate && endDate && startDate > endDate) {
        throw new Error(t("projects.detail.eventLinkInvalidDateRange"));
      }
    } catch (error) {
      eventLinkSearchRunId++;
      eventLinkSearchPending = false;
      eventLinkSearchError = error instanceof Error ? error.message : String(error);
      eventLinkResults = [];
      return;
    }

    const runId = ++eventLinkSearchRunId;
    eventLinkSearchPending = true;
    eventLinkSearchError = null;
    const timeoutId = setTimeout(() => {
      void projects.searchLinkableEvents(projectId, task.id, query, startDate, endDate, 12)
        .then((results) => {
          if (runId !== eventLinkSearchRunId) return;
          eventLinkResults = results;
        })
        .catch((error) => {
          if (runId !== eventLinkSearchRunId) return;
          eventLinkSearchError = t(
            "projects.detail.eventLinkSearchFailed",
            error instanceof Error ? error.message : String(error),
          );
          eventLinkResults = [];
        })
        .finally(() => {
          if (runId === eventLinkSearchRunId) {
            eventLinkSearchPending = false;
          }
        });
    }, query.trim() ? 150 : 0);

    return () => {
      clearTimeout(timeoutId);
    };
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

  function filteredProjectsForGroup(groupId: string) {
    const groupProjects = showInactiveProjects
      ? projects.projectsForGroupIncludingInactive(groupId)
      : projects.projectsForGroup(groupId);
    if (!normalizedProjectSearch) return groupProjects;
    return groupProjects.filter((project) =>
      project.name.toLowerCase().includes(normalizedProjectSearch)
    );
  }

  function groupVisible(groupId: string, groupName: string): boolean {
    return filteredProjectsForGroup(groupId).length > 0
      || groupName.toLowerCase().includes(normalizedProjectSearch);
  }

  function adjacentNavigatorGroup(group: ProjectGroup, direction: -1 | 1): ProjectGroup | undefined {
    const index = visibleProjectGroups.findIndex((entry) => entry.id === group.id);
    if (index < 0) return undefined;
    return visibleProjectGroups[index + direction];
  }

  function adjacentProjectInGroup(project: Project, direction: -1 | 1): Project | undefined {
    const ordered = filteredProjectsForGroup(project.groupId);
    const index = ordered.findIndex((entry) => entry.id === project.id);
    if (index < 0) return undefined;
    return ordered[index + direction];
  }

  function nextProjectSortOrderForGroup(groupId: string, excludeProjectId: string): number {
    return Math.max(
      0,
      ...projects.projectsForGroupIncludingInactive(groupId)
        .filter((project) => project.id !== excludeProjectId)
        .map((project) => project.sortOrder),
    ) + 1000;
  }

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

  function taskTypeLabel(taskType: ProjectTaskType): string {
    if (taskType === "bug") return t("projects.taskType.bug");
    if (taskType === "habit") return t("projects.taskType.habit");
    if (taskType === "milestone") return t("projects.taskType.milestone");
    return t("projects.taskType.task");
  }

  function projectTemplateLabel(templateId: ProjectTemplateId): string {
    if (templateId === "software") return t("projects.templates.software");
    if (templateId === "course") return t("projects.templates.course");
    if (templateId === "routine") return t("projects.templates.routine");
    if (templateId === "reading") return t("projects.templates.reading");
    if (templateId === "chores") return t("projects.templates.chores");
    return t("projects.templates.blank");
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

  function statusCategoryLabel(category: ProjectStatusCategory): string {
    if (category === "active") return t("projects.statusCategory.active");
    if (category === "blocked") return t("projects.statusCategory.blocked");
    if (category === "done") return t("projects.statusCategory.done");
    return t("projects.statusCategory.notStarted");
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

  function ganttRowsForSection(section: ProjectSection): ProjectGanttRow[] {
    const datedTaskIds = new Set(ganttTimeline.rows.map((row) => row.task.id));
    return ganttTimeline.rows.filter((row) =>
      row.task.sectionId === section.id
      && (!row.task.parentTaskId || !datedTaskIds.has(row.task.parentTaskId))
    );
  }

  function ganttSubtaskRows(parent: ProjectTask): ProjectGanttRow[] {
    return ganttTimeline.rows.filter((row) => row.task.parentTaskId === parent.id);
  }

  function percentStyle(leftPercent: number): string {
    return `left: ${leftPercent.toFixed(3)}%;`;
  }

  function ganttBarStyle(row: ProjectGanttRow): string {
    return `left: ${row.leftPercent.toFixed(3)}%; width: max(${row.widthPercent.toFixed(3)}%, 0.75rem);`;
  }

  function ganttDateControlClass(row: ProjectGanttRow): string {
    return cn(
      ganttRowTone(row),
      ganttDatePendingTaskId === row.task.id && "opacity-60",
      ganttDateDrag?.taskId === row.task.id && "ring-2 ring-ring",
    );
  }

  function ganttResizeHandleStyle(row: ProjectGanttRow, edge: "start" | "end"): string {
    const leftPercent = edge === "start"
      ? row.leftPercent
      : Math.min(100, row.leftPercent + row.widthPercent);
    return `left: ${leftPercent.toFixed(3)}%;`;
  }

  function ganttTrackForPointer(event: PointerEvent): HTMLElement | null {
    return (event.currentTarget as HTMLElement).closest<HTMLElement>("[data-gantt-track]");
  }

  function startGanttDateDrag(
    event: PointerEvent,
    row: ProjectGanttRow,
    interaction: ProjectGanttDateInteraction,
  ): void {
    if (row.task.archivedAt || ganttDatePendingTaskId) return;
    const track = ganttTrackForPointer(event);
    if (!track || ganttTimeline.totalDays <= 0) return;
    event.preventDefault();
    event.stopPropagation();
    const rect = track.getBoundingClientRect();
    const dayWidthPx = rect.width / ganttTimeline.totalDays;
    if (!Number.isFinite(dayWidthPx) || dayWidthPx <= 0) return;
    ganttDateDrag = {
      taskId: row.task.id,
      interaction,
      startClientX: event.clientX,
      dayWidthPx,
      pointerId: event.pointerId,
      dayDelta: 0,
    };
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function updateGanttDateDrag(event: PointerEvent): void {
    if (!ganttDateDrag || ganttDateDrag.pointerId !== event.pointerId) return;
    event.preventDefault();
    const dayDelta = Math.round((event.clientX - ganttDateDrag.startClientX) / ganttDateDrag.dayWidthPx);
    if (dayDelta === ganttDateDrag.dayDelta) return;
    ganttDateDrag = { ...ganttDateDrag, dayDelta };
  }

  function clearGanttSuppressedClick(taskId: string): void {
    setTimeout(() => {
      if (ganttSuppressClickTaskId === taskId) ganttSuppressClickTaskId = null;
    }, 0);
  }

  async function commitGanttDateDrag(event: PointerEvent, row: ProjectGanttRow): Promise<void> {
    const drag = ganttDateDrag;
    if (!drag || drag.pointerId !== event.pointerId) return;
    event.preventDefault();
    event.stopPropagation();
    ganttDateDrag = null;
    if (drag.dayDelta === 0) return;
    ganttSuppressClickTaskId = row.task.id;
    clearGanttSuppressedClick(row.task.id);
    const patch = buildProjectGanttDatePatch(row.task, drag.interaction, drag.dayDelta);
    if (!patch) return;
    ganttDatePendingTaskId = row.task.id;
    try {
      await projects.updateTask(row.task, patch);
    } finally {
      ganttDatePendingTaskId = null;
    }
  }

  function cancelGanttDateDrag(event: PointerEvent): void {
    if (!ganttDateDrag || ganttDateDrag.pointerId !== event.pointerId) return;
    event.preventDefault();
    ganttDateDrag = null;
  }

  function openGanttTaskFromBar(task: ProjectTask): void {
    if (ganttSuppressClickTaskId === task.id) {
      ganttSuppressClickTaskId = null;
      return;
    }
    openTaskDetail(task);
  }

  function ganttTickStyle(tick: ProjectGanttTick): string {
    return percentStyle(tick.leftPercent);
  }

  function ganttEdgeStyle(edge: ProjectGanttDependencyEdge): string {
    return `left: ${edge.lineStartPercent.toFixed(3)}%; width: max(${edge.lineWidthPercent.toFixed(3)}%, 0.5rem);`;
  }

  function ganttDateRangeLabel(startDate: string, endDate: string): string {
    return startDate === endDate ? startDate : t("projects.gantt.dateRange", startDate, endDate);
  }

  function cascadeItemRangeLabel(item: ProjectDependencyCascadeItem, next: boolean): string {
    return next
      ? ganttDateRangeLabel(item.nextRangeStart, item.nextRangeEnd)
      : ganttDateRangeLabel(item.originalRangeStart, item.originalRangeEnd);
  }

  function cascadeShiftLabel(item: ProjectDependencyCascadeItem): string {
    return t("projects.gantt.cascadeShiftDays", item.shiftDays);
  }

  function ganttRowTone(row: ProjectGanttRow): string {
    if (row.overdue) return "border-destructive bg-destructive text-destructive-foreground";
    if (row.blocked) return "border-amber-600 bg-amber-500 text-black";
    if (row.done) return "border-emerald-600 bg-emerald-500 text-white";
    return "border-primary bg-primary text-primary-foreground";
  }

  function ganttDependencyLabel(row: ProjectGanttRow): string | undefined {
    if (row.blockedByCount > 0 && row.blocksCount > 0) {
      return t("projects.gantt.dependenciesBoth", row.blockedByCount, row.blocksCount);
    }
    if (row.blockedByCount > 0) return t("projects.gantt.blockedBy", row.blockedByCount);
    if (row.blocksCount > 0) return t("projects.gantt.blocks", row.blocksCount);
    return undefined;
  }

  function ganttEdgesFrom(taskId: string): ProjectGanttDependencyEdge[] {
    return ganttTimeline.dependencyEdges.filter((edge) => edge.blockingTaskId === taskId);
  }

  function ganttConflictsForTask(taskId: string): ProjectGanttDependencyEdge[] {
    return ganttTimeline.dependencyEdges.filter((edge) =>
      edge.violated && (edge.blockingTaskId === taskId || edge.blockedTaskId === taskId)
    );
  }

  function ganttEdgeTitle(edge: ProjectGanttDependencyEdge): string {
    return edge.violated
      ? t("projects.gantt.dependencyConflict", edge.blockingTitle, edge.blockedTitle)
      : t("projects.gantt.dependencyOk", edge.blockingTitle, edge.blockedTitle);
  }

  async function applyDependencyCascadeProposal(): Promise<void> {
    if (dependencyCascadeProposal.items.length === 0) return;
    const itemsByTaskId = new Map(dependencyCascadeProposal.items.map((item) => [item.taskId, item]));
    const tasksToUpdate = tasks.filter((task) => itemsByTaskId.has(task.id));
    dependencyCascadeApplying = true;
    dependencyCascadeError = null;
    try {
      await projects.updateTasks(tasksToUpdate, (task) => {
        const item = itemsByTaskId.get(task.id);
        if (!item) return {};
        return {
          startDate: item.nextStartDate,
          dueDate: item.nextDueDate,
          targetEndDate: item.nextTargetEndDate,
        };
      });
      dependencyCascadeOpen = false;
    } catch (error) {
      dependencyCascadeError = t(
        "projects.gantt.cascadeApplyFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      dependencyCascadeApplying = false;
    }
  }

  function openGanttDependencyTarget(edge: ProjectGanttDependencyEdge): void {
    const blockedTask = taskById(edge.blockedTaskId);
    if (blockedTask) openTaskDetail(blockedTask);
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

  function pomodoroPresetLabel(preset: PomodoroPresetKey): string {
    if (preset === "creative") return t("projects.pomodoro.creative");
    if (preset === "balanced") return t("projects.pomodoro.balanced");
    if (preset === "deep") return t("projects.pomodoro.deep");
    if (preset === "extended") return t("projects.pomodoro.extended");
    return t("projects.pomodoro.adaptive");
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

  function tasksForStatus(status: ProjectStatus): ProjectTask[] {
    const statusTasks = tasks.filter((task) => task.statusId === status.id && !task.parentTaskId);
    if (taskSortMode !== "manual") return statusTasks;
    return [...statusTasks].sort((a, b) =>
      taskSortDirection === "asc" ? manualStatusCompare(a, b) : manualStatusCompare(b, a)
    );
  }

  function boardOrderTasksForStatus(status: ProjectStatus): ProjectTask[] {
    if (taskSortMode === "manual") return tasksForStatus(status);
    const projectId = selectedProjectId;
    if (!projectId) return [];
    return projects.topLevelTasksForStatus(projectId, status.id);
  }

  function resetBoardDragTarget(): void {
    boardDragOverStatusId = null;
    boardDragOverTaskId = null;
    boardDragOverPosition = null;
  }

  function boardDragTaskId(event: DragEvent): string | null {
    return event.dataTransfer?.getData(PROJECT_BOARD_DRAG_MIME) || boardDraggingTaskId;
  }

  function canDropBoardTask(task: ProjectTask | undefined, status: ProjectStatus): task is ProjectTask {
    return !!task
      && !task.archivedAt
      && !task.parentTaskId
      && task.projectId === status.projectId;
  }

  function handleBoardTaskDragStart(event: DragEvent, task: ProjectTask): void {
    if (task.archivedAt || task.parentTaskId) {
      event.preventDefault();
      return;
    }
    boardDraggingTaskId = task.id;
    event.dataTransfer?.setData(PROJECT_BOARD_DRAG_MIME, task.id);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function handleBoardTaskDragEnd(): void {
    boardDraggingTaskId = null;
    boardDropPendingTaskId = null;
    resetBoardDragTarget();
  }

  function boardCardDropPosition(event: DragEvent): ProjectBoardDropPosition {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return event.clientY >= rect.top + rect.height / 2 ? "after" : "before";
  }

  function handleBoardCardDragOver(event: DragEvent, status: ProjectStatus, task: ProjectTask): void {
    const dragged = taskById(boardDragTaskId(event) ?? "");
    if (!canDropBoardTask(dragged, status) || dragged.id === task.id) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    boardDragOverStatusId = status.id;
    boardDragOverTaskId = task.id;
    boardDragOverPosition = boardCardDropPosition(event);
  }

  function handleBoardColumnDragOver(event: DragEvent, status: ProjectStatus): void {
    const dragged = taskById(boardDragTaskId(event) ?? "");
    if (!canDropBoardTask(dragged, status)) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    boardDragOverStatusId = status.id;
    boardDragOverTaskId = null;
    boardDragOverPosition = "column";
  }

  async function dropBoardTask(
    event: DragEvent,
    status: ProjectStatus,
    targetTask?: ProjectTask,
    position?: ProjectBoardDropPosition,
  ): Promise<void> {
    event.preventDefault();
    event.stopPropagation();
    const dragged = taskById(boardDragTaskId(event) ?? "");
    if (!canDropBoardTask(dragged, status) || dragged.id === targetTask?.id) {
      resetBoardDragTarget();
      return;
    }

    const orderedTasks = boardOrderTasksForStatus(status);
    const nextStatusSortOrder = projectBoardDropSortOrder({
      orderedTasks,
      draggedTaskId: dragged.id,
      overTaskId: targetTask?.id,
      position,
      sortDirection: taskSortDirection,
    });

    if (dragged.statusId === status.id && dragged.statusSortOrder === nextStatusSortOrder) {
      resetBoardDragTarget();
      return;
    }

    boardDropPendingTaskId = dragged.id;
    resetBoardDragTarget();
    try {
      await projects.updateTask(dragged, {
        statusId: status.id,
        statusSortOrder: nextStatusSortOrder,
      });
    } finally {
      boardDropPendingTaskId = null;
      boardDraggingTaskId = null;
    }
  }

  function boardDropMarkerVisible(
    status: ProjectStatus,
    task: ProjectTask,
    position: ProjectBoardDropPosition,
  ): boolean {
    return boardDragOverStatusId === status.id
      && boardDragOverTaskId === task.id
      && boardDragOverPosition === position;
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

  function dependencyCandidateTasks(task: ProjectTask): ProjectTask[] {
    const existingBlockingTaskIds = new Set(
      blockedByDependencies(task).map((dependency) => dependency.blockingTaskId),
    );
    const query = dependencySearch.trim().toLowerCase();
    return allProjectTasks
      .filter((candidate) =>
        candidate.id !== task.id
        && !existingBlockingTaskIds.has(candidate.id)
        && (!query
          || candidate.title.toLowerCase().includes(query)
          || candidate.description.toLowerCase().includes(query))
      )
      .slice(0, 8);
  }

  function taskHasAnySubtasks(task: ProjectTask): boolean {
    return projects.subtasksForTaskIncludingArchived(task.id).length > 0;
  }

  function parentTaskCandidateTasks(task: ProjectTask): ProjectTask[] {
    const query = parentTaskSearch.trim().toLowerCase();
    return activeProjectTasks
      .filter((candidate) =>
        candidate.projectId === task.projectId
        && !candidate.parentTaskId
        && candidate.id !== task.id
        && (!query || candidate.title.toLowerCase().includes(query))
      )
      .sort((a, b) => a.sectionSortOrder - b.sectionSortOrder || a.title.localeCompare(b.title))
      .slice(0, 8);
  }

  function eventLinkCandidateEvents(task: ProjectTask): ProjectLinkableEvent[] {
    const linkedEventIds = new Set(projects.eventLinksForTask(task.id).map((link) => link.eventId));
    return eventLinkResults
      .filter((event) => !linkedEventIds.has(event.id))
      .slice(0, 8);
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

  function labelCandidateLabels(task: ProjectTask): ProjectLabel[] {
    const query = labelDraft.trim().toLowerCase();
    return projects.unlinkedLabelsForTask(task)
      .filter((label) => !query || label.name.toLowerCase().includes(query))
      .slice(0, 8);
  }

  function canCreateLabel(task: ProjectTask): boolean {
    const name = labelDraft.trim();
    if (!name) return false;
    return !projects.labelsForProject(task.projectId)
      .some((label) => label.name.trim().toLowerCase() === name.toLowerCase());
  }

  function customFieldTypeLabel(fieldType: ProjectCustomFieldType): string {
    if (fieldType === "number") return t("projects.customFields.typeNumber");
    if (fieldType === "date") return t("projects.customFields.typeDate");
    if (fieldType === "select") return t("projects.customFields.typeSelect");
    if (fieldType === "multi_select") return t("projects.customFields.typeMultiSelect");
    if (fieldType === "checkbox") return t("projects.customFields.typeCheckbox");
    if (fieldType === "url") return t("projects.customFields.typeUrl");
    return t("projects.customFields.typeText");
  }

  function customFieldAcceptsOptions(field: ProjectCustomField): boolean {
    return field.fieldType === "select" || field.fieldType === "multi_select";
  }

  function customFieldNameDraftValue(field: ProjectCustomField): string {
    return customFieldNameDrafts[field.id] ?? field.name;
  }

  function customFieldNameExists(name: string, ignoredFieldId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return projectCustomFields.some((field) =>
      field.id !== ignoredFieldId && field.name.trim().toLowerCase() === normalized
    );
  }

  function customFieldDraftDirty(field: ProjectCustomField): boolean {
    return customFieldNameDraftValue(field) !== field.name;
  }

  function adjacentCustomField(field: ProjectCustomField, direction: -1 | 1): ProjectCustomField | undefined {
    const index = projectCustomFields.findIndex((entry) => entry.id === field.id);
    if (index < 0) return undefined;
    return projectCustomFields[index + direction];
  }

  function customFieldOptions(field: ProjectCustomField): ProjectCustomFieldOption[] {
    return projects.customFieldOptionsForField(field.id);
  }

  function customFieldOptionNameDraftValue(option: ProjectCustomFieldOption): string {
    return customFieldOptionNameDrafts[option.id] ?? option.name;
  }

  function customFieldOptionNameExists(fieldId: string, name: string, ignoredOptionId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return projects.customFieldOptionsForField(fieldId).some((option) =>
      option.id !== ignoredOptionId && option.name.trim().toLowerCase() === normalized
    );
  }

  function customFieldOptionDraftDirty(option: ProjectCustomFieldOption): boolean {
    return customFieldOptionNameDraftValue(option) !== option.name;
  }

  function adjacentCustomFieldOption(
    option: ProjectCustomFieldOption,
    direction: -1 | 1,
  ): ProjectCustomFieldOption | undefined {
    const options = projects.customFieldOptionsForField(option.fieldId);
    const index = options.findIndex((entry) => entry.id === option.id);
    if (index < 0) return undefined;
    return options[index + direction];
  }

  function fieldForCustomFieldOption(option: ProjectCustomFieldOption | undefined): ProjectCustomField | undefined {
    return option ? projectCustomFields.find((field) => field.id === option.fieldId) : undefined;
  }

  function customFieldValueDirty(task: ProjectTask, field: ProjectCustomField): boolean {
    const value = projects.customFieldValueForTask(task.id, field.id);
    const optionValues = projects.customFieldOptionValuesForTask(task.id, field.id);
    if (field.fieldType === "text" || field.fieldType === "url") {
      return (customFieldTextDrafts[field.id] ?? "") !== (value?.textValue ?? "");
    }
    if (field.fieldType === "number") {
      return (customFieldNumberDrafts[field.id] ?? "") !== String(value?.numberValue ?? "");
    }
    if (field.fieldType === "date") {
      return (customFieldDateDrafts[field.id] ?? "") !== (value?.dateValue ?? "");
    }
    if (field.fieldType === "checkbox") {
      return (customFieldCheckboxDrafts[field.id] ?? false) !== (value?.checkboxValue ?? false);
    }
    if (field.fieldType === "select") {
      return (customFieldSelectDrafts[field.id] ?? "none") !== (optionValues[0]?.id ?? "none");
    }
    const currentIds = optionValues.map((option) => option.id).sort();
    const draftIds = [...(customFieldMultiDrafts[field.id] ?? [])].sort();
    return currentIds.join("\u0000") !== draftIds.join("\u0000");
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

  function labelNameDraftValue(label: ProjectLabel): string {
    return labelNameDrafts[label.id] ?? label.name;
  }

  function labelColorDraftValue(label: ProjectLabel): EventColor | undefined {
    const draft = labelColorDrafts[label.id];
    if (draft === "none") return undefined;
    return draft ?? label.color;
  }

  function newLabelColorValue(): EventColor | undefined {
    return newLabelColor === "none" ? undefined : newLabelColor;
  }

  function labelDraftDirty(label: ProjectLabel): boolean {
    return labelNameDraftValue(label) !== label.name
      || labelColorDraftValue(label) !== label.color;
  }

  function labelNameExists(name: string, ignoredLabelId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return projectLabels.some((label) =>
      label.id !== ignoredLabelId && label.name.trim().toLowerCase() === normalized
    );
  }

  function labelColorDotStyle(color: EventColor | undefined): string {
    if (color === undefined) return "";
    return `background-color: ${getEventColor(color, theme.current).bg};`;
  }

  function labelColorSwatchClass(color: EventColor | undefined): string {
    return color === undefined ? "border-border bg-muted/50" : "border-transparent";
  }

  function adjacentLabel(label: ProjectLabel, direction: -1 | 1): ProjectLabel | undefined {
    const index = projectLabels.findIndex((entry) => entry.id === label.id);
    if (index < 0) return undefined;
    return projectLabels[index + direction];
  }

  function linkedEventRowsForTask(task: ProjectTask): ProjectLinkableEvent[] {
    const linkedEventIds = new Set(projects.eventLinksForTask(task.id).map((link) => link.eventId));
    const rowsById = new Map(eventLinkResults.map((event) => [event.id, event]));
    for (const event of allProjectEvents) {
      if (linkedEventIds.has(event.id) && !rowsById.has(event.id)) {
        rowsById.set(event.id, calendarEventToLinkableEvent(event));
      }
    }
    return Array.from(rowsById.values())
      .filter((event) => linkedEventIds.has(event.id))
      .sort((a, b) => a.start.localeCompare(b.start));
  }

  function calendarEventToLinkableEvent(event: CalendarEvent): ProjectLinkableEvent {
    return {
      id: event.id,
      projectId: event.projectId ?? "",
      title: event.title,
      start: event.start,
      end: event.end,
      timezone: event.timezone,
      calendarId: event.calendarId,
      color: event.color,
      allDay: event.allDay === true,
      status: event.status ?? "confirmed",
      linkedTasks: [],
    };
  }

  function otherLinkedTaskText(event: ProjectLinkableEvent, task: ProjectTask): string {
    return event.linkedTasks
      .filter((linkedTask) => linkedTask.taskId !== task.id)
      .map((linkedTask) => linkedTask.title)
      .join(", ");
  }

  function historyTaskTitle(event: ProjectTaskChangeEvent): string {
    return projects.tasks.find((task) => task.id === event.taskId)?.title
      ?? t("projects.detail.missingHistoryTask");
  }

  function historyEventLabel(event: ProjectTaskChangeEvent): string {
    const field = event.fieldName ? historyFieldLabel(event.fieldName) : "";
    const oldValue = event.oldValue ?? t("projects.history.emptyValue");
    const newValue = event.newValue ?? t("projects.history.emptyValue");
    if (event.eventType === "created") return t("projects.history.created");
    if (event.eventType === "scheduled") return t("projects.history.scheduled");
    if (event.eventType === "event_unlinked") return t("projects.history.eventUnlinked");
    if (event.eventType === "dependency_added") return t("projects.history.dependencyAdded", newValue);
    if (event.eventType === "dependency_removed") return t("projects.history.dependencyRemoved", oldValue);
    if (event.eventType === "completed") return t("projects.history.completed", oldValue, newValue);
    if (event.eventType === "reopened") return t("projects.history.reopened", oldValue, newValue);
    if (event.eventType === "archived") return t("projects.history.archived");
    if (event.fieldName) return t("projects.history.fieldChanged", field, oldValue, newValue);
    return t("projects.history.updated");
  }

  function historyFieldLabel(fieldName: string): string {
    if (fieldName.startsWith("custom_field:")) return fieldName.slice("custom_field:".length);
    if (fieldName === "title") return t("projects.history.fields.title");
    if (fieldName === "description") return t("projects.history.fields.description");
    if (fieldName === "status") return t("projects.history.fields.status");
    if (fieldName === "section") return t("projects.history.fields.section");
    if (fieldName === "parent") return t("projects.history.fields.parent");
    if (fieldName === "priority") return t("projects.history.fields.priority");
    if (fieldName === "type") return t("projects.history.fields.type");
    if (fieldName === "estimate") return t("projects.history.fields.estimate");
    if (fieldName === "due_date") return t("projects.history.fields.dueDate");
    if (fieldName === "start_date") return t("projects.history.fields.startDate");
    if (fieldName === "target_date") return t("projects.history.fields.targetDate");
    if (fieldName === "archived_at") return t("projects.history.fields.archiveState");
    if (fieldName === "blocker_reason") return t("projects.history.fields.blockerReason");
    if (fieldName === "milestone") return t("projects.history.fields.milestone");
    if (fieldName === "checklist") return t("projects.history.fields.checklist");
    if (fieldName === "event_id") return t("projects.history.fields.event");
    if (fieldName === "blocking_task_id") return t("projects.history.fields.dependency");
    return fieldName;
  }

  function checklistItemDraftTitle(item: ProjectChecklistItem): string {
    return checklistTitleDrafts[item.id] ?? item.title;
  }

  function checklistItemDirty(item: ProjectChecklistItem): boolean {
    return checklistItemDraftTitle(item) !== item.title;
  }

  function adjacentChecklistItem(item: ProjectChecklistItem, direction: -1 | 1): ProjectChecklistItem | undefined {
    const ordered = projects.checklistItemsForTask(item.taskId);
    const index = ordered.findIndex((entry) => entry.id === item.id);
    if (index < 0) return undefined;
    return ordered[index + direction];
  }

  function adjacentSubtask(task: ProjectTask, direction: -1 | 1): ProjectTask | undefined {
    if (!task.parentTaskId) return undefined;
    const ordered = projects.subtasksForTask(task.parentTaskId);
    const index = ordered.findIndex((entry) => entry.id === task.id);
    if (index < 0) return undefined;
    return ordered[index + direction];
  }

  function adjacentStatus(task: ProjectTask, direction: -1 | 1): ProjectStatus | undefined {
    const index = statuses.findIndex((status) => status.id === task.statusId);
    if (index < 0) return undefined;
    return statuses[index + direction];
  }

  function adjacentWorkflowStatus(status: ProjectStatus, direction: -1 | 1): ProjectStatus | undefined {
    const index = statuses.findIndex((entry) => entry.id === status.id);
    if (index < 0) return undefined;
    return statuses[index + direction];
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

  function adjacentTaskInStatus(task: ProjectTask, direction: -1 | 1): ProjectTask | undefined {
    const ordered = projects.topLevelTasksForStatus(task.projectId, task.statusId);
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

  async function submitQuickTask(): Promise<void> {
    if (!selectedProjectId) return;
    await projects.addTask(selectedProjectId, quickTaskTitle);
    quickTaskTitle = "";
  }

  async function submitSectionTask(sectionId: string): Promise<void> {
    if (!selectedProjectId) return;
    const title = sectionTaskDrafts[sectionId] ?? "";
    await projects.addTask(selectedProjectId, title, sectionId);
    sectionTaskDrafts = { ...sectionTaskDrafts, [sectionId]: "" };
  }

  async function submitSubtask(parent: ProjectTask): Promise<void> {
    const statusId = projects.defaultStatus(parent.projectId)?.id ?? parent.statusId;
    await projects.addTask(parent.projectId, subtaskDraft, parent.sectionId, statusId, parent.id);
    subtaskDraft = "";
  }

  async function submitChecklistItem(task: ProjectTask): Promise<void> {
    await projects.addChecklistItem(task.id, checklistDraft);
    checklistDraft = "";
  }

  async function saveChecklistItem(item: ProjectChecklistItem): Promise<void> {
    const title = checklistItemDraftTitle(item).trim();
    if (!title) {
      detailError = t("projects.detail.checklistItemTitleRequired");
      return;
    }
    detailError = null;
    await projects.updateChecklistItem(item, { title });
    checklistTitleDrafts = { ...checklistTitleDrafts, [item.id]: title };
  }

  async function attachExistingLabel(task: ProjectTask, label: ProjectLabel): Promise<void> {
    detailError = null;
    try {
      await projects.linkTaskLabel(task.id, label.id);
      labelDraft = "";
    } catch (error) {
      detailError = t(
        "projects.detail.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitTaskLabel(task: ProjectTask): Promise<void> {
    const name = labelDraft.trim();
    if (!name) {
      detailError = t("projects.detail.labelNameRequired");
      return;
    }
    detailError = null;
    try {
      await projects.addAndLinkTaskLabel(task, name);
      labelDraft = "";
    } catch (error) {
      detailError = t(
        "projects.detail.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function detachTaskLabel(task: ProjectTask, label: ProjectLabel): Promise<void> {
    detailError = null;
    try {
      await projects.unlinkTaskLabel(task.id, label.id);
    } catch (error) {
      detailError = t(
        "projects.detail.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function moveChecklistItemInDetail(item: ProjectChecklistItem, direction: -1 | 1): Promise<void> {
    detailError = null;
    await projects.moveChecklistItem(item, direction);
  }

  async function moveSubtaskInDetail(task: ProjectTask, direction: -1 | 1): Promise<void> {
    detailError = null;
    await projects.moveSubtask(task, direction);
  }

  async function promoteSubtaskFromDetail(task: ProjectTask): Promise<void> {
    detailError = null;
    try {
      await projects.promoteSubtask(task);
    } catch (error) {
      detailError = t(
        "projects.detail.promoteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function demoteTaskFromDetail(task: ProjectTask, parentTask: ProjectTask): Promise<void> {
    detailError = null;
    try {
      await projects.demoteTaskToSubtask(task, parentTask);
      parentTaskSearch = "";
    } catch (error) {
      detailError = t(
        "projects.detail.demoteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function addBlockingDependency(blockingTask: ProjectTask, blockedTask: ProjectTask): Promise<void> {
    detailError = null;
    try {
      await projects.addTaskDependency(blockingTask.id, blockedTask.id);
      dependencySearch = "";
    } catch (error) {
      detailError = t(
        "projects.detail.dependencySaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function removeDependency(dependencyId: string): Promise<void> {
    detailError = null;
    try {
      await projects.removeTaskDependency(dependencyId);
    } catch (error) {
      detailError = t(
        "projects.detail.dependencySaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function linkExistingEvent(task: ProjectTask, event: ProjectLinkableEvent): Promise<void> {
    detailError = null;
    try {
      await projects.linkTaskEvent(task.id, event.id, "scheduled");
      eventLinkSearch = "";
    } catch (error) {
      detailError = t(
        "projects.detail.eventLinkSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function unlinkExistingEvent(task: ProjectTask, event: ProjectLinkableEvent): Promise<void> {
    detailError = null;
    try {
      await projects.unlinkTaskEvent(task.id, event.id);
    } catch (error) {
      detailError = t(
        "projects.detail.eventLinkSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
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

  async function submitGroup(): Promise<void> {
    await projects.addGroup(groupDraft);
    groupDraft = "";
    createGroupOpen = false;
  }

  function loadGroupEditorDraft(group: ProjectGroup): void {
    editingGroupId = group.id;
    groupEditorName = group.name;
    groupEditorIcon = group.icon;
    groupEditorColor = group.color;
    groupEditorError = null;
  }

  function toggleGroupEditor(group: ProjectGroup): void {
    if (editingGroupId === group.id) {
      editingGroupId = null;
      groupEditorError = null;
      return;
    }
    loadGroupEditorDraft(group);
  }

  function groupEditorDirty(group: ProjectGroup): boolean {
    return groupEditorName !== group.name
      || groupEditorIcon !== group.icon
      || groupEditorColor !== group.color;
  }

  async function saveGroupEditor(group: ProjectGroup): Promise<void> {
    const name = groupEditorName.trim();
    if (!name) {
      groupEditorError = t("projects.navigator.groupNameRequired");
      return;
    }
    groupEditorSaving = true;
    groupEditorError = null;
    try {
      await projects.updateGroup(group, {
        name,
        icon: groupEditorIcon,
        color: groupEditorColor,
      });
      editingGroupId = null;
    } catch (error) {
      groupEditorError = t(
        "projects.navigator.groupSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      groupEditorSaving = false;
    }
  }

  async function moveGroupInNavigator(group: ProjectGroup, direction: -1 | 1): Promise<void> {
    groupEditorError = null;
    try {
      await projects.moveGroup(group, direction);
    } catch (error) {
      groupEditorError = t(
        "projects.navigator.groupSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitProject(groupId: string): Promise<void> {
    const name = projectDraftByGroup[groupId] ?? "";
    const templateId = projectTemplateDraftByGroup[groupId] ?? "blank";
    await projects.addProject(groupId, name, templateId);
    projectDraftByGroup = { ...projectDraftByGroup, [groupId]: "" };
    projectTemplateDraftByGroup = { ...projectTemplateDraftByGroup, [groupId]: "blank" };
    createProjectGroupId = null;
  }

  async function moveProjectInNavigator(project: Project, direction: -1 | 1): Promise<void> {
    await projects.moveProject(project, direction, showInactiveProjects);
  }

  function loadTaskDetailDraft(task: ProjectTask): void {
    detailDraftTaskId = task.id;
    detailDraftUpdatedAt = task.updatedAt;
    detailTitle = task.title;
    detailDescription = task.description;
    detailSectionId = task.sectionId;
    detailStatusId = task.statusId;
    detailPriority = task.priority;
    detailTaskType = task.taskType;
    detailEstimateMinutes = String(task.estimateMinutes ?? "");
    detailStartDate = task.startDate ?? "";
    detailDueDate = task.dueDate ?? "";
    detailTargetEndDate = task.targetEndDate ?? "";
    detailBlockerReason = task.blockerReason ?? "";
    detailChangeReason = "";
    detailMilestone = task.milestone;
    detailError = null;
    subtaskDraft = "";
    checklistDraft = "";
    labelDraft = "";
    checklistTitleDrafts = Object.fromEntries(
      projects.checklistItemsForTask(task.id).map((item) => [item.id, item.title]),
    );
    const textDrafts: Record<string, string> = {};
    const numberDrafts: Record<string, string> = {};
    const dateDrafts: Record<string, string> = {};
    const checkboxDrafts: Record<string, boolean> = {};
    const selectDrafts: Record<string, string> = {};
    const multiDrafts: Record<string, string[]> = {};
    for (const field of projects.customFieldsForProject(task.projectId)) {
      const value = projects.customFieldValueForTask(task.id, field.id);
      const optionValues = projects.customFieldOptionValuesForTask(task.id, field.id);
      textDrafts[field.id] = value?.textValue ?? "";
      numberDrafts[field.id] = String(value?.numberValue ?? "");
      dateDrafts[field.id] = value?.dateValue ?? "";
      checkboxDrafts[field.id] = value?.checkboxValue ?? false;
      selectDrafts[field.id] = optionValues[0]?.id ?? "none";
      multiDrafts[field.id] = optionValues.map((option) => option.id);
    }
    customFieldTextDrafts = textDrafts;
    customFieldNumberDrafts = numberDrafts;
    customFieldDateDrafts = dateDrafts;
    customFieldCheckboxDrafts = checkboxDrafts;
    customFieldSelectDrafts = selectDrafts;
    customFieldMultiDrafts = multiDrafts;
    dependencySearch = "";
    parentTaskSearch = "";
    eventLinkSearch = "";
    eventLinkStartDate = "";
    eventLinkEndDate = "";
    eventLinkResults = [];
    eventLinkSearchError = null;
  }

  function openTaskDetail(task: ProjectTask): void {
    projectSettingsOpen = false;
    selectedTaskId = task.id;
    loadTaskDetailDraft(task);
  }

  function closeTaskDetail(): void {
    if (selectedTask) loadTaskDetailDraft(selectedTask);
    selectedTaskId = null;
  }

  function loadProjectSettingsDraft(project: Project): void {
    projectDraftId = project.id;
    projectDraftUpdatedAt = project.updatedAt;
    projectGroupDraft = project.groupId;
    projectNameDraft = project.name;
    projectIconDraft = project.icon;
    projectStatusDraft = project.status;
    projectColorDraft = project.color;
    projectDurationDraft = String(project.defaultEventDurationMinutes);
    projectPomodoroDraft = project.defaultPomodoroPresetKey ?? "none";
    projectIdleTimeoutDraft = String(project.defaultIdleTimeoutMinutes ?? "");
    projectFocusPlaylistDraft = project.focusPlaylistId ?? "";
    projectBreakPlaylistDraft = project.breakPlaylistId ?? "";
    projectWorkEnvironmentDraft = project.workEnvironmentId ?? "";
    projectBlockerRulesetDraft = project.blockerRulesetId ?? "";
    projectSettingsError = null;
    statusNameDrafts = Object.fromEntries(statuses.map((status) => [status.id, status.name]));
    statusCategoryDrafts = Object.fromEntries(
      statuses.map((status) => [status.id, status.category]),
    );
    labelNameDrafts = Object.fromEntries(projectLabels.map((label) => [label.id, label.name]));
    const nextLabelColorDrafts: Record<string, ProjectLabelColorDraft> = {};
    for (const label of projectLabels) {
      nextLabelColorDrafts[label.id] = label.color ?? "none";
    }
    labelColorDrafts = nextLabelColorDrafts;
    customFieldNameDrafts = Object.fromEntries(projectCustomFields.map((field) => [field.id, field.name]));
    customFieldOptionNameDrafts = Object.fromEntries(
      projectCustomFields.flatMap((field) =>
        projects.customFieldOptionsForField(field.id).map((option) => [option.id, option.name]),
      ),
    );
    newStatusName = "";
    newStatusCategory = "active";
    newLabelName = "";
    newLabelColor = "none";
    pendingDeleteLabelId = null;
    newCustomFieldName = "";
    newCustomFieldType = "text";
    newCustomFieldOptionDrafts = {};
    pendingDeleteCustomFieldId = null;
    pendingDeleteCustomFieldOptionId = null;
  }

  function openProjectSettings(): void {
    if (!selectedProject) return;
    selectedTaskId = null;
    projectSettingsOpen = true;
    loadProjectSettingsDraft(selectedProject);
  }

  function closeProjectSettings(): void {
    if (selectedProject) loadProjectSettingsDraft(selectedProject);
    projectSettingsOpen = false;
  }

  function normalizeOptionalDate(value: string): string | undefined {
    const trimmed = value.trim();
    if (!trimmed) return undefined;
    try {
      return Temporal.PlainDate.from(trimmed).toString();
    } catch {
      throw new Error(t("projects.detail.invalidDate"));
    }
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

  function normalizeOptionalPositiveInteger(value: string): number | undefined {
    const trimmed = value.trim();
    if (!trimmed) return undefined;
    const parsed = Number(trimmed);
    if (!Number.isInteger(parsed) || parsed <= 0) {
      throw new Error(t("projects.detail.invalidEstimate"));
    }
    return parsed;
  }

  function normalizeProjectPositiveInteger(value: string, errorMessage: string): number {
    const trimmed = value.trim();
    const parsed = Number(trimmed);
    if (!trimmed || !Number.isInteger(parsed) || parsed <= 0) {
      throw new Error(errorMessage);
    }
    return parsed;
  }

  function normalizeOptionalIdentifier(value: string): string | null {
    const trimmed = value.trim();
    return trimmed || null;
  }

  function statusDraftDirty(status: ProjectStatus): boolean {
    return (statusNameDrafts[status.id] ?? status.name) !== status.name
      || (statusCategoryDrafts[status.id] ?? status.category) !== status.category;
  }

  async function saveStatus(status: ProjectStatus): Promise<void> {
    const name = (statusNameDrafts[status.id] ?? status.name).trim();
    const category = statusCategoryDrafts[status.id] ?? status.category;
    if (!name) {
      projectSettingsError = t("projects.settings.statusNameRequired");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.updateStatus(status, { name, category });
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.statusSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitStatus(): Promise<void> {
    if (!selectedProjectId) return;
    const name = newStatusName.trim();
    if (!name) {
      projectSettingsError = t("projects.settings.statusNameRequired");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.addStatus(selectedProjectId, name, newStatusCategory);
      newStatusName = "";
      newStatusCategory = "active";
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.statusSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function saveLabel(label: ProjectLabel): Promise<void> {
    const name = labelNameDraftValue(label).trim();
    if (!name) {
      projectSettingsError = t("projects.settings.labelNameRequired");
      return;
    }
    if (labelNameExists(name, label.id)) {
      projectSettingsError = t("projects.settings.labelNameExists");
      return;
    }
    const color = labelColorDraftValue(label);
    projectSettingsError = null;
    try {
      await projects.updateLabel(label, { name, color });
      labelNameDrafts = { ...labelNameDrafts, [label.id]: name };
      labelColorDrafts = { ...labelColorDrafts, [label.id]: color ?? "none" };
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitLabel(): Promise<void> {
    if (!selectedProjectId) return;
    const name = newLabelName.trim();
    if (!name) {
      projectSettingsError = t("projects.settings.labelNameRequired");
      return;
    }
    if (labelNameExists(name)) {
      projectSettingsError = t("projects.settings.labelNameExists");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.addLabel(selectedProjectId, name, newLabelColorValue());
      newLabelName = "";
      newLabelColor = "none";
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function moveProjectLabel(label: ProjectLabel, direction: -1 | 1): Promise<void> {
    projectSettingsError = null;
    try {
      await projects.moveLabel(label, direction);
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function requestDeleteLabel(label: ProjectLabel): void {
    pendingDeleteLabelId = label.id;
  }

  function cancelDeleteLabel(): void {
    pendingDeleteLabelId = null;
  }

  async function confirmDeleteLabel(): Promise<void> {
    if (!pendingDeleteLabel) return;
    const label = pendingDeleteLabel;
    pendingDeleteLabelId = null;
    projectSettingsError = null;
    try {
      await projects.removeLabel(label.id);
      if (taskLabelFilter === label.id) {
        taskLabelFilter = "all";
      }
      const remainingNames = { ...labelNameDrafts };
      const remainingColors = { ...labelColorDrafts };
      delete remainingNames[label.id];
      delete remainingColors[label.id];
      labelNameDrafts = remainingNames;
      labelColorDrafts = remainingColors;
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.labelDeleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function saveCustomField(field: ProjectCustomField): Promise<void> {
    const name = customFieldNameDraftValue(field).trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.nameRequired");
      return;
    }
    if (customFieldNameExists(name, field.id)) {
      projectSettingsError = t("projects.customFields.nameExists");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.updateCustomField(field, { name });
      customFieldNameDrafts = { ...customFieldNameDrafts, [field.id]: name };
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.saveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitCustomField(): Promise<void> {
    if (!selectedProjectId) return;
    const name = newCustomFieldName.trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.nameRequired");
      return;
    }
    if (customFieldNameExists(name)) {
      projectSettingsError = t("projects.customFields.nameExists");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.addCustomField(selectedProjectId, name, newCustomFieldType);
      newCustomFieldName = "";
      newCustomFieldType = "text";
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.saveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function moveProjectCustomField(field: ProjectCustomField, direction: -1 | 1): Promise<void> {
    projectSettingsError = null;
    try {
      await projects.moveCustomField(field, direction);
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.saveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function requestDeleteCustomField(field: ProjectCustomField): void {
    pendingDeleteCustomFieldId = field.id;
  }

  function cancelDeleteCustomField(): void {
    pendingDeleteCustomFieldId = null;
  }

  async function confirmDeleteCustomField(): Promise<void> {
    if (!pendingDeleteCustomField) return;
    const field = pendingDeleteCustomField;
    pendingDeleteCustomFieldId = null;
    projectSettingsError = null;
    try {
      await projects.removeCustomField(field.id);
      const remainingNames = { ...customFieldNameDrafts };
      delete remainingNames[field.id];
      customFieldNameDrafts = remainingNames;
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.deleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function saveCustomFieldOption(option: ProjectCustomFieldOption): Promise<void> {
    const name = customFieldOptionNameDraftValue(option).trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.optionNameRequired");
      return;
    }
    if (customFieldOptionNameExists(option.fieldId, name, option.id)) {
      projectSettingsError = t("projects.customFields.optionNameExists");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.updateCustomFieldOption(option, { name });
      customFieldOptionNameDrafts = { ...customFieldOptionNameDrafts, [option.id]: name };
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.optionSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitCustomFieldOption(field: ProjectCustomField): Promise<void> {
    const name = (newCustomFieldOptionDrafts[field.id] ?? "").trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.optionNameRequired");
      return;
    }
    if (customFieldOptionNameExists(field.id, name)) {
      projectSettingsError = t("projects.customFields.optionNameExists");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.addCustomFieldOption(field.id, name);
      newCustomFieldOptionDrafts = { ...newCustomFieldOptionDrafts, [field.id]: "" };
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.optionSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function moveProjectCustomFieldOption(option: ProjectCustomFieldOption, direction: -1 | 1): Promise<void> {
    projectSettingsError = null;
    try {
      await projects.moveCustomFieldOption(option, direction);
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.optionSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function requestDeleteCustomFieldOption(option: ProjectCustomFieldOption): void {
    pendingDeleteCustomFieldOptionId = option.id;
  }

  function cancelDeleteCustomFieldOption(): void {
    pendingDeleteCustomFieldOptionId = null;
  }

  async function confirmDeleteCustomFieldOption(): Promise<void> {
    if (!pendingDeleteCustomFieldOption) return;
    const option = pendingDeleteCustomFieldOption;
    pendingDeleteCustomFieldOptionId = null;
    projectSettingsError = null;
    try {
      await projects.removeCustomFieldOption(option.id);
      const remainingNames = { ...customFieldOptionNameDrafts };
      delete remainingNames[option.id];
      customFieldOptionNameDrafts = remainingNames;
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.optionDeleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function toggleCustomFieldMultiOption(field: ProjectCustomField, option: ProjectCustomFieldOption): void {
    const current = customFieldMultiDrafts[field.id] ?? [];
    customFieldMultiDrafts = {
      ...customFieldMultiDrafts,
      [field.id]: current.includes(option.id)
        ? current.filter((optionId) => optionId !== option.id)
        : [...current, option.id],
    };
  }

  async function saveTaskCustomField(task: ProjectTask, field: ProjectCustomField): Promise<void> {
    detailError = null;
    try {
      let textValue: string | null = null;
      let numberValue: number | null = null;
      let dateValue: string | null = null;
      let checkboxValue: boolean | null = null;
      let optionIds: string[] = [];
      if (field.fieldType === "text" || field.fieldType === "url") {
        const text = (customFieldTextDrafts[field.id] ?? "").trim();
        textValue = text || null;
      } else if (field.fieldType === "number") {
        const numberText = (customFieldNumberDrafts[field.id] ?? "").trim();
        if (numberText) {
          const parsed = Number(numberText);
          if (!Number.isFinite(parsed)) {
            detailError = t("projects.customFields.invalidNumber");
            return;
          }
          numberValue = parsed;
        }
      } else if (field.fieldType === "date") {
        const dateText = (customFieldDateDrafts[field.id] ?? "").trim();
        if (dateText) {
          try {
            dateValue = Temporal.PlainDate.from(dateText).toString();
          } catch {
            detailError = t("projects.detail.invalidDate");
            return;
          }
        }
      } else if (field.fieldType === "checkbox") {
        checkboxValue = customFieldCheckboxDrafts[field.id] ?? false;
      } else if (field.fieldType === "select") {
        const optionId = customFieldSelectDrafts[field.id] ?? "none";
        optionIds = optionId === "none" ? [] : [optionId];
      } else {
        optionIds = customFieldMultiDrafts[field.id] ?? [];
      }
      await projects.saveCustomFieldValue({
        taskId: task.id,
        fieldId: field.id,
        textValue,
        numberValue,
        dateValue,
        checkboxValue,
        optionIds,
      });
      loadTaskDetailDraft(task);
    } catch (error) {
      detailError = t(
        "projects.customFields.valueSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function moveTaskToStatus(task: ProjectTask, status: ProjectStatus | undefined): Promise<void> {
    if (!status || status.id === task.statusId) return;
    await projects.setTaskStatus(task, status.id);
  }

  async function moveTaskWithinSection(task: ProjectTask, direction: -1 | 1): Promise<void> {
    if (taskSortMode !== "manual") return;
    await projects.moveTaskInSection(task, direction);
  }

  async function moveTaskWithinStatus(task: ProjectTask, direction: -1 | 1): Promise<void> {
    if (taskSortMode !== "manual") return;
    await projects.moveTaskInStatus(task, direction);
  }

  async function moveWorkflowStatus(status: ProjectStatus, direction: -1 | 1): Promise<void> {
    await projects.moveStatus(status, direction);
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

  async function archiveTaskFromDetail(task: ProjectTask): Promise<void> {
    detailError = null;
    detailSaving = true;
    try {
      await projects.archiveTasks([task]);
    } catch (error) {
      detailError = t("projects.detail.archiveFailed", error instanceof Error ? error.message : String(error));
    } finally {
      detailSaving = false;
    }
  }

  async function restoreTaskFromDetail(task: ProjectTask): Promise<void> {
    detailError = null;
    detailSaving = true;
    showArchivedTasks = true;
    try {
      await projects.restoreTasks([task]);
    } catch (error) {
      detailError = t("projects.detail.restoreFailed", error instanceof Error ? error.message : String(error));
    } finally {
      detailSaving = false;
    }
  }

  async function saveProjectSettings(): Promise<void> {
    if (!selectedProject) return;
    const name = projectNameDraft.trim();
    if (!name) {
      projectSettingsError = t("projects.settings.nameRequired");
      return;
    }
    if (!visibleProjectGroups.some((group) => group.id === projectGroupDraft)) {
      projectSettingsError = t("projects.settings.groupRequired");
      return;
    }
    projectSettingsSaving = true;
    projectSettingsError = null;
    try {
      const defaultEventDurationMinutes = normalizeProjectPositiveInteger(
        projectDurationDraft,
        t("projects.settings.invalidDuration"),
      );
      const defaultIdleTimeoutMinutes = projectIdleTimeoutDraft.trim()
        ? normalizeProjectPositiveInteger(projectIdleTimeoutDraft, t("projects.settings.invalidIdleTimeout"))
        : undefined;
      await projects.updateProject({
        id: selectedProject.id,
        groupId: projectGroupDraft,
        name,
        icon: projectIconDraft,
        color: projectColorDraft ?? null,
        sortOrder: projectGroupDraft === selectedProject.groupId
          ? selectedProject.sortOrder
          : nextProjectSortOrderForGroup(projectGroupDraft, selectedProject.id),
        status: projectStatusDraft,
        defaultEventDurationMinutes,
        defaultPomodoroPresetKey: projectPomodoroDraft === "none" ? null : projectPomodoroDraft,
        defaultIdleTimeoutMinutes: defaultIdleTimeoutMinutes ?? null,
        focusPlaylistId: normalizeOptionalIdentifier(projectFocusPlaylistDraft),
        breakPlaylistId: normalizeOptionalIdentifier(projectBreakPlaylistDraft),
        workEnvironmentId: normalizeOptionalIdentifier(projectWorkEnvironmentDraft),
        blockerRulesetId: normalizeOptionalIdentifier(projectBlockerRulesetDraft),
      });
      if (projectStatusDraft !== "active") {
        showInactiveProjects = true;
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      projectSettingsError = message === t("projects.settings.invalidDuration")
        || message === t("projects.settings.invalidIdleTimeout")
        ? message
        : t("projects.settings.saveFailed", message);
    } finally {
      projectSettingsSaving = false;
    }
  }

  async function saveTaskDetail(): Promise<void> {
    if (!selectedTask) return;
    const title = detailTitle.trim();
    if (!title) {
      detailError = t("projects.detail.titleRequired");
      return;
    }
    if (!detailSectionId || !detailStatusId) return;
    detailSaving = true;
    detailError = null;
    try {
      const estimateMinutes = normalizeOptionalPositiveInteger(detailEstimateMinutes);
      const startDate = normalizeOptionalDate(detailStartDate);
      const dueDate = normalizeOptionalDate(detailDueDate);
      const targetEndDate = normalizeOptionalDate(detailTargetEndDate);
      await projects.updateTask(selectedTask, {
        title,
        description: detailDescription.trim(),
        sectionId: detailSectionId,
        statusId: detailStatusId,
        priority: detailPriority,
        taskType: detailTaskType,
        estimateMinutes,
        startDate,
        dueDate,
        targetEndDate,
        blockerReason: detailBlockerReason.trim() || undefined,
        milestone: detailMilestone,
        changeReason: detailChangeReason.trim() || null,
      });
      detailChangeReason = "";
      detailDraftUpdatedAt = selectedTask.updatedAt;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      detailError = message === t("projects.detail.invalidEstimate")
        || message === t("projects.detail.invalidDate")
        ? message
        : t("projects.detail.saveFailed", message);
    } finally {
      detailSaving = false;
    }
  }

  function projectCompletionPercent(): number {
    if (tasks.length === 0) return 0;
    return Math.round((completedTaskCount / tasks.length) * 100);
  }

  function isTaskDone(task: ProjectTask): boolean {
    return projects.statusById(task.statusId)?.terminal === true;
  }

  function taskDateForDeadline(task: ProjectTask): string | undefined {
    return task.dueDate ?? task.targetEndDate;
  }

  function upcomingDeadlineTasks(): ProjectTask[] {
    return tasks
      .filter((task) => {
        const deadline = taskDateForDeadline(task);
        return deadline !== undefined && !isTaskDone(task) && deadline >= todayDate;
      })
      .sort((a, b) => (taskDateForDeadline(a) ?? "").localeCompare(taskDateForDeadline(b) ?? ""))
      .slice(0, 5);
  }

  function overdueTasks(): ProjectTask[] {
    return tasks
      .filter((task) => {
        const deadline = taskDateForDeadline(task);
        return deadline !== undefined && !isTaskDone(task) && deadline < todayDate;
      })
      .sort((a, b) => (taskDateForDeadline(a) ?? "").localeCompare(taskDateForDeadline(b) ?? ""))
      .slice(0, 5);
  }

  function unscheduledDueTasks(): ProjectTask[] {
    return tasks
      .filter((task) =>
        Boolean(task.dueDate)
        && !isTaskDone(task)
        && scheduledLinksForTask(task.id).length === 0
      )
      .sort((a, b) => (a.dueDate ?? "").localeCompare(b.dueDate ?? ""))
      .slice(0, 5);
  }

  function blockedSummaryTasks(): ProjectTask[] {
    return tasks
      .filter((task) =>
        projects.statusById(task.statusId)?.category === "blocked"
        || Boolean(task.blockerReason?.trim())
        || blockedByDependencies(task).length > 0
      )
      .slice(0, 5);
  }

  function recentlyCompletedTasks(): ProjectTask[] {
    return tasks
      .filter((task) => isTaskDone(task) && Boolean(task.completedAt))
      .sort((a, b) => (b.completedAt ?? "").localeCompare(a.completedAt ?? ""))
      .slice(0, 5);
  }

  function tasksWithoutEstimates(): ProjectTask[] {
    return tasks
      .filter((task) => !isTaskDone(task) && task.estimateMinutes === undefined)
      .slice(0, 5);
  }

  function recentProjectChangeEvents(): ProjectTaskChangeEvent[] {
    return projects.recentTaskChangeEventsForProject(selectedProjectId, 6);
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

  function totalOpenEstimateMinutes(): number {
    return tasks
      .filter((task) => !isTaskDone(task))
      .reduce((total, task) => total + (task.estimateMinutes ?? 0), 0);
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
  <aside class="flex min-h-0 w-[min(17rem,42vw)] min-w-40 shrink-0 flex-col border-r border-border bg-card/70">
    <div class="flex shrink-0 items-center gap-2 border-b border-border px-2.5 py-2">
      <Search size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
      <input
        bind:value={projectSearch}
        placeholder={t("projects.navigator.searchPlaceholder")}
        class="min-h-8 min-w-0 flex-1 bg-transparent text-[0.866667rem] placeholder:text-muted-foreground"
      />
      <button
        type="button"
        class={cn(
          "flex h-8 w-8 shrink-0 items-center justify-center rounded-md hover:bg-accent hover:text-foreground",
          showInactiveProjects ? "text-foreground" : "text-muted-foreground",
        )}
        aria-label={showInactiveProjects ? t("projects.navigator.hideInactive") : t("projects.navigator.showInactive")}
        title={showInactiveProjects ? t("projects.navigator.hideInactive") : t("projects.navigator.showInactive")}
        onclick={() => {
          showInactiveProjects = !showInactiveProjects;
        }}
      >
        {#if showInactiveProjects}
          <EyeOff size={14} strokeWidth={1.75} />
        {:else}
          <Eye size={14} strokeWidth={1.75} />
        {/if}
      </button>
    </div>
    <div class="min-h-0 flex-1 overflow-y-auto py-1">
      {#if projects.loading && !projects.loaded}
        <div class="px-3 py-2 text-[0.8rem] text-muted-foreground">{t("projects.loading")}</div>
      {:else if projects.loadError}
        <div class="px-3 py-2 text-[0.8rem] text-destructive">
          {t("projects.loadFailed", projects.loadError)}
        </div>
      {:else if projects.groups.length === 0}
        <div class="px-3 py-2 text-[0.8rem] text-muted-foreground">{t("projects.navigator.empty")}</div>
      {:else}
        {#each visibleProjectGroups.filter((group) => groupVisible(group.id, group.name)) as group (group.id)}
          {@const groupProjects = filteredProjectsForGroup(group.id)}
          {@const expanded = normalizedProjectSearch.length > 0 || !group.collapsed}
          {@const previousGroup = adjacentNavigatorGroup(group, -1)}
          {@const nextGroup = adjacentNavigatorGroup(group, 1)}
          <div class="px-1">
            <div class="flex items-center gap-1">
              <button
                type="button"
                class="flex min-h-8 min-w-0 flex-1 items-center gap-1.5 rounded px-1.5 text-left text-[0.8rem] font-medium hover:bg-accent"
                aria-label={expanded ? t("projects.actions.collapseGroup") : t("projects.actions.expandGroup")}
                onclick={() => { void projects.setGroupCollapsed(group.id, !group.collapsed); }}
              >
                {#if expanded}
                  <ChevronDown size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                {:else}
                  <ChevronRight size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                {/if}
                <ProjectIcon name={group.icon} size={14} class="shrink-0" />
                <span class="truncate">{group.name}</span>
              </button>
              <button
                type="button"
                class="flex h-8 w-8 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground"
                aria-label={t("projects.navigator.createProject")}
                onclick={() => {
                  createProjectGroupId = createProjectGroupId === group.id ? null : group.id;
                }}
              >
                <Plus size={14} strokeWidth={1.75} />
              </button>
              <button
                type="button"
                class={cn(
                  "flex h-8 w-8 shrink-0 items-center justify-center rounded hover:bg-accent hover:text-foreground",
                  editingGroupId === group.id ? "text-foreground" : "text-muted-foreground",
                )}
                aria-label={t("projects.navigator.editGroup")}
                title={t("projects.navigator.editGroup")}
                onclick={() => toggleGroupEditor(group)}
              >
                <MoreHorizontal size={14} strokeWidth={1.75} />
              </button>
            </div>
            {#if editingGroupId === group.id}
              <div class="grid gap-2 px-5 py-1">
                <input
                  bind:value={groupEditorName}
                  class="min-h-8 min-w-0 rounded border border-border bg-background px-2 text-[0.8rem]"
                  aria-label={t("projects.navigator.groupName")}
                />
                <div class="grid grid-cols-6 gap-1">
                  {#each PROJECT_ICON_OPTIONS as icon}
                    <button
                      type="button"
                      class={cn(
                        "flex h-7 items-center justify-center rounded border",
                        groupEditorIcon === icon
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      aria-label={t("projects.navigator.selectGroupIcon", icon)}
                      onclick={() => {
                        groupEditorIcon = icon;
                      }}
                    >
                      <ProjectIcon name={icon} size={14} />
                    </button>
                  {/each}
                </div>
                <div class="flex min-h-8 items-center justify-between gap-2 rounded border border-border bg-background px-2">
                  <span class="text-[0.733333rem] text-muted-foreground">{t("projects.navigator.groupColor")}</span>
                  <div class="flex items-center gap-1">
                    <button
                      type="button"
                      class="rounded border border-border px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                      onclick={() => {
                        groupEditorColor = undefined;
                      }}
                    >
                      {t("common.none")}
                    </button>
                    <ColorPicker
                      color={groupEditorColor}
                      theme={theme.current}
                      title={t("projects.navigator.groupColor")}
                      ariaLabel={t("projects.navigator.selectGroupColor")}
                      onselect={(color) => {
                        groupEditorColor = color;
                      }}
                    />
                  </div>
                </div>
                <div class="flex items-center gap-1">
                  <button
                    type="button"
                    class="flex h-8 w-8 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!previousGroup}
                    aria-label={t("projects.actions.moveGroupUp", group.name)}
                    title={t("projects.actions.moveGroupUp", group.name)}
                    onclick={() => { void moveGroupInNavigator(group, -1); }}
                  >
                    <ArrowUp size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class="flex h-8 w-8 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!nextGroup}
                    aria-label={t("projects.actions.moveGroupDown", group.name)}
                    title={t("projects.actions.moveGroupDown", group.name)}
                    onclick={() => { void moveGroupInNavigator(group, 1); }}
                  >
                    <ArrowDown size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class="ml-auto flex min-h-8 items-center gap-1 rounded border border-border bg-background px-2 text-[0.733333rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={groupEditorSaving || !groupEditorDirty(group)}
                    onclick={() => { void saveGroupEditor(group); }}
                  >
                    <Save size={13} strokeWidth={1.75} />
                    <span>{groupEditorSaving ? t("common.loading") : t("common.save")}</span>
                  </button>
                </div>
                {#if groupEditorError}
                  <div class="rounded border border-destructive/40 bg-destructive/10 px-2 py-1 text-[0.733333rem] text-destructive">
                    {groupEditorError}
                  </div>
                {/if}
              </div>
            {/if}
            {#if createProjectGroupId === group.id}
              <form class="grid gap-1 px-5 py-1" onsubmit={(event) => { event.preventDefault(); void submitProject(group.id); }}>
                <div class="flex gap-1">
                  <input
                    value={projectDraftByGroup[group.id] ?? ""}
                    oninput={(event) => {
                      projectDraftByGroup = {
                        ...projectDraftByGroup,
                        [group.id]: event.currentTarget.value,
                      };
                    }}
                    placeholder={t("projects.navigator.projectNamePlaceholder")}
                    class="min-h-8 min-w-0 flex-1 rounded border border-border bg-background px-2 text-[0.8rem]"
                  />
                  <button type="submit" class="min-h-8 rounded bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground">
                    {t("common.save")}
                  </button>
                </div>
                <div class="flex flex-wrap gap-1" aria-label={t("projects.navigator.projectTemplate")}>
                  {#each PROJECT_TEMPLATE_IDS as templateId}
                    <button
                      type="button"
                      class={cn(
                        "min-h-7 rounded border px-2 text-[0.733333rem]",
                        (projectTemplateDraftByGroup[group.id] ?? "blank") === templateId
                          ? "border-primary/60 bg-primary/10 text-primary"
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        projectTemplateDraftByGroup = {
                          ...projectTemplateDraftByGroup,
                          [group.id]: templateId,
                        };
                      }}
                    >
                      {projectTemplateLabel(templateId)}
                    </button>
                  {/each}
                </div>
              </form>
            {/if}
            {#if expanded}
              <div class="pb-1 pl-5">
                {#each groupProjects as project (project.id)}
                  {@const previousProject = adjacentProjectInGroup(project, -1)}
                  {@const nextProject = adjacentProjectInGroup(project, 1)}
                  <div class="flex items-center gap-1">
                    <button
                      type="button"
                      class={cn(
                        "flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded px-2 text-left text-[0.833333rem] hover:bg-accent",
                        selectedProjectId === project.id ? "bg-accent text-accent-foreground" : "text-foreground",
                        project.status !== "active" && selectedProjectId !== project.id && "text-muted-foreground",
                      )}
                      aria-label={t("projects.actions.selectProject", project.name, group.name)}
                      onclick={() => {
                        void projects.selectProject(project.id).catch((error) => {
                          console.error("select project failed", error);
                        });
                        selectedTaskId = null;
                        projectSettingsOpen = false;
                      }}
                    >
                      <ProjectIcon name={project.icon} size={14} class="shrink-0" />
                      <span class="min-w-0 flex-1 truncate">{project.name}</span>
                      {#if project.status !== "active"}
                        <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(project.status))}>
                          {projectLifecycleLabel(project.status)}
                        </span>
                      {/if}
                      {#if project.color !== undefined}
                        <span class="h-2 w-2 shrink-0 rounded-full bg-primary/70"></span>
                      {/if}
                    </button>
                    {#if selectedProjectId === project.id}
                      <button
                        type="button"
                        class="flex h-7 w-7 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                        disabled={!previousProject}
                        aria-label={previousProject ? t("projects.actions.moveProjectUp", project.name) : t("projects.actions.noPreviousProject")}
                        title={previousProject ? t("projects.actions.moveProjectUp", project.name) : t("projects.actions.noPreviousProject")}
                        onclick={() => { void moveProjectInNavigator(project, -1); }}
                      >
                        <ArrowUp size={13} strokeWidth={1.75} />
                      </button>
                      <button
                        type="button"
                        class="flex h-7 w-7 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                        disabled={!nextProject}
                        aria-label={nextProject ? t("projects.actions.moveProjectDown", project.name) : t("projects.actions.noNextProject")}
                        title={nextProject ? t("projects.actions.moveProjectDown", project.name) : t("projects.actions.noNextProject")}
                        onclick={() => { void moveProjectInNavigator(project, 1); }}
                      >
                        <ArrowDown size={13} strokeWidth={1.75} />
                      </button>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
    <div class="shrink-0 border-t border-border p-2">
      {#if createGroupOpen}
        <form class="flex gap-1" onsubmit={(event) => { event.preventDefault(); void submitGroup(); }}>
          <input
            bind:value={groupDraft}
            placeholder={t("projects.navigator.groupNamePlaceholder")}
            class="min-h-8 min-w-0 flex-1 rounded border border-border bg-background px-2 text-[0.8rem]"
          />
          <button type="submit" class="min-h-8 rounded bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground">
            {t("common.save")}
          </button>
        </form>
      {:else}
        <button
          type="button"
          class="flex min-h-8 w-full items-center justify-center gap-1.5 rounded text-[0.8rem] text-foreground hover:bg-accent"
          onclick={() => { createGroupOpen = true; }}
        >
          <Plus size={14} strokeWidth={1.75} />
          <span>{t("projects.navigator.createGroup")}</span>
        </button>
      {/if}
    </div>
  </aside>

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
          <form class="flex min-w-48 flex-1 gap-1" onsubmit={(event) => { event.preventDefault(); void submitQuickTask(); }}>
            <input
              bind:value={quickTaskTitle}
              placeholder={t("projects.header.quickAddPlaceholder")}
              class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem] placeholder:text-muted-foreground"
            />
            <button type="submit" class="flex min-h-8 items-center gap-1.5 rounded-md bg-primary px-2 text-[0.8rem] font-medium text-primary-foreground">
              <Plus size={14} strokeWidth={1.75} />
              <span>{t("projects.header.addTask")}</span>
            </button>
          </form>
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
                  <form class="flex gap-1 pl-7" onsubmit={(event) => { event.preventDefault(); void submitSectionTask(section.id); }}>
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
                    <button type="submit" class="flex min-h-8 items-center justify-center rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent">
                      <Plus size={14} strokeWidth={1.75} />
                    </button>
                  </form>
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
          <div class="flex min-h-full gap-3 overflow-x-auto p-3">
            {#each statuses as status (status.id)}
              {@const statusTasks = tasksForStatus(status)}
              <section
                class={cn(
                  "flex w-64 shrink-0 flex-col gap-2 rounded-lg border border-transparent p-1",
                  boardDragOverStatusId === status.id && "border-primary/40 bg-primary/5",
                )}
                role="list"
                aria-label={status.name}
                ondragover={(event) => handleBoardColumnDragOver(event, status)}
                ondrop={(event) => { void dropBoardTask(event, status); }}
              >
                <div class={cn("rounded-md border px-2 py-1.5 text-[0.8rem] font-semibold", statusBadgeClass(status))}>
                  {status.name} ({statusTasks.length})
                </div>
                <div class="flex flex-col gap-2">
                  {#each statusTasks as task (task.id)}
                    {@const previousStatus = adjacentStatus(task, -1)}
                    {@const nextStatus = adjacentStatus(task, 1)}
                    {@const previousStatusTask = adjacentTaskInStatus(task, -1)}
                    {@const nextStatusTask = adjacentTaskInStatus(task, 1)}
                    {@const blockedByCount = blockedByDependencies(task).length}
                    {@const blocksCount = blocksDependencies(task).length}
                    {#if boardDropMarkerVisible(status, task, "before")}
                      <div class="h-1 rounded-full bg-primary"></div>
                    {/if}
                    <article
                      class={cn(
                        "rounded-md border border-border bg-card p-2",
                        task.archivedAt && "opacity-70",
                        boardDraggingTaskId === task.id && "opacity-50",
                        boardDropPendingTaskId === task.id && "opacity-60",
                      )}
                      ondragover={(event) => handleBoardCardDragOver(event, status, task)}
                      ondrop={(event) => { void dropBoardTask(event, status, task, boardCardDropPosition(event)); }}
                    >
                      <div class="grid grid-cols-[auto_auto_minmax(0,1fr)] gap-2">
                        <button
                          type="button"
                          class="mt-0.5 flex h-5 w-5 shrink-0 cursor-grab items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground active:cursor-grabbing disabled:cursor-not-allowed disabled:opacity-40"
                          draggable={!task.archivedAt && boardDropPendingTaskId === null}
                          disabled={Boolean(task.archivedAt) || boardDropPendingTaskId !== null}
                          aria-label={t("projects.actions.dragTask", task.title)}
                          title={t("projects.actions.dragTask", task.title)}
                          ondragstart={(event) => handleBoardTaskDragStart(event, task)}
                          ondragend={handleBoardTaskDragEnd}
                        >
                          <GripVertical size={13} strokeWidth={1.75} />
                        </button>
                        <button
                          type="button"
                          class={cn(
                            "mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded border",
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
                          class="min-w-0 text-left hover:text-primary"
                          onclick={() => openTaskDetail(task)}
                        >
                          <span class="block truncate text-[0.866667rem]">{task.title}</span>
                          <span class="mt-1 flex items-center gap-1 text-[0.733333rem] text-muted-foreground">
                            {priorityLabel(task.priority)}
                            {#if task.dueDate}
                              <span>/</span>
                              <span>{task.dueDate}</span>
                            {/if}
                          </span>
                          {#if task.archivedAt}
                            <span class={cn("mt-1 inline-flex rounded border px-1.5 py-0.5 text-[0.733333rem]", taskArchivedBadgeClass(task))}>
                              {t("projects.taskLifecycle.archived")}
                            </span>
                          {/if}
                          {#if blockedByCount > 0 || blocksCount > 0}
                            <span class="mt-1 flex flex-wrap items-center gap-1 text-[0.733333rem]">
                              {#if blockedByCount > 0}
                                <span class="rounded border border-destructive/40 bg-destructive/10 px-1.5 py-0.5 text-destructive">
                                  {t("projects.list.blockedBy", blockedByCount)}
                                </span>
                              {/if}
                              {#if blocksCount > 0}
                                <span class="rounded border border-amber-500/40 bg-amber-500/10 px-1.5 py-0.5 text-amber-700 dark:text-amber-300">
                                  {t("projects.list.blocks", blocksCount)}
                                </span>
                              {/if}
                            </span>
                          {/if}
                        </button>
                      </div>
                      <div class="mt-2 grid grid-cols-[auto_auto_minmax(0,1fr)_auto_auto] items-center gap-1 border-t border-border/70 pt-2">
                        <button
                          type="button"
                          class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                          disabled={Boolean(task.archivedAt) || !previousStatus}
                          aria-label={previousStatus ? t("projects.actions.moveTaskToStatus", task.title, previousStatus.name) : t("projects.actions.noPreviousStatus")}
                          title={previousStatus ? t("projects.actions.moveTaskToStatus", task.title, previousStatus.name) : t("projects.actions.noPreviousStatus")}
                          onclick={() => { void moveTaskToStatus(task, previousStatus); }}
                        >
                          <ArrowLeft size={13} strokeWidth={1.75} />
                        </button>
                        <button
                          type="button"
                          class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                          disabled={Boolean(task.archivedAt) || taskSortMode !== "manual" || !previousStatusTask}
                          aria-label={previousStatusTask ? t("projects.actions.moveTaskUp", task.title) : t("projects.actions.noPreviousTask")}
                          title={previousStatusTask ? t("projects.actions.moveTaskUp", task.title) : t("projects.actions.noPreviousTask")}
                          onclick={() => { void moveTaskWithinStatus(task, -1); }}
                        >
                          <ArrowUp size={13} strokeWidth={1.75} />
                        </button>
                        <button
                          type="button"
                          class="min-w-0 rounded border border-border px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                          onclick={() => openTaskDetail(task)}
                        >
                          <span class="block truncate">{t("projects.board.openDetails")}</span>
                        </button>
                        <button
                          type="button"
                          class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                          disabled={Boolean(task.archivedAt) || taskSortMode !== "manual" || !nextStatusTask}
                          aria-label={nextStatusTask ? t("projects.actions.moveTaskDown", task.title) : t("projects.actions.noNextTask")}
                          title={nextStatusTask ? t("projects.actions.moveTaskDown", task.title) : t("projects.actions.noNextTask")}
                          onclick={() => { void moveTaskWithinStatus(task, 1); }}
                        >
                          <ArrowDown size={13} strokeWidth={1.75} />
                        </button>
                        <button
                          type="button"
                          class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                          disabled={Boolean(task.archivedAt) || !nextStatus}
                          aria-label={nextStatus ? t("projects.actions.moveTaskToStatus", task.title, nextStatus.name) : t("projects.actions.noNextStatus")}
                          title={nextStatus ? t("projects.actions.moveTaskToStatus", task.title, nextStatus.name) : t("projects.actions.noNextStatus")}
                          onclick={() => { void moveTaskToStatus(task, nextStatus); }}
                        >
                          <ArrowRight size={13} strokeWidth={1.75} />
                        </button>
                      </div>
                    </article>
                    {#if boardDropMarkerVisible(status, task, "after")}
                      <div class="h-1 rounded-full bg-primary"></div>
                    {/if}
                  {/each}
                  {#if boardDragOverStatusId === status.id && boardDragOverPosition === "column"}
                    <div class="h-1 rounded-full bg-primary"></div>
                  {/if}
                  {#if statusTasks.length === 0}
                    <div class="rounded-md border border-dashed border-border px-2 py-3 text-[0.8rem] text-muted-foreground">
                      {t("projects.board.emptyColumn")}
                    </div>
                  {/if}
                </div>
              </section>
            {/each}
          </div>
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
          <div class="flex min-h-full flex-col gap-3 p-3">
            <div class="flex flex-wrap items-center justify-between gap-2">
              <div>
                <h2 class="text-[0.866667rem] font-semibold">{t("projects.gantt.timeline")}</h2>
                <div class="text-[0.733333rem] text-muted-foreground">
                  {#if ganttTimeline.startDate && ganttTimeline.endDate}
                    {ganttDateRangeLabel(ganttTimeline.startDate, ganttTimeline.endDate)}
                  {:else}
                    {t("projects.gantt.noDateRange")}
                  {/if}
                </div>
              </div>
              {#if ganttTimeline.rows.length > 0}
                <div class="flex flex-wrap items-center gap-1 text-[0.733333rem] text-muted-foreground">
                  <span class="rounded border border-border bg-card px-1.5 py-0.5">{t("projects.gantt.taskCount", ganttTimeline.rows.length)}</span>
                  <span class="rounded border border-border bg-card px-1.5 py-0.5">{t("projects.gantt.dependencyCount", ganttTimeline.dependencyEdges.length)}</span>
                  <span class="rounded border border-destructive/40 bg-destructive/10 px-1.5 py-0.5 text-destructive">{t("projects.gantt.lateCount", ganttTimeline.rows.filter((row) => row.overdue).length)}</span>
                  <span class="rounded border border-amber-500/40 bg-amber-500/10 px-1.5 py-0.5 text-amber-700 dark:text-amber-300">{t("projects.gantt.blockedCount", ganttTimeline.rows.filter((row) => row.blocked).length)}</span>
                  <span class="rounded border border-destructive/40 bg-destructive/10 px-1.5 py-0.5 text-destructive">{t("projects.gantt.dependencyConflictCount", ganttTimeline.dependencyEdges.filter((edge) => edge.violated).length)}</span>
                  {#if dependencyCascadeProposal.items.length > 0}
                    <button
                      type="button"
                      class={cn(
                        "rounded border px-1.5 py-0.5 font-medium",
                        dependencyCascadeOpen
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        dependencyCascadeOpen = !dependencyCascadeOpen;
                      }}
                    >
                      {t("projects.gantt.cascadeReview", dependencyCascadeProposal.items.length)}
                    </button>
                  {/if}
                </div>
              {/if}
            </div>

            {#if dependencyCascadeOpen && dependencyCascadeProposal.items.length > 0}
              <section class="grid gap-2 rounded-md border border-border bg-card p-2">
                <div class="flex flex-wrap items-center justify-between gap-2">
                  <div>
                    <h3 class="text-[0.8rem] font-semibold">{t("projects.gantt.cascadeTitle")}</h3>
                    <div class="text-[0.733333rem] text-muted-foreground">
                      {t("projects.gantt.cascadeDescription")}
                    </div>
                  </div>
                  <div class="flex items-center gap-1">
                    <button
                      type="button"
                      class="rounded-md border border-border bg-background px-2 py-1 text-[0.766667rem] hover:bg-accent"
                      onclick={() => {
                        dependencyCascadeOpen = false;
                      }}
                    >
                      {t("common.cancel")}
                    </button>
                    <button
                      type="button"
                      class="rounded-md bg-primary px-2 py-1 text-[0.766667rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
                      disabled={dependencyCascadeApplying}
                      onclick={() => { void applyDependencyCascadeProposal(); }}
                    >
                      {dependencyCascadeApplying ? t("common.loading") : t("projects.gantt.cascadeApply")}
                    </button>
                  </div>
                </div>
                <div class="grid gap-1">
                  {#each dependencyCascadeProposal.items as item (item.taskId)}
                    <button
                      type="button"
                      class="grid gap-1 rounded border border-border bg-background px-2 py-1.5 text-left hover:bg-accent"
                      onclick={() => {
                        const task = taskById(item.taskId);
                        if (task) openTaskDetail(task);
                      }}
                    >
                      <span class="flex min-w-0 flex-wrap items-center gap-1">
                        <span class="min-w-0 flex-1 truncate text-[0.8rem] font-medium">{item.title}</span>
                        <span class="rounded border border-sky-500/40 bg-sky-500/10 px-1.5 py-0.5 text-[0.733333rem] text-sky-700 dark:text-sky-300">
                          {cascadeShiftLabel(item)}
                        </span>
                      </span>
                      <span class="text-[0.733333rem] text-muted-foreground">
                        {cascadeItemRangeLabel(item, false)} {t("projects.gantt.cascadeTo")} {cascadeItemRangeLabel(item, true)}
                      </span>
                      <span class="text-[0.733333rem] text-muted-foreground">
                        {#each item.reasons as reason, index (reason.dependencyId)}
                          {#if index > 0} / {/if}
                          {t("projects.gantt.cascadeReason", reason.blockingTitle, reason.requiredStartDate)}
                        {/each}
                      </span>
                    </button>
                  {/each}
                </div>
                {#if dependencyCascadeError}
                  <div class="rounded border border-destructive/30 bg-destructive/10 px-2 py-1 text-[0.766667rem] text-destructive">
                    {dependencyCascadeError}
                  </div>
                {/if}
              </section>
            {/if}

            {#if ganttTimeline.rows.length > 0}
              <div class="min-h-0 overflow-x-auto rounded-md border border-border bg-card">
                <div class="min-w-208">
                  <div class="grid min-h-10 grid-cols-[minmax(12rem,16rem)_minmax(34rem,1fr)] border-b border-border bg-muted/40">
                    <div class="flex items-center border-r border-border px-2 text-[0.733333rem] font-medium text-muted-foreground">
                      {t("projects.gantt.taskColumn")}
                    </div>
                    <div class="relative">
                      {#each ganttTimeline.ticks as tick (tick.date)}
                        <div class="absolute inset-y-0 border-l border-border/70" style={ganttTickStyle(tick)}>
                          <span class="absolute left-1 top-1 text-[0.666667rem] text-muted-foreground">{tick.date}</span>
                        </div>
                      {/each}
                      {#if ganttTimeline.todayPercent !== undefined}
                        <div class="absolute inset-y-0 border-l border-primary/70" style={percentStyle(ganttTimeline.todayPercent)}>
                          <span class="absolute bottom-1 left-1 text-[0.666667rem] font-medium text-primary">{t("projects.gantt.today")}</span>
                        </div>
                      {/if}
                    </div>
                  </div>

                  {#each sections as section (section.id)}
                    {@const sectionRows = ganttRowsForSection(section)}
                    {#if sectionRows.length > 0}
                      <div class="grid min-h-8 grid-cols-[minmax(12rem,16rem)_minmax(34rem,1fr)] border-b border-border/70 bg-background/70">
                        <button
                          type="button"
                          class="flex min-w-0 items-center gap-1 border-r border-border px-2 text-left text-[0.8rem] font-medium hover:bg-accent"
                          aria-label={section.collapsed ? t("projects.actions.expandSection", section.name) : t("projects.actions.collapseSection", section.name)}
                          onclick={() => { void toggleSectionCollapsed(section); }}
                        >
                          {#if section.collapsed}
                            <ChevronRight size={13} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                          {:else}
                            <ChevronDown size={13} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                          {/if}
                          <span class="truncate">{section.name}</span>
                          <span class="ml-auto text-[0.733333rem] text-muted-foreground">{sectionRows.length}</span>
                        </button>
                        <div class="relative">
                          {#each ganttTimeline.ticks as tick (tick.date)}
                            <div class="absolute inset-y-0 border-l border-border/50" style={ganttTickStyle(tick)}></div>
                          {/each}
                        </div>
                      </div>
                      {#if !section.collapsed}
                        {#each sectionRows as row (row.task.id)}
                          {@const rowDependencyLabel = ganttDependencyLabel(row)}
                          {@const rowDependencyConflicts = ganttConflictsForTask(row.task.id)}
                          <div class="grid min-h-11 grid-cols-[minmax(12rem,16rem)_minmax(34rem,1fr)] border-b border-border/50">
                            <button
                              type="button"
                              class="grid min-w-0 content-center gap-0.5 border-r border-border px-2 text-left hover:bg-accent"
                              aria-label={t("projects.actions.openTaskDetails", row.task.title)}
                              onclick={() => openTaskDetail(row.task)}
                            >
                              <span class="truncate text-[0.8rem] font-medium">{row.task.title}</span>
                              <span class="truncate text-[0.733333rem] text-muted-foreground">
                                {ganttDateRangeLabel(row.startDate, row.endDate)}
                                {#if rowDependencyLabel}
                                  / {rowDependencyLabel}
                                {/if}
                                {#if rowDependencyConflicts.length > 0}
                                  / {t("projects.gantt.conflictCount", rowDependencyConflicts.length)}
                                {/if}
                              </span>
                            </button>
                            <div class="relative bg-muted/20" data-gantt-track>
                              {#each ganttTimeline.ticks as tick (tick.date)}
                                <div class="absolute inset-y-0 border-l border-border/50" style={ganttTickStyle(tick)}></div>
                              {/each}
                              {#if ganttTimeline.todayPercent !== undefined}
                                <div class="absolute inset-y-0 border-l border-primary/60" style={percentStyle(ganttTimeline.todayPercent)}></div>
                              {/if}
                              {#each ganttEdgesFrom(row.task.id) as edge (edge.id)}
                                <button
                                  type="button"
                                  class="absolute top-1 z-10 flex h-4 min-w-2 items-center overflow-hidden rounded text-[0.666667rem]"
                                  style={ganttEdgeStyle(edge)}
                                  aria-label={ganttEdgeTitle(edge)}
                                  title={ganttEdgeTitle(edge)}
                                  onclick={() => openGanttDependencyTarget(edge)}
                                >
                                  <span class={cn("h-px min-w-2 flex-1", edge.violated ? "bg-destructive" : "bg-muted-foreground/60")}></span>
                                  <ArrowRight
                                    size={10}
                                    strokeWidth={2}
                                    class={edge.violated ? "shrink-0 text-destructive" : "shrink-0 text-muted-foreground"}
                                  />
                                </button>
                              {/each}
                              {#if row.milestone}
                                <button
                                  type="button"
                                  class={cn(
                                    "absolute top-1/2 h-4 w-4 -translate-x-1/2 -translate-y-1/2 rotate-45 cursor-grab rounded-sm border shadow-sm active:cursor-grabbing disabled:cursor-not-allowed",
                                    ganttDateControlClass(row),
                                  )}
                                  style={percentStyle(row.markerPercent)}
                                  aria-label={t("projects.actions.openTaskDetails", row.task.title)}
                                  title={`${row.task.title} / ${row.startDate}`}
                                  disabled={Boolean(row.task.archivedAt) || ganttDatePendingTaskId === row.task.id}
                                  onpointerdown={(event) => startGanttDateDrag(event, row, "move")}
                                  onpointermove={updateGanttDateDrag}
                                  onpointerup={(event) => { void commitGanttDateDrag(event, row); }}
                                  onpointercancel={cancelGanttDateDrag}
                                  onclick={() => openGanttTaskFromBar(row.task)}
                                ></button>
                              {:else}
                                <button
                                  type="button"
                                  class="absolute top-1/2 z-20 h-6 w-3 -translate-x-1/2 -translate-y-1/2 rounded border border-background/80 bg-foreground/20 hover:bg-foreground/35 disabled:cursor-not-allowed disabled:opacity-40"
                                  style={ganttResizeHandleStyle(row, "start")}
                                  disabled={Boolean(row.task.archivedAt) || ganttDatePendingTaskId === row.task.id}
                                  aria-label={t("projects.actions.resizeGanttTaskStart", row.task.title)}
                                  title={t("projects.actions.resizeGanttTaskStart", row.task.title)}
                                  onpointerdown={(event) => startGanttDateDrag(event, row, "resize-start")}
                                  onpointermove={updateGanttDateDrag}
                                  onpointerup={(event) => { void commitGanttDateDrag(event, row); }}
                                  onpointercancel={cancelGanttDateDrag}
                                  onclick={(event) => event.stopPropagation()}
                                ></button>
                                <button
                                  type="button"
                                  class={cn(
                                    "absolute top-1/2 flex h-5 -translate-y-1/2 cursor-grab items-center rounded border px-1.5 text-left text-[0.733333rem] shadow-sm active:cursor-grabbing disabled:cursor-not-allowed",
                                    ganttDateControlClass(row),
                                  )}
                                  style={ganttBarStyle(row)}
                                  aria-label={t("projects.actions.openTaskDetails", row.task.title)}
                                  title={`${row.task.title} / ${ganttDateRangeLabel(row.startDate, row.endDate)}`}
                                  disabled={Boolean(row.task.archivedAt) || ganttDatePendingTaskId === row.task.id}
                                  onpointerdown={(event) => startGanttDateDrag(event, row, "move")}
                                  onpointermove={updateGanttDateDrag}
                                  onpointerup={(event) => { void commitGanttDateDrag(event, row); }}
                                  onpointercancel={cancelGanttDateDrag}
                                  onclick={() => openGanttTaskFromBar(row.task)}
                                >
                                  <span class="truncate">{row.task.title}</span>
                                </button>
                                <button
                                  type="button"
                                  class="absolute top-1/2 z-20 h-6 w-3 -translate-x-1/2 -translate-y-1/2 rounded border border-background/80 bg-foreground/20 hover:bg-foreground/35 disabled:cursor-not-allowed disabled:opacity-40"
                                  style={ganttResizeHandleStyle(row, "end")}
                                  disabled={Boolean(row.task.archivedAt) || ganttDatePendingTaskId === row.task.id}
                                  aria-label={t("projects.actions.resizeGanttTaskEnd", row.task.title)}
                                  title={t("projects.actions.resizeGanttTaskEnd", row.task.title)}
                                  onpointerdown={(event) => startGanttDateDrag(event, row, "resize-end")}
                                  onpointermove={updateGanttDateDrag}
                                  onpointerup={(event) => { void commitGanttDateDrag(event, row); }}
                                  onpointercancel={cancelGanttDateDrag}
                                  onclick={(event) => event.stopPropagation()}
                                ></button>
                              {/if}
                            </div>
                          </div>
                          {#each ganttSubtaskRows(row.task) as subtaskRow (subtaskRow.task.id)}
                            {@const subtaskDependencyLabel = ganttDependencyLabel(subtaskRow)}
                            {@const subtaskDependencyConflicts = ganttConflictsForTask(subtaskRow.task.id)}
                            <div class="grid min-h-10 grid-cols-[minmax(12rem,16rem)_minmax(34rem,1fr)] border-b border-border/50 bg-muted/10">
                              <button
                                type="button"
                                class="grid min-w-0 content-center gap-0.5 border-r border-border px-2 pl-6 text-left hover:bg-accent"
                                aria-label={t("projects.actions.openTaskDetails", subtaskRow.task.title)}
                                onclick={() => openTaskDetail(subtaskRow.task)}
                              >
                                <span class="truncate text-[0.8rem]">{subtaskRow.task.title}</span>
                                <span class="truncate text-[0.733333rem] text-muted-foreground">
                                  {ganttDateRangeLabel(subtaskRow.startDate, subtaskRow.endDate)}
                                  {#if subtaskDependencyLabel}
                                    / {subtaskDependencyLabel}
                                  {/if}
                                  {#if subtaskDependencyConflicts.length > 0}
                                    / {t("projects.gantt.conflictCount", subtaskDependencyConflicts.length)}
                                  {/if}
                                </span>
                              </button>
                              <div class="relative bg-muted/20" data-gantt-track>
                                {#each ganttTimeline.ticks as tick (tick.date)}
                                  <div class="absolute inset-y-0 border-l border-border/50" style={ganttTickStyle(tick)}></div>
                                {/each}
                                {#if ganttTimeline.todayPercent !== undefined}
                                  <div class="absolute inset-y-0 border-l border-primary/60" style={percentStyle(ganttTimeline.todayPercent)}></div>
                                {/if}
                                {#each ganttEdgesFrom(subtaskRow.task.id) as edge (edge.id)}
                                  <button
                                    type="button"
                                    class="absolute top-1 z-10 flex h-4 min-w-2 items-center overflow-hidden rounded text-[0.666667rem]"
                                    style={ganttEdgeStyle(edge)}
                                    aria-label={ganttEdgeTitle(edge)}
                                    title={ganttEdgeTitle(edge)}
                                    onclick={() => openGanttDependencyTarget(edge)}
                                  >
                                    <span class={cn("h-px min-w-2 flex-1", edge.violated ? "bg-destructive" : "bg-muted-foreground/60")}></span>
                                    <ArrowRight
                                      size={10}
                                      strokeWidth={2}
                                      class={edge.violated ? "shrink-0 text-destructive" : "shrink-0 text-muted-foreground"}
                                    />
                                  </button>
                                {/each}
                                {#if subtaskRow.milestone}
                                  <button
                                    type="button"
                                    class={cn(
                                      "absolute top-1/2 h-4 w-4 -translate-x-1/2 -translate-y-1/2 rotate-45 cursor-grab rounded-sm border shadow-sm active:cursor-grabbing disabled:cursor-not-allowed",
                                      ganttDateControlClass(subtaskRow),
                                    )}
                                    style={percentStyle(subtaskRow.markerPercent)}
                                    aria-label={t("projects.actions.openTaskDetails", subtaskRow.task.title)}
                                    title={`${subtaskRow.task.title} / ${subtaskRow.startDate}`}
                                    disabled={Boolean(subtaskRow.task.archivedAt) || ganttDatePendingTaskId === subtaskRow.task.id}
                                    onpointerdown={(event) => startGanttDateDrag(event, subtaskRow, "move")}
                                    onpointermove={updateGanttDateDrag}
                                    onpointerup={(event) => { void commitGanttDateDrag(event, subtaskRow); }}
                                    onpointercancel={cancelGanttDateDrag}
                                    onclick={() => openGanttTaskFromBar(subtaskRow.task)}
                                  ></button>
                                {:else}
                                  <button
                                    type="button"
                                    class="absolute top-1/2 z-20 h-6 w-3 -translate-x-1/2 -translate-y-1/2 rounded border border-background/80 bg-foreground/20 hover:bg-foreground/35 disabled:cursor-not-allowed disabled:opacity-40"
                                    style={ganttResizeHandleStyle(subtaskRow, "start")}
                                    disabled={Boolean(subtaskRow.task.archivedAt) || ganttDatePendingTaskId === subtaskRow.task.id}
                                    aria-label={t("projects.actions.resizeGanttTaskStart", subtaskRow.task.title)}
                                    title={t("projects.actions.resizeGanttTaskStart", subtaskRow.task.title)}
                                    onpointerdown={(event) => startGanttDateDrag(event, subtaskRow, "resize-start")}
                                    onpointermove={updateGanttDateDrag}
                                    onpointerup={(event) => { void commitGanttDateDrag(event, subtaskRow); }}
                                    onpointercancel={cancelGanttDateDrag}
                                    onclick={(event) => event.stopPropagation()}
                                  ></button>
                                  <button
                                    type="button"
                                    class={cn(
                                      "absolute top-1/2 flex h-5 -translate-y-1/2 cursor-grab items-center rounded border px-1.5 text-left text-[0.733333rem] shadow-sm active:cursor-grabbing disabled:cursor-not-allowed",
                                      ganttDateControlClass(subtaskRow),
                                    )}
                                    style={ganttBarStyle(subtaskRow)}
                                    aria-label={t("projects.actions.openTaskDetails", subtaskRow.task.title)}
                                    title={`${subtaskRow.task.title} / ${ganttDateRangeLabel(subtaskRow.startDate, subtaskRow.endDate)}`}
                                    disabled={Boolean(subtaskRow.task.archivedAt) || ganttDatePendingTaskId === subtaskRow.task.id}
                                    onpointerdown={(event) => startGanttDateDrag(event, subtaskRow, "move")}
                                    onpointermove={updateGanttDateDrag}
                                    onpointerup={(event) => { void commitGanttDateDrag(event, subtaskRow); }}
                                    onpointercancel={cancelGanttDateDrag}
                                    onclick={() => openGanttTaskFromBar(subtaskRow.task)}
                                  >
                                    <span class="truncate">{subtaskRow.task.title}</span>
                                  </button>
                                  <button
                                    type="button"
                                    class="absolute top-1/2 z-20 h-6 w-3 -translate-x-1/2 -translate-y-1/2 rounded border border-background/80 bg-foreground/20 hover:bg-foreground/35 disabled:cursor-not-allowed disabled:opacity-40"
                                    style={ganttResizeHandleStyle(subtaskRow, "end")}
                                    disabled={Boolean(subtaskRow.task.archivedAt) || ganttDatePendingTaskId === subtaskRow.task.id}
                                    aria-label={t("projects.actions.resizeGanttTaskEnd", subtaskRow.task.title)}
                                    title={t("projects.actions.resizeGanttTaskEnd", subtaskRow.task.title)}
                                    onpointerdown={(event) => startGanttDateDrag(event, subtaskRow, "resize-end")}
                                    onpointermove={updateGanttDateDrag}
                                    onpointerup={(event) => { void commitGanttDateDrag(event, subtaskRow); }}
                                    onpointercancel={cancelGanttDateDrag}
                                    onclick={(event) => event.stopPropagation()}
                                  ></button>
                                {/if}
                              </div>
                            </div>
                          {/each}
                        {/each}
                      {/if}
                    {/if}
                  {/each}
                </div>
              </div>
            {:else}
              <div class="rounded-md border border-dashed border-border px-2 py-3 text-[0.8rem] text-muted-foreground">
                {t("projects.gantt.noDatedTasks")}
              </div>
            {/if}
          </div>
        {:else}
          <div class="grid gap-3 p-3 min-[760px]:grid-cols-2">
            <section class="rounded-md border border-border bg-card p-3">
              <h2 class="mb-2 text-[0.866667rem] font-semibold">{t("projects.summary.taskCounts")}</h2>
              <div class="grid gap-2 min-[520px]:grid-cols-3">
                <div>
                  <div class="text-[1.6rem] font-semibold">{projectCompletionPercent()}%</div>
                  <div class="text-[0.733333rem] text-muted-foreground">{t("projects.summary.complete")}</div>
                </div>
                <div>
                  <div class="text-[1.2rem] font-semibold">{formatMinutesAsHours(totalOpenEstimateMinutes())}</div>
                  <div class="text-[0.733333rem] text-muted-foreground">{t("projects.summary.openEstimate")}</div>
                </div>
                <div>
                  <div class="text-[1.2rem] font-semibold">{formatMinutesAsHours(thisWeekScheduledMinutes())}</div>
                  <div class="text-[0.733333rem] text-muted-foreground">{t("projects.summary.scheduledThisWeek")}</div>
                </div>
              </div>
              <div class="mt-3 grid gap-1 text-[0.8rem] text-muted-foreground">
                {#each statuses as status (status.id)}
                  <div class="flex items-center justify-between gap-2">
                    <span>{status.name}</span>
                    <span>{tasksForStatus(status).length}</span>
                  </div>
                {/each}
              </div>
            </section>
            <section class="rounded-md border border-border bg-card p-3">
              <h2 class="mb-2 text-[0.866667rem] font-semibold">{t("projects.summary.blocked")}</h2>
              <div class="grid gap-1">
                {#each blockedSummaryTasks() as task (task.id)}
                  <button
                    type="button"
                    class="flex min-w-0 items-center gap-2 rounded border border-border px-2 py-1 text-left text-[0.8rem] hover:bg-accent"
                    onclick={() => openTaskDetail(task)}
                  >
                    <CircleAlert size={14} strokeWidth={1.75} class="shrink-0 text-destructive" />
                    <span class="truncate">{task.title}</span>
                  </button>
                {:else}
                  <div class="text-[0.8rem] text-muted-foreground">{t("projects.summary.noBlockedTasks")}</div>
                {/each}
              </div>
            </section>
            <section class="rounded-md border border-border bg-card p-3">
              <h2 class="mb-2 text-[0.866667rem] font-semibold">{t("projects.summary.upcomingDeadlines")}</h2>
              <div class="grid gap-1">
                {#each upcomingDeadlineTasks() as task (task.id)}
                  <button
                    type="button"
                    class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded border border-border px-2 py-1 text-left text-[0.8rem] hover:bg-accent"
                    onclick={() => openTaskDetail(task)}
                  >
                    <span class="truncate">{task.title}</span>
                    <span class="text-[0.733333rem] text-muted-foreground">{taskDateForDeadline(task)}</span>
                  </button>
                {:else}
                  <div class="text-[0.8rem] text-muted-foreground">{t("projects.summary.noDeadlines")}</div>
                {/each}
              </div>
            </section>
            <section class="rounded-md border border-border bg-card p-3">
              <h2 class="mb-2 text-[0.866667rem] font-semibold">{t("projects.summary.overdue")}</h2>
              <div class="grid gap-1">
                {#each overdueTasks() as task (task.id)}
                  <button
                    type="button"
                    class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded border border-destructive/30 bg-destructive/5 px-2 py-1 text-left text-[0.8rem] hover:bg-destructive/10"
                    onclick={() => openTaskDetail(task)}
                  >
                    <span class="truncate">{task.title}</span>
                    <span class="text-[0.733333rem] text-destructive">{taskDateForDeadline(task)}</span>
                  </button>
                {:else}
                  <div class="text-[0.8rem] text-muted-foreground">{t("projects.summary.noOverdueTasks")}</div>
                {/each}
              </div>
            </section>
            <section class="rounded-md border border-border bg-card p-3">
              <h2 class="mb-2 text-[0.866667rem] font-semibold">{t("projects.summary.unscheduledWithDueDate")}</h2>
              <div class="grid gap-1">
                {#each unscheduledDueTasks() as task (task.id)}
                  <button
                    type="button"
                    class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded border border-border px-2 py-1 text-left text-[0.8rem] hover:bg-accent"
                    onclick={() => openTaskDetail(task)}
                  >
                    <span class="truncate">{task.title}</span>
                    <span class="text-[0.733333rem] text-muted-foreground">{task.dueDate}</span>
                  </button>
                {:else}
                  <div class="text-[0.8rem] text-muted-foreground">{t("projects.summary.noUnscheduledDueTasks")}</div>
                {/each}
              </div>
            </section>
            <section class="rounded-md border border-border bg-card p-3">
              <h2 class="mb-2 text-[0.866667rem] font-semibold">{t("projects.summary.needsEstimates")}</h2>
              <div class="grid gap-1">
                {#each tasksWithoutEstimates() as task (task.id)}
                  <button
                    type="button"
                    class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded border border-border px-2 py-1 text-left text-[0.8rem] hover:bg-accent"
                    onclick={() => openTaskDetail(task)}
                  >
                    <span class="truncate">{task.title}</span>
                    <span class="text-[0.733333rem] text-muted-foreground">{priorityLabel(task.priority)}</span>
                  </button>
                {:else}
                  <div class="text-[0.8rem] text-muted-foreground">{t("projects.summary.noMissingEstimates")}</div>
                {/each}
              </div>
            </section>
            <section class="rounded-md border border-border bg-card p-3 min-[760px]:col-span-2">
              <h2 class="mb-2 text-[0.866667rem] font-semibold">{t("projects.summary.recentChanges")}</h2>
              <div class="grid gap-1 min-[760px]:grid-cols-2">
                {#each recentProjectChangeEvents() as event (event.id)}
                  {@const historyTask = taskById(event.taskId)}
                  <button
                    type="button"
                    class="grid min-h-11 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded border border-border px-2 py-1 text-left hover:bg-accent disabled:cursor-default disabled:opacity-70"
                    disabled={!historyTask}
                    onclick={() => {
                      if (historyTask) openTaskDetail(historyTask);
                    }}
                  >
                      <span class="min-w-0">
                        <span class="block truncate text-[0.8rem]">{historyTaskTitle(event)}</span>
                        <span class="block truncate text-[0.733333rem] text-muted-foreground">{historyEventLabel(event)}</span>
                        {#if event.reason}
                          <span class="block truncate text-[0.733333rem] text-muted-foreground">
                            {t("projects.history.reason", event.reason)}
                          </span>
                        {/if}
                      </span>
                    <span class="text-[0.733333rem] text-muted-foreground">{event.occurredAt}</span>
                  </button>
                {:else}
                  <div class="text-[0.8rem] text-muted-foreground">{t("projects.summary.noRecentChanges")}</div>
                {/each}
              </div>
            </section>
            <section class="rounded-md border border-border bg-card p-3 min-[760px]:col-span-2">
              <h2 class="mb-2 text-[0.866667rem] font-semibold">{t("projects.summary.recentlyCompleted")}</h2>
              <div class="grid gap-1 min-[760px]:grid-cols-2">
                {#each recentlyCompletedTasks() as task (task.id)}
                  <button
                    type="button"
                    class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded border border-border px-2 py-1 text-left text-[0.8rem] hover:bg-accent"
                    onclick={() => openTaskDetail(task)}
                  >
                    <span class="truncate">{task.title}</span>
                    <span class="text-[0.733333rem] text-muted-foreground">{task.completedAt?.slice(0, 10)}</span>
                  </button>
                {:else}
                  <div class="text-[0.8rem] text-muted-foreground">{t("projects.summary.noRecentlyCompleted")}</div>
                {/each}
              </div>
            </section>
          </div>
        {/if}
      </div>
    {:else}
      <div class="flex h-full items-center justify-center p-4 text-[0.866667rem] text-muted-foreground">
        {projects.loading ? t("projects.loading") : t("projects.navigator.empty")}
      </div>
    {/if}
  </section>

  {#if projectSettingsOpen && selectedProject}
    <aside class="flex min-h-0 w-[min(23rem,42vw)] min-w-64 shrink-0 flex-col border-l border-border bg-card max-[760px]:fixed max-[760px]:inset-2 max-[760px]:z-30 max-[760px]:w-auto max-[760px]:rounded-md max-[760px]:border">
      <header class="flex shrink-0 items-center gap-2 border-b border-border px-3 py-2">
        <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-accent text-accent-foreground">
          <ProjectIcon name={projectIconDraft} size={16} />
        </span>
        <div class="min-w-0 flex-1">
          <div class="truncate text-[0.933333rem] font-semibold">{t("projects.settings.title")}</div>
          <div class="truncate text-[0.733333rem] text-muted-foreground">{selectedGroup?.name ?? ""}</div>
        </div>
        <button
          type="button"
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
          aria-label={t("projects.settings.discard")}
          title={t("projects.settings.discard")}
          disabled={!projectSettingsDirty}
          onclick={() => loadProjectSettingsDraft(selectedProject)}
        >
          <RotateCcw size={14} strokeWidth={1.75} />
        </button>
        <button
          type="button"
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
          aria-label={t("projects.settings.close")}
          title={t("projects.settings.close")}
          onclick={closeProjectSettings}
        >
          <X size={15} strokeWidth={1.75} />
        </button>
      </header>

      <form class="flex min-h-0 flex-1 flex-col" onsubmit={(event) => { event.preventDefault(); void saveProjectSettings(); }}>
        <div class="min-h-0 flex-1 overflow-y-auto px-3 py-3">
          <div class="grid gap-3">
            <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
              <span>{t("projects.settings.name")}</span>
              <input
                bind:value={projectNameDraft}
                class="min-h-9 rounded-md border border-border bg-background px-2 text-[0.9rem] font-medium text-foreground"
              />
            </label>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <h2 class="text-[0.8rem] font-semibold">{t("projects.settings.identity")}</h2>
              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.settings.group")}</div>
                <div class="grid gap-1">
                  {#each visibleProjectGroups as group (group.id)}
                    <button
                      type="button"
                      class={cn(
                        "flex min-h-8 items-center gap-2 rounded-md border px-2 text-left text-[0.8rem]",
                        projectGroupDraft === group.id
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-background text-foreground hover:bg-accent",
                      )}
                      onclick={() => {
                        projectGroupDraft = group.id;
                      }}
                    >
                      <ProjectIcon name={group.icon} size={14} class="shrink-0" />
                      <span class="min-w-0 flex-1 truncate">{group.name}</span>
                    </button>
                  {/each}
                </div>
              </div>
              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.settings.lifecycle")}</div>
                <div class="grid grid-cols-3 gap-1">
                  {#each PROJECT_LIFECYCLE_STATUSES as status}
                    <button
                      type="button"
                      class={cn(
                        "min-h-8 rounded-md border px-2 text-[0.766667rem] font-medium",
                        projectStatusDraft === status
                          ? projectLifecycleBadgeClass(status)
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        projectStatusDraft = status;
                      }}
                    >
                      {projectLifecycleLabel(status)}
                    </button>
                  {/each}
                </div>
              </div>
              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.settings.icon")}</div>
                <div class="grid grid-cols-6 gap-1">
                  {#each PROJECT_ICON_OPTIONS as icon}
                    <button
                      type="button"
                      class={cn(
                        "flex h-8 items-center justify-center rounded-md border",
                        projectIconDraft === icon
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      aria-label={t("projects.settings.selectIcon", icon)}
                      onclick={() => {
                        projectIconDraft = icon;
                      }}
                    >
                      <ProjectIcon name={icon} size={15} />
                    </button>
                  {/each}
                </div>
              </div>

              <div class="flex min-h-8 items-center justify-between gap-3 rounded-md border border-border bg-background px-2">
                <span class="text-[0.8rem]">{t("projects.settings.color")}</span>
                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    class="rounded-md border border-border px-2 py-1 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                    onclick={() => {
                      projectColorDraft = undefined;
                    }}
                  >
                    {t("common.none")}
                  </button>
                  <ColorPicker
                    color={projectColorDraft}
                    theme={theme.current}
                    title={t("projects.settings.color")}
                    ariaLabel={t("projects.settings.selectColor")}
                    onselect={(color) => {
                      projectColorDraft = color;
                    }}
                  />
                </div>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <h2 class="text-[0.8rem] font-semibold">{t("projects.settings.defaults")}</h2>
              <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                <span>{t("projects.settings.defaultDuration")}</span>
                <input
                  bind:value={projectDurationDraft}
                  inputmode="numeric"
                  class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                />
              </label>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.settings.defaultPomodoro")}</div>
                <div class="flex flex-wrap gap-1">
                  <button
                    type="button"
                    class={cn(
                      "rounded-md border px-2 py-1 text-[0.766667rem]",
                      projectPomodoroDraft === "none"
                        ? "border-primary/50 bg-primary/10 text-primary"
                        : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                    )}
                    onclick={() => {
                      projectPomodoroDraft = "none";
                    }}
                  >
                    {t("common.none")}
                  </button>
                  {#each PROJECT_POMODORO_OPTIONS as preset}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.766667rem]",
                        projectPomodoroDraft === preset
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        projectPomodoroDraft = preset;
                      }}
                    >
                      {pomodoroPresetLabel(preset)}
                    </button>
                  {/each}
                </div>
              </div>

              <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                <span>{t("projects.settings.defaultIdleTimeout")}</span>
                <input
                  bind:value={projectIdleTimeoutDraft}
                  inputmode="numeric"
                  placeholder={t("common.disabled")}
                  class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                />
              </label>

              <div class="grid gap-2 border-t border-border/60 pt-2">
                <h3 class="text-[0.766667rem] font-semibold">{t("projects.settings.automationDefaults")}</h3>
                <div class="grid gap-2 min-[980px]:grid-cols-2">
                  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                    <span>{t("projects.settings.focusPlaylist")}</span>
                    <input
                      bind:value={projectFocusPlaylistDraft}
                      placeholder={t("common.none")}
                      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                    />
                  </label>
                  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                    <span>{t("projects.settings.breakPlaylist")}</span>
                    <input
                      bind:value={projectBreakPlaylistDraft}
                      placeholder={t("common.none")}
                      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                    />
                  </label>
                  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                    <span>{t("projects.settings.workEnvironment")}</span>
                    <input
                      bind:value={projectWorkEnvironmentDraft}
                      placeholder={t("common.none")}
                      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                    />
                  </label>
                  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                    <span>{t("projects.settings.blockerRuleset")}</span>
                    <input
                      bind:value={projectBlockerRulesetDraft}
                      placeholder={t("common.none")}
                      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                    />
                  </label>
                </div>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.settings.labels")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{projectLabels.length}</span>
              </div>
              <div class="grid gap-2">
                {#each projectLabels as label (label.id)}
                  {@const previousLabel = adjacentLabel(label, -1)}
                  {@const nextLabel = adjacentLabel(label, 1)}
                  {@const draftColor = labelColorDraftValue(label)}
                  <div class="grid gap-2 rounded-md border border-border bg-background p-2">
                    <div class="flex gap-1">
                      <span
                        class={cn("mt-2.5 h-2.5 w-2.5 shrink-0 rounded-full border", labelColorSwatchClass(draftColor))}
                        style={labelColorDotStyle(draftColor)}
                      ></span>
                      <input
                        value={labelNameDraftValue(label)}
                        class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
                        aria-label={t("projects.settings.labelName")}
                        oninput={(event) => {
                          labelNameDrafts = {
                            ...labelNameDrafts,
                            [label.id]: event.currentTarget.value,
                          };
                        }}
                        onkeydown={(event) => {
                          if (event.key === "Enter") {
                            event.preventDefault();
                            void saveLabel(label);
                          }
                        }}
                      />
                      <button
                        type="button"
                        class="flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
                        disabled={!labelDraftDirty(label)}
                        onclick={() => { void saveLabel(label); }}
                      >
                        <Save size={13} strokeWidth={1.75} />
                        <span>{t("projects.settings.saveLabel")}</span>
                      </button>
                    </div>
                    <div class="flex flex-wrap items-center gap-1">
                      <span class="mr-1 text-[0.733333rem] font-medium text-muted-foreground">
                        {t("projects.settings.labelColor")}
                      </span>
                      <button
                        type="button"
                        class={cn(
                          "rounded-md border px-2 py-1 text-[0.733333rem]",
                          draftColor === undefined
                            ? "border-primary/50 bg-primary/10 text-primary"
                            : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                        )}
                        onclick={() => {
                          labelColorDrafts = {
                            ...labelColorDrafts,
                            [label.id]: "none",
                          };
                        }}
                      >
                        {t("common.none")}
                      </button>
                      <ColorPicker
                        color={draftColor}
                        theme={theme.current}
                        title={t("projects.settings.labelColor")}
                        ariaLabel={t("projects.settings.selectLabelColor", label.name)}
                        onselect={(color) => {
                          labelColorDrafts = {
                            ...labelColorDrafts,
                            [label.id]: color ?? "none",
                          };
                        }}
                      />
                      <button
                        type="button"
                        class="ml-auto flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                        disabled={!previousLabel}
                        aria-label={t("projects.actions.moveLabelUp", label.name)}
                        title={t("projects.actions.moveLabelUp", label.name)}
                        onclick={() => { void moveProjectLabel(label, -1); }}
                      >
                        <ArrowUp size={13} strokeWidth={1.75} />
                      </button>
                      <button
                        type="button"
                        class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                        disabled={!nextLabel}
                        aria-label={t("projects.actions.moveLabelDown", label.name)}
                        title={t("projects.actions.moveLabelDown", label.name)}
                        onclick={() => { void moveProjectLabel(label, 1); }}
                      >
                        <ArrowDown size={13} strokeWidth={1.75} />
                      </button>
                      <button
                        type="button"
                        class="flex h-7 w-7 items-center justify-center rounded-md text-destructive hover:bg-destructive/10 disabled:cursor-not-allowed disabled:opacity-40"
                        aria-label={t("projects.actions.deleteLabel", label.name)}
                        title={t("projects.actions.deleteLabel", label.name)}
                        onclick={() => requestDeleteLabel(label)}
                      >
                        <Trash2 size={13} strokeWidth={1.75} />
                      </button>
                    </div>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.settings.noLabels")}
                  </div>
                {/each}
              </div>

              <div class="grid gap-1 rounded-md border border-dashed border-border p-2">
                <div class="flex gap-1">
                  <span
                    class={cn("mt-2.5 h-2.5 w-2.5 shrink-0 rounded-full border", labelColorSwatchClass(newLabelColorValue()))}
                    style={labelColorDotStyle(newLabelColorValue())}
                  ></span>
                  <input
                    bind:value={newLabelName}
                    class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                    placeholder={t("projects.settings.newLabelPlaceholder")}
                    onkeydown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        void submitLabel();
                      }
                    }}
                  />
                  <button
                    type="button"
                    class="flex min-h-8 items-center gap-1 rounded-md bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground"
                    onclick={() => { void submitLabel(); }}
                  >
                    <Plus size={13} strokeWidth={1.75} />
                    <span>{t("projects.settings.addLabel")}</span>
                  </button>
                </div>
                <div class="flex flex-wrap items-center gap-1">
                  <span class="mr-1 text-[0.733333rem] font-medium text-muted-foreground">
                    {t("projects.settings.labelColor")}
                  </span>
                  <button
                    type="button"
                    class={cn(
                      "rounded-md border px-2 py-1 text-[0.733333rem]",
                      newLabelColor === "none"
                        ? "border-primary/50 bg-primary/10 text-primary"
                        : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                    )}
                    onclick={() => {
                      newLabelColor = "none";
                    }}
                  >
                    {t("common.none")}
                  </button>
                  <ColorPicker
                    color={newLabelColorValue()}
                    theme={theme.current}
                    title={t("projects.settings.labelColor")}
                    ariaLabel={t("projects.settings.selectNewLabelColor")}
                    onselect={(color) => {
                      newLabelColor = color ?? "none";
                    }}
                  />
                </div>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.customFields.title")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{projectCustomFields.length}</span>
              </div>
              <div class="grid gap-2">
                {#each projectCustomFields as field (field.id)}
                  {@const previousField = adjacentCustomField(field, -1)}
                  {@const nextField = adjacentCustomField(field, 1)}
                  <div class="grid gap-2 rounded-md border border-border bg-background p-2">
                    <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-1">
                      <input
                        value={customFieldNameDraftValue(field)}
                        class="min-h-8 min-w-0 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
                        aria-label={t("projects.customFields.fieldName")}
                        oninput={(event) => {
                          customFieldNameDrafts = {
                            ...customFieldNameDrafts,
                            [field.id]: event.currentTarget.value,
                          };
                        }}
                        onkeydown={(event) => {
                          if (event.key === "Enter") {
                            event.preventDefault();
                            void saveCustomField(field);
                          }
                        }}
                      />
                      <button
                        type="button"
                        class="flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
                        disabled={!customFieldDraftDirty(field)}
                        onclick={() => { void saveCustomField(field); }}
                      >
                        <Save size={13} strokeWidth={1.75} />
                        <span>{t("projects.customFields.saveField")}</span>
                      </button>
                    </div>
                    <div class="flex flex-wrap items-center gap-1">
                      <span class="rounded border border-border bg-card px-2 py-1 text-[0.733333rem] text-muted-foreground">
                        {customFieldTypeLabel(field.fieldType)}
                      </span>
                      <button
                        type="button"
                        class="ml-auto flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                        disabled={!previousField}
                        aria-label={t("projects.actions.moveCustomFieldUp", field.name)}
                        title={t("projects.actions.moveCustomFieldUp", field.name)}
                        onclick={() => { void moveProjectCustomField(field, -1); }}
                      >
                        <ArrowUp size={13} strokeWidth={1.75} />
                      </button>
                      <button
                        type="button"
                        class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                        disabled={!nextField}
                        aria-label={t("projects.actions.moveCustomFieldDown", field.name)}
                        title={t("projects.actions.moveCustomFieldDown", field.name)}
                        onclick={() => { void moveProjectCustomField(field, 1); }}
                      >
                        <ArrowDown size={13} strokeWidth={1.75} />
                      </button>
                      <button
                        type="button"
                        class="flex h-7 w-7 items-center justify-center rounded-md text-destructive hover:bg-destructive/10 disabled:cursor-not-allowed disabled:opacity-40"
                        aria-label={t("projects.actions.deleteCustomField", field.name)}
                        title={t("projects.actions.deleteCustomField", field.name)}
                        onclick={() => requestDeleteCustomField(field)}
                      >
                        <Trash2 size={13} strokeWidth={1.75} />
                      </button>
                    </div>

                    {#if customFieldAcceptsOptions(field)}
                      <div class="grid gap-1 border-t border-border/60 pt-2">
                        <div class="text-[0.733333rem] font-medium text-muted-foreground">
                          {t("projects.customFields.options")}
                        </div>
                        {#each customFieldOptions(field) as option (option.id)}
                          {@const previousOption = adjacentCustomFieldOption(option, -1)}
                          {@const nextOption = adjacentCustomFieldOption(option, 1)}
                          <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto_auto_auto_auto] items-center gap-1 rounded-md border border-border bg-card px-2">
                            <input
                              value={customFieldOptionNameDraftValue(option)}
                              class="min-h-7 min-w-0 bg-transparent px-1 text-[0.8rem] text-foreground"
                              aria-label={t("projects.customFields.optionName")}
                              oninput={(event) => {
                                customFieldOptionNameDrafts = {
                                  ...customFieldOptionNameDrafts,
                                  [option.id]: event.currentTarget.value,
                                };
                              }}
                              onkeydown={(event) => {
                                if (event.key === "Enter") {
                                  event.preventDefault();
                                  void saveCustomFieldOption(option);
                                }
                              }}
                            />
                            <button
                              type="button"
                              class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                              disabled={!customFieldOptionDraftDirty(option)}
                              aria-label={t("projects.actions.saveCustomFieldOption", option.name)}
                              title={t("projects.actions.saveCustomFieldOption", option.name)}
                              onclick={() => { void saveCustomFieldOption(option); }}
                            >
                              <Save size={13} strokeWidth={1.75} />
                            </button>
                            <button
                              type="button"
                              class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                              disabled={!previousOption}
                              aria-label={t("projects.actions.moveCustomFieldOptionUp", option.name)}
                              title={t("projects.actions.moveCustomFieldOptionUp", option.name)}
                              onclick={() => { void moveProjectCustomFieldOption(option, -1); }}
                            >
                              <ArrowUp size={13} strokeWidth={1.75} />
                            </button>
                            <button
                              type="button"
                              class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                              disabled={!nextOption}
                              aria-label={t("projects.actions.moveCustomFieldOptionDown", option.name)}
                              title={t("projects.actions.moveCustomFieldOptionDown", option.name)}
                              onclick={() => { void moveProjectCustomFieldOption(option, 1); }}
                            >
                              <ArrowDown size={13} strokeWidth={1.75} />
                            </button>
                            <button
                              type="button"
                              class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                              aria-label={t("projects.actions.deleteCustomFieldOption", option.name)}
                              title={t("projects.actions.deleteCustomFieldOption", option.name)}
                              onclick={() => requestDeleteCustomFieldOption(option)}
                            >
                              <Trash2 size={13} strokeWidth={1.75} />
                            </button>
                          </div>
                        {:else}
                          <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                            {t("projects.customFields.noOptions")}
                          </div>
                        {/each}
                        <div class="flex gap-1">
                          <input
                            value={newCustomFieldOptionDrafts[field.id] ?? ""}
                            class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
                            placeholder={t("projects.customFields.newOptionPlaceholder")}
                            oninput={(event) => {
                              newCustomFieldOptionDrafts = {
                                ...newCustomFieldOptionDrafts,
                                [field.id]: event.currentTarget.value,
                              };
                            }}
                            onkeydown={(event) => {
                              if (event.key === "Enter") {
                                event.preventDefault();
                                void submitCustomFieldOption(field);
                              }
                            }}
                          />
                          <button
                            type="button"
                            class="flex min-h-8 items-center gap-1 rounded-md border border-border bg-card px-2 text-[0.733333rem] hover:bg-accent"
                            onclick={() => { void submitCustomFieldOption(field); }}
                          >
                            <Plus size={13} strokeWidth={1.75} />
                            <span>{t("projects.customFields.addOption")}</span>
                          </button>
                        </div>
                      </div>
                    {/if}
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.customFields.noFields")}
                  </div>
                {/each}
              </div>

              <div class="grid gap-2 rounded-md border border-dashed border-border p-2">
                <div class="flex gap-1">
                  <input
                    bind:value={newCustomFieldName}
                    class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                    placeholder={t("projects.customFields.newFieldPlaceholder")}
                    onkeydown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        void submitCustomField();
                      }
                    }}
                  />
                  <button
                    type="button"
                    class="flex min-h-8 items-center gap-1 rounded-md bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground"
                    onclick={() => { void submitCustomField(); }}
                  >
                    <Plus size={13} strokeWidth={1.75} />
                    <span>{t("projects.customFields.addField")}</span>
                  </button>
                </div>
                <div class="flex flex-wrap gap-1">
                  {#each PROJECT_CUSTOM_FIELD_TYPES as fieldType}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.733333rem]",
                        newCustomFieldType === fieldType
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        newCustomFieldType = fieldType;
                      }}
                    >
                      {customFieldTypeLabel(fieldType)}
                    </button>
                  {/each}
                </div>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <h2 class="text-[0.8rem] font-semibold">{t("projects.settings.workflow")}</h2>
              <div class="grid gap-2">
                {#each statuses as status (status.id)}
                  {@const previousStatus = adjacentWorkflowStatus(status, -1)}
                  {@const nextStatus = adjacentWorkflowStatus(status, 1)}
                  <div class="grid gap-1 rounded-md border border-border bg-background p-2">
                    <div class="flex gap-1">
                      <input
                        value={statusNameDrafts[status.id] ?? status.name}
                        class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
                        aria-label={t("projects.settings.statusName")}
                        oninput={(event) => {
                          statusNameDrafts = {
                            ...statusNameDrafts,
                            [status.id]: event.currentTarget.value,
                          };
                        }}
                        onkeydown={(event) => {
                          if (event.key === "Enter") {
                            event.preventDefault();
                            void saveStatus(status);
                          }
                        }}
                      />
                      <button
                        type="button"
                        class="flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
                        disabled={!statusDraftDirty(status)}
                        onclick={() => { void saveStatus(status); }}
                      >
                        <Save size={13} strokeWidth={1.75} />
                        <span>{t("projects.settings.saveStatus")}</span>
                      </button>
                    </div>
                    <div class="flex flex-wrap gap-1">
                      {#each PROJECT_STATUS_CATEGORIES as category}
                        <button
                          type="button"
                          class={cn(
                            "rounded-md border px-2 py-1 text-[0.733333rem]",
                            (statusCategoryDrafts[status.id] ?? status.category) === category
                              ? "border-primary/50 bg-primary/10 text-primary"
                              : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                          )}
                          onclick={() => {
                            statusCategoryDrafts = {
                              ...statusCategoryDrafts,
                              [status.id]: category,
                            };
                          }}
                        >
                          {statusCategoryLabel(category)}
                        </button>
                      {/each}
                      <button
                        type="button"
                        class="ml-auto flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                        disabled={!previousStatus}
                        aria-label={t("projects.actions.moveStatusUp", status.name)}
                        title={t("projects.actions.moveStatusUp", status.name)}
                        onclick={() => { void moveWorkflowStatus(status, -1); }}
                      >
                        <ArrowUp size={13} strokeWidth={1.75} />
                      </button>
                      <button
                        type="button"
                        class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                        disabled={!nextStatus}
                        aria-label={t("projects.actions.moveStatusDown", status.name)}
                        title={t("projects.actions.moveStatusDown", status.name)}
                        onclick={() => { void moveWorkflowStatus(status, 1); }}
                      >
                        <ArrowDown size={13} strokeWidth={1.75} />
                      </button>
                    </div>
                  </div>
                {/each}
              </div>

              <div class="grid gap-1 rounded-md border border-dashed border-border p-2">
                <input
                  bind:value={newStatusName}
                  class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  placeholder={t("projects.settings.newStatusPlaceholder")}
                  onkeydown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      void submitStatus();
                    }
                  }}
                />
                <div class="flex flex-wrap gap-1">
                  {#each PROJECT_STATUS_CATEGORIES as category}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.733333rem]",
                        newStatusCategory === category
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        newStatusCategory = category;
                      }}
                    >
                      {statusCategoryLabel(category)}
                    </button>
                  {/each}
                  <button
                    type="button"
                    class="ml-auto flex min-h-7 items-center gap-1 rounded-md bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground"
                    onclick={() => { void submitStatus(); }}
                  >
                    <Plus size={13} strokeWidth={1.75} />
                    <span>{t("projects.settings.addStatus")}</span>
                  </button>
                </div>
              </div>
            </section>

            {#if projectSettingsError}
              <div class="rounded-md border border-destructive/30 bg-destructive/10 px-2 py-2 text-[0.8rem] text-destructive">
                {projectSettingsError}
              </div>
            {/if}
          </div>
        </div>

        <footer class="flex shrink-0 items-center justify-end gap-2 border-t border-border px-3 py-2">
          <button
            type="button"
            class="flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
            disabled={!projectSettingsDirty}
            onclick={() => loadProjectSettingsDraft(selectedProject)}
          >
            <RotateCcw size={14} strokeWidth={1.75} />
            <span>{t("projects.settings.discard")}</span>
          </button>
          <button
            type="submit"
            class="flex min-h-8 items-center gap-1.5 rounded-md bg-primary px-2 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
            disabled={projectSettingsSaving || !projectSettingsDirty}
          >
            <Save size={14} strokeWidth={1.75} />
            <span>{projectSettingsSaving ? t("common.loading") : t("projects.settings.save")}</span>
          </button>
        </footer>
      </form>
    </aside>
  {:else if selectedTask}
    {@const selectedTaskStatus = statusForTask(selectedTask)}
    {@const selectedTaskSection = sectionForTask(selectedTask)}
    {@const selectedTaskEvents = linkedEventRowsForTask(selectedTask)}
    {@const selectedTaskHistory = projects.taskChangeEventsForTask(selectedTask.id).slice(0, 8)}
    {@const selectedTaskChecklist = projects.checklistItemsForTask(selectedTask.id)}
    {@const selectedTaskLabels = labelsForTask(selectedTask)}
    {@const selectedTaskLabelCandidates = labelCandidateLabels(selectedTask)}
    {@const selectedTaskSubtasks = subtasksForTask(selectedTask)}
    {@const selectedTaskBlockedBy = blockedByDependencies(selectedTask)}
    {@const selectedTaskBlocks = blocksDependencies(selectedTask)}
    {@const selectedTaskParent = selectedTask.parentTaskId ? taskById(selectedTask.parentTaskId) : undefined}
    {@const selectedTaskDependencyCandidates = dependencyCandidateTasks(selectedTask)}
    {@const selectedTaskEventCandidates = eventLinkCandidateEvents(selectedTask)}
    <aside class="flex min-h-0 w-[min(23rem,42vw)] min-w-64 shrink-0 flex-col border-l border-border bg-card max-[760px]:fixed max-[760px]:inset-2 max-[760px]:z-30 max-[760px]:w-auto max-[760px]:rounded-md max-[760px]:border">
      <header class="flex shrink-0 items-center gap-2 border-b border-border px-3 py-2">
        <div class="min-w-0 flex-1">
          <div class="truncate text-[0.933333rem] font-semibold">{t("projects.detail.title")}</div>
          <div class="flex min-w-0 items-center gap-1.5">
            <span class="truncate text-[0.733333rem] text-muted-foreground">
              {selectedTaskSection?.name ?? t("projects.list.general")} / {selectedTaskStatus?.name ?? t("projects.list.status")}
            </span>
            {#if selectedTask.archivedAt}
              <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", taskArchivedBadgeClass(selectedTask))}>
                {t("projects.taskLifecycle.archived")}
              </span>
            {/if}
          </div>
        </div>
        <button
          type="button"
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
          aria-label={t("projects.detail.discard")}
          title={t("projects.detail.discard")}
          disabled={!detailDirty}
          onclick={() => loadTaskDetailDraft(selectedTask)}
        >
          <RotateCcw size={14} strokeWidth={1.75} />
        </button>
        <button
          type="button"
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
          aria-label={t("projects.detail.close")}
          title={t("projects.detail.close")}
          onclick={closeTaskDetail}
        >
          <X size={15} strokeWidth={1.75} />
        </button>
      </header>

      <form class="flex min-h-0 flex-1 flex-col" onsubmit={(event) => { event.preventDefault(); void saveTaskDetail(); }}>
        <div class="min-h-0 flex-1 overflow-y-auto px-3 py-3">
          <div class="grid gap-3">
            <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
              <span>{t("projects.detail.titleLabel")}</span>
              <input
                bind:value={detailTitle}
                class="min-h-9 rounded-md border border-border bg-background px-2 text-[0.9rem] font-medium text-foreground"
              />
            </label>

            <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
              <span>{t("projects.detail.description")}</span>
              <textarea
                bind:value={detailDescription}
                rows="5"
                class="min-h-28 resize-none rounded-md border border-border bg-background px-2 py-2 text-[0.833333rem] text-foreground"
              ></textarea>
            </label>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.properties")}</h2>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.status")}</div>
                <div class="flex flex-wrap gap-1">
                  {#each statuses as status (status.id)}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.766667rem]",
                        detailStatusId === status.id
                          ? statusBadgeClass(status)
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        detailStatusId = status.id;
                      }}
                    >
                      {status.name}
                    </button>
                  {/each}
                </div>
              </div>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.section")}</div>
                <div class="flex flex-wrap gap-1">
                  {#each sections as section (section.id)}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.766667rem]",
                        detailSectionId === section.id
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        detailSectionId = section.id;
                      }}
                    >
                      {section.name}
                    </button>
                  {/each}
                </div>
              </div>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.priority")}</div>
                <div class="flex flex-wrap gap-1">
                  {#each PROJECT_PRIORITIES as priority}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.766667rem]",
                        detailPriority === priority
                          ? priorityClass(priority)
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        detailPriority = priority;
                      }}
                    >
                      {priorityLabel(priority)}
                    </button>
                  {/each}
                </div>
              </div>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.type")}</div>
                <div class="flex flex-wrap gap-1">
                  {#each PROJECT_TASK_TYPES as taskType}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.766667rem]",
                        detailTaskType === taskType
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        detailTaskType = taskType;
                      }}
                    >
                      {taskTypeLabel(taskType)}
                    </button>
                  {/each}
                </div>
              </div>

              <label class="flex min-h-8 items-center gap-2 rounded-md border border-border bg-background px-2 text-[0.8rem]">
                <input type="checkbox" bind:checked={detailMilestone} class="h-4 w-4 accent-primary" />
                <span>{t("projects.detail.milestone")}</span>
              </label>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.labels")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{selectedTaskLabels.length}</span>
              </div>
              {#if selectedTaskLabels.length > 0}
                <div class="flex flex-wrap gap-1">
                  {#each selectedTaskLabels as label (label.id)}
                    <span class="inline-flex min-h-7 max-w-full items-center gap-1 rounded-md border border-border bg-background px-2 text-[0.766667rem]">
                      <span
                        class={cn("h-2 w-2 shrink-0 rounded-full border", labelColorSwatchClass(label.color))}
                        style={labelColorDotStyle(label.color)}
                      ></span>
                      <span class="truncate">{label.name}</span>
                      <button
                        type="button"
                        class="flex h-5 w-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                        aria-label={t("projects.actions.removeLabel", label.name)}
                        title={t("projects.actions.removeLabel", label.name)}
                        onclick={() => { void detachTaskLabel(selectedTask, label); }}
                      >
                        <X size={12} strokeWidth={1.75} />
                      </button>
                    </span>
                  {/each}
                </div>
              {:else}
                <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                  {t("projects.detail.noLabels")}
                </div>
              {/if}
              <div class="grid gap-1">
                <div class="flex gap-1">
                  <input
                    bind:value={labelDraft}
                    placeholder={t("projects.detail.addLabelPlaceholder")}
                    class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem]"
                    onkeydown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        void submitTaskLabel(selectedTask);
                      }
                    }}
                  />
                  <button
                    type="button"
                    class="flex min-h-8 items-center justify-center rounded-md border border-border bg-background px-2 text-[0.8rem] hover:bg-accent"
                    aria-label={t("projects.detail.addLabel")}
                    onclick={() => { void submitTaskLabel(selectedTask); }}
                  >
                    <Plus size={14} strokeWidth={1.75} />
                  </button>
                </div>
                <div class="grid gap-1">
                  {#each selectedTaskLabelCandidates as label (label.id)}
                    <button
                      type="button"
                      class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
                      onclick={() => { void attachExistingLabel(selectedTask, label); }}
                    >
                      <span class="flex min-w-0 items-center gap-2">
                        <span
                          class={cn("h-2 w-2 shrink-0 rounded-full border", labelColorSwatchClass(label.color))}
                          style={labelColorDotStyle(label.color)}
                        ></span>
                        <span class="truncate text-[0.8rem]">{label.name}</span>
                      </span>
                      <Plus size={13} strokeWidth={1.75} class="text-muted-foreground" />
                    </button>
                  {:else}
                    {#if canCreateLabel(selectedTask)}
                      <button
                        type="button"
                        class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
                        onclick={() => { void submitTaskLabel(selectedTask); }}
                      >
                        <span class="truncate text-[0.8rem]">{t("projects.detail.createLabel", labelDraft.trim())}</span>
                        <Plus size={13} strokeWidth={1.75} class="text-muted-foreground" />
                      </button>
                    {:else}
                      <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                        {t("projects.detail.noLabelCandidates")}
                      </div>
                    {/if}
                  {/each}
                </div>
              </div>
            </section>

            {#if projectCustomFields.length > 0}
              <section class="grid gap-2 border-t border-border/70 pt-3">
                <div class="flex items-center justify-between gap-2">
                  <h2 class="text-[0.8rem] font-semibold">{t("projects.customFields.taskValues")}</h2>
                  <span class="text-[0.733333rem] text-muted-foreground">{projectCustomFields.length}</span>
                </div>
                <div class="grid gap-2">
                  {#each projectCustomFields as field (field.id)}
                    <div class="grid gap-1 rounded-md border border-border bg-background p-2">
                      <div class="flex min-w-0 items-center justify-between gap-2">
                        <div class="min-w-0">
                          <div class="truncate text-[0.8rem] font-medium">{field.name}</div>
                          <div class="truncate text-[0.733333rem] text-muted-foreground">
                            {customFieldTypeLabel(field.fieldType)}
                          </div>
                        </div>
                        <button
                          type="button"
                          class="flex min-h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 text-[0.733333rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
                          disabled={!customFieldValueDirty(selectedTask, field)}
                          onclick={() => { void saveTaskCustomField(selectedTask, field); }}
                        >
                          <Save size={13} strokeWidth={1.75} />
                          <span>{t("projects.customFields.saveValue")}</span>
                        </button>
                      </div>

                      {#if field.fieldType === "text" || field.fieldType === "url"}
                        <input
                          value={customFieldTextDrafts[field.id] ?? ""}
                          inputmode={field.fieldType === "url" ? "url" : "text"}
                          class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
                          placeholder={t("projects.customFields.emptyValue")}
                          oninput={(event) => {
                            customFieldTextDrafts = {
                              ...customFieldTextDrafts,
                              [field.id]: event.currentTarget.value,
                            };
                          }}
                          onkeydown={(event) => {
                            if (event.key === "Enter") {
                              event.preventDefault();
                              void saveTaskCustomField(selectedTask, field);
                            }
                          }}
                        />
                      {:else if field.fieldType === "number"}
                        <input
                          value={customFieldNumberDrafts[field.id] ?? ""}
                          inputmode="decimal"
                          class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
                          placeholder={t("projects.customFields.emptyValue")}
                          oninput={(event) => {
                            customFieldNumberDrafts = {
                              ...customFieldNumberDrafts,
                              [field.id]: event.currentTarget.value,
                            };
                          }}
                          onkeydown={(event) => {
                            if (event.key === "Enter") {
                              event.preventDefault();
                              void saveTaskCustomField(selectedTask, field);
                            }
                          }}
                        />
                      {:else if field.fieldType === "date"}
                        <input
                          value={customFieldDateDrafts[field.id] ?? ""}
                          placeholder="YYYY-MM-DD"
                          class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
                          oninput={(event) => {
                            customFieldDateDrafts = {
                              ...customFieldDateDrafts,
                              [field.id]: event.currentTarget.value,
                            };
                          }}
                          onkeydown={(event) => {
                            if (event.key === "Enter") {
                              event.preventDefault();
                              void saveTaskCustomField(selectedTask, field);
                            }
                          }}
                        />
                      {:else if field.fieldType === "checkbox"}
                        <label class="flex min-h-8 items-center gap-2 rounded-md border border-border bg-card px-2 text-[0.8rem]">
                          <input
                            type="checkbox"
                            checked={customFieldCheckboxDrafts[field.id] ?? false}
                            class="h-4 w-4 accent-primary"
                            onchange={(event) => {
                              customFieldCheckboxDrafts = {
                                ...customFieldCheckboxDrafts,
                                [field.id]: event.currentTarget.checked,
                              };
                            }}
                          />
                          <span>{field.name}</span>
                        </label>
                      {:else if field.fieldType === "select"}
                        <div class="flex flex-wrap gap-1">
                          <button
                            type="button"
                            class={cn(
                              "rounded-md border px-2 py-1 text-[0.733333rem]",
                              (customFieldSelectDrafts[field.id] ?? "none") === "none"
                                ? "border-primary/50 bg-primary/10 text-primary"
                                : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                            )}
                            onclick={() => {
                              customFieldSelectDrafts = {
                                ...customFieldSelectDrafts,
                                [field.id]: "none",
                              };
                            }}
                          >
                            {t("projects.customFields.selectNone")}
                          </button>
                          {#each customFieldOptions(field) as option (option.id)}
                            <button
                              type="button"
                              class={cn(
                                "rounded-md border px-2 py-1 text-[0.733333rem]",
                                customFieldSelectDrafts[field.id] === option.id
                                  ? "border-primary/50 bg-primary/10 text-primary"
                                  : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                              )}
                              onclick={() => {
                                customFieldSelectDrafts = {
                                  ...customFieldSelectDrafts,
                                  [field.id]: option.id,
                                };
                              }}
                            >
                              {option.name}
                            </button>
                          {:else}
                            <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                              {t("projects.customFields.noOptions")}
                            </div>
                          {/each}
                        </div>
                      {:else}
                        <div class="flex flex-wrap gap-1">
                          {#each customFieldOptions(field) as option (option.id)}
                            {@const optionSelected = (customFieldMultiDrafts[field.id] ?? []).includes(option.id)}
                            <button
                              type="button"
                              class={cn(
                                "flex min-h-7 items-center gap-1 rounded-md border px-2 text-[0.733333rem]",
                                optionSelected
                                  ? "border-primary/50 bg-primary/10 text-primary"
                                  : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                              )}
                              onclick={() => toggleCustomFieldMultiOption(field, option)}
                            >
                              {#if optionSelected}
                                <Check size={12} strokeWidth={1.75} />
                              {/if}
                              <span>{option.name}</span>
                            </button>
                          {:else}
                            <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                              {t("projects.customFields.noOptions")}
                            </div>
                          {/each}
                        </div>
                      {/if}
                    </div>
                  {/each}
                </div>
              </section>
            {/if}

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.dates")}</h2>
              <div class="grid gap-2 min-[980px]:grid-cols-2">
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.estimateMinutes")}</span>
                  <input
                    bind:value={detailEstimateMinutes}
                    inputmode="numeric"
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.startDate")}</span>
                  <input
                    bind:value={detailStartDate}
                    placeholder="YYYY-MM-DD"
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.dueDate")}</span>
                  <input
                    bind:value={detailDueDate}
                    placeholder="YYYY-MM-DD"
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.targetEndDate")}</span>
                  <input
                    bind:value={detailTargetEndDate}
                    placeholder="YYYY-MM-DD"
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
              </div>
              <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                <span>{t("projects.detail.blockerReason")}</span>
                <input
                  bind:value={detailBlockerReason}
                  class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                />
              </label>
              <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                <span>{t("projects.detail.changeReason")}</span>
                <input
                  bind:value={detailChangeReason}
                  maxlength="1000"
                  placeholder={t("projects.detail.changeReasonPlaceholder")}
                  class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                />
              </label>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.parentTask")}</h2>
                {#if selectedTask.parentTaskId}
                  <button
                    type="button"
                    class="flex min-h-7 items-center gap-1 rounded-md border border-border bg-background px-2 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                    onclick={() => { void promoteSubtaskFromDetail(selectedTask); }}
                  >
                    <ArrowLeft size={13} strokeWidth={1.75} />
                    <span>{t("projects.detail.promoteSubtask")}</span>
                  </button>
                {/if}
              </div>
              {#if selectedTask.parentTaskId}
                <div class="rounded-md border border-border bg-background px-2 py-1.5 text-[0.8rem]">
                  {selectedTaskParent?.title ?? t("projects.detail.missingDependencyTask")}
                </div>
              {:else if taskHasAnySubtasks(selectedTask)}
                <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                  {t("projects.detail.demoteBlockedBySubtasks")}
                </div>
              {:else}
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.demoteToParent")}</span>
                  <input
                    bind:value={parentTaskSearch}
                    placeholder={t("projects.detail.demoteToParentPlaceholder")}
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                  />
                </label>
                <div class="grid gap-1">
                  {#each parentTaskCandidateTasks(selectedTask) as candidate (candidate.id)}
                    <button
                      type="button"
                      class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left text-muted-foreground hover:bg-accent hover:text-foreground"
                      onclick={() => { void demoteTaskFromDetail(selectedTask, candidate); }}
                    >
                      <span class="truncate text-[0.8rem] text-foreground">{candidate.title}</span>
                      <ArrowRight size={13} strokeWidth={1.75} />
                    </button>
                  {:else}
                    <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                      {t("projects.detail.noParentCandidates")}
                    </div>
                  {/each}
                </div>
              {/if}
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.dependencies")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">
                  {selectedTaskBlockedBy.length + selectedTaskBlocks.length}
                </span>
              </div>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.blockedBy")}</div>
                {#each selectedTaskBlockedBy as dependency (dependency.id)}
                  {@const blockingTask = taskById(dependency.blockingTaskId)}
                  <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2">
                    <span class="truncate text-[0.8rem]">
                      {blockingTask?.title ?? t("projects.detail.missingDependencyTask")}
                    </span>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                      aria-label={t("projects.actions.deleteDependency")}
                      title={t("projects.actions.deleteDependency")}
                      onclick={() => { void removeDependency(dependency.id); }}
                    >
                      <Trash2 size={13} strokeWidth={1.75} />
                    </button>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noBlockedBy")}
                  </div>
                {/each}
              </div>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.blocks")}</div>
                {#each selectedTaskBlocks as dependency (dependency.id)}
                  {@const blockedTask = taskById(dependency.blockedTaskId)}
                  <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2">
                    <span class="truncate text-[0.8rem]">
                      {blockedTask?.title ?? t("projects.detail.missingDependencyTask")}
                    </span>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                      aria-label={t("projects.actions.deleteDependency")}
                      title={t("projects.actions.deleteDependency")}
                      onclick={() => { void removeDependency(dependency.id); }}
                    >
                      <Trash2 size={13} strokeWidth={1.75} />
                    </button>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noBlocks")}
                  </div>
                {/each}
              </div>

              <div class="grid gap-1">
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.addBlockedBy")}</span>
                  <input
                    bind:value={dependencySearch}
                    placeholder={t("projects.detail.addBlockedByPlaceholder")}
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <div class="grid gap-1">
                  {#each selectedTaskDependencyCandidates as candidate (candidate.id)}
                    <button
                      type="button"
                      class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
                      onclick={() => { void addBlockingDependency(candidate, selectedTask); }}
                    >
                      <span class="truncate text-[0.8rem]">{candidate.title}</span>
                      <Plus size={13} strokeWidth={1.75} class="text-muted-foreground" />
                    </button>
                  {:else}
                    <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                      {t("projects.detail.noDependencyCandidates")}
                    </div>
                  {/each}
                </div>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.checklist")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{selectedTaskChecklist.length}</span>
              </div>
              <div class="grid gap-1">
                {#each selectedTaskChecklist as item (item.id)}
                  {@const previousChecklistItem = adjacentChecklistItem(item, -1)}
                  {@const nextChecklistItem = adjacentChecklistItem(item, 1)}
                  <div class="grid min-h-8 grid-cols-[auto_minmax(0,1fr)_auto_auto_auto_auto] items-center gap-1 rounded-md border border-border bg-background px-2">
                    <button
                      type="button"
                      class={cn(
                        "flex h-5 w-5 shrink-0 items-center justify-center rounded border",
                        item.completedAt ? "border-emerald-500 bg-emerald-500 text-white" : "border-border hover:bg-accent",
                      )}
                      aria-label={t("projects.actions.toggleChecklistItem")}
                      onclick={() => { void projects.setChecklistItemCompleted(item, !item.completedAt); }}
                    >
                      {#if item.completedAt}
                        <Check size={13} strokeWidth={2} />
                      {/if}
                    </button>
                    <input
                      value={checklistItemDraftTitle(item)}
                      class={cn(
                        "min-h-7 min-w-0 bg-transparent px-1 text-[0.8rem]",
                        item.completedAt ? "text-muted-foreground line-through" : "text-foreground",
                      )}
                      aria-label={t("projects.detail.checklistItemTitle")}
                      oninput={(event) => {
                        checklistTitleDrafts = {
                          ...checklistTitleDrafts,
                          [item.id]: event.currentTarget.value,
                        };
                      }}
                      onkeydown={(event) => {
                        if (event.key === "Enter") {
                          event.preventDefault();
                          void saveChecklistItem(item);
                        }
                      }}
                    />
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={!previousChecklistItem}
                      aria-label={previousChecklistItem ? t("projects.actions.moveChecklistItemUp", item.title) : t("projects.actions.noPreviousChecklistItem")}
                      title={previousChecklistItem ? t("projects.actions.moveChecklistItemUp", item.title) : t("projects.actions.noPreviousChecklistItem")}
                      onclick={() => { void moveChecklistItemInDetail(item, -1); }}
                    >
                      <ArrowUp size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={!nextChecklistItem}
                      aria-label={nextChecklistItem ? t("projects.actions.moveChecklistItemDown", item.title) : t("projects.actions.noNextChecklistItem")}
                      title={nextChecklistItem ? t("projects.actions.moveChecklistItemDown", item.title) : t("projects.actions.noNextChecklistItem")}
                      onclick={() => { void moveChecklistItemInDetail(item, 1); }}
                    >
                      <ArrowDown size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={!checklistItemDirty(item)}
                      aria-label={t("projects.actions.saveChecklistItem", item.title)}
                      title={t("projects.actions.saveChecklistItem", item.title)}
                      onclick={() => { void saveChecklistItem(item); }}
                    >
                      <Save size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                      aria-label={t("projects.actions.deleteChecklistItem", item.title)}
                      onclick={() => { void projects.removeChecklistItem(item.id); }}
                    >
                      <Trash2 size={13} strokeWidth={1.75} />
                    </button>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noChecklistItems")}
                  </div>
                {/each}
              </div>
              <div class="flex gap-1">
                <input
                  bind:value={checklistDraft}
                  placeholder={t("projects.detail.addChecklistItemPlaceholder")}
                  class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem]"
                  onkeydown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      void submitChecklistItem(selectedTask);
                    }
                  }}
                />
                <button
                  type="button"
                  class="flex min-h-8 items-center justify-center rounded-md border border-border bg-background px-2 text-[0.8rem] hover:bg-accent"
                  aria-label={t("projects.detail.addChecklistItem")}
                  onclick={() => { void submitChecklistItem(selectedTask); }}
                >
                  <Plus size={14} strokeWidth={1.75} />
                </button>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.subtasks")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{selectedTaskSubtasks.length}</span>
              </div>
              <div class="grid gap-1">
                {#each selectedTaskSubtasks as subtask (subtask.id)}
                  {@const subtaskStatus = statusForTask(subtask)}
                  {@const previousSubtask = adjacentSubtask(subtask, -1)}
                  {@const nextSubtask = adjacentSubtask(subtask, 1)}
                  <div class="grid min-h-8 grid-cols-[auto_minmax(0,1fr)_auto_auto_auto_auto] items-center gap-1 rounded-md border border-border bg-background px-2">
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
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={Boolean(subtask.archivedAt)}
                      aria-label={t("projects.actions.promoteSubtask", subtask.title)}
                      title={t("projects.actions.promoteSubtask", subtask.title)}
                      onclick={() => { void promoteSubtaskFromDetail(subtask); }}
                    >
                      <ArrowLeft size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={!previousSubtask}
                      aria-label={previousSubtask ? t("projects.actions.moveSubtaskUp", subtask.title) : t("projects.actions.noPreviousSubtask")}
                      title={previousSubtask ? t("projects.actions.moveSubtaskUp", subtask.title) : t("projects.actions.noPreviousSubtask")}
                      onclick={() => { void moveSubtaskInDetail(subtask, -1); }}
                    >
                      <ArrowUp size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={!nextSubtask}
                      aria-label={nextSubtask ? t("projects.actions.moveSubtaskDown", subtask.title) : t("projects.actions.noNextSubtask")}
                      title={nextSubtask ? t("projects.actions.moveSubtaskDown", subtask.title) : t("projects.actions.noNextSubtask")}
                      onclick={() => { void moveSubtaskInDetail(subtask, 1); }}
                    >
                      <ArrowDown size={13} strokeWidth={1.75} />
                    </button>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noSubtasks")}
                  </div>
                {/each}
              </div>
              <div class="flex gap-1">
                <input
                  bind:value={subtaskDraft}
                  placeholder={t("projects.detail.addSubtaskPlaceholder")}
                  class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem]"
                  onkeydown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      void submitSubtask(selectedTask);
                    }
                  }}
                />
                <button
                  type="button"
                  class="flex min-h-8 items-center justify-center rounded-md border border-border bg-background px-2 text-[0.8rem] hover:bg-accent"
                  aria-label={t("projects.detail.addSubtask")}
                  onclick={() => { void submitSubtask(selectedTask); }}
                >
                  <Plus size={14} strokeWidth={1.75} />
                </button>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.scheduledBlocks")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{selectedTaskEvents.length}</span>
              </div>
              <div class="grid gap-1">
                {#each selectedTaskEvents as event (event.id)}
                  <div class="grid grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-2 rounded-md border border-border bg-background px-2 py-1.5">
                    <div class="min-w-0">
                      <span class="block truncate text-[0.8rem]">{event.title || t("calendar.event.noTitle")}</span>
                      <span class="block truncate text-[0.733333rem] text-muted-foreground">{event.start}</span>
                    </div>
                    <span class="text-[0.733333rem] text-muted-foreground">{event.end}</span>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                      aria-label={t("projects.actions.unlinkScheduledBlock")}
                      title={t("projects.actions.unlinkScheduledBlock")}
                      onclick={() => { void unlinkExistingEvent(selectedTask, event); }}
                    >
                      <Trash2 size={13} strokeWidth={1.75} />
                    </button>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noScheduledBlocks")}
                  </div>
                {/each}
              </div>
              {#if eventLinkSearchError}
                <div class="rounded-md border border-destructive/30 bg-destructive/10 px-2 py-2 text-[0.8rem] text-destructive">
                  {eventLinkSearchError}
                </div>
              {/if}
              <div class="grid gap-1">
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.linkExistingBlock")}</span>
                  <input
                    bind:value={eventLinkSearch}
                    placeholder={t("projects.detail.linkExistingBlockPlaceholder")}
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <div class="grid gap-2 min-[980px]:grid-cols-2">
                  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                    <span>{t("projects.detail.eventLinkStartDate")}</span>
                    <input
                      bind:value={eventLinkStartDate}
                      placeholder="YYYY-MM-DD"
                      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                    />
                  </label>
                  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                    <span>{t("projects.detail.eventLinkEndDate")}</span>
                    <input
                      bind:value={eventLinkEndDate}
                      placeholder="YYYY-MM-DD"
                      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                    />
                  </label>
                </div>
                <div class="grid gap-1">
                  {#if eventLinkSearchPending}
                    <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                      {t("projects.detail.searchingBlocks")}
                    </div>
                  {:else}
                    {#each selectedTaskEventCandidates as event (event.id)}
                      {@const otherLinkedTasks = otherLinkedTaskText(event, selectedTask)}
                      <button
                        type="button"
                        class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
                        onclick={() => { void linkExistingEvent(selectedTask, event); }}
                      >
                        <span class="min-w-0">
                          <span class="block truncate text-[0.8rem]">{event.title || t("calendar.event.noTitle")}</span>
                          <span class="block truncate text-[0.733333rem] text-muted-foreground">{event.start}</span>
                          {#if otherLinkedTasks}
                            <span class="block truncate text-[0.7rem] text-muted-foreground">
                              {t("projects.detail.linkedToTasks", otherLinkedTasks)}
                            </span>
                          {/if}
                        </span>
                        <Plus size={13} strokeWidth={1.75} class="text-muted-foreground" />
                      </button>
                    {:else}
                      <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                        {t("projects.detail.noLinkableBlocks")}
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.history")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{selectedTaskHistory.length}</span>
              </div>
              <div class="grid gap-1">
                {#each selectedTaskHistory as event (event.id)}
                  <div class="rounded-md border border-border bg-background px-2 py-1.5">
                    <div class="truncate text-[0.8rem]">{historyEventLabel(event)}</div>
                    {#if event.reason}
                      <div class="truncate text-[0.733333rem] text-muted-foreground">
                        {t("projects.history.reason", event.reason)}
                      </div>
                    {/if}
                    <div class="truncate text-[0.733333rem] text-muted-foreground">{event.occurredAt}</div>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noHistory")}
                  </div>
                {/each}
              </div>
            </section>

            {#if detailError}
              <div class="rounded-md border border-destructive/30 bg-destructive/10 px-2 py-2 text-[0.8rem] text-destructive">
                {detailError}
              </div>
            {/if}
          </div>
        </div>

        <footer class="flex shrink-0 items-center justify-end gap-2 border-t border-border px-3 py-2">
          {#if selectedTask.archivedAt}
            <button
              type="button"
              class="mr-auto flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
              disabled={detailSaving}
              onclick={() => { void restoreTaskFromDetail(selectedTask); }}
            >
              <ArchiveRestore size={14} strokeWidth={1.75} />
              <span>{t("projects.detail.restore")}</span>
            </button>
          {:else}
            <button
              type="button"
              class="mr-auto flex min-h-8 items-center gap-1.5 rounded-md border border-destructive/30 bg-destructive/10 px-2 text-[0.8rem] text-destructive hover:bg-destructive/15 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={detailSaving}
              onclick={() => { void archiveTaskFromDetail(selectedTask); }}
            >
              <Archive size={14} strokeWidth={1.75} />
              <span>{t("projects.detail.archive")}</span>
            </button>
          {/if}
          <button
            type="button"
            class="flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
            disabled={!detailDirty}
            onclick={() => loadTaskDetailDraft(selectedTask)}
          >
            <RotateCcw size={14} strokeWidth={1.75} />
            <span>{t("projects.detail.discard")}</span>
          </button>
          <button
            type="submit"
            class="flex min-h-8 items-center gap-1.5 rounded-md bg-primary px-2 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
            disabled={detailSaving || !detailDirty}
          >
            <Save size={14} strokeWidth={1.75} />
            <span>{detailSaving ? t("common.loading") : t("projects.detail.save")}</span>
          </button>
        </footer>
      </form>
    </aside>
  {/if}
</div>

{#if pendingDeleteLabel}
  <ConfirmDialog
    title={t("projects.settings.deleteLabelTitle", pendingDeleteLabel.name)}
    message={t("projects.settings.deleteLabelMessage", pendingDeleteLabel.name)}
    confirmLabel={t("projects.settings.deleteLabelConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={() => { void confirmDeleteLabel(); }}
    onCancel={cancelDeleteLabel}
  />
{/if}

{#if pendingDeleteCustomField}
  <ConfirmDialog
    title={t("projects.customFields.deleteFieldTitle", pendingDeleteCustomField.name)}
    message={t("projects.customFields.deleteFieldMessage", pendingDeleteCustomField.name)}
    confirmLabel={t("projects.customFields.deleteFieldConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={() => { void confirmDeleteCustomField(); }}
    onCancel={cancelDeleteCustomField}
  />
{/if}

{#if pendingDeleteCustomFieldOption}
  <ConfirmDialog
    title={t("projects.customFields.deleteOptionTitle", pendingDeleteCustomFieldOption.name)}
    message={t(
      "projects.customFields.deleteOptionMessage",
      pendingDeleteCustomFieldOption.name,
      fieldForCustomFieldOption(pendingDeleteCustomFieldOption)?.name ?? "",
    )}
    confirmLabel={t("projects.customFields.deleteOptionConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={() => { void confirmDeleteCustomFieldOption(); }}
    onCancel={cancelDeleteCustomFieldOption}
  />
{/if}
