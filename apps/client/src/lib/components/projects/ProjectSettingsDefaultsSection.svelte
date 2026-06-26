<script lang="ts">
  import ColorPicker from "$lib/components/calendar/ColorPicker.svelte";
  import type { EventColor } from "$lib/components/calendar/types";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { PomodoroPresetKey } from "$lib/pomodoro/rhythm";
  import {
    isProjectCustomDurationInputShape,
    PROJECT_DURATION_PRESET_MINUTES,
    projectCustomDurationDraftFromMinutes,
    projectDurationMinutesFromCustomInput,
    projectDurationPresetFromMinutes,
    type ProjectDurationPresetValue,
    type ProjectDurationUnit,
  } from "$lib/projects/project-settings-duration";
  import CustomSelect from "$lib/components/settings/CustomSelect.svelte";
  import type { Theme } from "$lib/stores/themes";

  let {
    theme,
    pomodoroOptions,
    pomodoroPresetLabel,
    projectColorDraft = $bindable<EventColor | undefined>(),
    projectDurationDraft = $bindable<string>(),
    projectPomodoroDraft = $bindable<PomodoroPresetKey | "none">(),
    projectIdleTimeoutDraft = $bindable<string>(),
    projectFocusPlaylistDraft = $bindable<string>(),
    projectBreakPlaylistDraft = $bindable<string>(),
    projectWorkEnvironmentDraft = $bindable<string>(),
    projectBlockerRulesetDraft = $bindable<string>(),
  }: {
    theme: Theme;
    pomodoroOptions: readonly PomodoroPresetKey[];
    pomodoroPresetLabel: (preset: PomodoroPresetKey) => string;
    projectColorDraft: EventColor | undefined;
    projectDurationDraft: string;
    projectPomodoroDraft: PomodoroPresetKey | "none";
    projectIdleTimeoutDraft: string;
    projectFocusPlaylistDraft: string;
    projectBreakPlaylistDraft: string;
    projectWorkEnvironmentDraft: string;
    projectBlockerRulesetDraft: string;
  } = $props();

  const { t } = getLocalization();

  type SelectOption = { value: string; label: string };

  let durationPreset = $state<ProjectDurationPresetValue>("default");
  let customDurationValue = $state("");
  let customDurationUnit = $state<ProjectDurationUnit>("hours");
  let lastSyncedDurationDraft = $state("");

  const durationPresetOptions = $derived<SelectOption[]>([
    { value: "default", label: t("projects.settings.defaultDurationNone") },
    { value: "10", label: t("projects.settings.durationMinutes", 10) },
    { value: "15", label: t("projects.settings.durationMinutes", 15) },
    { value: "30", label: t("projects.settings.durationMinutes", 30) },
    { value: "60", label: t("projects.settings.durationHours", 1) },
    { value: "120", label: t("projects.settings.durationHours", 2) },
    { value: "180", label: t("projects.settings.durationHours", 3) },
    { value: "240", label: t("projects.settings.durationHours", 4) },
    { value: "custom", label: t("projects.settings.durationCustom") },
  ]);
  const durationUnitOptions = $derived<SelectOption[]>([
    { value: "hours", label: t("projects.settings.durationUnitHours") },
    { value: "minutes", label: t("projects.settings.durationUnitMinutes") },
  ]);
  const pomodoroSelectOptions = $derived<SelectOption[]>([
    { value: "none", label: t("common.none") },
    ...pomodoroOptions.map((preset) => ({
      value: preset,
      label: pomodoroPresetLabel(preset),
    })),
  ]);

  function parseStoredDurationDraft(value: string): number | null | "custom" {
    const trimmed = value.trim();
    if (!trimmed) return null;
    const parsed = Number(value.trim());
    return Number.isFinite(parsed) && parsed > 0
      ? Math.round(parsed)
      : "custom";
  }

  function syncDurationControlsFromDraft(value: string): void {
    const minutes = parseStoredDurationDraft(value);
    durationPreset = minutes === "custom" ? "custom" : projectDurationPresetFromMinutes(minutes);
    if (durationPreset === "custom") {
      if (typeof minutes === "number") {
        const draft = projectCustomDurationDraftFromMinutes(minutes);
        customDurationValue = draft.value;
        customDurationUnit = draft.unit;
      } else {
        customDurationValue = "";
        customDurationUnit = "hours";
      }
    } else {
      customDurationValue = "";
      customDurationUnit = "hours";
    }
    lastSyncedDurationDraft = value;
  }

  syncDurationControlsFromDraft(projectDurationDraft);

  $effect(() => {
    if (projectDurationDraft !== lastSyncedDurationDraft) {
      syncDurationControlsFromDraft(projectDurationDraft);
    }
  });

  function setProjectDurationDraft(value: string): void {
    projectDurationDraft = value;
    lastSyncedDurationDraft = value;
  }

  function isDurationPresetValue(value: string): value is ProjectDurationPresetValue {
    return value === "default" || value === "custom" || value in PROJECT_DURATION_PRESET_MINUTES;
  }

  function isDurationUnit(value: string): value is ProjectDurationUnit {
    return value === "minutes" || value === "hours";
  }

  function setDurationPreset(value: string): void {
    if (!isDurationPresetValue(value)) return;
    durationPreset = value;
    if (value === "custom") {
      customDurationValue = "";
      customDurationUnit = "hours";
      setProjectDurationDraft("custom");
      return;
    }
    if (value === "default") {
      setProjectDurationDraft("");
      return;
    }
    setProjectDurationDraft(String(PROJECT_DURATION_PRESET_MINUTES[value]));
  }

  function syncCustomDurationDraft(): void {
    const minutes = projectDurationMinutesFromCustomInput(customDurationValue, customDurationUnit);
    setProjectDurationDraft(minutes === null ? "custom" : String(minutes));
  }

  function setCustomDurationValue(input: HTMLInputElement): void {
    const normalized = input.value.trim().replace(",", ".");
    if (!isProjectCustomDurationInputShape(normalized)) {
      input.value = customDurationValue;
      return;
    }
    customDurationValue = normalized;
    syncCustomDurationDraft();
  }

  function setCustomDurationUnit(value: string): void {
    if (!isDurationUnit(value)) return;
    customDurationUnit = value;
    syncCustomDurationDraft();
  }

  function setPomodoroPreset(value: string): void {
    if (value === "none") {
      projectPomodoroDraft = "none";
      return;
    }
    if (pomodoroOptions.includes(value as PomodoroPresetKey)) {
      projectPomodoroDraft = value as PomodoroPresetKey;
    }
  }
