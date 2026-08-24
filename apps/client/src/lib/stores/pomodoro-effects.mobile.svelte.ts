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

/** Provide foreground-only timer effects without desktop media, tray, or overlay code. */
export function createPomodoroEffects(context: PomodoroEffectsContext): PomodoroEffects {
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
