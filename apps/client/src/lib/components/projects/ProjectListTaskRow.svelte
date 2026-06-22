<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectLabelColorDotStyle,
    projectLabelColorSwatchClass,
    projectTaskArchivedBadgeClass,
  } from "$lib/projects/project-display";
  import type {
    ProjectCustomField,
    ProjectLabel,
    ProjectPriority,
    ProjectStatus,
    ProjectTask,
    ProjectTaskListColumn,
  } from "$lib/projects/types";
  import type { Theme } from "$lib/stores/themes";
  import { cn } from "$lib/utils";
  import ProjectListColumnCell from "./ProjectListColumnCell.svelte";
  import ProjectListSubtaskRows from "./ProjectListSubtaskRows.svelte";

  let {
    task,
    status,
    statuses,
    subtasks,
    scheduled,
    taskLabels,
    hiddenLabels,
    blockedByCount,
    blocksCount,
    taskListColumns,
    projectCustomFields,
    selectedTaskId,
    taskSelected,
    gridTemplate,
    gridMinWidth,
    sectionName,
    draggable,
    dragging,
    dropPending,
    statusMenuOpen,
    priorityMenuOpen,
    startDateMenuOpen,
    dueDateMenuOpen,
    theme,
    estimateLabel,
    customFieldDisplayValue,
    statusForTask,
    onToggleTaskSelection,
    onOpenTask,
    onPointerDown,
    onPointerUp,
    onPointerCancel,
    onDragStart,
    onDragEnd,
    onDragOver,
    onDrop,
    onToggleStatusMenu,
    onSetStatus,
    onTogglePriorityMenu,
    onSetPriority,
    onToggleStartDateMenu,
    onCloseStartDateMenu,
    onSetStartDate,
    onClearStartDate,
    onSetStartTime,
    onClearStartTime,
    onToggleDueDateMenu,
    onCloseDueDateMenu,
    onSetDueDate,
    onClearDueDate,
    onSetDueTime,
    onClearDueTime,
    onToggleSubtaskDone,
  }: {
    task: ProjectTask;
    status: ProjectStatus | undefined;
    statuses: ProjectStatus[];
    subtasks: ProjectTask[];
    scheduled: string | null;
    taskLabels: ProjectLabel[];
    hiddenLabels: number;
    blockedByCount: number;
    blocksCount: number;
    taskListColumns: ProjectTaskListColumn[];
    projectCustomFields: ProjectCustomField[];
    selectedTaskId: string | null;
    taskSelected: boolean;
    gridTemplate: string;
    gridMinWidth: string;
    sectionName?: string;
    draggable: boolean;
    dragging: boolean;
    dropPending: boolean;
    statusMenuOpen: boolean;
    priorityMenuOpen: boolean;
    startDateMenuOpen: boolean;
    dueDateMenuOpen: boolean;
    theme: Theme;
    estimateLabel: (minutes: number) => string;
    customFieldDisplayValue: (task: ProjectTask, field: ProjectCustomField) => string | undefined;
    statusForTask: (task: ProjectTask) => ProjectStatus | undefined;
    onToggleTaskSelection: (task: ProjectTask) => void;
    onOpenTask: (task: ProjectTask) => void;
    onPointerDown?: (event: PointerEvent) => void;
    onPointerUp?: (event: PointerEvent) => void;
    onPointerCancel?: (event: PointerEvent) => void;
    onDragStart?: (event: DragEvent) => void;
    onDragEnd?: (event: DragEvent) => void;
    onDragOver?: (event: DragEvent) => void;
    onDrop?: (event: DragEvent) => void;
    onToggleStatusMenu: () => void;
    onSetStatus: (status: ProjectStatus) => void;
    onTogglePriorityMenu: () => void;
    onSetPriority: (priority: ProjectPriority) => void;
    onToggleStartDateMenu: () => void;
    onCloseStartDateMenu: () => void;
    onSetStartDate: (startDate: string) => void;
    onClearStartDate: () => void;
    onSetStartTime: (startTime: string) => void;
    onClearStartTime: () => void;
    onToggleDueDateMenu: () => void;
    onCloseDueDateMenu: () => void;
    onSetDueDate: (dueDate: string) => void;
    onClearDueDate: () => void;
    onSetDueTime: (dueTime: string) => void;
    onClearDueTime: () => void;
    onToggleSubtaskDone: (task: ProjectTask) => void;
  } = $props();

  const { t } = getLocalization();
</script>

<div
  role="listitem"
  class={cn(
    "project-list-divider group/row relative grid min-h-11 items-center px-1 transition-colors hover:bg-accent/20",
    selectedTaskId === task.id && "bg-accent/40 ring-1 ring-inset ring-primary/20",
    task.archivedAt && "opacity-70",
    dragging && "opacity-50",
    dropPending && "opacity-60",
  )}
  style={`grid-template-columns: ${gridTemplate}; min-width: ${gridMinWidth};`}
  {draggable}
  onpointerdown={(event) => onPointerDown?.(event)}
  onpointerup={(event) => onPointerUp?.(event)}
  onpointercancel={(event) => onPointerCancel?.(event)}
  ondragstart={(event) => onDragStart?.(event)}
  ondragend={(event) => onDragEnd?.(event)}
  ondragover={(event) => onDragOver?.(event)}
  ondrop={(event) => onDrop?.(event)}
