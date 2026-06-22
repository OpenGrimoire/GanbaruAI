<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { PomodoroPresetKey } from "$lib/pomodoro/rhythm";
  import { cn } from "$lib/utils";

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
    pomodoroOptions: PomodoroPresetKey[];
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
</script>

<section class="grid gap-2 border-t border-border/70 pt-3">
  <h2 class="text-[0.8rem] font-semibold">{t("projects.settings.defaults")}</h2>
  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
    <span>{t("projects.settings.defaultDuration")}</span>
    <input
      bind:value={projectDurationDraft}
      inputmode="numeric"
      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
    />
  </label>

  <div class="grid gap-1">
    <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.settings.defaultPomodoro")}</div>
    <div class="flex flex-wrap gap-1">
      <button
        type="button"
        class={cn(
          "rounded-md border px-2 py-1 text-[0.766667rem]",
          projectPomodoroDraft === "none"
            ? "border-primary/50 bg-primary/10 text-primary"
            : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
        )}
        onclick={() => {
          projectPomodoroDraft = "none";
        }}
      >
        {t("common.none")}
      </button>
      {#each pomodoroOptions as preset}
        <button
          type="button"
          class={cn(
            "rounded-md border px-2 py-1 text-[0.766667rem]",
            projectPomodoroDraft === preset
              ? "border-primary/50 bg-primary/10 text-primary"
              : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
          )}
          onclick={() => {
            projectPomodoroDraft = preset;
          }}
        >
          {pomodoroPresetLabel(preset)}
        </button>
      {/each}
    </div>
  </div>

  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
    <span>{t("projects.settings.defaultIdleTimeout")}</span>
    <input
      bind:value={projectIdleTimeoutDraft}
      inputmode="numeric"
      placeholder={t("common.disabled")}
      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
    />
  </label>

  <div class="grid gap-2 border-t border-border/60 pt-2">
    <h3 class="text-[0.766667rem] font-semibold">{t("projects.settings.automationDefaults")}</h3>
    <div class="grid gap-2 min-[980px]:grid-cols-2">
      <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
        <span>{t("projects.settings.focusPlaylist")}</span>
        <input
          bind:value={projectFocusPlaylistDraft}
          placeholder={t("common.none")}
          class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
        />
      </label>
      <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
        <span>{t("projects.settings.breakPlaylist")}</span>
        <input
          bind:value={projectBreakPlaylistDraft}
          placeholder={t("common.none")}
          class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
        />
      </label>
      <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
        <span>{t("projects.settings.workEnvironment")}</span>
        <input
          bind:value={projectWorkEnvironmentDraft}
          placeholder={t("common.none")}
          class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
        />
      </label>
      <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
        <span>{t("projects.settings.blockerRuleset")}</span>
        <input
          bind:value={projectBlockerRulesetDraft}
          placeholder={t("common.none")}
          class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
        />
      </label>
    </div>
  </div>
</section>
