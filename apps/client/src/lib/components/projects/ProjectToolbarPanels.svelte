<script lang="ts">
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import ArrowUpDown from "@lucide/svelte/icons/arrow-up-down";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import MoreHorizontal from "@lucide/svelte/icons/more-horizontal";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import Save from "@lucide/svelte/icons/save";
  import Search from "@lucide/svelte/icons/search";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectLabelColorDotStyle,
    projectLabelColorSwatchClass,
    projectPriorityLabel,
  } from "$lib/projects/project-display";
  import {
    type ProjectListColumnControl,
    type ProjectToolbarPanel,
  } from "$lib/projects/project-toolbar";
  import {
    customFieldIdFromCustomFieldReference,
    customFieldReference,
  } from "$lib/projects/task-list-columns";
  import {
    PROJECT_PRIORITIES,
    PROJECT_TASK_GROUP_MODES,
    PROJECT_TASK_SORT_MODES,
    type ProjectCustomField,
    type ProjectCustomFieldFilter,
    type ProjectLabel,
    type ProjectPriority,
    type ProjectSavedTaskView,
    type ProjectSection,
    type ProjectTaskDependencyFilter,
    type ProjectTaskDueFilter,
    type ProjectTaskGroupMode,
    type ProjectTaskLabelFilter,
    type ProjectTaskListColumn,
    type ProjectTaskScheduleFilter,
    type ProjectTaskSortDirection,
    type ProjectTaskSortMode,
    type ProjectTaskStatusFilter,
  } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { cn } from "$lib/utils";

  const TASK_STATUS_FILTERS: ProjectTaskStatusFilter[] = ["all", "open", "blocked", "done"];
  const TASK_DUE_FILTERS: ProjectTaskDueFilter[] = ["all", "overdue", "today", "week", "none", "range"];
  const TASK_SCHEDULE_FILTERS: ProjectTaskScheduleFilter[] = ["all", "scheduled", "unscheduled"];
  const TASK_DEPENDENCY_FILTERS: ProjectTaskDependencyFilter[] = ["all", "linked", "blocked_by", "blocking", "none"];
  const TASK_SORT_MODES: ProjectTaskSortMode[] = [...PROJECT_TASK_SORT_MODES];

  let {
    panel,
    sections,
    projectLabels,
    projectCustomFields,
    savedTaskViews,
    taskListColumnControls,
    matchingTaskCount,
    totalTaskCount,
    archivedProjectTaskCount,
    inactiveSectionCount,
    taskFiltersActive,
    savedViewSaving,
    savedViewError,
    taskStatusFilter = $bindable<ProjectTaskStatusFilter>(),
    taskSectionFilter = $bindable<string | "all">(),
    taskPriorityFilter = $bindable<ProjectPriority | "all">(),
    taskDueFilter = $bindable<ProjectTaskDueFilter>(),
    taskDueRangeStart = $bindable<string>(),
    taskDueRangeEnd = $bindable<string>(),
    taskScheduleFilter = $bindable<ProjectTaskScheduleFilter>(),
    taskDependencyFilter = $bindable<ProjectTaskDependencyFilter>(),
    taskLabelFilter = $bindable<ProjectTaskLabelFilter>(),
    taskCustomFieldFilters = $bindable<ProjectCustomFieldFilter[]>(),
    taskGroupBy = $bindable<ProjectTaskGroupMode>(),
    taskSortMode = $bindable<ProjectTaskSortMode>(),
    taskSortDirection = $bindable<ProjectTaskSortDirection>(),
    showArchivedTasks = $bindable<boolean>(),
    showInactiveSections = $bindable<boolean>(),
    savedViewNameDraft = $bindable<string>(),
    onClearTaskFilters,
    onSaveCurrentTaskView,
    onApplyTaskView,
    onDeleteSavedTaskView,
    onToggleTaskListColumn,
    onOpenProjectSettings,
    onOpenTaskFinder,
  }: {
    panel: ProjectToolbarPanel | null;
    sections: ProjectSection[];
    projectLabels: ProjectLabel[];
    projectCustomFields: ProjectCustomField[];
    savedTaskViews: ProjectSavedTaskView[];
    taskListColumnControls: ProjectListColumnControl[];
    matchingTaskCount: number;
    totalTaskCount: number;
    archivedProjectTaskCount: number;
    inactiveSectionCount: number;
    taskFiltersActive: boolean;
    savedViewSaving: boolean;
    savedViewError: string | null;
    taskStatusFilter: ProjectTaskStatusFilter;
    taskSectionFilter: string | "all";
    taskPriorityFilter: ProjectPriority | "all";
    taskDueFilter: ProjectTaskDueFilter;
    taskDueRangeStart: string;
    taskDueRangeEnd: string;
    taskScheduleFilter: ProjectTaskScheduleFilter;
    taskDependencyFilter: ProjectTaskDependencyFilter;
    taskLabelFilter: ProjectTaskLabelFilter;
    taskCustomFieldFilters: ProjectCustomFieldFilter[];
    taskGroupBy: ProjectTaskGroupMode;
    taskSortMode: ProjectTaskSortMode;
    taskSortDirection: ProjectTaskSortDirection;
    showArchivedTasks: boolean;
    showInactiveSections: boolean;
    savedViewNameDraft: string;
    onClearTaskFilters: () => void;
    onSaveCurrentTaskView: () => void | Promise<void>;
    onApplyTaskView: (view: ProjectSavedTaskView) => void | Promise<void>;
    onDeleteSavedTaskView: (view: ProjectSavedTaskView) => void | Promise<void>;
    onToggleTaskListColumn: (column: ProjectTaskListColumn) => void | Promise<void>;
    onOpenProjectSettings: () => void;
    onOpenTaskFinder: () => void;
  } = $props();

  const projects = getProjects();
  const theme = getTheme();
  const { t } = getLocalization();

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

  function customFieldFilterFor(fieldId: string): ProjectCustomFieldFilter | undefined {
    return taskCustomFieldFilters.find((filter) => filter.fieldId === fieldId);
  }

  function setTaskCustomFieldFilter(filter: ProjectCustomFieldFilter): void {
    taskCustomFieldFilters = [
      ...taskCustomFieldFilters.filter((entry) => entry.fieldId !== filter.fieldId),
      filter,
    ];
  }

  function clearTaskCustomFieldFilter(fieldId: string): void {
    taskCustomFieldFilters = taskCustomFieldFilters.filter((filter) => filter.fieldId !== fieldId);
  }

  function customFieldFilterButtonClass(active: boolean): string {
    return active
      ? "bg-background text-foreground shadow-sm"
      : "text-muted-foreground hover:bg-background/60 hover:text-foreground";
  }

  function customFieldSortMode(field: ProjectCustomField): ProjectTaskSortMode {
    return customFieldReference(field.id);
  }
