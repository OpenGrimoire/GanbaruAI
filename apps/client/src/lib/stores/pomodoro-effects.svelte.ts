import type { PomodoroPhase } from "@ganbaru-ai/shared-types";
import { invoke } from "@tauri-apps/api/core";

import { APP_SOUND_IDS, playAppSound } from "$lib/app-sounds";
import { getMusicPlayer } from "$lib/stores/music-player.svelte";
import { getPreferences } from "$lib/stores/preferences.svelte";
import { createPomodoroNativeTrayPolicy } from "./pomodoro-native-update-policy";
import { shouldResumePomodoroPausedMusic } from "$lib/music/music-automation-ownership";
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

interface PomodoroTrayUpdatePayload {
  phase: PomodoroPhase;
  remainingSeconds: number;
  totalSeconds: number;
  isRunning: boolean;
  isActive: boolean;
  canPauseResume: boolean;
  canAddFocusTime: boolean;
  pausedPulseFrame: number | null;
}

const PAUSED_PULSE_AMOUNTS = [
  0, 0, 0, 0, 0, 0.067, 0.25, 0.5, 0.75, 0.933, 1, 1, 1, 1, 1, 1,
  0.933, 0.75, 0.5, 0.25, 0.067, 0,
] as const;
const PAUSED_TRAY_PULSE_FRAME_COUNT = PAUSED_PULSE_AMOUNTS.length;
const PAUSED_TRAY_PULSE_FRAME_MS = 180;

