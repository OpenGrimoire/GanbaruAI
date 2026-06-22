<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";

  let {
    scheduleDate,
    scheduleStartTime,
    scheduleDurationMinutes,
    schedulePending,
    scheduleError,
    onScheduleDateChange,
    onScheduleStartTimeChange,
    onScheduleDurationMinutesChange,
    onSubmit,
    onCancel,
  }: {
    scheduleDate: string;
    scheduleStartTime: string;
    scheduleDurationMinutes: number;
    schedulePending: boolean;
    scheduleError: string | null;
    onScheduleDateChange: (value: string) => void;
    onScheduleStartTimeChange: (value: string) => void;
    onScheduleDurationMinutesChange: (value: number) => void;
    onSubmit: () => void;
    onCancel: () => void;
  } = $props();

  const { t } = getLocalization();
</script>

<form
  class="project-list-inline-divider col-span-full mt-2 grid gap-2 pt-2 min-[720px]:grid-cols-[minmax(0,1fr)_7rem_6rem_auto_auto]"
  onsubmit={(event) => {
    event.preventDefault();
    onSubmit();
  }}
>
  <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
    <span>{t("projects.schedule.date")}</span>
    <input
      value={scheduleDate}
      placeholder="YYYY-MM-DD"
      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
      oninput={(event) => onScheduleDateChange(event.currentTarget.value)}
    />
  </label>
  <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
    <span>{t("projects.schedule.start")}</span>
    <input
      value={scheduleStartTime}
      placeholder="HH:MM"
      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
      oninput={(event) => onScheduleStartTimeChange(event.currentTarget.value)}
    />
  </label>
  <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
    <span>{t("projects.schedule.duration")}</span>
    <input
      type="number"
      min="1"
      step="5"
      value={scheduleDurationMinutes}
      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
      oninput={(event) => {
        if (Number.isFinite(event.currentTarget.valueAsNumber)) {
          onScheduleDurationMinutesChange(event.currentTarget.valueAsNumber);
        }
      }}
    />
  </label>
  <button
    type="submit"
    disabled={schedulePending}
    class="self-end cursor-pointer rounded-md bg-primary px-2 py-1.5 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
  >
    {t("projects.schedule.schedule")}
  </button>
  <button
    type="button"
    class="self-end cursor-pointer rounded-md border border-border bg-card px-2 py-1.5 text-[0.8rem] hover:bg-accent"
    onclick={onCancel}
  >
    {t("common.cancel")}
  </button>
  {#if scheduleError}
    <div class="text-[0.733333rem] text-destructive min-[720px]:col-span-5">
      {scheduleError}
    </div>
  {/if}
</form>
