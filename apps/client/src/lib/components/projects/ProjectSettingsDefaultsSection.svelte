<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { PomodoroPresetKey } from "$lib/pomodoro/rhythm";
  import CustomSelect from "$lib/components/settings/CustomSelect.svelte";

  let {
    pomodoroOptions,
    pomodoroPresetLabel,
    projectDurationDraft = $bindable<string>(),
    projectPomodoroDraft = $bindable<PomodoroPresetKey | "none">(),
    projectIdleTimeoutDraft = $bindable<string>(),
    projectFocusPlaylistDraft = $bindable<string>(),
    projectBreakPlaylistDraft = $bindable<string>(),
    projectWorkEnvironmentDraft = $bindable<string>(),
    projectBlockerRulesetDraft = $bindable<string>(),
  }: {
    pomodoroOptions: readonly PomodoroPresetKey[];
    pomodoroPresetLabel: (preset: PomodoroPresetKey) => string;
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

  const pomodoroSelectOptions = $derived<SelectOption[]>([
    { value: "none", label: t("common.none") },
    ...pomodoroOptions.map((preset) => ({
      value: preset,
      label: pomodoroPresetLabel(preset),
    })),
  ]);

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
    <label class="flex items-center justify-between gap-4 px-1 py-1 max-[480px]:flex-col max-[480px]:items-stretch max-[480px]:gap-2">
      <span class="min-w-0 flex-1 text-[0.866667rem] text-foreground">{t("projects.settings.defaultDuration")}</span>
      <input
        bind:value={projectDurationDraft}
        inputmode="numeric"
        class="h-7 w-44 min-w-0 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground outline-none transition-colors focus:border-ring dark:bg-transparent max-[480px]:w-full"
      />
    </label>

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