export function createPomodoroEffects(context: PomodoroEffectsContext): PomodoroEffects {
  const nativeTrayPolicy = createPomodoroNativeTrayPolicy();
  let pausedTrayPulseFrame = $state(0);
  let pausedTrayPulseIntervalId: ReturnType<typeof setInterval> | null = null;
  let breakEndWarningTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let musicPausedByPomodoroPause = false;
  let musicPauseInFlight: Promise<void> | null = null;
  let musicManualActionVersionAtPause: number | null = null;
  let pausedFocusNotificationTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let pausedFocusNotificationSuppressed = false;

  function stopPausedTrayPulse(): void {
    if (pausedTrayPulseIntervalId !== null) {
      clearInterval(pausedTrayPulseIntervalId);
      pausedTrayPulseIntervalId = null;
    }
    pausedTrayPulseFrame = 0;
  }

  function currentPausedTrayPulseFrame(): number | null {
    return context.pausedFocusPulseActive() ? pausedTrayPulseFrame : null;
  }

  function currentPausedPulseAmount(): number | null {
    const frame = currentPausedTrayPulseFrame();
    if (frame === null) return null;
    return PAUSED_PULSE_AMOUNTS[frame % PAUSED_TRAY_PULSE_FRAME_COUNT];
  }

  function syncPausedTrayPulse(): void {
    if (!context.isCoordinator()) return;
    if (!context.pausedFocusPulseActive()) {
      stopPausedTrayPulse();
      return;
    }
    if (pausedTrayPulseIntervalId !== null) return;

    pausedTrayPulseFrame = 0;
    pausedTrayPulseIntervalId = setInterval(() => {
      if (!context.pausedFocusPulseActive()) {
        stopPausedTrayPulse();
        updateTray({ publishSnapshot: false });
        return;
      }

      pausedTrayPulseFrame = (pausedTrayPulseFrame + 1) % PAUSED_TRAY_PULSE_FRAME_COUNT;
      updateTray({ publishSnapshot: false });
    }, PAUSED_TRAY_PULSE_FRAME_MS);
  }

  function clearPausedFocusNotificationTimeout(): void {
    if (pausedFocusNotificationTimeoutId === null) return;
    clearTimeout(pausedFocusNotificationTimeoutId);
    pausedFocusNotificationTimeoutId = null;
  }

  function resetPausedFocusNotificationState(): void {
    clearPausedFocusNotificationTimeout();
    pausedFocusNotificationSuppressed = false;
  }

  function suppressPausedFocusNotificationsForCurrentPause(): void {
    pausedFocusNotificationSuppressed = true;
    clearPausedFocusNotificationTimeout();
  }

  function pausedFocusNotificationIntervalMs(): number {
    const minutes = getPreferences().focusPauseNotificationIntervalMinutes;
    return minutes > 0 ? minutes * 60_000 : 0;
  }

  function showPausedFocusNotification(): void {
    invoke("show_paused_focus_notification").catch((error) => {
      console.warn("Failed to show paused focus notification:", error);
    });
  }

  function scheduleNextPausedFocusNotification(): void {
    if (pausedFocusNotificationTimeoutId !== null) return;
    if (pausedFocusNotificationSuppressed || !context.pausedFocusPulseActive()) return;

    const delayMs = pausedFocusNotificationIntervalMs();
    if (delayMs <= 0) return;

    pausedFocusNotificationTimeoutId = setTimeout(() => {
      pausedFocusNotificationTimeoutId = null;
      if (pausedFocusNotificationSuppressed || !context.pausedFocusPulseActive()) return;
      if (pausedFocusNotificationIntervalMs() <= 0) return;
      showPausedFocusNotification();
      scheduleNextPausedFocusNotification();
    }, delayMs);
  }

  function syncPausedFocusNotification(): void {
    if (!context.isCoordinator()) return;
    if (
      pausedFocusNotificationSuppressed ||
      !context.pausedFocusPulseActive() ||
      pausedFocusNotificationIntervalMs() <= 0
    ) {
      clearPausedFocusNotificationTimeout();
      return;
    }
    scheduleNextPausedFocusNotification();
  }

  function updateTray(options: PomodoroTrayUpdateOptions = {}): void {
    if (options.publishSnapshot !== false) context.publishWindowSnapshot();
    if (!context.isCoordinator()) return;
    if (!context.desktopIntegrationsAvailable()) return;
    context.writeDoomscrollingRuntimeState();
    syncPausedTrayPulse();
    syncPausedFocusNotification();
    const update: PomodoroTrayUpdatePayload = {
      phase: context.phase(),
      remainingSeconds: context.remainingSeconds(),
      totalSeconds: context.totalSeconds(),
      isRunning: context.isRunning(),
      isActive: context.isActive(),
      canPauseResume: context.canPauseResume(),
      canAddFocusTime: context.canAddFocusTime(),
      pausedPulseFrame: currentPausedTrayPulseFrame(),
    };

    if (!nativeTrayPolicy.shouldSend(update)) return;

    invoke("update_tray", { update }).catch(() => {});
  }

  function clearBreakEndWarning(): void {
    if (breakEndWarningTimeoutId === null) return;
    clearTimeout(breakEndWarningTimeoutId);
    breakEndWarningTimeoutId = null;
  }

  function scheduleBreakEndWarning(): void {
    clearBreakEndWarning();
    if (
      !context.isRunning() ||
      (context.phase() !== "short_break" && context.phase() !== "long_break") ||
      context.phaseEndTime() === null
    ) {
      return;
    }

    const warningSeconds = getPreferences().focusBreakEndWarningSeconds;
    if (warningSeconds <= 0) return;

    const targetMs = context.phaseEndTime()! - warningSeconds * 1000;
    const delayMs = targetMs - Date.now();
    if (delayMs <= 0) return;

    breakEndWarningTimeoutId = setTimeout(() => {
      breakEndWarningTimeoutId = null;
      if (
        !context.isRunning() ||
        (context.phase() !== "short_break" && context.phase() !== "long_break") ||
        context.phaseEndTime() === null
      ) {
        return;
      }
      playBreakFinishedAlert();
    }, delayMs);
  }

  function clearMusicPausedByPomodoro(): void {
    musicPausedByPomodoroPause = false;
    musicPauseInFlight = null;
    musicManualActionVersionAtPause = null;
  }

  function pauseMusicForPomodoroPause(): void {
    if (
      !context.desktopIntegrationsAvailable()
      || !context.isCoordinator()
      || context.phase() !== "focus"
      || !context.isActive()
      || !getPreferences().musicPauseOnPomodoroPause
    ) {
      clearMusicPausedByPomodoro();
      return;
    }
    const music = getMusicPlayer();
    if (!music.isPlaying) {
      clearMusicPausedByPomodoro();
      return;
    }
    musicPausedByPomodoroPause = true;
    musicManualActionVersionAtPause = music.manualPlaybackActionVersion;
    const trackedPause = music.pausePlayback("pomodoro-pause").catch((error) => {
      console.warn("Failed to pause music with pomodoro:", error);
    });
    musicPauseInFlight = trackedPause;
    void trackedPause.finally(() => {
      if (musicPauseInFlight === trackedPause) musicPauseInFlight = null;
    });
  }

  function resumeMusicFromPomodoroPause(): void {
    if (!musicPausedByPomodoroPause) return;
    musicPausedByPomodoroPause = false;
    if (
      !context.desktopIntegrationsAvailable()
      || !context.isCoordinator()
      || !getPreferences().musicPauseOnPomodoroPause
    ) {
      clearMusicPausedByPomodoro();
      return;
    }
    const music = getMusicPlayer();
    const pausePromise = musicPauseInFlight;
    const manualActionVersionAtPause = musicManualActionVersionAtPause;
    musicPauseInFlight = null;
    musicManualActionVersionAtPause = null;
    void (async () => {
      if (pausePromise) await pausePromise;
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
    })().catch((error) => {
      console.warn("Failed to resume music with pomodoro:", error);
    });
  }

  function showBreakOverlay(breakSeconds: number): void {
    if (!context.desktopIntegrationsAvailable()) return;
    context.initListeners();
    const breakEndsAtMs = Math.max(
      0,
      Math.floor(context.phaseEndTime() ?? (Date.now() + breakSeconds * 1000)),
    );
    invoke("show_break_overlay", {
      breakEndsAtMs,
      breakEndEscPresses: getPreferences().focusBreakEndEscPresses,
      breakExtensionLimit: getPreferences().focusBreakExtensionLimit,
    }).catch((e) =>
      console.warn("Failed to show break overlay:", e),
    );
    scheduleBreakEndWarning();
  }

  function closePomodoroOverlay(): void {
    if (!context.desktopIntegrationsAvailable()) return;
    invoke("close_pomodoro_overlay").catch((e) =>
      console.warn("Failed to close pomodoro overlay:", e),
    );
  }

  function showNotification(): void {
    if (context.notificationShown()) return;
    context.setNotificationShown(true);
    if (!context.desktopIntegrationsAvailable()) return;

    invoke("show_pomodoro_notification", {
      remainingSeconds: 60,
      allowAddTime: context.canAddFocusTime(),
    }).catch((e) => {
      console.warn("Failed to show notification:", e);
      context.setNotificationShown(false);
    });
  }

  function playBreakFinishedAlert(): void {
    if (!context.desktopIntegrationsAvailable()) return;
    playAppSound(APP_SOUND_IDS.breakFinished).catch(() => {});
  }

  function startConfiguredBreakFinishedAlertInterval(): ReturnType<typeof setInterval> | null {
    if (!context.desktopIntegrationsAvailable()) return null;
    const repeatSeconds = getPreferences().focusBreakFinishedRepeatSeconds;
    if (repeatSeconds <= 0) return null;
    return setInterval(playBreakFinishedAlert, repeatSeconds * 1000);
  }

  return {
    currentPausedTrayPulseFrame,
    currentPausedPulseAmount,
    clearBreakEndWarning,
    scheduleBreakEndWarning,
    clearMusicPausedByPomodoro,
    pauseMusicForPomodoroPause,
    resumeMusicFromPomodoroPause,
    resetPausedFocusNotificationState,
    suppressPausedFocusNotificationsForCurrentPause,
    updateTray,
    showBreakOverlay,
    closePomodoroOverlay,
    showNotification,
    playBreakFinishedAlert,
    startConfiguredBreakFinishedAlertInterval,
  };
}
