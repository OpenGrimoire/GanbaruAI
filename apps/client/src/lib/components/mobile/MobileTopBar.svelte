<script lang="ts">
  import Settings from "@lucide/svelte/icons/settings";
  import Music from "@lucide/svelte/icons/music";
  import StickyNote from "@lucide/svelte/icons/sticky-note";
  import type { Component } from "svelte";
  import type { View } from "$lib/navigation";
  import MobileNavigation from "$lib/components/mobile/MobileNavigation.svelte";
  import PomodoroProgressRing from "$lib/components/pomodoro/PomodoroProgressRing.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    classifyLoadFailure,
    recoverLoadFailure,
    type LoadFailure,
  } from "$lib/module-load-recovery";
  import { cn } from "$lib/utils";

  interface PomodoroMenuProps {
    includeMusic: boolean;
    touch?: boolean;
    onDismiss: () => void;
    onOpenMusic: () => void;
  }

  type PomodoroMenuComponent = Component<PomodoroMenuProps>;

  let {
    current,
    pomodoroTime,
    pomodoroActive,
    pomodoroRemainingSeconds,
    pomodoroTotalSeconds,
    pomodoroPaused,
    pomodoroPausedPulseAmount,
    pomodoroOpen,
    quickNotesOpen,
    quickNotesLoading,
    quickNotesDisabled,
    musicOpen,
    musicLoading,
    musicDisabled,
    musicVisible = true,
    primaryNavigationVisible = true,
    onTogglePomodoro,
    onClosePomodoro,
    onOpenQuickNotes,
    onOpenMusic,
    onOpenSettings,
    onNavigate,
  }: {
    current: View;
    pomodoroTime: string;
    pomodoroActive: boolean;
    pomodoroRemainingSeconds: number;
    pomodoroTotalSeconds: number;
    pomodoroPaused: boolean;
    pomodoroPausedPulseAmount: number | null;
    pomodoroOpen: boolean;
    quickNotesOpen: boolean;
    quickNotesLoading: boolean;
    quickNotesDisabled: boolean;
    musicOpen: boolean;
    musicLoading: boolean;
    musicDisabled: boolean;
    musicVisible?: boolean;
    primaryNavigationVisible?: boolean;
    onTogglePomodoro: () => void;
    onClosePomodoro: () => void;
    onOpenQuickNotes: () => void;
    onOpenMusic: () => void;
    onOpenSettings: () => void;
    onNavigate: (view: View) => void;
  } = $props();

  const { t } = getLocalization();
  const utilityButtonClass = "group flex h-full items-center justify-center text-muted-foreground transition-colors disabled:opacity-40";
  const utilityIconClass = "grid h-8 w-8 place-items-center rounded-full transition-colors group-active:bg-accent/70";
  let PomodoroMenuSurface = $state<PomodoroMenuComponent | null>(null);
  let pomodoroMenuLoading = $state(false);
  let pomodoroMenuLoadError = $state<LoadFailure | null>(null);

  async function loadPomodoroMenu(): Promise<void> {
    if (PomodoroMenuSurface || pomodoroMenuLoading) return;
    pomodoroMenuLoading = true;
    pomodoroMenuLoadError = null;
    try {
      const module = await import("$lib/components/pomodoro/PomodoroMenuContent.svelte");
      PomodoroMenuSurface = module.default;
    } catch (error) {
      pomodoroMenuLoadError = classifyLoadFailure(error);
      console.error("Failed to load the mobile Pomodoro menu", error);
    } finally {
      pomodoroMenuLoading = false;
    }
  }

  function retryPomodoroMenuLoad(): void {
    const failure = pomodoroMenuLoadError;
    if (!failure) return;
    recoverLoadFailure(failure, () => void loadPomodoroMenu());
  }

  function togglePomodoro(): void {
    if (!pomodoroOpen) void loadPomodoroMenu();
    onTogglePomodoro();
  }

  $effect(() => {
    if (pomodoroOpen) void loadPomodoroMenu();
  });
</script>

<header
  class={cn(
    "mobile-top-bar h-(--cal-header-row-h) shrink-0 items-center border-b px-0.5 text-foreground",
    primaryNavigationVisible
      ? musicVisible
        ? "grid grid-cols-8"
        : "grid grid-cols-7"
      : "flex justify-end",
  )}
  style="background-color: var(--cal-header-bg); border-color: var(--sidebar);"
