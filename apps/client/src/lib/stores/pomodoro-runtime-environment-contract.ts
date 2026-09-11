export interface PomodoroNativeEvent<Payload> {
  payload: Payload;
}

export type PomodoroNativeEventListener = <Payload>(
  eventName: string,
  handler: (event: PomodoroNativeEvent<Payload>) => void,
) => Promise<() => void>;

export interface PomodoroRuntimeEnvironment {
  readonly isCoordinator: boolean;
  readonly nativeEventListener: PomodoroNativeEventListener | null;
}
