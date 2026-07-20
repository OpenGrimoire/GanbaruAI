// @vitest-environment jsdom

import { afterEach, describe, expect, it } from "vitest";
import {
  loadMusicPlayerSettings,
  persistMusicPlayerSettings,
} from "./music-player-settings";

const settingsKey = "ganbaru-ai-music-player";

describe("Music player settings", () => {
  afterEach(() => {
    localStorage.clear();
  });

  it("keeps the playlist closed when no visibility preference has been saved", () => {
    expect(loadMusicPlayerSettings().playlistVisible).toBe(false);
  });

  it("loads a saved open playlist preference", () => {
    localStorage.setItem(settingsKey, JSON.stringify({ playlistVisible: true }));
    expect(loadMusicPlayerSettings().playlistVisible).toBe(true);
  });

  it("treats malformed playlist visibility as closed", () => {
    localStorage.setItem(settingsKey, JSON.stringify({ playlistVisible: "true" }));
    expect(loadMusicPlayerSettings().playlistVisible).toBe(false);
  });

  it("persists playlist visibility with the existing Music settings", () => {
    persistMusicPlayerSettings({
      volume: 0.8,
      rate: 1,
      shuffleEnabled: true,
      shuffleExplicit: false,
      muted: false,
      playlistVisible: true,
    });
    expect(JSON.parse(localStorage.getItem(settingsKey) ?? "{}")).toMatchObject({
      playlistVisible: true,
    });
  });
});
