<script lang="ts">
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import Check from "@lucide/svelte/icons/check";
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectKanbanDropSortOrder,
    type ProjectKanbanDropPosition,
  } from "$lib/projects/kanban-drag";
  import {
    projectPriorityDisplayColor,
    projectPriorityDisplayLabel,
    projectTaskArchivedBadgeClass,
  } from "$lib/projects/project-display";
  import { manualStatusCompare } from "$lib/projects/task-view";
  import type {
    ProjectPriorityConfig,
    ProjectStatus,
    ProjectTask,
    ProjectTaskSortDirection,
    ProjectTaskSortMode,
  } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { cn } from "$lib/utils";
  import PriorityFlagIcon from "./PriorityFlagIcon.svelte";
  import ProjectStatusBadge from "./ProjectStatusBadge.svelte";

  let {
    tasks,
    statuses,
    priorities,
    selectedTaskIds,
    taskSortMode,
    taskSortDirection,
    onOpenTask,
    onToggleTaskSelection,
  }: {
    tasks: ProjectTask[];
    statuses: ProjectStatus[];
    priorities: ProjectPriorityConfig[];
    selectedTaskIds: string[];
    taskSortMode: ProjectTaskSortMode;
    taskSortDirection: ProjectTaskSortDirection;
    onOpenTask: (task: ProjectTask) => void;
    onToggleTaskSelection: (task: ProjectTask) => void;
  } = $props();

  const projects = getProjects();
  const theme = getTheme();
  const { t } = getLocalization();

  const PROJECT_KANBAN_DRAG_MIME = "application/x-ganbaru-project-task";

  let kanbanDraggingTaskId = $state<string | null>(null);
  let kanbanDragOverStatusId = $state<string | null>(null);
  let kanbanDragOverTaskId = $state<string | null>(null);
  let kanbanDragOverPosition = $state<ProjectKanbanDropPosition | "column" | null>(null);
  let kanbanDropPendingTaskId = $state<string | null>(null);

  const selectedTaskIdSet = $derived.by(() => new Set(selectedTaskIds));

  function tasksForStatus(status: ProjectStatus): ProjectTask[] {
    const statusTasks = tasks.filter((task) => task.statusId === status.id && !task.parentTaskId);
    if (taskSortMode !== "manual") return statusTasks;
    return [...statusTasks].sort((a, b) =>
      taskSortDirection === "asc" ? manualStatusCompare(a, b) : manualStatusCompare(b, a)
    );
  }

  function kanbanOrderTasksForStatus(status: ProjectStatus): ProjectTask[] {
    if (taskSortMode === "manual") return tasksForStatus(status);
    return projects.topLevelTasksForStatus(status.projectId, status.id);
  }

  function resetKanbanDragTarget(): void {
    kanbanDragOverStatusId = null;
    kanbanDragOverTaskId = null;
    kanbanDragOverPosition = null;
  }

  function kanbanDragTaskId(event: DragEvent): string | null {
    return event.dataTransfer?.getData(PROJECT_KANBAN_DRAG_MIME) || kanbanDraggingTaskId;
  }

  function taskById(taskId: string): ProjectTask | undefined {
    return tasks.find((task) => task.id === taskId);
  }

  function canDropKanbanTask(task: ProjectTask | undefined, status: ProjectStatus): task is ProjectTask {
    return !!task
      && !task.archivedAt
      && !task.parentTaskId
      && task.projectId === status.projectId;
  }

  function handleKanbanTaskDragStart(event: DragEvent, task: ProjectTask): void {
    if (task.archivedAt || task.parentTaskId) {
      event.preventDefault();
      return;
    }
    kanbanDraggingTaskId = task.id;
    event.dataTransfer?.setData(PROJECT_KANBAN_DRAG_MIME, task.id);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function handleKanbanTaskDragEnd(): void {
    kanbanDraggingTaskId = null;
    kanbanDropPendingTaskId = null;
    resetKanbanDragTarget();
  }

  function kanbanCardDropPosition(event: DragEvent): ProjectKanbanDropPosition {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return event.clientY >= rect.top + rect.height / 2 ? "after" : "before";
  }

  function handleKanbanCardDragOver(event: DragEvent, status: ProjectStatus, task: ProjectTask): void {
    const dragged = taskById(kanbanDragTaskId(event) ?? "");
    if (!canDropKanbanTask(dragged, status) || dragged.id === task.id) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    kanbanDragOverStatusId = status.id;
    kanbanDragOverTaskId = task.id;
    kanbanDragOverPosition = kanbanCardDropPosition(event);
  }

  function handleKanbanColumnDragOver(event: DragEvent, status: ProjectStatus): void {
    const dragged = taskById(kanbanDragTaskId(event) ?? "");
    if (!canDropKanbanTask(dragged, status)) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    kanbanDragOverStatusId = status.id;
    kanbanDragOverTaskId = null;
    kanbanDragOverPosition = "column";
  }

  async function dropKanbanTask(
    event: DragEvent,
    status: ProjectStatus,
    targetTask?: ProjectTask,
    position?: ProjectKanbanDropPosition,
  ): Promise<void> {
    event.preventDefault();
    event.stopPropagation();
    const dragged = taskById(kanbanDragTaskId(event) ?? "");
    if (!canDropKanbanTask(dragged, status) || dragged.id === targetTask?.id) {
      resetKanbanDragTarget();
      return;
    }

    const orderedTasks = kanbanOrderTasksForStatus(status);
    const nextStatusSortOrder = projectKanbanDropSortOrder({
      orderedTasks,
      draggedTaskId: dragged.id,
      overTaskId: targetTask?.id,
      position,
      sortDirection: taskSortDirection,
    });

    if (dragged.statusId === status.id && dragged.statusSortOrder === nextStatusSortOrder) {
      resetKanbanDragTarget();
      return;
    }

    kanbanDropPendingTaskId = dragged.id;
    resetKanbanDragTarget();
    try {
      await projects.updateTask(dragged, {
        statusId: status.id,
        statusSortOrder: nextStatusSortOrder,
      });
    } finally {
      kanbanDropPendingTaskId = null;
      kanbanDraggingTaskId = null;
    }
  }

  function kanbanDropMarkerVisible(
    status: ProjectStatus,
    task: ProjectTask,
    position: ProjectKanbanDropPosition,
  ): boolean {
    return kanbanDragOverStatusId === status.id
      && kanbanDragOverTaskId === task.id
      && kanbanDragOverPosition === position;
  }

  function taskSelected(task: ProjectTask): boolean {
    return selectedTaskIdSet.has(task.id);
  }

  function blockedByDependencies(task: ProjectTask) {
    return projects.dependenciesBlockingTask(task.id);
  }

  function blocksDependencies(task: ProjectTask) {
    return projects.dependenciesBlockedByTask(task.id);
  }

  function adjacentStatus(task: ProjectTask, direction: -1 | 1): ProjectStatus | undefined {
    const index = statuses.findIndex((status) => status.id === task.statusId);
    if (index < 0) return undefined;
    return statuses[index + direction];
  }

  function adjacentTaskInStatus(task: ProjectTask, direction: -1 | 1): ProjectTask | undefined {
    const ordered = projects.topLevelTasksForStatus(task.projectId, task.statusId);
    const index = ordered.findIndex((entry) => entry.id === task.id);
    if (index < 0) return undefined;
    return ordered[index + direction];
  }

  async function moveTaskToStatus(task: ProjectTask, status: ProjectStatus | undefined): Promise<void> {
    if (!status || status.id === task.statusId) return;
    await projects.setTaskStatus(task, status.id);
  }

  async function moveTaskWithinStatus(task: ProjectTask, direction: -1 | 1): Promise<void> {
    if (taskSortMode !== "manual") return;
    await projects.moveTaskInStatus(task, direction);
  }
</script>

<div class="flex min-h-full gap-3 overflow-x-auto p-3">
  {#each statuses as status (status.id)}
    {@const statusTasks = tasksForStatus(status)}
    <section
      class={cn(
        "flex w-64 shrink-0 flex-col gap-2 rounded-lg border border-transparent p-1",
        kanbanDragOverStatusId === status.id && "border-primary/40 bg-primary/5",
      )}
      role="list"
      aria-label={status.name}
      ondragover={(event) => handleKanbanColumnDragOver(event, status)}
      ondrop={(event) => { void dropKanbanTask(event, status); }}
    >
      <div class="min-w-0 px-1 py-1">
        <ProjectStatusBadge
          {status}
          theme={theme.current}
          label={`${status.name} (${statusTasks.length})`}
          class="text-[0.8rem]"
        />
      </div>
      <div class="flex flex-col gap-2">
        {#each statusTasks as task (task.id)}
          {@const previousStatus = adjacentStatus(task, -1)}
          {@const nextStatus = adjacentStatus(task, 1)}
          {@const previousStatusTask = adjacentTaskInStatus(task, -1)}
          {@const nextStatusTask = adjacentTaskInStatus(task, 1)}
          {@const blockedByCount = blockedByDependencies(task).length}
          {@const blocksCount = blocksDependencies(task).length}
          {#if kanbanDropMarkerVisible(status, task, "before")}
            <div class="h-1 rounded-full bg-primary"></div>
          {/if}
          <article
            class={cn(
              "rounded-md border border-border bg-card p-2",
              task.archivedAt && "opacity-70",
              kanbanDraggingTaskId === task.id && "opacity-50",
              kanbanDropPendingTaskId === task.id && "opacity-60",
            )}
            ondragover={(event) => handleKanbanCardDragOver(event, status, task)}
            ondrop={(event) => { void dropKanbanTask(event, status, task, kanbanCardDropPosition(event)); }}
          >
            <div class="grid grid-cols-[auto_auto_minmax(0,1fr)] gap-2">
              <button
                type="button"
                class="mt-0.5 flex h-5 w-5 shrink-0 cursor-grab items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground active:cursor-grabbing disabled:cursor-not-allowed disabled:opacity-40"
                draggable={!task.archivedAt && kanbanDropPendingTaskId === null}
                disabled={Boolean(task.archivedAt) || kanbanDropPendingTaskId !== null}
                aria-label={t("projects.actions.dragTask", task.title)}
                title={t("projects.actions.dragTask", task.title)}
                ondragstart={(event) => handleKanbanTaskDragStart(event, task)}
                ondragend={handleKanbanTaskDragEnd}
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
                onclick={() => onToggleTaskSelection(task)}
              >
                {#if taskSelected(task)}
                  <Check size={13} strokeWidth={2} />
                {/if}
              </button>
              <button
                type="button"
                class="min-w-0 text-left hover:text-primary"
                onclick={() => onOpenTask(task)}
              >
                <span class="block truncate text-[0.866667rem]">{task.title}</span>
                <span class="mt-1 flex items-center gap-1 text-[0.733333rem] text-muted-foreground">
                  <PriorityFlagIcon
                    color={projectPriorityDisplayColor(task.priority, priorities)}
                    theme={theme.current}
                    size={12}
                    class="shrink-0"
                  />
                  <span>{projectPriorityDisplayLabel(task.priority, priorities, t)}</span>
                  {#if task.dueDate}
                    <span>/</span>
                    <span>{task.dueDate}</span>
                  {/if}
                </span>
                {#if task.archivedAt}
                  <span class={cn("mt-1 inline-flex rounded border px-1.5 py-0.5 text-[0.733333rem]", projectTaskArchivedBadgeClass(task))}>
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
                onclick={() => onOpenTask(task)}
              >
                <span class="block truncate">{t("projects.kanban.openDetails")}</span>
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
          {#if kanbanDropMarkerVisible(status, task, "after")}
            <div class="h-1 rounded-full bg-primary"></div>
          {/if}
        {/each}
        {#if kanbanDragOverStatusId === status.id && kanbanDragOverPosition === "column"}
          <div class="h-1 rounded-full bg-primary"></div>
        {/if}
        {#if statusTasks.length === 0}
          <div class="rounded-md border border-dashed border-border px-2 py-3 text-[0.8rem] text-muted-foreground">
            {t("projects.kanban.emptyColumn")}
          </div>
        {/if}
      </div>
    </section>
  {/each}
</div>
