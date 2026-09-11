import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PomodoroEffectsContext } from "./pomodoro-effects-contracts";

const preferenceState = vi.hoisted(() => ({ pauseMusic: true }));
const musicState = vi.hoisted(() => ({
  isPlaying: true,
  isBusy: false,
  currentSource: { identity: "local:test" } as { identity: string } | null,
  manualPlaybackActionVersion: 0,
  pausePlayback: vi.fn<() => Promise<void>>(),
  playPlayback: vi.fn<() => Promise<void>>(),
}));

vi.mock("$lib/stores/preferences.svelte", () => ({
  getPreferences: () => ({ musicPauseOnPomodoroPause: preferenceState.pauseMusic }),
}));

vi.mock("$lib/stores/music-player.svelte", () => ({
  getMusicPlayer: () => musicState,
}));

import { createPomodoroEffects } from "./pomodoro-effects.mobile.svelte";

function context(): PomodoroEffectsContext {
  return {
    isCoordinator: () => true,
    phase: () => "focus",
    remainingSeconds: () => 60,
    totalSeconds: () => 60,
    isRunning: () => false,
    phaseEndTime: () => null,
    isActive: () => true,
    canPauseResume: () => true,
    canAddFocusTime: () => false,
    pausedFocusPulseActive: () => false,
    desktopIntegrationsAvailable: () => false,
    mobileNotificationState: () => null,
    notificationShown: () => false,
    setNotificationShown: () => {},
    publishWindowSnapshot: () => {},
    writeDoomscrollingRuntimeState: () => {},
    initListeners: () => {},
  };
}

describe("mobile Pomodoro music automation", () => {
  beforeEach(() => {
    preferenceState.pauseMusic = true;
    musicState.isPlaying = true;
    musicState.isBusy = false;
    musicState.currentSource = { identity: "local:test" };
    musicState.manualPlaybackActionVersion = 0;
    musicState.pausePlayback.mockReset().mockImplementation(async () => {
      musicState.isPlaying = false;
    });
    musicState.playPlayback.mockReset().mockImplementation(async () => {
      musicState.isPlaying = true;
    });
  });

  it("pauses and resumes music owned by a Focus pause", async () => {
    const effects = createPomodoroEffects(context());

    effects.pauseMusicForPomodoroPause();
    await vi.waitFor(() => expect(musicState.pausePlayback).toHaveBeenCalledWith("pomodoro-pause"));

    effects.resumeMusicFromPomodoroPause();
    await vi.waitFor(() => expect(musicState.playPlayback).toHaveBeenCalledWith("pomodoro-pause"));
  });

  it("does not resume after a manual playback action", async () => {
    const effects = createPomodoroEffects(context());

    effects.pauseMusicForPomodoroPause();
    await vi.waitFor(() => expect(musicState.pausePlayback).toHaveBeenCalledOnce());
    musicState.manualPlaybackActionVersion += 1;

    effects.resumeMusicFromPomodoroPause();
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(musicState.playPlayback).not.toHaveBeenCalled();
  });

  it("leaves playing music alone when the preference is disabled", async () => {
    preferenceState.pauseMusic = false;
    const effects = createPomodoroEffects(context());

    effects.pauseMusicForPomodoroPause();
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(musicState.pausePlayback).not.toHaveBeenCalled();
  });
});
