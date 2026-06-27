<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Clock from "@lucide/svelte/icons/clock";
  import Eraser from "@lucide/svelte/icons/eraser";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import MiniDatePicker from "$lib/components/calendar/MiniDatePicker.svelte";
  import TimePicker from "$lib/components/calendar/TimePicker.svelte";
  import { formatTimeLabel } from "$lib/components/calendar/utils";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectPersonInitials,
    projectPriorityBadgeClass,
    projectPriorityLabel,
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
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { cn } from "$lib/utils";
  import ProjectStatusBadge from "./ProjectStatusBadge.svelte";

  let {
    column,
    task,
    status,
    statuses,
    statusMenuOpen,
    priorityMenuOpen,
    startDateMenuOpen,
    dueDateMenuOpen,
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
  }: {
    column: ProjectTaskListColumn;
    task: ProjectTask;
    status: ProjectStatus | undefined;
    statuses: ProjectStatus[];
    statusMenuOpen: boolean;
    priorityMenuOpen: boolean;
    startDateMenuOpen: boolean;
    dueDateMenuOpen: boolean;
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
  } = $props();

  const { t } = getLocalization();
  const theme = getTheme();
  const preferences = getPreferences();
  const FLOATING_PANEL_GAP = 6;
  const FLOATING_PANEL_MARGIN = 8;
  const DATE_PICKER_PANEL_WIDTH = 240;
  const DATE_PICKER_PANEL_MIN_HEIGHT = 180;
  const TIME_PICKER_PANEL_WIDTH = 160;
  const TIME_PICKER_PANEL_HEIGHT = 200;
  const TIME_PICKER_PANEL_MIN_HEIGHT = 96;
  const DEFAULT_TIME_PICKER_ANCHOR = "12:00";

  let timePickerOpen = $state(false);
  let dateTriggerEl: HTMLButtonElement | undefined = $state();
  let timeTriggerEl: HTMLButtonElement | undefined = $state();

  const todayDate = $derived.by(() => {
    const now = new Date();
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
  });
  const ownDateMenuOpen = $derived(column === "start" ? startDateMenuOpen : column === "due" ? dueDateMenuOpen : false);

  $effect(() => {
    if (!ownDateMenuOpen) {
      timePickerOpen = false;
    }
  });

  function dateButtonText(dateValue: string | undefined, timeValue: string | undefined, emptyDateLabel: string): string {
    if (!dateValue) return emptyDateLabel;
    if (!timeValue) return dateValue;
    return `${dateValue} ${formatTimeLabel(timeValue, preferences.calendarTimeFormat)}`;
  }

  function positionPanel(
    node: HTMLElement,
    anchor: HTMLElement | undefined,
    options: {
      fallbackWidth: number;
      minHeight: number;
      preferredHeight?: number;
      applyNodeMaxHeight?: boolean;
      maxHeightProperty?: string;
    },
  ): void {
    if (!anchor) return;
    const triggerRect = anchor.getBoundingClientRect();
    const panelWidth = node.offsetWidth || options.fallbackWidth;
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const usableViewportHeight = Math.max(0, viewportHeight - FLOATING_PANEL_MARGIN * 2);
    const preferredHeight = Math.min(
      options.preferredHeight ?? Math.max(node.scrollHeight, node.offsetHeight, options.minHeight),
      usableViewportHeight,
    );
    const minimumHeight = Math.min(options.minHeight, usableViewportHeight);
    const left = Math.max(
      FLOATING_PANEL_MARGIN,
      Math.min(
        triggerRect.left,
        viewportWidth - panelWidth - FLOATING_PANEL_MARGIN,
      ),
    );
    const belowTop = triggerRect.bottom + FLOATING_PANEL_GAP;
    const aboveBottom = triggerRect.top - FLOATING_PANEL_GAP;
    const belowSpace = Math.max(0, viewportHeight - FLOATING_PANEL_MARGIN - belowTop);
    const aboveSpace = Math.max(0, aboveBottom - FLOATING_PANEL_MARGIN);
    const preferBelow = belowSpace >= preferredHeight || belowSpace >= aboveSpace;
    const availableHeight = preferBelow ? belowSpace : aboveSpace;
    const maxHeight = Math.max(
      minimumHeight,
      Math.min(preferredHeight, availableHeight || usableViewportHeight),
    );
    const unclampedTop = preferBelow
      ? belowTop
      : aboveBottom - maxHeight;
    const top = Math.max(
      FLOATING_PANEL_MARGIN,
      Math.min(
        unclampedTop,
        viewportHeight - FLOATING_PANEL_MARGIN - maxHeight,
      ),
    );

    node.style.left = `${Math.round(left)}px`;
    node.style.top = `${Math.round(top)}px`;
    if (options.applyNodeMaxHeight ?? true) {
      node.style.maxHeight = `${Math.round(maxHeight)}px`;
      node.style.overflowY = "auto";
    }
    if (options.maxHeightProperty) {
      node.style.setProperty(options.maxHeightProperty, `${Math.round(maxHeight)}px`);
    }
  }

  function positionDatePickerPanel(node: HTMLElement) {
    function updatePosition(): void {
      positionPanel(node, dateTriggerEl, {
        fallbackWidth: DATE_PICKER_PANEL_WIDTH,
        minHeight: DATE_PICKER_PANEL_MIN_HEIGHT,
      });
    }

    const frame = requestAnimationFrame(updatePosition);
    const resizeObserver = new ResizeObserver(updatePosition);
    resizeObserver.observe(node);
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);

    return {
      destroy() {
        cancelAnimationFrame(frame);
        resizeObserver.disconnect();
        window.removeEventListener("resize", updatePosition);
        window.removeEventListener("scroll", updatePosition, true);
      },
    };
  }

  function positionTimePickerPanel(node: HTMLElement) {
    function updatePosition(): void {
      positionPanel(node, timeTriggerEl, {
        fallbackWidth: TIME_PICKER_PANEL_WIDTH,
        minHeight: TIME_PICKER_PANEL_MIN_HEIGHT,
        preferredHeight: TIME_PICKER_PANEL_HEIGHT,
        applyNodeMaxHeight: false,
        maxHeightProperty: "--project-list-time-picker-max-height",
      });
    }

    const frame = requestAnimationFrame(updatePosition);
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);

    return {
      destroy() {
        cancelAnimationFrame(frame);
        window.removeEventListener("resize", updatePosition);
        window.removeEventListener("scroll", updatePosition, true);
      },
    };
  }
