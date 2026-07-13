<script lang="ts">
  import ClockPlus from "@lucide/svelte/icons/clock-plus";
  import Coffee from "@lucide/svelte/icons/coffee";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Music from "@lucide/svelte/icons/music";
  import PauseIcon from "@lucide/svelte/icons/pause";
  import PlayIcon from "@lucide/svelte/icons/play";
  import SkipBack from "@lucide/svelte/icons/skip-back";
  import SkipForward from "@lucide/svelte/icons/skip-forward";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { PlaybackStatus } from "$lib/music/playback";
  import { getMusicPlayer } from "$lib/stores/music-player.svelte";
  import { getNavigation } from "$lib/stores/navigation.svelte";
  import { getPomodoro } from "$lib/stores/pomodoro.svelte";
  import { cn } from "$lib/utils";

  let {
    showPomodoro,
    showMusic,
    showMenu = $bindable(),
    isMainWindow,
    onMenuOpened,
  }: {
    showPomodoro: boolean;
    showMusic: boolean;
    showMenu: boolean;
    isMainWindow: boolean;
    onMenuOpened: () => void;
  } = $props();

  const musicPlayer = getMusicPlayer();
  const nav = getNavigation();
  const pomodoro = getPomodoro();
  const { t } = getLocalization();

  const TITLE_BAR_ICON_COLOR_CLASS = "text-foreground/68 dark:text-white/76";
  const TITLE_BAR_ICON_STROKE_CLASS = "stroke-foreground/68 dark:stroke-white/76";
  const TITLE_BAR_SUBTLE_STROKE_CLASS = "stroke-foreground/20 dark:stroke-white/20";
  const TITLE_BAR_ICON_STROKE_WIDTH = 1.5;
  const TITLE_BAR_ICON_SIZE = 14;
  const TITLE_BAR_MENU_ICON_SIZE = 14;
  const TITLE_BAR_MENU_ICON_STROKE_WIDTH = 1.8;
  const POMODORO_RING_SIZE = TITLE_BAR_ICON_SIZE + 0.5;
  const POMODORO_RING_STROKE_WIDTH = 2.15;
  const volumeStep = 0.05;

  const progressPercent = $derived(() => {
    const total = pomodoro.totalSecondsForPhase;
    return total === 0 ? 0 : ((total - pomodoro.remainingSeconds) / total) * 100;
  });
  const isActive = $derived(pomodoro.isActive);
  const pomodoroPauseResumeLabel = $derived(
    isActive && !pomodoro.isRunning
      ? t("titleBar.pomodoro.resumeFocus")
      : t("titleBar.pomodoro.pauseFocus"),
  );
  const pomodoroPausedPulseActive = $derived(
    isActive && pomodoro.phase === "focus" && !pomodoro.isRunning
      && !pomodoro.suspendedAway && !pomodoro.idlePaused,
  );
  const pomodoroPausedPulseStyle = $derived.by(() => {
    const amount = pomodoro.pausedPulseAmount;
    return pomodoroPausedPulseActive && amount !== null
      ? `--pomodoro-ring-paused-pulse-amount: ${Math.round(amount * 100)}%;`
      : undefined;
  });
  const phaseAdvanceLabel = $derived(
    isActive
      ? pomodoro.phase === "focus"
        ? t("titleBar.pomodoro.goToBreakNow")
        : t("titleBar.pomodoro.startFocusNow")
      : t("titleBar.pomodoro.goToBreakNow"),
  );
  const canPauseResumePomodoro = $derived(pomodoro.canPauseResume);
  const canAdvancePomodoro = $derived(isActive);
  const titleBarVolumeStep = volumeStep;
  const titleBarVolumeSliderProgress = $derived(musicPlayer.volumeMax > 0
    ? `${Math.min(100, Math.max(0, (musicPlayer.volumeControlValue / musicPlayer.volumeMax) * 100))}%`
    : "0%");
  const musicStatusText = $derived.by(() => {
    const title = musicPlayer.currentSource ? musicPlayer.loadedTitle.trim() : "";
    if (title) return title;
    if (musicPlayer.currentSource || musicPlayer.snapshot.status !== "idle") {
      return musicStatusLabel(musicPlayer.snapshot.status);
    }
    return t("titleBar.music.noMusicLoaded");
  });
  const canPlayPauseMusic = $derived(Boolean(musicPlayer.currentSource) && !musicPlayer.isBusy);
  const musicPlayPauseLabel = $derived(
    musicPlayer.isPlaying ? t("titleBar.music.pause") : t("titleBar.music.play"),
  );
  const musicVolumeTooltipLine = $derived(
    t("titleBar.music.volumeTooltip", musicPlayer.volumePercentLabel),
  );
  const pomodoroButtonTooltip = $derived(
    `${isActive ? t("titleBar.pomodoro.remaining", pomodoro.formattedTime) : t("titleBar.control.pomodoro")}\n${musicVolumeTooltipLine}`,
  );
  const musicButtonTooltip = $derived(
    `${t("titleBar.control.music")}\n${musicVolumeTooltipLine}`,
  );

  function musicStatusLabel(status: PlaybackStatus): string {
    switch (status) {
      case "playing": return t("titleBar.music.status.playing");
      case "paused": return t("titleBar.music.status.paused");
      case "loading": return t("titleBar.music.status.loading");
      case "ready": return t("titleBar.music.status.ready");
      case "ended": return t("titleBar.music.status.ended");
      case "error": return t("titleBar.music.status.error");
      case "idle": return t("titleBar.music.status.idle");
    }
  }

  function toggleMenu(): void {
    const nextOpen = !showMenu;
    showMenu = nextOpen;
    if (nextOpen) onMenuOpened();
  }

  function snappedVolume(value: number): number {
    if (!Number.isFinite(value)) return musicPlayer.volumeControlValue;
    return Number((Math.round(value / volumeStep) * volumeStep).toFixed(2));
  }

  function setTitleBarVolume(value: number): void {
    void musicPlayer.setVolume(snappedVolume(value));
  }

  export function handleVolumeWheel(event: WheelEvent): void {
    event.preventDefault();
    event.stopPropagation();
    if (event.ctrlKey) return;
    const delta = event.deltaY === 0 ? -event.deltaX : event.deltaY;
    if (delta === 0) return;
    setTitleBarVolume(
      musicPlayer.volumeControlValue + (delta > 0 ? -volumeStep : volumeStep),
    );
  }

  function openMusicFromTitleBarMenu(): void {
    showMenu = false;
    nav.navigate("music");
  }
