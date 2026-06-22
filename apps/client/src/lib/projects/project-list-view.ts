import type {
  ProjectCustomField,
  ProjectCustomFieldOption,
  ProjectCustomFieldFilter,
  ProjectLabel,
  ProjectPriority,
  ProjectSection,
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
} from "./types";
import type { Translate } from "$lib/i18n/translator.svelte";
import { projectPriorityLabel } from "./project-display";
import { deriveProjectFilterChips, type ProjectFilterChip } from "./project-toolbar";

export interface ProjectTaskFilterState {
  search: string;
  statusFilter: ProjectTaskStatusFilter;
  sectionFilter: string | "all";
  priorityFilter: ProjectPriority | "all";
  dueFilter: ProjectTaskDueFilter;
  dueRangeStart: string;
  dueRangeEnd: string;
  scheduleFilter: ProjectTaskScheduleFilter;
  dependencyFilter: ProjectTaskDependencyFilter;
  labelFilter: ProjectTaskLabelFilter;
  customFieldFilters: readonly ProjectCustomFieldFilter[];
  groupBy: ProjectTaskGroupMode;
  sortMode: ProjectTaskSortMode;
  sortDirection: ProjectTaskSortDirection;
}

export const PROJECT_TASK_FILTER_DEFAULTS = Object.freeze({
  search: "",
  statusFilter: "all",
  sectionFilter: "all",
  priorityFilter: "all",
  dueFilter: "all",
  dueRangeStart: "",
  dueRangeEnd: "",
  scheduleFilter: "all",
  dependencyFilter: "all",
  labelFilter: "all",
  customFieldFilters: [],
  groupBy: "section",
  sortMode: "manual",
  sortDirection: "asc",
} satisfies ProjectTaskFilterState);

export function projectTaskListColumnTrack(column: ProjectTaskListColumn): string {
  if (column === "priority" || column === "estimate" || column === "start" || column === "due") return "minmax(7rem, 0.7fr)";
  if (column === "assignee" || column === "reviewer") return "minmax(6rem, 0.55fr)";
  if (column === "status" || column === "scheduled" || column === "dependencies") return "minmax(8rem, 0.8fr)";
  return "minmax(9rem, 0.85fr)";
}

function projectTaskListColumnMinWidthRem(column: ProjectTaskListColumn): number {
  if (column === "priority" || column === "estimate" || column === "start" || column === "due") return 7;
  if (column === "assignee" || column === "reviewer") return 6;
  if (column === "status" || column === "scheduled" || column === "dependencies") return 8;
  return 9;
}

export function projectTaskListGridTemplate(columns: readonly ProjectTaskListColumn[]): string {
  return [
    "1.5rem",
    "1.75rem",
    "minmax(16rem, 2fr)",
    ...columns.map(projectTaskListColumnTrack),
  ].join(" ");
}

export function projectTaskListGridMinWidth(columns: readonly ProjectTaskListColumn[]): string {
  const remWidth = Math.max(47, 25 + columns.reduce((total, column) => total + projectTaskListColumnMinWidthRem(column), 0));
  return `${remWidth}rem`;
}

export function selectedProjectTaskIdsInView(
  selectedTaskIds: readonly string[],
  visibleTasks: readonly ProjectTask[],
): string[] {
  const visibleTaskIds = new Set(visibleTasks.map((task) => task.id));
  return selectedTaskIds.filter((taskId) => visibleTaskIds.has(taskId));
}

export function projectTaskDataFiltersActive(input: ProjectTaskFilterState): boolean {
  return input.search.trim().length > 0
    || input.statusFilter !== "all"
    || input.sectionFilter !== "all"
    || input.priorityFilter !== "all"
    || input.dueFilter !== "all"
    || input.scheduleFilter !== "all"
    || input.dependencyFilter !== "all"
    || input.labelFilter !== "all"
    || input.customFieldFilters.length > 0;
}

export function projectTaskFiltersActive(input: ProjectTaskFilterState): boolean {
  return projectTaskDataFiltersActive(input)
    || input.groupBy !== "section"
    || input.sortMode !== "manual"
    || input.sortDirection !== "asc";
}

export interface ProjectTaskActiveFilterChipInput extends ProjectTaskFilterState {
  sections: readonly ProjectSection[];
  labels: readonly ProjectLabel[];
  customFields: readonly ProjectCustomField[];
  customFieldOptionsForField: (fieldId: string) => readonly ProjectCustomFieldOption[];
  normalizedDueRangeStart?: string;
  normalizedDueRangeEnd?: string;
  t: Translate;
}

export function projectTaskStatusFilterLabel(filter: ProjectTaskStatusFilter, t: Translate): string {
  if (filter === "open") return t("projects.filters.open");
  if (filter === "blocked") return t("projects.filters.blocked");
  if (filter === "done") return t("projects.filters.done");
  return t("projects.filters.allStatuses");
}

