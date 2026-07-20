export type MusicAutomationActionOrigin = "manual" | "context" | "pomodoro-pause" | "system";
export type MusicAutomationContextState = "playing" | "prepared" | "paused" | "kept" | "unavailable" | "overridden";

/** Manual transport actions take ownership, while system actions preserve contextual ownership. */
export function musicContextStateAfterAction(
  state: MusicAutomationContextState | null,
  origin: MusicAutomationActionOrigin,
): MusicAutomationContextState | null {
  if (state === null || state === "overridden" || origin !== "manual") return state;
  return "overridden";
}

export interface PomodoroMusicResumeInput {
  pauseOwned: boolean;
  manualActionVersionAtPause: number | null;
  currentManualActionVersion: number;
  coordinator: boolean;
  preferenceEnabled: boolean;
  hasSource: boolean;
  playing: boolean;
  busy: boolean;
}

/** Resumes only the music paused by this focus pause, unless the user acted afterward. */
export function shouldResumePomodoroPausedMusic(input: PomodoroMusicResumeInput): boolean {
  return input.pauseOwned
    && input.manualActionVersionAtPause !== null
    && input.manualActionVersionAtPause === input.currentManualActionVersion
    && input.coordinator
    && input.preferenceEnabled
    && input.hasSource
    && !input.playing
    && !input.busy;
}