>
  <div class="flex h-full items-center justify-center">
    <button
      type="button"
      class={cn(
        "flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded border transition-opacity",
        taskSelected
          ? "border-primary bg-primary text-primary-foreground opacity-100"
          : "border-border bg-background opacity-0 hover:bg-accent group-hover/row:opacity-100 group-focus-within/row:opacity-100",
      )}
      aria-label={taskSelected ? t("projects.actions.unselectTask", task.title) : t("projects.actions.selectTask", task.title)}
      onclick={() => onToggleTaskSelection(task)}
    >
      {#if taskSelected}
        <Check size={13} strokeWidth={2} />
      {/if}
    </button>
  </div>
  <div class="flex h-full items-center justify-center">
    <button
      type="button"
      class="flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground opacity-0 transition-opacity hover:bg-accent hover:text-foreground group-hover/row:opacity-100 group-focus-within/row:opacity-100"
      aria-label={t("projects.actions.openTaskDetails", task.title)}
      onclick={() => onOpenTask(task)}
    >
      <ChevronRight size={14} strokeWidth={1.75} />
    </button>
  </div>
  <button
    type="button"
    data-list-row-drag-source="true"
    class="project-list-cell-frame relative flex min-h-11 min-w-0 cursor-pointer flex-col justify-center self-stretch rounded-md px-2 py-1 text-left"
    aria-label={t("projects.actions.openTaskDetails", task.title)}
    onclick={() => onOpenTask(task)}
  >
    <div class="truncate text-[0.866667rem]">{task.title}</div>
    {#if sectionName || subtasks.length > 0}
      <div class="mt-0.5 flex min-w-0 flex-wrap items-center gap-1 text-[0.733333rem] text-muted-foreground">
        {#if sectionName}
          <span class="truncate">{sectionName}</span>
        {/if}
        {#if subtasks.length > 0}
          <span>{t("projects.list.subtasks", subtasks.length)}</span>
        {/if}
      </div>
    {/if}
    {#if taskLabels.length > 0}
      <div class="mt-1 flex min-w-0 flex-wrap gap-1">
        {#each taskLabels as label (label.id)}
          <span class="inline-flex max-w-full items-center gap-1 rounded border border-border bg-muted/50 px-1.5 py-0.5 text-[0.666667rem] text-muted-foreground">
            <span
              class={cn("h-1.5 w-1.5 shrink-0 rounded-full border", projectLabelColorSwatchClass(label.color))}
              style={projectLabelColorDotStyle(label.color, theme)}
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
      <span class={cn("mt-1 inline-flex w-fit rounded border px-1.5 py-0.5 text-[0.733333rem]", projectTaskArchivedBadgeClass(task))}>
        {t("projects.taskLifecycle.archived")}
      </span>
    {/if}
  </button>
  {#each taskListColumns as column (column)}
    <ProjectListColumnCell
      {column}
      {task}
      {status}
      {statuses}
      statusMenuOpen={statusMenuOpen}
      priorityMenuOpen={priorityMenuOpen}
      startDateMenuOpen={startDateMenuOpen}
      dueDateMenuOpen={dueDateMenuOpen}
      {projectCustomFields}
      {scheduled}
      {blockedByCount}
      {blocksCount}
      {estimateLabel}
      {customFieldDisplayValue}
      onToggleStatusMenu={onToggleStatusMenu}
      onSetStatus={onSetStatus}
      onTogglePriorityMenu={onTogglePriorityMenu}
      onSetPriority={onSetPriority}
      onToggleStartDateMenu={onToggleStartDateMenu}
      onCloseStartDateMenu={onCloseStartDateMenu}
      onSetStartDate={onSetStartDate}
      onClearStartDate={onClearStartDate}
      onSetStartTime={onSetStartTime}
      onClearStartTime={onClearStartTime}
      onToggleDueDateMenu={onToggleDueDateMenu}
      onCloseDueDateMenu={onCloseDueDateMenu}
      onSetDueDate={onSetDueDate}
      onClearDueDate={onClearDueDate}
      onSetDueTime={onSetDueTime}
      onClearDueTime={onClearDueTime}
    />
  {/each}
  <ProjectListSubtaskRows
    {subtasks}
    {selectedTaskId}
    {statusForTask}
    onToggleDone={onToggleSubtaskDone}
    onOpenTask={onOpenTask}
  />
</div>

<style>
  .project-list-cell-frame::before {
    position: absolute;
    inset: 0;
    z-index: 1;
    border: 1px solid transparent;
    border-radius: 0.375rem;
    content: "";
    pointer-events: none;
    transition: border-color 150ms ease;
  }

  .project-list-cell-frame:hover::before,
  .project-list-cell-frame:focus-visible::before {
    border-color: color-mix(in srgb, var(--foreground) 25%, transparent);
  }
</style>
