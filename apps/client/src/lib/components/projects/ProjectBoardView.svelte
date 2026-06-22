<script lang="ts">
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import Check from "@lucide/svelte/icons/check";
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectBoardDropSortOrder,
    type ProjectBoardDropPosition,
  } from "$lib/projects/board-drag";
  import {
    projectPriorityLabel,
    projectStatusBadgeClass,
    projectTaskArchivedBadgeClass,
  } from "$lib/projects/project-display";
  import { manualStatusCompare } from "$lib/projects/task-view";
  import type {
    ProjectStatus,
    ProjectTask,
    ProjectTaskSortDirection,
    ProjectTaskSortMode,
  } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { cn } from "$lib/utils";

  let {
    tasks,
    statuses,
    selectedTaskIds,
    taskSortMode,
    taskSortDirection,
    onOpenTask,
    onToggleTaskSelection,
  }: {
    tasks: ProjectTask[];
    statuses: ProjectStatus[];
    selectedTaskIds: string[];
    taskSortMode: ProjectTaskSortMode;
    taskSortDirection: ProjectTaskSortDirection;
    onOpenTask: (task: ProjectTask) => void;
    onToggleTaskSelection: (task: ProjectTask) => void;
  } = $props();

  const projects = getProjects();
  const { t } = getLocalization();

  const PROJECT_BOARD_DRAG_MIME = "application/x-ganbaru-project-task";

  let boardDraggingTaskId = $state<string | null>(null);
  let boardDragOverStatusId = $state<string | null>(null);
  let boardDragOverTaskId = $state<string | null>(null);
  let boardDragOverPosition = $state<ProjectBoardDropPosition | "column" | null>(null);
  let boardDropPendingTaskId = $state<string | null>(null);

  const selectedTaskIdSet = $derived.by(() => new Set(selectedTaskIds));

  function tasksForStatus(status: ProjectStatus): ProjectTask[] {
    const statusTasks = tasks.filter((task) => task.statusId === status.id && !task.parentTaskId);
    if (taskSortMode !== "manual") return statusTasks;
    return [...statusTasks].sort((a, b) =>
      taskSortDirection === "asc" ? manualStatusCompare(a, b) : manualStatusCompare(b, a)
    );
  }

  function boardOrderTasksForStatus(status: ProjectStatus): ProjectTask[] {
    if (taskSortMode === "manual") return tasksForStatus(status);
    return projects.topLevelTasksForStatus(status.projectId, status.id);
  }

  function resetBoardDragTarget(): void {
    boardDragOverStatusId = null;
    boardDragOverTaskId = null;
    boardDragOverPosition = null;
  }

  function boardDragTaskId(event: DragEvent): string | null {
    return event.dataTransfer?.getData(PROJECT_BOARD_DRAG_MIME) || boardDraggingTaskId;
  }

  function taskById(taskId: string): ProjectTask | undefined {
    return tasks.find((task) => task.id === taskId);
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
        boardDragOverStatusId === status.id && "border-primary/40 bg-primary/5",
      )}
      role="list"
      aria-label={status.name}
      ondragover={(event) => handleBoardColumnDragOver(event, status)}
      ondrop={(event) => { void dropBoardTask(event, status); }}
    >
      <div class={cn("rounded-md border px-2 py-1.5 text-[0.8rem] font-semibold", projectStatusBadgeClass(status))}>
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
                  {projectPriorityLabel(task.priority, t)}
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