</script>

<div
  class="project-list-cell-frame relative flex min-h-11 min-w-0 self-stretch items-center rounded-md px-2 py-1"
  data-list-status-menu-root={column === "status" ? "true" : undefined}
  data-list-priority-menu-root={column === "priority" ? "true" : undefined}
  data-list-date-menu-root={column === "start" || column === "due" ? "true" : undefined}
>
  {#if column === "status"}
    <button
      type="button"
      class="absolute inset-0 z-0 cursor-pointer rounded-md disabled:cursor-not-allowed"
      disabled={Boolean(task.archivedAt)}
      aria-label={status?.name ?? t("projects.list.status")}
      data-app-tooltip-disabled="true"
      aria-haspopup="menu"
      aria-expanded={statusMenuOpen}
      onclick={onToggleStatusMenu}
    ></button>
    <div class="pointer-events-none relative z-10 min-w-0 max-w-full">
      <ProjectStatusBadge
        {status}
        theme={theme.current}
        label={status?.name ?? t("projects.list.status")}
        class="text-[0.733333rem]"
      />
    </div>
      {#if statusMenuOpen}
        <div
          class="absolute left-0 top-full z-30 mt-1 w-44 rounded-lg border border-border bg-popover p-1 text-[0.8rem] text-popover-foreground shadow-sm"
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
              <ProjectStatusBadge
                status={nextStatus}
                theme={theme.current}
                label={nextStatus.name}
                class="text-[0.733333rem]"
              />
              {#if task.statusId === nextStatus.id}
                <Check size={13} strokeWidth={2} class="shrink-0 text-muted-foreground" />
              {/if}
            </button>
          {/each}
        </div>
      {/if}
  {:else if column === "priority"}
    <button
      type="button"
      class="absolute inset-0 z-0 cursor-pointer rounded-md disabled:cursor-not-allowed"
      disabled={Boolean(task.archivedAt)}
      aria-label={projectPriorityLabel(task.priority, t)}
      data-app-tooltip-disabled="true"
      aria-haspopup="menu"
      aria-expanded={priorityMenuOpen}
      onclick={onTogglePriorityMenu}
    ></button>
    <div class="pointer-events-none relative z-10 min-w-0 max-w-full">
      <span class="block truncate text-[0.8rem] text-foreground">
        {projectPriorityLabel(task.priority, t)}
      </span>
    </div>
      {#if priorityMenuOpen}
        <div
          class="absolute left-0 top-full z-30 mt-1 w-44 rounded-lg border border-border bg-popover p-1 text-[0.8rem] text-popover-foreground shadow-sm"
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
  {:else if column === "estimate"}
    {#if task.estimateMinutes !== undefined}
      <span class="truncate rounded border border-border bg-background px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground">
        {estimateLabel(task.estimateMinutes)}
      </span>
    {/if}
  {:else if column === "assignee" || column === "reviewer"}
    {@const personName = t("projects.people.you")}
    <span
      class="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-foreground text-[0.733333rem] font-semibold leading-none text-background"
      aria-label={personName}
      title={personName}
    >
      {projectPersonInitials(personName)}
    </span>
  {:else if column === "start" || column === "due"}
    {@const dateValue = column === "start" ? task.startDate : task.dueDate}
    {@const timeValue = column === "start" ? task.startTime : task.dueTime}
    {@const dateMenuOpen = column === "start" ? startDateMenuOpen : dueDateMenuOpen}
    {@const dateLabel = column === "start" ? t("projects.columns.start") : t("projects.columns.due")}
    {@const emptyDateLabel = column === "start" ? t("projects.detail.noDate") : t("projects.filters.noDueDate")}
    {@const datePickerAnchor = dateValue || (column === "start" ? task.dueDate : task.startDate) || todayDate}
      <button
        bind:this={dateTriggerEl}
        type="button"
        class="absolute inset-0 z-0 cursor-pointer rounded-md disabled:cursor-not-allowed"
        disabled={Boolean(task.archivedAt)}
        aria-label={dateButtonText(dateValue, timeValue, emptyDateLabel)}
        data-app-tooltip-disabled="true"
        aria-haspopup="dialog"
        aria-expanded={dateMenuOpen}
        onclick={() => {
          if (column === "start") onToggleStartDateMenu();
          else onToggleDueDateMenu();
        }}
      ></button>
      <span class={cn("pointer-events-none relative z-10 block min-w-0 truncate text-[0.8rem]", dateValue ? "text-foreground" : "text-muted-foreground")}>
        {dateButtonText(dateValue, timeValue, emptyDateLabel)}
      </span>
      {#if dateMenuOpen}
        <div
          class="fixed z-30 w-60 rounded-lg border border-border bg-popover p-2 text-popover-foreground shadow-sm"
          role="dialog"
          aria-label={dateLabel}
          use:positionDatePickerPanel
        >
          <MiniDatePicker
            selectedDate={datePickerAnchor}
            rangeStartDate={task.startDate}
            rangeEndDate={task.dueDate}
            small
            highlightToday={false}
            activeHighlight="primary"
            onselect={(dateStr) => {
              if (column === "start") onSetStartDate(dateStr);
              else onSetDueDate(dateStr);
            }}
            oncancel={() => {
              if (column === "start") onCloseStartDateMenu();
              else onCloseDueDateMenu();
            }}
          />
          {#if dateValue}
            <div class="mt-2 border-t border-border/70 pt-2">
              <div class="flex items-center gap-1">
                <button
                  bind:this={timeTriggerEl}
                  type="button"
                  class="flex min-h-8 min-w-0 flex-1 cursor-pointer items-center gap-2 rounded-md px-2 text-left text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                  aria-haspopup="dialog"
                  aria-expanded={timePickerOpen}
                  onclick={() => {
                    timePickerOpen = !timePickerOpen;
                  }}
                >
                  <Clock size={13} strokeWidth={1.75} class="shrink-0" />
                  <span class="truncate">
                    {timeValue ? formatTimeLabel(timeValue, preferences.calendarTimeFormat) : t("projects.columns.selectHour")}
                  </span>
                </button>
                {#if timeValue}
                  <button
                    type="button"
                    class="flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
                    aria-label={t("projects.columns.clearHour")}
                    data-app-tooltip-disabled="true"
                    onclick={() => {
                      if (column === "start") onClearStartTime();
                      else onClearDueTime();
                      timePickerOpen = false;
                    }}
                  >
                    <Eraser size={13} strokeWidth={1.75} />
                  </button>
                {/if}
                <button
                  type="button"
                  class="flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
                  aria-label={t("projects.detail.clearDate", dateLabel)}
                  data-app-tooltip-disabled="true"
                  onclick={() => {
                    if (column === "start") onClearStartDate();
                    else onClearDueDate();
                    timePickerOpen = false;
                  }}
                >
                  <Trash2 size={13} strokeWidth={1.75} />
                </button>
              </div>
              {#if timePickerOpen}
                <div
                  class="project-list-time-panel fixed z-40 w-40 overflow-hidden rounded-md border border-border bg-popover text-popover-foreground shadow-sm"
                  role="dialog"
                  aria-label={`${dateLabel} ${t("projects.columns.selectHour")}`}
                  use:positionTimePickerPanel
                >
                  <TimePicker
                    currentTime={timeValue ?? ""}
                    activeTime={timeValue ?? DEFAULT_TIME_PICKER_ANCHOR}
                    scrollTime={timeValue ?? DEFAULT_TIME_PICKER_ANCHOR}
                    emphasizedTime={timeValue ? undefined : null}
                    focusOnOpen
                    onselect={(time) => {
                      if (column === "start") onSetStartTime(time);
                      else onSetDueTime(time);
                      timePickerOpen = false;
                    }}
                    oncancel={() => {
                      timePickerOpen = false;
                    }}
                  />
                </div>
              {/if}
            </div>
          {/if}
        </div>
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
        <span class="min-w-0 truncate text-[0.8rem] text-muted-foreground">
          {customValue}
        </span>
      {/if}
    {/if}
  {/if}
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
  }

  .project-list-cell-frame:hover::before,
  .project-list-cell-frame:focus-within::before {
    border-color: color-mix(in srgb, var(--foreground) 25%, transparent);
  }

  :global(.project-list-time-panel .time-picker-scroll) {
    max-height: var(--project-list-time-picker-max-height, 12.5rem);
  }
</style>
