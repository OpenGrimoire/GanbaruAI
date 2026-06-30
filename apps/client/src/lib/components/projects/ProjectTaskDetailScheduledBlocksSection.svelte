<script lang="ts">
  import Plus from "@lucide/svelte/icons/plus";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { ProjectLinkableEvent, ProjectTask } from "$lib/projects/types";
  import ProjectTaskDetailDateField from "./ProjectTaskDetailDateField.svelte";

  type ActionResult = void | Promise<void>;

  let {
    task,
    events,
    candidates,
    search,
    searchPending,
    searchError,
    startDate,
    endDate,
    todayDate,
    startPickerOpen,
    endPickerOpen,
    onSearchChange,
    onToggleStartPicker,
    onToggleEndPicker,
    onClearStartDate,
    onClearEndDate,
    onSelectDate,
    onCancelDatePicker,
    onLinkEvent,
    onUnlinkEvent,
  }: {
    task: ProjectTask;
    events: ProjectLinkableEvent[];
    candidates: ProjectLinkableEvent[];
    search: string;
    searchPending: boolean;
    searchError: string | null;
    startDate: string;
    endDate: string;
    todayDate: string;
    startPickerOpen: boolean;
    endPickerOpen: boolean;
    onSearchChange: (value: string) => void;
    onToggleStartPicker: () => void;
    onToggleEndPicker: () => void;
    onClearStartDate: () => void;
    onClearEndDate: () => void;
    onSelectDate: (date: string) => void;
    onCancelDatePicker: () => void;
    onLinkEvent: (task: ProjectTask, event: ProjectLinkableEvent) => ActionResult;
    onUnlinkEvent: (task: ProjectTask, event: ProjectLinkableEvent) => ActionResult;
  } = $props();

  const { t } = getLocalization();

  function otherLinkedTaskText(event: ProjectLinkableEvent, currentTask: ProjectTask): string {
    return event.linkedTasks
      .filter((linkedTask) => linkedTask.taskId !== currentTask.id)
      .map((linkedTask) => linkedTask.title)
      .join(", ");
  }
</script>

<div class="flex items-center justify-between gap-2">
  <h2 class="text-[0.8rem] font-semibold tracking-normal">{t("projects.detail.scheduledBlocks")}</h2>
  <span class="text-[0.733333rem] text-muted-foreground">{events.length}</span>
</div>
<div class="grid gap-1">
  {#each events as event (event.id)}
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
        onclick={() => { void onUnlinkEvent(task, event); }}
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
{#if searchError}
  <div class="rounded-md border border-destructive/30 bg-destructive/10 px-2 py-2 text-[0.8rem] text-destructive">
    {searchError}
  </div>
{/if}
<div class="grid gap-1">
  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
    <span>{t("projects.detail.linkExistingBlock")}</span>
    <input
      value={search}
      placeholder={t("projects.detail.linkExistingBlockPlaceholder")}
      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
      oninput={(event) => onSearchChange(event.currentTarget.value)}
    />
  </label>
  <div class="grid gap-2 min-[760px]:grid-cols-2">
    <ProjectTaskDetailDateField
      label={t("projects.detail.eventLinkStartDate")}
      value={startDate}
      noDateLabel={t("projects.detail.noDate")}
      clearLabel={t("projects.detail.clearDate", t("projects.detail.eventLinkStartDate"))}
      pickerOpen={startPickerOpen}
      selectedDate={startDate || todayDate}
      highlightMode="none"
      onToggle={onToggleStartPicker}
      onClear={onClearStartDate}
      onSelect={onSelectDate}
      onCancel={onCancelDatePicker}
    />
    <ProjectTaskDetailDateField
      label={t("projects.detail.eventLinkEndDate")}
      value={endDate}
      noDateLabel={t("projects.detail.noDate")}
      clearLabel={t("projects.detail.clearDate", t("projects.detail.eventLinkEndDate"))}
      pickerOpen={endPickerOpen}
      selectedDate={endDate || todayDate}
      highlightMode="none"
      onToggle={onToggleEndPicker}
      onClear={onClearEndDate}
      onSelect={onSelectDate}
      onCancel={onCancelDatePicker}
    />
  </div>
  <div class="grid gap-1">
    {#if searchPending}
      <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
        {t("projects.detail.searchingBlocks")}
      </div>
    {:else}
      {#each candidates as event (event.id)}
        {@const otherLinkedTasks = otherLinkedTaskText(event, task)}
        <button
          type="button"
          class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
          onclick={() => { void onLinkEvent(task, event); }}
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
