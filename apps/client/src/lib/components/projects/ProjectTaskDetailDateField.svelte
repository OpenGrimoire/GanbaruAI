<script lang="ts">
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import X from "@lucide/svelte/icons/x";
  import MiniDatePicker from "$lib/components/calendar/MiniDatePicker.svelte";
  import { cn } from "$lib/utils";

  type MiniDatePickerHighlightMode = "day" | "week" | "workweek" | "none";

  let {
    label = null,
    value,
    noDateLabel,
    clearLabel,
    pickerOpen,
    selectedDate,
    rangeStartDate = undefined,
    rangeEndDate = undefined,
    highlightToday = undefined,
    highlightMode = undefined,
    onToggle,
    onClear,
    onSelect,
    onCancel,
  }: {
    label?: string | null;
    value: string;
    noDateLabel: string;
    clearLabel: string;
    pickerOpen: boolean;
    selectedDate: string;
    rangeStartDate?: string | undefined;
    rangeEndDate?: string | undefined;
    highlightToday?: boolean | undefined;
    highlightMode?: MiniDatePickerHighlightMode | undefined;
    onToggle: () => void;
    onClear: () => void;
    onSelect: (date: string) => void;
    onCancel: () => void;
  } = $props();
</script>

<div class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
  {#if label}
    <span>{label}</span>
  {/if}
  <div class="flex gap-1">
    <button
      type="button"
      class={cn(
        "flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded-md border border-border bg-card px-2 text-left text-[0.8rem] text-foreground hover:bg-accent",
        !value && "text-muted-foreground",
      )}
      onclick={onToggle}
    >
      <CalendarDays size={14} strokeWidth={1.75} class="shrink-0" />
      <span class="truncate">{value || noDateLabel}</span>
    </button>
    {#if value}
      <button
        type="button"
        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground"
        aria-label={clearLabel}
        onclick={onClear}
      >
        <X size={13} strokeWidth={1.75} />
      </button>
    {/if}
  </div>
  {#if pickerOpen}
    <div class="w-fit rounded-lg border border-border bg-card p-2">
      <MiniDatePicker
        {selectedDate}
        {rangeStartDate}
        {rangeEndDate}
        small
        {highlightToday}
        {highlightMode}
        activeHighlight="primary"
        onselect={onSelect}
        oncancel={onCancel}
      />
    </div>
  {/if}
</div>
