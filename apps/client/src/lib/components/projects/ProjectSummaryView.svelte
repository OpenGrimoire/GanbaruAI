<script lang="ts">
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type {
    ProjectStatus,
    ProjectTask,
    ProjectTaskChangeEvent,
  } from "$lib/projects/types";
  import {
    projectPriorityLabel,
    projectTaskHistoryEventLabel,
  } from "$lib/projects/project-display";
  import { getProjects } from "$lib/stores/projects.svelte";

  let {
    projectId,
    tasks,
    statuses,
    todayDate,
    scheduledTaskIds,
    scheduledThisWeekMinutes,
    onOpenTask,
  }: {
    projectId: string | null;
    tasks: ProjectTask[];
    statuses: ProjectStatus[];
    todayDate: string;
    scheduledTaskIds: ReadonlySet<string>;
    scheduledThisWeekMinutes: number;
    onOpenTask: (task: ProjectTask) => void;
  } = $props();

  const projects = getProjects();
  const { t } = getLocalization();

  const completedTaskCount = $derived(tasks.filter((task) => isTaskDone(task)).length);

  function taskCountForStatus(status: ProjectStatus): number {
    return tasks.filter((task) => task.statusId === status.id && !task.parentTaskId).length;
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
        && !scheduledTaskIds.has(task.id)
      )
      .sort((a, b) => (a.dueDate ?? "").localeCompare(b.dueDate ?? ""))
      .slice(0, 5);
  }

  function blockedSummaryTasks(): ProjectTask[] {
    return tasks
      .filter((task) =>
        projects.statusById(task.statusId)?.category === "blocked"
        || Boolean(task.blockerReason?.trim())
        || projects.dependenciesBlockingTask(task.id).length > 0
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
    return projects.recentTaskChangeEventsForProject(projectId, 6);
  }

  function taskById(taskId: string): ProjectTask | undefined {
    return projects.tasks.find((task) => task.id === taskId);
  }

  function historyTaskTitle(event: ProjectTaskChangeEvent): string {
    return taskById(event.taskId)?.title ?? t("projects.detail.missingHistoryTask");
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
</script>

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
        <div class="text-[1.2rem] font-semibold">{formatMinutesAsHours(scheduledThisWeekMinutes)}</div>
        <div class="text-[0.733333rem] text-muted-foreground">{t("projects.summary.scheduledThisWeek")}</div>
      </div>
    </div>
    <div class="mt-3 grid gap-1 text-[0.8rem] text-muted-foreground">
      {#each statuses as status (status.id)}
        <div class="flex items-center justify-between gap-2">
          <span>{status.name}</span>
          <span>{taskCountForStatus(status)}</span>
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
          onclick={() => onOpenTask(task)}
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
          onclick={() => onOpenTask(task)}
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
          onclick={() => onOpenTask(task)}
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
          onclick={() => onOpenTask(task)}
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
          onclick={() => onOpenTask(task)}
        >
          <span class="truncate">{task.title}</span>
          <span class="text-[0.733333rem] text-muted-foreground">{projectPriorityLabel(task.priority, t)}</span>
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
            if (historyTask) onOpenTask(historyTask);
          }}
        >
          <span class="min-w-0">
            <span class="block truncate text-[0.8rem]">{historyTaskTitle(event)}</span>
            <span class="block truncate text-[0.733333rem] text-muted-foreground">{projectTaskHistoryEventLabel(event, t)}</span>
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
          onclick={() => onOpenTask(task)}
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
