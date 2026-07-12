export interface DoomscrollingObservationPlan {
  coordinatorEnabled: boolean;
  observeForeground: boolean;
  scanProcesses: boolean;
}

/** Select the bounded observations for one main-window coordinator interval. */
export function doomscrollingObservationPlan(
  isMainWindow: boolean,
  desktopBlockingEnabled: boolean,
  desktopUsageEnabled: boolean,
): DoomscrollingObservationPlan {
  if (!isMainWindow) {
    return { coordinatorEnabled: false, observeForeground: false, scanProcesses: false };
  }
  return {
    coordinatorEnabled: desktopBlockingEnabled || desktopUsageEnabled,
    observeForeground: desktopUsageEnabled,
    scanProcesses: desktopBlockingEnabled,
  };
}
