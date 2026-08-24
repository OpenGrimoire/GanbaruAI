import type { PomodoroPhase } from "@ganbaru-ai/shared-types";

export interface PomodoroTrayUpdateOptions {
  publishSnapshot?: boolean;
}

export interface PomodoroEffectsContext {
  isCoordinator(): boolean;
  phase(): PomodoroPhase;
  remainingSeconds(): number;
  totalSeconds(): number;
  isRunning(): boolean;
  phaseEndTime(): number | null;
  isActive(): boolean;
  canPauseResume(): boolean;
  canAddFocusTime(): boolean;
  pausedFocusPulseActive(): boolean;
  desktopIntegrationsAvailable(): boolean;
  notificationShown(): boolean;
  setNotificationShown(value: boolean): void;
  publishWindowSnapshot(): void;
  writeDoomscrollingRuntimeState(force?: boolean): void;
  initListeners(): void;
}

export interface PomodoroEffects {
  currentPausedTrayPulseFrame(): number | null;
  currentPausedPulseAmount(): number | null;
  clearBreakEndWarning(): void;
  scheduleBreakEndWarning(): void;
  clearMusicPausedByPomodoro(): void;
  pauseMusicForPomodoroPause(): void;
  resumeMusicFromPomodoroPause(): void;
  resetPausedFocusNotificationState(): void;
  suppressPausedFocusNotificationsForCurrentPause(): void;
  updateTray(options?: PomodoroTrayUpdateOptions): void;
  showBreakOverlay(breakSeconds: number): void;
  closePomodoroOverlay(): void;
  showNotification(): void;
  playBreakFinishedAlert(): void;
  startConfiguredBreakFinishedAlertInterval(): ReturnType<typeof setInterval> | null;
}
