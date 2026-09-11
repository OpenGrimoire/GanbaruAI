import { emit, listen } from "@tauri-apps/api/event";

import {
  createWindowSyncEnvelope,
  isForeignWindowSyncEnvelope,
  isWindowSyncEnvelope,
} from "$lib/window-sync";
import {
  isPomodoroWindowCommand,
  isPomodoroWindowSnapshot,
  type PomodoroWindowCommand,
} from "./pomodoro-window-sync";
import type {
  PomodoroWindowCoordinator,
  PomodoroWindowCoordinatorContext,
} from "./pomodoro-window-coordinator-contract";

export type {
  PomodoroWindowCoordinator,
  PomodoroWindowCoordinatorContext,
} from "./pomodoro-window-coordinator-contract";

const POMODORO_WINDOW_SYNC_EVENT = "pomodoro-window-sync";
const POMODORO_WINDOW_COMMAND_EVENT = "pomodoro-window-command";

export function createPomodoroWindowCoordinator(
  context: PomodoroWindowCoordinatorContext,
): PomodoroWindowCoordinator {
  let initialized = false;

  function publishSnapshot(): void {
    if (!context.isCoordinator()) return;
    context.beforePublishSnapshot();
    emit(
      POMODORO_WINDOW_SYNC_EVENT,
      createWindowSyncEnvelope(context.buildSnapshot()),
    ).catch((err) => console.warn("pomodoro window sync failed", err));
  }

  function sendCommand(command: PomodoroWindowCommand): void {
    emit(
      POMODORO_WINDOW_COMMAND_EVENT,
      createWindowSyncEnvelope(command),
    ).catch((err) => console.warn("pomodoro window command failed", err));
  }

  function forwardCommand(command: PomodoroWindowCommand): boolean {
    if (context.isCoordinator()) return false;
    sendCommand(command);
    return true;
  }

  function init(): void {
    if (initialized) return;
    initialized = true;

    listen<unknown>(POMODORO_WINDOW_SYNC_EVENT, (event) => {
      const envelope = event.payload;
      if (!isWindowSyncEnvelope(envelope, isPomodoroWindowSnapshot)) return;
      if (!isForeignWindowSyncEnvelope(envelope)) return;
      context.applySnapshot(envelope.payload);
    }).catch((err) => console.warn("pomodoro window sync listener failed", err));

    listen<unknown>(POMODORO_WINDOW_COMMAND_EVENT, (event) => {
      const envelope = event.payload;
      if (!isWindowSyncEnvelope(envelope, isPomodoroWindowCommand)) return;
      if (!isForeignWindowSyncEnvelope(envelope)) return;
      context.handleCommand(envelope.payload);
    }).catch((err) => console.warn("pomodoro window command listener failed", err));

    if (!context.isCoordinator()) {
      sendCommand({ kind: "request-snapshot" });
    } else {
      publishSnapshot();
    }
  }

  return {
    publishSnapshot,
    sendCommand,
    forwardCommand,
    init,
  };
}
