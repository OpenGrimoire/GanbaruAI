import type { PomodoroRuntimeEnvironment } from "./pomodoro-runtime-environment-contract";

export type {
  PomodoroNativeEvent,
  PomodoroNativeEventListener,
  PomodoroRuntimeEnvironment,
} from "./pomodoro-runtime-environment-contract";

const MOBILE_POMODORO_ENVIRONMENT: PomodoroRuntimeEnvironment = Object.freeze({
  isCoordinator: true,
  nativeEventListener: null,
});

/** Describe the single-webview mobile runtime without desktop native event listeners. */
export function getPomodoroRuntimeEnvironment(): PomodoroRuntimeEnvironment {
  return MOBILE_POMODORO_ENVIRONMENT;
}
