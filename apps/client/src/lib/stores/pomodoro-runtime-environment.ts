import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  PomodoroNativeEventListener,
  PomodoroRuntimeEnvironment,
} from "./pomodoro-runtime-environment-contract";

export type {
  PomodoroNativeEvent,
  PomodoroNativeEventListener,
  PomodoroRuntimeEnvironment,
} from "./pomodoro-runtime-environment-contract";

const nativeEventListener: PomodoroNativeEventListener = (eventName, handler) =>
  listen(eventName, handler);

/** Describe desktop coordination and native Pomodoro event delivery. */
export function getPomodoroRuntimeEnvironment(): PomodoroRuntimeEnvironment {
  return {
    isCoordinator: getCurrentWindow().label === "main",
    nativeEventListener,
  };
}
