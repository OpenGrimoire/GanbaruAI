// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import { MusicFocusRecovery } from "./music-focus-recovery";

describe("music focus recovery", () => {
  it("returns focus to the remembered row after a surface closes", async () => {
    const root = document.createElement("div");
    const button = document.createElement("button");
    button.dataset.musicFocusKey = "item:1";
    root.append(button);
    document.body.append(root);
    const recovery = new MusicFocusRecovery();
    recovery.remember(button);
    expect(await recovery.restore(root)).toBe(true);
    expect(document.activeElement).toBe(button);
    root.remove();
  });

  it("fails safely when filtering removed the intended target", async () => {
    const recovery = new MusicFocusRecovery();
    recovery.rememberKey("missing");
    expect(await recovery.restore(document)).toBe(false);
  });
});