</script>

{#if panel === "filters"}
  <div class="mx-3 my-2 flex min-w-0 flex-wrap items-center gap-2 rounded-md border border-border bg-card p-2 text-[0.766667rem]">
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
          {projectPriorityLabel(priority, t)}
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
          bind:value={taskDueRangeStart}
          placeholder={t("projects.filters.dueRangeStart")}
          aria-label={t("projects.filters.dueRangeStart")}
          class="h-7 w-28 shrink-0 rounded bg-transparent px-2 text-[0.766667rem] text-foreground placeholder:text-muted-foreground"
        />
        <span class="shrink-0 text-muted-foreground">{t("projects.filters.dueRangeTo")}</span>
        <input
          bind:value={taskDueRangeEnd}
          placeholder={t("projects.filters.dueRangeEnd")}
          aria-label={t("projects.filters.dueRangeEnd")}
          class="h-7 w-28 shrink-0 rounded bg-transparent px-2 text-[0.766667rem] text-foreground placeholder:text-muted-foreground"
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
            class={cn("h-2 w-2 shrink-0 rounded-full border", projectLabelColorSwatchClass(label.color))}
            style={projectLabelColorDotStyle(label.color, theme.current)}
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
          {@const currentCustomFieldFilter = customFieldFilterFor(field.id)}
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
              {#each projects.customFieldOptionsForField(field.id) as option (option.id)}
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
      {t("projects.filters.matchingTasks", matchingTaskCount, totalTaskCount)}
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
        onclick={onClearTaskFilters}
      >
        <RotateCcw size={13} strokeWidth={1.75} />
        <span>{t("projects.filters.reset")}</span>
      </button>
    {/if}
  </div>
{/if}

{#if panel === "customize"}
  <div class="mx-3 my-2 flex min-w-0 flex-wrap items-center gap-2 rounded-md border border-border bg-card p-2 text-[0.766667rem]">
    <form
      class="flex min-w-52 max-w-full flex-1 gap-1 min-[820px]:max-w-sm"
      onsubmit={(event) => { event.preventDefault(); void onSaveCurrentTaskView(); }}
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
              onclick={() => { void onApplyTaskView(view); }}
            >
              <span class="truncate">{view.name}</span>
            </button>
            <button
              type="button"
              class="flex h-full w-7 shrink-0 items-center justify-center border-l border-border text-muted-foreground hover:bg-destructive/10 hover:text-destructive disabled:cursor-not-allowed disabled:opacity-50"
              disabled={savedViewSaving}
              aria-label={t("projects.savedViews.delete", view.name)}
              title={t("projects.savedViews.delete", view.name)}
              onclick={() => { void onDeleteSavedTaskView(view); }}
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
          onclick={() => { void onToggleTaskListColumn(control.column); }}
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

{#if panel === "more"}
  <div class="mx-3 my-2 flex min-w-0 flex-wrap items-center gap-2 rounded-md border border-border bg-card p-2 text-[0.766667rem]">
    <button
      type="button"
      class="flex h-8 shrink-0 items-center gap-1.5 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
      onclick={onOpenProjectSettings}
    >
      <MoreHorizontal size={13} strokeWidth={1.75} />
      <span>{t("projects.header.projectSettings")}</span>
    </button>
    <button
      type="button"
      class="flex h-8 shrink-0 items-center gap-1.5 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
      onclick={onOpenTaskFinder}
    >
      <Search size={13} strokeWidth={1.75} />
      <span>{t("projects.finder.open")}</span>
    </button>
    {#if taskFiltersActive}
      <button
        type="button"
        class="flex h-8 shrink-0 items-center gap-1.5 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
        onclick={onClearTaskFilters}
      >
        <RotateCcw size={13} strokeWidth={1.75} />
        <span>{t("projects.filters.reset")}</span>
      </button>
    {/if}
    <span class="h-8 shrink-0 rounded-md border border-border bg-muted/60 px-2 py-1.5 text-muted-foreground">
      {t("projects.filters.matchingTasks", matchingTaskCount, totalTaskCount)}
    </span>
  </div>
{/if}