>
  {#if primaryNavigationVisible}
    <MobileNavigation {current} presentation="top" {onNavigate} />
  {/if}
  <div class={cn("relative h-full", primaryNavigationVisible ? "min-w-0" : "w-11 shrink-0")}>
    <button
      type="button"
      data-mobile-pomodoro-trigger
      onpointerdown={() => void loadPomodoroMenu()}
      onclick={togglePomodoro}
      aria-label={pomodoroActive
        ? t("titleBar.pomodoro.remaining", pomodoroTime)
        : t("titleBar.control.pomodoro")}
      aria-haspopup="menu"
      aria-expanded={pomodoroOpen}
      class={cn(utilityButtonClass, primaryNavigationVisible ? "w-full min-w-0" : "w-11 shrink-0", pomodoroOpen && "text-foreground")}
    >
      <span class={cn(utilityIconClass, pomodoroOpen && "bg-accent/70")}>
        <PomodoroProgressRing
          active={pomodoroActive}
          remainingSeconds={pomodoroRemainingSeconds}
          totalSeconds={pomodoroTotalSeconds}
          paused={pomodoroPaused}
          pausedPulseAmount={pomodoroPausedPulseAmount}
          size={20}
          trackClass="stroke-muted-foreground/25"
          progressClass="text-foreground/76 stroke-foreground/76"
        />
      </span>
    </button>
    {#if pomodoroOpen}
      <button
        type="button"
        class="fixed inset-0 z-40 h-full w-full bg-transparent"
        aria-label={t("common.close")}
        onclick={onClosePomodoro}
      ></button>
      <div
        role="menu"
        class="absolute right-0 top-[calc(100%+0.25rem)] z-50 max-h-[calc(var(--visual-viewport-height)-var(--safe-area-top)-var(--mobile-topbar-h)-0.75rem)] w-64 max-w-[calc(100vw-1rem)] overflow-y-auto rounded-xl border border-border bg-popover py-1 text-popover-foreground shadow-xl"
      >
        {#if PomodoroMenuSurface}
          <PomodoroMenuSurface
            includeMusic={musicVisible}
            touch
            onDismiss={onClosePomodoro}
            onOpenMusic={onOpenMusic}
          />
        {:else if pomodoroMenuLoadError}
          <div class="flex flex-col gap-2 p-3 text-sm" role="alert">
            <p class="wrap-break-word text-xs text-muted-foreground">{pomodoroMenuLoadError.message}</p>
            <button
              type="button"
              class="min-h-12 rounded-lg border border-border px-3 font-medium active:bg-accent"
              onclick={retryPomodoroMenuLoad}
            >{t("common.retry")}</button>
          </div>
        {:else}
          <p class="p-4 text-center text-sm text-muted-foreground" aria-busy="true">
            {t("common.loading")}
          </p>
        {/if}
      </div>
    {/if}
  </div>
  <button
    type="button"
    data-mobile-quick-notes-trigger
    disabled={quickNotesDisabled}
    onclick={onOpenQuickNotes}
    aria-label={t("titleBar.control.quickNotes")}
    aria-haspopup="dialog"
    aria-expanded={quickNotesOpen}
    aria-busy={quickNotesLoading}
    class={cn(utilityButtonClass, primaryNavigationVisible ? "w-full min-w-0" : "w-11 shrink-0", quickNotesOpen && "text-foreground")}
  >
    <span class={cn(utilityIconClass, quickNotesOpen && "bg-accent/70")}><StickyNote size={19} strokeWidth={1.8} aria-hidden="true" /></span>
  </button>
  {#if musicVisible}
    <button
      type="button"
      data-mobile-music-trigger
      disabled={musicDisabled}
      onclick={onOpenMusic}
      aria-label={t("titleBar.control.music")}
      aria-haspopup="dialog"
      aria-expanded={musicOpen}
      aria-busy={musicLoading}
      class={cn(utilityButtonClass, primaryNavigationVisible ? "w-full min-w-0" : "w-11 shrink-0", musicOpen && "text-foreground")}
    >
      <span class={cn(utilityIconClass, musicOpen && "bg-accent/70")}><Music size={19} strokeWidth={1.8} aria-hidden="true" /></span>
    </button>
  {/if}
  <button
    type="button"
    onclick={onOpenSettings}
    aria-label={t("titleBar.control.settings")}
    aria-haspopup="dialog"
    class={cn(utilityButtonClass, primaryNavigationVisible ? "w-full min-w-0" : "w-11 shrink-0")}
  >
    <span class={utilityIconClass}><Settings size={19} strokeWidth={1.8} aria-hidden="true" /></span>
  </button>
</header>
