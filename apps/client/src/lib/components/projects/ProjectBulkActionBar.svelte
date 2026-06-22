<script lang="ts">
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import Check from "@lucide/svelte/icons/check";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import X from "@lucide/svelte/icons/x";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectPriorityBadgeClass,
    projectPriorityLabel,
  } from "$lib/projects/project-display";
  import { PROJECT_PRIORITIES, type ProjectPriority } from "$lib/projects/types";
  import { cn } from "$lib/utils";

  let {
    selectedTaskCount,
    selectableTaskCount,
    selectedActiveTaskCount,
    selectedArchivedTaskCount,
    bulkSchedulableCount,
    bulkTaskActionPending,
    bulkScheduleOpen,
    bulkScheduleDate,
    bulkScheduleStartTime,
    bulkScheduleDurationMinutes,
    bulkTaskError,
    canMarkDone,
    canReopen,
    onSelectFiltered,
    onMarkDone,
    onReopen,
    onToggleBulkSchedule,
    onSetPriority,
    onArchive,
    onRestore,
    onClear,
    onBulkScheduleDateChange,
    onBulkScheduleStartTimeChange,
    onBulkScheduleDurationMinutesChange,
    onBulkScheduleSubmit,
    onCloseBulkSchedule,
  }: {
    selectedTaskCount: number;
    selectableTaskCount: number;
    selectedActiveTaskCount: number;
    selectedArchivedTaskCount: number;
    bulkSchedulableCount: number;
    bulkTaskActionPending: boolean;
    bulkScheduleOpen: boolean;
    bulkScheduleDate: string;
    bulkScheduleStartTime: string;
    bulkScheduleDurationMinutes: number;
    bulkTaskError: string | null;
    canMarkDone: boolean;
    canReopen: boolean;
    onSelectFiltered: () => void;
    onMarkDone: () => void;
    onReopen: () => void;
    onToggleBulkSchedule: () => void;
    onSetPriority: (priority: ProjectPriority) => void;
    onArchive: () => void;
    onRestore: () => void;
    onClear: () => void;
    onBulkScheduleDateChange: (value: string) => void;
    onBulkScheduleStartTimeChange: (value: string) => void;
    onBulkScheduleDurationMinutesChange: (value: number) => void;
    onBulkScheduleSubmit: () => void;
    onCloseBulkSchedule: () => void;
  } = $props();

  const { t } = getLocalization();
</script>

<div class="mx-3 my-2 flex min-w-0 flex-wrap items-center gap-1 rounded-md border border-border bg-card px-2 py-1 text-[0.766667rem]">
  <span class="mr-1 shrink-0 font-medium">{t("projects.bulk.selected", selectedTaskCount)}</span>
  <button
    type="button"
    class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
    disabled={bulkTaskActionPending || selectableTaskCount === selectedTaskCount}
    onclick={onSelectFiltered}
  >
    <Check size={13} strokeWidth={1.75} />
    <span>{t("projects.bulk.selectFiltered", selectableTaskCount)}</span>
  </button>
  <button
    type="button"
    class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
    disabled={bulkTaskActionPending || !canMarkDone}
    onclick={onMarkDone}
  >
    <Check size={13} strokeWidth={1.75} />
    <span>{t("projects.bulk.markDone")}</span>
  </button>
  <button
    type="button"
    class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
    disabled={bulkTaskActionPending || !canReopen}
    onclick={onReopen}
  >
    <RotateCcw size={13} strokeWidth={1.75} />
    <span>{t("projects.bulk.reopen")}</span>
  </button>
  <button
    type="button"
    class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
    disabled={bulkTaskActionPending || bulkSchedulableCount === 0}
    onclick={onToggleBulkSchedule}
  >
    <CalendarDays size={13} strokeWidth={1.75} />
    <span>{t("projects.bulk.schedule")}</span>
  </button>
  <div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-md bg-muted/60 p-0.5">
    {#each PROJECT_PRIORITIES as priority}
      <button
        type="button"
        class={cn("h-6 shrink-0 rounded px-2 font-medium", projectPriorityBadgeClass(priority))}
        disabled={bulkTaskActionPending}
        onclick={() => onSetPriority(priority)}
      >
        {projectPriorityLabel(priority, t)}
      </button>
    {/each}
  </div>
  {#if selectedActiveTaskCount > 0}
    <button
      type="button"
      class="flex h-7 shrink-0 items-center gap-1 rounded-md border border-destructive/30 bg-destructive/10 px-2 font-medium text-destructive hover:bg-destructive/15 disabled:cursor-not-allowed disabled:opacity-50"
      disabled={bulkTaskActionPending}
      onclick={onArchive}
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
      onclick={onRestore}
    >
      <ArchiveRestore size={13} strokeWidth={1.75} />
      <span>{t("projects.bulk.restore")}</span>
    </button>
  {/if}
  <button
    type="button"
    class="ml-auto flex h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
    onclick={onClear}
  >
    <X size={13} strokeWidth={1.75} />
    <span>{t("projects.bulk.clear")}</span>
  </button>
  {#if bulkScheduleOpen}
    <form
      class="basis-full grid gap-2 border-t border-border/70 pt-2 min-[860px]:grid-cols-[minmax(0,1fr)_8rem_7rem_6rem_auto_auto]"
      onsubmit={(event) => {
        event.preventDefault();
        onBulkScheduleSubmit();
      }}
    >
      <div class="self-end text-[0.733333rem] text-muted-foreground">
        {t("projects.bulk.scheduleHint", bulkSchedulableCount)}
      </div>
      <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
        <span>{t("projects.schedule.date")}</span>
        <input
          value={bulkScheduleDate}
          placeholder="YYYY-MM-DD"
          class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
          oninput={(event) => onBulkScheduleDateChange(event.currentTarget.value)}
        />
      </label>
      <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
        <span>{t("projects.schedule.start")}</span>
        <input
          value={bulkScheduleStartTime}
          placeholder="HH:MM"
          class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
          oninput={(event) => onBulkScheduleStartTimeChange(event.currentTarget.value)}
        />
      </label>
      <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
        <span>{t("projects.schedule.duration")}</span>
        <input
          type="number"
          min="1"
          step="5"
          value={bulkScheduleDurationMinutes}
          class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
          oninput={(event) => onBulkScheduleDurationMinutesChange(event.currentTarget.valueAsNumber)}
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
        onclick={onCloseBulkSchedule}
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
