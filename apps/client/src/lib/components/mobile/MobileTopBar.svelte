<script lang="ts">
  import Settings from "@lucide/svelte/icons/settings";
  import Music from "@lucide/svelte/icons/music";
  import StickyNote from "@lucide/svelte/icons/sticky-note";
  import type { Component } from "svelte";
  import PomodoroProgressRing from "$lib/components/pomodoro/PomodoroProgressRing.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";

  interface PomodoroMenuProps {
    includeMusic: boolean;
    touch?: boolean;
    onDismiss: () => void;
    onOpenMusic: () => void;
  }

  type PomodoroMenuComponent = Component<PomodoroMenuProps>;

  let {
    title,
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
    onTogglePomodoro,
    onClosePomodoro,
    onOpenQuickNotes,
    onOpenMusic,
    onOpenSettings,
  }: {
    title: string;
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
    onTogglePomodoro: () => void;
    onClosePomodoro: () => void;
    onOpenQuickNotes: () => void;
    onOpenMusic: () => void;
    onOpenSettings: () => void;
  } = $props();

  const { t } = getLocalization();
  let PomodoroMenuSurface = $state<PomodoroMenuComponent | null>(null);
  let pomodoroMenuLoading = $state(false);
  let pomodoroMenuLoadError = $state("");

  async function loadPomodoroMenu(): Promise<void> {
    if (PomodoroMenuSurface || pomodoroMenuLoading) return;
    pomodoroMenuLoading = true;
    pomodoroMenuLoadError = "";
    try {
      const module = await import("$lib/components/pomodoro/PomodoroMenuContent.svelte");
      PomodoroMenuSurface = module.default;
    } catch (error) {
      pomodoroMenuLoadError = error instanceof Error ? error.message : String(error);
      console.error("Failed to load the mobile Pomodoro menu", error);
    } finally {
      pomodoroMenuLoading = false;
    }
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
  class="mobile-top-bar flex min-h-(--mobile-topbar-h) shrink-0 items-center gap-2 border-b border-sidebar-border bg-sidebar px-2 text-sidebar-foreground"
>
  <h1 class="min-w-0 flex-1 truncate px-2 text-base font-semibold">{title}</h1>
  <button
    type="button"
    data-mobile-quick-notes-trigger
    disabled={quickNotesDisabled}
    onclick={onOpenQuickNotes}
    aria-label={t("titleBar.control.quickNotes")}
    aria-haspopup="dialog"
    aria-expanded={quickNotesOpen}
    aria-busy={quickNotesLoading}
    class="flex min-h-12 min-w-12 items-center justify-center rounded-xl active:bg-sidebar-accent disabled:opacity-40"
  >
    <StickyNote size={20} strokeWidth={1.8} aria-hidden="true" />
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
      class="flex min-h-12 min-w-12 items-center justify-center rounded-xl active:bg-sidebar-accent disabled:opacity-40"
    >
      <Music size={20} strokeWidth={1.8} aria-hidden="true" />
    </button>
  {/if}
  <div class="relative">
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
      class="flex min-h-12 min-w-12 items-center justify-center rounded-xl active:bg-sidebar-accent"
    >
      <PomodoroProgressRing
        active={pomodoroActive}
        remainingSeconds={pomodoroRemainingSeconds}
        totalSeconds={pomodoroTotalSeconds}
        paused={pomodoroPaused}
        pausedPulseAmount={pomodoroPausedPulseAmount}
        size={21}
        trackClass="stroke-sidebar-foreground/20"
        progressClass="text-sidebar-foreground/76 stroke-sidebar-foreground/76"
      />
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
            <p class="wrap-break-word text-xs text-muted-foreground">{pomodoroMenuLoadError}</p>
            <button
              type="button"
              class="min-h-12 rounded-lg border border-border px-3 font-medium active:bg-accent"
              onclick={() => void loadPomodoroMenu()}
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
    onclick={onOpenSettings}
    aria-label={t("titleBar.control.settings")}
    aria-haspopup="dialog"
    class="flex min-h-12 min-w-12 items-center justify-center rounded-xl active:bg-sidebar-accent"
  >
    <Settings size={20} strokeWidth={1.8} aria-hidden="true" />
  </button>
</header>
