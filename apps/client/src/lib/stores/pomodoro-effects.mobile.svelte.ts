import { invoke } from "@tauri-apps/api/core";
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
    clearMusicPausedByPomodoro: () => {},
    pauseMusicForPomodoroPause: () => {},
    resumeMusicFromPomodoroPause: () => {},
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