</script>

    <!-- Pomodoro progress ring with dropdown -->
    {#if showPomodoro}
      <div class="relative">
        <button
          onclick={toggleMenu}
          onwheel={handleVolumeWheel}
          class={cn(
            "titlebar-icon-button flex items-center justify-center rounded-lg transition-colors",
            showMenu ? "bg-sidebar-accent" : "hover:bg-sidebar-accent",
          )}
          title={pomodoroButtonTooltip}
          aria-haspopup="menu"
          aria-expanded={showMenu}
        >
          <svg viewBox="0 0 20 20" width={POMODORO_RING_SIZE} height={POMODORO_RING_SIZE}>
            <circle
              cx="10"
              cy="10"
              r="8"
              fill="none"
              stroke-width={POMODORO_RING_STROKE_WIDTH}
              class={TITLE_BAR_SUBTLE_STROKE_CLASS}
            />
            {#if isActive}
              <circle
                cx="10"
                cy="10"
                r="8"
                fill="none"
                stroke-width={POMODORO_RING_STROKE_WIDTH}
                stroke-dasharray={`${((100 - progressPercent()) / 100) * 50.27} 50.27`}
                stroke-linecap="round"
                class={cn(
                  `${TITLE_BAR_ICON_COLOR_CLASS} ${TITLE_BAR_ICON_STROKE_CLASS} -rotate-90 origin-center`,
                  pomodoroPausedPulseActive ? "pomodoro-ring-paused-pulse" : "",
                )}
                style={pomodoroPausedPulseStyle}
              />
            {/if}
          </svg>
        </button>
        {#if showMenu}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="fixed inset-0 z-40"
            onclick={() => { showMenu = false; }}
            onkeydown={(e) => { if (e.key === "Escape") showMenu = false; }}
            onwheel={handleVolumeWheel}
          ></div>
          <div
            class="absolute right-0 top-9 z-50 w-60 max-w-[calc(100vw-1rem)] rounded-lg border border-border bg-popover py-1 shadow-lg"
            onwheel={handleVolumeWheel}
          >
            {#if isActive}
              <div class="px-3 py-1.5 text-xs text-muted-foreground">
                {t("titleBar.pomodoro.left", pomodoro.formattedTime)}
              </div>
            {:else}
              <div class="px-3 py-1.5 text-xs text-muted-foreground">
                {t("titleBar.pomodoro.noActiveSession")}
              </div>
            {/if}
            <button
              onclick={() => {
                if (pomodoro.isRunning) {
                  pomodoro.pause();
                } else {
                  pomodoro.start();
                }
                showMenu = false;
              }}
              disabled={!canPauseResumePomodoro}
              class={cn(
                "flex w-full items-center justify-between gap-4 whitespace-nowrap px-3 py-1.5 text-left text-sm transition-colors",
                canPauseResumePomodoro
                  ? "text-foreground hover:bg-accent"
                  : "cursor-not-allowed text-muted-foreground/50",
              )}
            >
              <span>{pomodoroPauseResumeLabel}</span>
              {#if isActive && !pomodoro.isRunning}
                <PlayIcon
                  class="shrink-0 opacity-70"
                  size={TITLE_BAR_MENU_ICON_SIZE}
                  strokeWidth={TITLE_BAR_MENU_ICON_STROKE_WIDTH}
                />
              {:else}
                <PauseIcon
                  class="shrink-0 opacity-70"
                  size={TITLE_BAR_MENU_ICON_SIZE}
                  strokeWidth={TITLE_BAR_MENU_ICON_STROKE_WIDTH}
                />
              {/if}
            </button>
            <button
              onclick={() => { pomodoro.addFocusTime(); showMenu = false; }}
              disabled={!pomodoro.canAddFocusTime}
              class={cn(
                "flex w-full items-center justify-between gap-4 whitespace-nowrap px-3 py-1.5 text-left text-sm transition-colors",
                pomodoro.canAddFocusTime
                  ? "text-foreground hover:bg-accent"
                  : "cursor-not-allowed text-muted-foreground/50",
              )}
            >
              <span>{t("titleBar.pomodoro.extendFocusMinutes", 3)}</span>
              <ClockPlus
                class="shrink-0 opacity-70"
                size={TITLE_BAR_MENU_ICON_SIZE}
                strokeWidth={TITLE_BAR_MENU_ICON_STROKE_WIDTH}
              />
            </button>
            <button
              onclick={() => { pomodoro.skip(); showMenu = false; }}
              disabled={!canAdvancePomodoro}
              class={cn(
                "flex w-full items-center justify-between gap-4 whitespace-nowrap px-3 py-1.5 text-left text-sm transition-colors",
                canAdvancePomodoro
                  ? "text-foreground hover:bg-accent"
                  : "cursor-not-allowed text-muted-foreground/50",
              )}
            >
              <span>{phaseAdvanceLabel}</span>
              {#if pomodoro.phase === "focus" || !isActive}
                <Coffee
                  class="shrink-0 opacity-70"
                  size={TITLE_BAR_MENU_ICON_SIZE}
                  strokeWidth={TITLE_BAR_MENU_ICON_STROKE_WIDTH}
                />
              {:else}
                <PlayIcon
                  class="shrink-0 opacity-70"
                  size={TITLE_BAR_MENU_ICON_SIZE}
                  strokeWidth={TITLE_BAR_MENU_ICON_STROKE_WIDTH}
                />
              {/if}
            </button>
            {#if isMainWindow}
              <div class="mx-3 my-1.5 h-px bg-border"></div>
              <div class="px-3 pb-1.5 pt-2 text-xs text-muted-foreground">
                <span class="block truncate">{musicStatusText}</span>
              </div>
              <button
                onclick={() => { void musicPlayer.togglePlay(); showMenu = false; }}
                disabled={!canPlayPauseMusic}
                class={cn(
                  "flex w-full items-center justify-between gap-4 whitespace-nowrap px-3 py-1.5 text-left text-sm transition-colors",
                  canPlayPauseMusic
                    ? "text-foreground hover:bg-accent"
                    : "cursor-not-allowed text-muted-foreground/50",
                )}
              >
                <span>{musicPlayPauseLabel}</span>
                {#if musicPlayer.isPlaying}
                  <PauseIcon
                    class="shrink-0 opacity-70"
                    size={TITLE_BAR_MENU_ICON_SIZE}
                    strokeWidth={TITLE_BAR_MENU_ICON_STROKE_WIDTH}
                  />
                {:else}
                  <PlayIcon
                    class="shrink-0 opacity-70"
                    size={TITLE_BAR_MENU_ICON_SIZE}
                    strokeWidth={TITLE_BAR_MENU_ICON_STROKE_WIDTH}
                  />
                {/if}
              </button>
              <button
                onclick={() => { void musicPlayer.playPreviousTrack(); showMenu = false; }}
                disabled={!musicPlayer.canPlayPreviousTrack}
                class={cn(
                  "flex w-full items-center justify-between gap-4 whitespace-nowrap px-3 py-1.5 text-left text-sm transition-colors",
                  musicPlayer.canPlayPreviousTrack
                    ? "text-foreground hover:bg-accent"
                    : "cursor-not-allowed text-muted-foreground/50",
                )}
              >
                <span>{t("titleBar.music.previous")}</span>
                <SkipBack
                  class="shrink-0 opacity-70"
                  size={TITLE_BAR_MENU_ICON_SIZE}
                  strokeWidth={TITLE_BAR_MENU_ICON_STROKE_WIDTH}
                />
              </button>
              <button
                onclick={() => { void musicPlayer.playNextTrack(); showMenu = false; }}
                disabled={!musicPlayer.canPlayNextTrack}
                class={cn(
                  "flex w-full items-center justify-between gap-4 whitespace-nowrap px-3 py-1.5 text-left text-sm transition-colors",
                  musicPlayer.canPlayNextTrack
                    ? "text-foreground hover:bg-accent"
                    : "cursor-not-allowed text-muted-foreground/50",
                )}
              >
                <span>{t("titleBar.music.next")}</span>
                <SkipForward
                  class="shrink-0 opacity-70"
                  size={TITLE_BAR_MENU_ICON_SIZE}
                  strokeWidth={TITLE_BAR_MENU_ICON_STROKE_WIDTH}
                />
              </button>
              <div class="flex items-center gap-3 px-3 py-2 text-sm text-foreground">
                <span class="shrink-0">{t("titleBar.music.volume")}</span>
                <input
                  type="range"
                  min="0"
                  max={musicPlayer.volumeMax}
                  step={titleBarVolumeStep}
                  value={musicPlayer.volumeControlValue}
                  class="titlebar-volume-slider min-w-0 flex-1"
                  style={`--titlebar-volume-progress: ${titleBarVolumeSliderProgress};`}
                  aria-label={t("titleBar.music.volumeLabel")}
                  tabindex="-1"
                  oninput={(event) => { setTitleBarVolume(Number(event.currentTarget.value)); }}
                />
              </div>
              <button
                onclick={openMusicFromTitleBarMenu}
                class="flex w-full items-center justify-between gap-4 whitespace-nowrap px-3 py-1.5 text-left text-sm text-foreground hover:bg-accent"
              >
                <span>{t("titleBar.music.open")}</span>
                <ExternalLink
                  class="shrink-0 opacity-70"
                  size={TITLE_BAR_MENU_ICON_SIZE}
                  strokeWidth={TITLE_BAR_MENU_ICON_STROKE_WIDTH}
                />
              </button>
            {/if}
          </div>
        {/if}
      </div>
    {/if}

    {#if isMainWindow && showMusic}
      <button
        type="button"
        onclick={() => nav.navigate("music")}
        onwheel={handleVolumeWheel}
        class={cn(
          "titlebar-icon-button flex items-center justify-center rounded-lg transition-colors",
          nav.current === "music"
            ? "bg-background text-foreground dark:bg-accent dark:text-white"
            : `${TITLE_BAR_ICON_COLOR_CLASS} hover:bg-sidebar-accent`,
        )}
        title={musicButtonTooltip}
        aria-label={t("titleBar.control.music")}
      >
        <Music size={TITLE_BAR_ICON_SIZE} strokeWidth={TITLE_BAR_ICON_STROKE_WIDTH} />
      </button>
    {/if}

<style>
  .titlebar-icon-button {
    width: 32px;
    height: 32px;
  }

  .pomodoro-ring-paused-pulse {
    stroke: color-mix(
      in srgb,
      color-mix(in srgb, currentColor 30%, transparent)
        var(--pomodoro-ring-paused-pulse-amount, 0%),
      currentColor
    );
  }

  @media (prefers-reduced-motion: reduce) {
    .pomodoro-ring-paused-pulse {
      stroke: color-mix(in srgb, currentColor 50%, transparent);
    }
  }

  .titlebar-volume-slider {
    --titlebar-volume-thumb-size: 0.5rem;
    --titlebar-volume-track-height: 0.125rem;
    --titlebar-volume-track-color: color-mix(in srgb, var(--foreground) 18%, transparent);
    --titlebar-volume-fill-color: color-mix(in srgb, var(--foreground) 58%, transparent);
    height: var(--titlebar-volume-thumb-size);
    appearance: none;
    cursor: pointer;
    background: linear-gradient(
      to right,
      var(--titlebar-volume-fill-color) 0%,
      var(--titlebar-volume-fill-color) var(--titlebar-volume-progress),
      var(--titlebar-volume-track-color) var(--titlebar-volume-progress),
      var(--titlebar-volume-track-color) 100%
    ) center / calc(100% - var(--titlebar-volume-thumb-size)) var(--titlebar-volume-track-height) no-repeat;
  }

  .titlebar-volume-slider::-webkit-slider-runnable-track {
    height: var(--titlebar-volume-track-height);
    border-radius: 999px;
    background: transparent;
  }

  .titlebar-volume-slider::-webkit-slider-thumb {
    width: var(--titlebar-volume-thumb-size);
    height: var(--titlebar-volume-thumb-size);
    margin-top: calc((var(--titlebar-volume-track-height) - var(--titlebar-volume-thumb-size)) / 2);
    appearance: none;
    border: 0;
    border-radius: 999px;
    background: var(--foreground);
  }

  .titlebar-volume-slider::-moz-range-track,
  .titlebar-volume-slider::-moz-range-progress {
    height: var(--titlebar-volume-track-height);
    border-radius: 999px;
    background: transparent;
  }

  .titlebar-volume-slider::-moz-range-thumb {
    width: var(--titlebar-volume-thumb-size);
    height: var(--titlebar-volume-thumb-size);
    border: 0;
    border-radius: 999px;
    background: var(--foreground);
  }
</style>
