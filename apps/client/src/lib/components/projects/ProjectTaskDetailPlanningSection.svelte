<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import ProjectTaskDetailDateField from "./ProjectTaskDetailDateField.svelte";

  let {
    estimateMinutes,
    startDate,
    dueDate,
    targetEndDate,
    blockerReason,
    changeReason,
    todayDate,
    startPickerOpen,
    duePickerOpen,
    targetPickerOpen,
    onEstimateMinutesChange,
    onBlockerReasonChange,
    onChangeReasonChange,
    onToggleStartPicker,
    onToggleDuePicker,
    onToggleTargetPicker,
    onClearStartDate,
    onClearDueDate,
    onClearTargetEndDate,
    onSelectDate,
    onCancelDatePicker,
  }: {
    estimateMinutes: string;
    startDate: string;
    dueDate: string;
    targetEndDate: string;
    blockerReason: string;
    changeReason: string;
    todayDate: string;
    startPickerOpen: boolean;
    duePickerOpen: boolean;
    targetPickerOpen: boolean;
    onEstimateMinutesChange: (value: string) => void;
    onBlockerReasonChange: (value: string) => void;
    onChangeReasonChange: (value: string) => void;
    onToggleStartPicker: () => void;
    onToggleDuePicker: () => void;
    onToggleTargetPicker: () => void;
    onClearStartDate: () => void;
    onClearDueDate: () => void;
    onClearTargetEndDate: () => void;
    onSelectDate: (date: string) => void;
    onCancelDatePicker: () => void;
  } = $props();

  const { t } = getLocalization();
</script>

<h2 class="text-[0.8rem] font-semibold tracking-normal">{t("projects.detail.dates")}</h2>
<div class="grid gap-3 min-[760px]:grid-cols-2">
  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
    <span>{t("projects.detail.estimateMinutes")}</span>
    <input
      value={estimateMinutes}
      inputmode="numeric"
      class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground outline-none focus:ring-2 focus:ring-ring/30"
      oninput={(event) => onEstimateMinutesChange(event.currentTarget.value)}
    />
  </label>

  <ProjectTaskDetailDateField
    label={t("projects.detail.startDate")}
    value={startDate}
    noDateLabel={t("projects.detail.noDate")}
    clearLabel={t("projects.detail.clearDate", t("projects.detail.startDate"))}
    pickerOpen={startPickerOpen}
    selectedDate={startDate || todayDate}
    rangeStartDate={startDate || undefined}
    rangeEndDate={dueDate || undefined}
    highlightToday={false}
    onToggle={onToggleStartPicker}
    onClear={onClearStartDate}
    onSelect={onSelectDate}
    onCancel={onCancelDatePicker}
  />

  <ProjectTaskDetailDateField
    label={t("projects.detail.dueDate")}
    value={dueDate}
    noDateLabel={t("projects.detail.noDate")}
    clearLabel={t("projects.detail.clearDate", t("projects.detail.dueDate"))}
    pickerOpen={duePickerOpen}
    selectedDate={dueDate || todayDate}
    rangeStartDate={startDate || undefined}
    rangeEndDate={dueDate || undefined}
    highlightToday={false}
    onToggle={onToggleDuePicker}
    onClear={onClearDueDate}
    onSelect={onSelectDate}
    onCancel={onCancelDatePicker}
  />

  <ProjectTaskDetailDateField
    label={t("projects.detail.targetEndDate")}
    value={targetEndDate}
    noDateLabel={t("projects.detail.noDate")}
    clearLabel={t("projects.detail.clearDate", t("projects.detail.targetEndDate"))}
    pickerOpen={targetPickerOpen}
    selectedDate={targetEndDate || todayDate}
    highlightMode="none"
    onToggle={onToggleTargetPicker}
    onClear={onClearTargetEndDate}
    onSelect={onSelectDate}
    onCancel={onCancelDatePicker}
  />
</div>
<label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
  <span>{t("projects.detail.blockerReason")}</span>
  <input
    value={blockerReason}
    class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground outline-none focus:ring-2 focus:ring-ring/30"
    oninput={(event) => onBlockerReasonChange(event.currentTarget.value)}
  />
</label>
<label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
  <span>{t("projects.detail.changeReason")}</span>
  <input
    value={changeReason}
    maxlength="1000"
    placeholder={t("projects.detail.changeReasonPlaceholder")}
    class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground outline-none placeholder:text-muted-foreground focus:ring-2 focus:ring-ring/30"
    oninput={(event) => onChangeReasonChange(event.currentTarget.value)}
  />
</label>
