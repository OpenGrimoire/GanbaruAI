<script lang="ts">
  import Settings from "@lucide/svelte/icons/settings";
  import Music from "@lucide/svelte/icons/music";
  import StickyNote from "@lucide/svelte/icons/sticky-note";
  import Timer from "@lucide/svelte/icons/timer";
  import { getLocalization } from "$lib/i18n/translator.svelte";

  let {
    title,
    pomodoroTime,
    pomodoroActive,
    quickNotesOpen,
    quickNotesLoading,
    quickNotesDisabled,
    musicOpen,
    musicLoading,
    musicDisabled,
    musicVisible = true,
    onOpenPomodoro,
    onOpenQuickNotes,
    onOpenMusic,
    onOpenSettings,
  }: {
    title: string;
    pomodoroTime: string;
    pomodoroActive: boolean;
    quickNotesOpen: boolean;
    quickNotesLoading: boolean;
    quickNotesDisabled: boolean;
    musicOpen: boolean;
    musicLoading: boolean;
    musicDisabled: boolean;
    musicVisible?: boolean;
    onOpenPomodoro: () => void;
    onOpenQuickNotes: () => void;
    onOpenMusic: () => void;
    onOpenSettings: () => void;
  } = $props();

  const { t } = getLocalization();
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
  <button
    type="button"
    onclick={onOpenPomodoro}
    aria-label={t("titleBar.control.pomodoro")}
    aria-haspopup="dialog"
    class="flex min-h-12 min-w-12 items-center justify-center gap-1.5 rounded-xl px-2 text-sm active:bg-sidebar-accent"
  >
    <Timer size={20} strokeWidth={1.8} aria-hidden="true" />
    {#if pomodoroActive}
      <span class="font-mono text-xs font-semibold tabular-nums">{pomodoroTime}</span>
    {/if}
  </button>
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