</script>

<section class="flex flex-col gap-1.5">
  <h2 class="px-1 text-[0.866667rem] font-semibold text-foreground">{t("projects.settings.defaults")}</h2>
  <div class="flex flex-col gap-1.5">
    <div class="flex items-center justify-between gap-4 px-1 py-1 max-[480px]:flex-col max-[480px]:items-stretch max-[480px]:gap-2">
      <div class="min-w-0 flex-1 text-[0.866667rem] text-foreground">{t("projects.settings.color")}</div>
      <ColorPicker
        color={projectColorDraft}
        {theme}
        title={t("projects.settings.color")}
        ariaLabel={t("projects.settings.selectColor")}
        displayLabel
        class="w-44 max-[480px]:w-full"
        onselect={(color) => {
          projectColorDraft = color;
        }}
      />
    </div>

    <CustomSelect
      label={t("projects.settings.defaultDuration")}
      value={durationPreset}
      options={durationPresetOptions}
      onChange={setDurationPreset}
      class="w-44"
    />

    {#if durationPreset === "custom"}
      <div class="flex justify-end px-1 py-1">
        <div class="flex w-44 min-w-0 items-center gap-1.5 max-[480px]:w-full">
          <input
            value={customDurationValue}
            inputmode="decimal"
            aria-label={t("projects.settings.customDurationValue")}
            class="h-7 min-w-0 flex-1 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground outline-none transition-colors focus:border-ring dark:bg-transparent"
            oninput={(event) => {
              setCustomDurationValue(event.currentTarget);
            }}
          />
          <CustomSelect
            inline
            value={customDurationUnit}
            options={durationUnitOptions}
            onChange={setCustomDurationUnit}
            ariaLabel={t("projects.settings.customDurationUnit")}
            class="w-24 shrink-0 max-[480px]:w-28 max-[480px]:flex-none"
          />
        </div>
      </div>
    {/if}

    <CustomSelect
      label={t("projects.settings.defaultPomodoro")}
      value={projectPomodoroDraft}
      options={pomodoroSelectOptions}
      onChange={setPomodoroPreset}
      class="w-44"
    />

    <label class="flex items-center justify-between gap-4 px-1 py-1 max-[480px]:flex-col max-[480px]:items-stretch max-[480px]:gap-2">
      <span class="min-w-0 flex-1 text-[0.866667rem] text-foreground">{t("projects.settings.defaultIdleTimeout")}</span>
      <input
        bind:value={projectIdleTimeoutDraft}
        inputmode="numeric"
        placeholder={t("common.disabled")}
        class="h-7 w-44 min-w-0 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground outline-none transition-colors placeholder:text-muted-foreground focus:border-ring dark:bg-transparent max-[480px]:w-full"
      />
    </label>
  </div>
</section>

<div class="h-px bg-border/70" aria-hidden="true"></div>

<section class="flex flex-col gap-1.5">
  <h2 class="px-1 text-[0.866667rem] font-semibold text-foreground">{t("projects.settings.automationDefaults")}</h2>
  <div class="flex flex-col gap-1.5">
    <label class="flex items-center justify-between gap-4 px-1 py-1 max-[480px]:flex-col max-[480px]:items-stretch max-[480px]:gap-2">
      <span class="min-w-0 flex-1 text-[0.866667rem] text-foreground">{t("projects.settings.focusPlaylist")}</span>
      <input
        bind:value={projectFocusPlaylistDraft}
        placeholder={t("common.none")}
        class="h-7 w-44 min-w-0 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground outline-none transition-colors placeholder:text-muted-foreground focus:border-ring dark:bg-transparent max-[480px]:w-full"
      />
    </label>

    <label class="flex items-center justify-between gap-4 px-1 py-1 max-[480px]:flex-col max-[480px]:items-stretch max-[480px]:gap-2">
      <span class="min-w-0 flex-1 text-[0.866667rem] text-foreground">{t("projects.settings.breakPlaylist")}</span>
      <input
        bind:value={projectBreakPlaylistDraft}
        placeholder={t("common.none")}
        class="h-7 w-44 min-w-0 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground outline-none transition-colors placeholder:text-muted-foreground focus:border-ring dark:bg-transparent max-[480px]:w-full"
      />
    </label>

    <label class="flex items-center justify-between gap-4 px-1 py-1 max-[480px]:flex-col max-[480px]:items-stretch max-[480px]:gap-2">
      <span class="min-w-0 flex-1 text-[0.866667rem] text-foreground">{t("projects.settings.workEnvironment")}</span>
      <input
        bind:value={projectWorkEnvironmentDraft}
        placeholder={t("common.none")}
        class="h-7 w-44 min-w-0 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground outline-none transition-colors placeholder:text-muted-foreground focus:border-ring dark:bg-transparent max-[480px]:w-full"
      />
    </label>

    <label class="flex items-center justify-between gap-4 px-1 py-1 max-[480px]:flex-col max-[480px]:items-stretch max-[480px]:gap-2">
      <span class="min-w-0 flex-1 text-[0.866667rem] text-foreground">{t("projects.settings.blockerRuleset")}</span>
      <input
        bind:value={projectBlockerRulesetDraft}
        placeholder={t("common.none")}
        class="h-7 w-44 min-w-0 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground outline-none transition-colors placeholder:text-muted-foreground focus:border-ring dark:bg-transparent max-[480px]:w-full"
      />
    </label>
  </div>
</section>
