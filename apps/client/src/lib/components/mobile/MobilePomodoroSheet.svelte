<script lang="ts">
  import ClockPlus from "@lucide/svelte/icons/clock-plus";
  import CalendarClock from "@lucide/svelte/icons/calendar-clock";
  import Coffee from "@lucide/svelte/icons/coffee";
  import Pause from "@lucide/svelte/icons/pause";
  import Play from "@lucide/svelte/icons/play";
  import Square from "@lucide/svelte/icons/square";
  import X from "@lucide/svelte/icons/x";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { activateModalFocus } from "$lib/modal-focus";
  import { BUILD_PLATFORM_PROFILE, platformHasCapability } from "$lib/platform";
  import { getPomodoro } from "$lib/stores/pomodoro.svelte";
  import { cn } from "$lib/utils";
  import { onMount, tick } from "svelte";

  let {
    onClose,
    onOpenCalendar,
  }: {
    onClose: () => void;
    onOpenCalendar: () => void;
  } = $props();

  const pomodoro = getPomodoro();
  const { t } = getLocalization();
  const backgroundAlertsAvailable = platformHasCapability(
    BUILD_PLATFORM_PROFILE,
    "notifications.native-scheduling",
  );
  const active = $derived(pomodoro.isActive);
  const pauseResumeLabel = $derived(
    active && !pomodoro.isRunning
      ? t("titleBar.pomodoro.resumeFocus")
      : t("titleBar.pomodoro.pauseFocus"),
  );
  const advanceLabel = $derived(
    active && pomodoro.phase !== "focus"
      ? t("titleBar.pomodoro.startFocusNow")
      : t("titleBar.pomodoro.goToBreakNow"),
  );
  let dialogElement = $state<HTMLDivElement | null>(null);
  let closeButtonElement = $state<HTMLButtonElement | null>(null);
  let showStopConfirmation = $state(false);
  let stopping = $state(false);

  async function confirmStopSession(): Promise<void> {
    if (stopping) return;
    stopping = true;
    try {
      await pomodoro.stopSession();
      showStopConfirmation = false;
      await tick();
      onClose();
    } catch (error) {
      console.error("Failed to stop the Pomodoro session", error);
    } finally {
      stopping = false;
    }
  }

  onMount(() => {
    if (!dialogElement) return;
    return activateModalFocus(dialogElement, closeButtonElement);
  });
</script>

<div
  data-mobile-pomodoro-sheet
  class="fixed z-80 flex items-end"
  style="left: var(--visual-viewport-offset-left); top: var(--visual-viewport-offset-top); width: var(--visual-viewport-width); height: var(--visual-viewport-height); padding-top: var(--safe-area-top);"
  role="presentation"
  inert={showStopConfirmation}
  aria-hidden={showStopConfirmation ? "true" : undefined}
