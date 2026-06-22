<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectPriorityBadgeClass,
    projectPriorityLabel,
    projectStatusBadgeClass,
  } from "$lib/projects/project-display";
  import { customFieldIdFromTaskListColumn } from "$lib/projects/task-list-columns";
  import {
    PROJECT_PRIORITIES,
    type ProjectCustomField,
    type ProjectPriority,
    type ProjectStatus,
    type ProjectTask,
    type ProjectTaskListColumn,
  } from "$lib/projects/types";
  import { cn } from "$lib/utils";

  let {
    column,
    task,
    status,
    statuses,
    statusMenuOpen,
    priorityMenuOpen,
    projectCustomFields,
    scheduled,
    blockedByCount,
    blocksCount,
    estimateLabel,
    customFieldDisplayValue,
    onToggleStatusMenu,
    onSetStatus,
    onTogglePriorityMenu,
    onSetPriority,
  }: {
    column: ProjectTaskListColumn;
    task: ProjectTask;
    status: ProjectStatus | undefined;
    statuses: ProjectStatus[];
    statusMenuOpen: boolean;
    priorityMenuOpen: boolean;
    projectCustomFields: ProjectCustomField[];
    scheduled: string | null;
    blockedByCount: number;
    blocksCount: number;
    estimateLabel: (minutes: number) => string;
    customFieldDisplayValue: (task: ProjectTask, field: ProjectCustomField) => string | undefined;
    onToggleStatusMenu: () => void;
    onSetStatus: (status: ProjectStatus) => void;
    onTogglePriorityMenu: () => void;
    onSetPriority: (priority: ProjectPriority) => void;
  } = $props();

  const { t } = getLocalization();
</script>

<div class="flex min-w-0 items-center px-2">
  {#if column === "status"}
    <div class="relative max-w-full" data-list-status-menu-root="true">
      <button
        type="button"
        class={cn(
          "max-w-full cursor-pointer truncate rounded border px-1.5 py-0.5 text-[0.733333rem] disabled:cursor-not-allowed disabled:opacity-60",
          projectStatusBadgeClass(status),
        )}
        disabled={Boolean(task.archivedAt)}
        aria-haspopup="menu"
        aria-expanded={statusMenuOpen}
        onclick={onToggleStatusMenu}
      >
        {status?.name ?? t("projects.list.status")}
      </button>
      {#if statusMenuOpen}
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
              onclick={() => onSetStatus(nextStatus)}
            >
              <span class={cn("min-w-0 truncate rounded border px-1.5 py-0.5 text-[0.733333rem]", projectStatusBadgeClass(nextStatus))}>
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
    <div class="relative max-w-full" data-list-priority-menu-root="true">
      <button
        type="button"
        class={cn(
          "max-w-full cursor-pointer truncate rounded border px-1.5 py-0.5 text-[0.733333rem] disabled:cursor-not-allowed disabled:opacity-60",
          projectPriorityBadgeClass(task.priority),
        )}
        disabled={Boolean(task.archivedAt)}
        aria-haspopup="menu"
        aria-expanded={priorityMenuOpen}
        onclick={onTogglePriorityMenu}
      >
        {projectPriorityLabel(task.priority, t)}
      </button>
      {#if priorityMenuOpen}
        <div
          class="absolute left-0 top-7 z-30 w-44 rounded-lg border border-border bg-popover p-1 text-[0.8rem] text-popover-foreground shadow-sm"
          role="menu"
        >
          {#each PROJECT_PRIORITIES as priority}
            <button
              type="button"
              class="flex min-h-8 w-full cursor-pointer items-center justify-between gap-2 rounded-md px-2 text-left hover:bg-accent hover:text-foreground"
              role="menuitemradio"
              aria-checked={task.priority === priority}
              onclick={() => onSetPriority(priority)}
            >
              <span class={cn("min-w-0 truncate rounded border px-1.5 py-0.5 text-[0.733333rem]", projectPriorityBadgeClass(priority))}>
                {projectPriorityLabel(priority, t)}
              </span>
              {#if task.priority === priority}
                <Check size={13} strokeWidth={2} class="shrink-0 text-muted-foreground" />
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>
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