export function projectTaskDueFilterLabel(filter: ProjectTaskDueFilter, t: Translate): string {
  if (filter === "overdue") return t("projects.filters.overdue");
  if (filter === "today") return t("projects.filters.today");
  if (filter === "week") return t("projects.filters.thisWeek");
  if (filter === "none") return t("projects.filters.noDueDate");
  if (filter === "range") return t("projects.filters.dueRange");
  return t("projects.filters.allDueDates");
}

export function projectTaskScheduleFilterLabel(filter: ProjectTaskScheduleFilter, t: Translate): string {
  if (filter === "scheduled") return t("projects.filters.scheduled");
  if (filter === "unscheduled") return t("projects.filters.unscheduled");
  return t("projects.filters.allSchedule");
}

export function projectTaskDependencyFilterLabel(filter: ProjectTaskDependencyFilter, t: Translate): string {
  if (filter === "linked") return t("projects.filters.hasDependencies");
  if (filter === "blocked_by") return t("projects.filters.blockedByDependencies");
  if (filter === "blocking") return t("projects.filters.blockingDependencies");
  if (filter === "none") return t("projects.filters.noDependencies");
  return t("projects.filters.allDependencies");
}

export function projectTaskLabelFilterLabel(
  filter: ProjectTaskLabelFilter,
  labels: readonly ProjectLabel[],
  t: Translate,
): string {
  if (filter === "all") return t("projects.filters.allLabels");
  if (filter === "none") return t("projects.filters.noLabels");
  return labels.find((label) => label.id === filter)?.name ?? t("projects.filters.allLabels");
}

export function projectTaskSectionFilterLabel(
  filter: string | "all",
  sections: readonly ProjectSection[],
  t: Translate,
): string {
  if (filter === "all") return t("projects.filters.allSections");
  return sections.find((section) => section.id === filter)?.name ?? t("projects.filters.allSections");
}

export function projectTaskDueFilterChipLabel(input: {
  filter: ProjectTaskDueFilter;
  normalizedStart?: string;
  normalizedEnd?: string;
  t: Translate;
}): string {
  if (input.filter !== "range") return projectTaskDueFilterLabel(input.filter, input.t);
  const start = input.normalizedStart ?? input.t("projects.filters.dueRangeStart");
  const end = input.normalizedEnd ?? input.t("projects.filters.dueRangeEnd");
  return input.t("projects.filters.dueRangeChip", start, end);
}

export function projectTaskCustomFieldFilterLabel(
  filter: ProjectCustomFieldFilter,
  customFields: readonly ProjectCustomField[],
  customFieldOptionsForField: (fieldId: string) => readonly ProjectCustomFieldOption[],
  t: Translate,
): string {
  const field = customFields.find((entry) => entry.id === filter.fieldId);
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
    const optionName = customFields
      .flatMap((entry) => customFieldOptionsForField(entry.id))
      .find((option) => option.id === filter.optionId)?.name ?? t("projects.filters.allValues");
    return t("projects.filters.customFieldChip", fieldName, optionName);
  }
  return fieldName;
}

export function projectTaskActiveFilterChips(input: ProjectTaskActiveFilterChipInput): ProjectFilterChip[] {
  return deriveProjectFilterChips({
    search: input.search,
    statusLabel: input.statusFilter === "all" ? undefined : projectTaskStatusFilterLabel(input.statusFilter, input.t),
    sectionLabel: input.sectionFilter === "all"
      ? undefined
      : projectTaskSectionFilterLabel(input.sectionFilter, input.sections, input.t),
    priorityLabel: input.priorityFilter === "all" ? undefined : projectPriorityLabel(input.priorityFilter, input.t),
    dueLabel: input.dueFilter === "all"
      ? undefined
      : projectTaskDueFilterChipLabel({
        filter: input.dueFilter,
        normalizedStart: input.normalizedDueRangeStart,
        normalizedEnd: input.normalizedDueRangeEnd,
        t: input.t,
      }),
    scheduleLabel: input.scheduleFilter === "all"
      ? undefined
      : projectTaskScheduleFilterLabel(input.scheduleFilter, input.t),
    dependencyLabel: input.dependencyFilter === "all"
      ? undefined
      : projectTaskDependencyFilterLabel(input.dependencyFilter, input.t),
    labelFilterLabel: input.labelFilter === "all"
      ? undefined
      : projectTaskLabelFilterLabel(input.labelFilter, input.labels, input.t),
    customFieldFilters: input.customFieldFilters,
    customFieldFilterLabel: (filter) =>
      projectTaskCustomFieldFilterLabel(filter, input.customFields, input.customFieldOptionsForField, input.t),
  });
}
