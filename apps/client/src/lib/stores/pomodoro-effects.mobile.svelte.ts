import { invoke } from "@tauri-apps/api/core";
import { getPreferences } from "$lib/stores/preferences.svelte";
import type {
  PomodoroEffects,
  PomodoroEffectsContext,
  PomodoroTrayUpdateOptions,
} from "./pomodoro-effects-contracts";

export type {
  PomodoroEffects,
  PomodoroEffectsContext,
  PomodoroTrayUpdateOptions,
} from "./pomodoro-effects-contracts";

/** Provide mobile timer effects without desktop media, tray, or overlay code. */
export function createPomodoroEffects(context: PomodoroEffectsContext): PomodoroEffects {
  let nativeUpdateGeneration = 0;
  let previousNativeKey: string | null = null;
  let musicAutomationGeneration = 0;
  let musicAutomationQueue: Promise<void> = Promise.resolve();
  let musicPausedByPomodoroPause = false;
  let musicManualActionVersionAtPause: number | null = null;

  function queueMusicAutomation(generation: number, action: () => Promise<void>): void {
    musicAutomationQueue = musicAutomationQueue
      .then(async () => {
        if (generation !== musicAutomationGeneration) return;
        await action();
      })
      .catch((error: unknown) => {
        console.warn("Failed to apply Android Pomodoro music automation:", error);
      });
  }

  function clearMusicPausedByPomodoro(): void {
    musicAutomationGeneration += 1;
    musicPausedByPomodoroPause = false;
    musicManualActionVersionAtPause = null;
  }

  function pauseMusicForPomodoroPause(): void {
    if (
      !context.isCoordinator()
      || context.phase() !== "focus"
      || !context.isActive()
      || !getPreferences().musicPauseOnPomodoroPause
    ) {
      clearMusicPausedByPomodoro();
      return;
    }

    const generation = ++musicAutomationGeneration;
    queueMusicAutomation(generation, async () => {
      const { getMusicPlayer } = await import("$lib/stores/music-player.svelte");
      if (generation !== musicAutomationGeneration) return;
      const music = getMusicPlayer();
      if (!music.isPlaying) {
        musicPausedByPomodoroPause = false;
        musicManualActionVersionAtPause = null;
        return;
      }
      musicPausedByPomodoroPause = true;
      musicManualActionVersionAtPause = music.manualPlaybackActionVersion;
      try {
        await music.pausePlayback("pomodoro-pause");
      } catch (error: unknown) {
        if (generation === musicAutomationGeneration) {
          musicPausedByPomodoroPause = false;
          musicManualActionVersionAtPause = null;
        }
        throw error;
      }
    });
  }

  function resumeMusicFromPomodoroPause(): void {
    const generation = ++musicAutomationGeneration;
    queueMusicAutomation(generation, async () => {
      if (!musicPausedByPomodoroPause) return;
      musicPausedByPomodoroPause = false;
      const manualActionVersionAtPause = musicManualActionVersionAtPause;
      musicManualActionVersionAtPause = null;
      const [{ getMusicPlayer }, { shouldResumePomodoroPausedMusic }] = await Promise.all([
        import("$lib/stores/music-player.svelte"),
        import("$lib/music/music-automation-ownership"),
      ]);
      if (generation !== musicAutomationGeneration) return;
      const music = getMusicPlayer();
      if (!shouldResumePomodoroPausedMusic({
        pauseOwned: true,
        manualActionVersionAtPause,
        currentManualActionVersion: music.manualPlaybackActionVersion,
        coordinator: context.isCoordinator(),
        preferenceEnabled: getPreferences().musicPauseOnPomodoroPause,
        hasSource: Boolean(music.currentSource),
        playing: music.isPlaying,
        busy: music.isBusy,
      })) return;
      await music.playPlayback("pomodoro-pause");
    });
  }

  function nativeStateKey(state: ReturnType<PomodoroEffectsContext["mobileNotificationState"]>): string {
    if (!state) return "inactive";
    const progress = Math.floor(
      Math.max(0, Math.min(1, 1 - state.remainingSeconds / state.totalSeconds)) * 100,
    );
    return JSON.stringify({
      runId: state.runId,
      eventId: state.eventId,
      eventTitle: state.eventTitle,
      eventEndsAtEpochMs: state.eventEndsAtEpochMs,
      isRunning: state.isRunning,
      totalSeconds: state.totalSeconds,
      configJson: state.configJson,
      progress,
      phases: state.phases,
      copy: state.copy,
    });
  }

  function syncNativeNotification(): void {
    const state = context.mobileNotificationState();
    const key = nativeStateKey(state);
    if (key === previousNativeKey) return;
    previousNativeKey = key;
    const generation = ++nativeUpdateGeneration;
    const request = state
      ? invoke("plugin:ganbaru-mobile-notifications|updatePomodoroNotification", { state })
      : invoke("plugin:ganbaru-mobile-notifications|cancelPomodoroNotification");
    void request.catch((error) => {
      if (generation === nativeUpdateGeneration) {
        console.warn("Failed to update Android Pomodoro notification:", error);
      }
    });
  }

  return {
    currentPausedTrayPulseFrame: () => null,
    currentPausedPulseAmount: () => null,
    clearBreakEndWarning: () => {},
    scheduleBreakEndWarning: () => {},
    clearMusicPausedByPomodoro,
    pauseMusicForPomodoroPause,
    resumeMusicFromPomodoroPause,
    resetPausedFocusNotificationState: () => {},
    suppressPausedFocusNotificationsForCurrentPause: () => {},
    updateTray: (options: PomodoroTrayUpdateOptions = {}) => {
      if (options.publishSnapshot !== false) context.publishWindowSnapshot();
      if (context.isCoordinator()) syncNativeNotification();
    },
    showBreakOverlay: () => {},
    closePomodoroOverlay: () => {},
    showNotification: () => {
      if (!context.notificationShown()) context.setNotificationShown(true);
    },
    playBreakFinishedAlert: () => {},
    startConfiguredBreakFinishedAlertInterval: () => null,
  };
}