>
  <button
    type="button"
    class="absolute inset-0 h-full w-full bg-black/45"
    aria-label={t("common.close")}
    onclick={onClose}
  ></button>
  <div
    bind:this={dialogElement}
    role="dialog"
    aria-modal="true"
    aria-label={t("titleBar.control.pomodoro")}
    class="relative z-10 mx-auto max-h-full w-full max-w-2xl overflow-y-auto rounded-t-3xl border border-b-0 border-border bg-card pb-[calc(var(--safe-area-bottom)+1rem)] pt-2 shadow-2xl"
    style="padding-left: calc(var(--safe-area-left) + 1rem); padding-right: calc(var(--safe-area-right) + 1rem);"
    tabindex="-1"
    onkeydown={(event) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      onClose();
    }}
  >
    <div class="mx-auto mb-2 h-1 w-10 rounded-full bg-muted-foreground/35" aria-hidden="true"></div>
    <div class="flex min-h-12 items-center gap-3">
      <div class="min-w-0 flex-1">
        <h2 class="font-semibold">{t("titleBar.control.pomodoro")}</h2>
        <p class="text-sm text-muted-foreground">
          {active
            ? t("titleBar.pomodoro.remaining", pomodoro.formattedTime)
            : t("titleBar.pomodoro.noActiveSession")}
        </p>
      </div>
      <button
        bind:this={closeButtonElement}
        type="button"
        class="flex min-h-12 min-w-12 items-center justify-center rounded-xl active:bg-accent"
        aria-label={t("common.close")}
        onclick={onClose}
      >
        <X size={21} aria-hidden="true" />
      </button>
    </div>

    {#if active}
      <div class="py-5 text-center font-mono text-5xl font-semibold tabular-nums">
        {pomodoro.formattedTime}
      </div>

      {#if !backgroundAlertsAvailable}
        <p class="mb-4 rounded-xl border border-border bg-background/65 px-3 py-2 text-sm leading-5 text-muted-foreground">
          {t("mobile.pomodoroForegroundOnly")}
        </p>
      {/if}

      <div class="grid grid-cols-2 gap-2">
      <button
        type="button"
        disabled={!pomodoro.canPauseResume}
        onclick={() => {
          if (pomodoro.isRunning) pomodoro.pause();
          else pomodoro.start();
        }}
        class={cn(
          "flex min-h-14 flex-col items-center justify-center gap-1 rounded-xl border border-border px-2 text-xs font-medium",
          pomodoro.canPauseResume ? "active:bg-accent" : "opacity-45",
        )}
      >
        {#if active && !pomodoro.isRunning}
          <Play size={21} aria-hidden="true" />
        {:else}
          <Pause size={21} aria-hidden="true" />
        {/if}
        <span>{pauseResumeLabel}</span>
      </button>
      <button
        type="button"
        disabled={!pomodoro.canAddFocusTime}
        onclick={() => pomodoro.addFocusTime()}
        class={cn(
          "flex min-h-14 flex-col items-center justify-center gap-1 rounded-xl border border-border px-2 text-xs font-medium",
          pomodoro.canAddFocusTime ? "active:bg-accent" : "opacity-45",
        )}
      >
        <ClockPlus size={21} aria-hidden="true" />
        <span>{t("titleBar.pomodoro.extendFocusMinutes", 3)}</span>
      </button>
      <button
        type="button"
        disabled={!active}
        onclick={() => pomodoro.skip()}
        class={cn(
          "flex min-h-14 flex-col items-center justify-center gap-1 rounded-xl border border-border px-2 text-xs font-medium",
          active ? "active:bg-accent" : "opacity-45",
        )}
      >
        <Coffee size={21} aria-hidden="true" />
        <span>{advanceLabel}</span>
      </button>
      <button
        type="button"
        data-mobile-pomodoro-stop
        disabled={!active || stopping}
        onclick={() => { showStopConfirmation = true; }}
        class={cn(
          "flex min-h-14 flex-col items-center justify-center gap-1 rounded-xl border border-destructive/45 px-2 text-xs font-medium text-destructive",
          active && !stopping ? "active:bg-destructive/10" : "opacity-45",
        )}
      >
        <Square size={20} aria-hidden="true" />
        <span>{t("focusDialog.stopSession")}</span>
      </button>
      </div>
    {:else}
      <div class="flex flex-col items-center gap-4 py-6 text-center" data-mobile-pomodoro-inactive>
        <div class="flex size-14 items-center justify-center rounded-2xl bg-accent text-accent-foreground">
          <CalendarClock size={28} aria-hidden="true" />
        </div>
        <p class="max-w-md text-sm leading-6 text-muted-foreground">
          {t("mobile.pomodoroInactiveDescription")}
        </p>
        <button
          type="button"
          class="flex min-h-12 w-full items-center justify-center gap-2 rounded-xl bg-primary px-4 text-sm font-semibold text-primary-foreground active:opacity-85"
          onclick={onOpenCalendar}
        >
          <CalendarClock size={20} aria-hidden="true" />
          <span>{t("mobile.pomodoroOpenCalendar")}</span>
        </button>
      </div>
    {/if}
  </div>
</div>

{#if showStopConfirmation}
  <ConfirmDialog
    title={t("focusDialog.stopTitle")}
    message={t("focusDialog.stopMessage")}
    confirmLabel={t("focusDialog.stopSession")}
    cancelLabel={t("common.cancel")}
    onConfirm={() => { void confirmStopSession(); }}
    onCancel={() => { showStopConfirmation = false; }}
  />
{/if}
