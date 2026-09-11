import type {
  PomodoroWindowCommand,
  PomodoroWindowSnapshot,
} from "./pomodoro-window-sync";

export interface PomodoroWindowCoordinatorContext {
  isCoordinator(): boolean;
  beforePublishSnapshot(): void;
  buildSnapshot(): PomodoroWindowSnapshot;
  applySnapshot(snapshot: PomodoroWindowSnapshot): void;
  handleCommand(command: PomodoroWindowCommand): void;
}

export interface PomodoroWindowCoordinator {
  publishSnapshot(): void;
  sendCommand(command: PomodoroWindowCommand): void;
  forwardCommand(command: PomodoroWindowCommand): boolean;
  init(): void;
}
