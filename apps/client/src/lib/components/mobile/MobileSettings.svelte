<script lang="ts">
  import X from "@lucide/svelte/icons/x";
  import { themeDisplayName } from "$lib/i18n/theme-labels";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { activateModalFocus } from "$lib/modal-focus";
  import { isLanguagePreference } from "$lib/stores/preferences";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { onMount } from "svelte";

  let { onClose }: { onClose: () => void } = $props();

  const localization = getLocalization();
  const { t } = localization;
  const preferences = getPreferences();
  const theme = getTheme();
  const themeOptions = $derived(
    Object.values(theme.registry).map((entry) => ({
      value: entry.id,
      label: themeDisplayName(entry, t),
    })),
  );
  let dialogElement = $state<HTMLDivElement | null>(null);
  let closeButtonElement = $state<HTMLButtonElement | null>(null);

  onMount(() => {
    if (!dialogElement) return;
    return activateModalFocus(dialogElement, closeButtonElement);
  });

  function languageLabel(value: string): string {
    if (value === "system") return t("language.systemOption");
    if (value === "en") return t("language.englishOption");
    return t("language.spanishOption");
  }

  function updateLanguage(value: string): void {
    if (isLanguagePreference(value)) void preferences.setLanguagePreference(value);
  }

  function updateFontScale(value: string): void {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) preferences.setFontScale(parsed);
  }

  function updateTimeFormat(value: string): void {
    if (value === "12h" || value === "24h") preferences.setCalendarTimeFormat(value);
  }
</script>

<div
  bind:this={dialogElement}
  role="dialog"
  aria-modal="true"
  aria-label={t("settings.title")}
  class="fixed inset-0 z-80 flex flex-col bg-background pb-(--safe-area-bottom) pt-(--safe-area-top)"
  style="padding-left: var(--safe-area-left); padding-right: var(--safe-area-right);"
  tabindex="-1"
  onkeydown={(event) => {
    if (event.key !== "Escape") return;
    event.preventDefault();
    event.stopPropagation();
    onClose();
  }}
>
  <header class="flex min-h-14 shrink-0 items-center gap-2 border-b border-border px-2">
    <h2 class="min-w-0 flex-1 truncate px-2 text-lg font-semibold">{t("settings.title")}</h2>
    <button
      bind:this={closeButtonElement}
      type="button"
      onclick={onClose}
      aria-label={t("settings.close")}
      class="flex min-h-12 min-w-12 items-center justify-center rounded-xl active:bg-accent"
    >
      <X size={22} aria-hidden="true" />
    </button>
  </header>

  <div class="min-h-0 flex-1 overflow-y-auto px-4 py-5">
    <div class="mx-auto flex w-full max-w-xl flex-col gap-6">
      <section class="flex flex-col gap-4">
        <h3 class="text-base font-semibold">{t("settings.section.appearance")}</h3>

        <label class="flex flex-col gap-1.5 text-sm">
          <span class="font-medium">{t("settings.appearance.languageHeading")}</span>
          <select
            value={preferences.languagePreference}
            onchange={(event) => updateLanguage(event.currentTarget.value)}
            class="min-h-12 rounded-xl border border-border bg-card px-3 text-base"
          >
            {#each preferences.languagePreferences as language}
              <option value={language}>{languageLabel(language)}</option>
            {/each}
          </select>
        </label>

        <label class="flex flex-col gap-1.5 text-sm">
          <span class="font-medium">{t("mobile.theme")}</span>
          <select
            value={theme.id}
            onchange={(event) => theme.setTheme(event.currentTarget.value)}
            class="min-h-12 rounded-xl border border-border bg-card px-3 text-base"
          >
            {#each themeOptions as option}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </label>

        <label class="flex flex-col gap-1.5 text-sm">
          <span class="font-medium">{t("settings.appearance.textSize")}</span>
          <select
            value={String(preferences.fontScale)}
            onchange={(event) => updateFontScale(event.currentTarget.value)}
            class="min-h-12 rounded-xl border border-border bg-card px-3 text-base"
          >
            <option value="0.9">90%</option>
            <option value="1">100%</option>
            <option value="1.1">110%</option>
            <option value="1.25">125%</option>
            <option value="1.5">150%</option>
          </select>
          <span class="text-xs text-muted-foreground">
            {t("settings.appearance.textSizeDescription")}
          </span>
        </label>

        <label class="flex flex-col gap-1.5 text-sm">
          <span class="font-medium">{t("settings.appearance.timeFormat")}</span>
          <select
            value={preferences.calendarTimeFormat}
            onchange={(event) => updateTimeFormat(event.currentTarget.value)}
            class="min-h-12 rounded-xl border border-border bg-card px-3 text-base"
          >
            <option value="24h">{t("settings.appearance.timeFormat24h")}</option>
            <option value="12h">{t("settings.appearance.timeFormat12h")}</option>
          </select>
        </label>
      </section>

      <section class="rounded-2xl border border-border bg-card p-4 text-sm leading-6">
        <h3 class="font-semibold">{t("mobile.privateDataHeading")}</h3>
        <p class="mt-1 text-muted-foreground">
          {t("mobile.privateDataDescription")}
        </p>
      </section>
    </div>
  </div>
</div>
