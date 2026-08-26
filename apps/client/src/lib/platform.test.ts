import { describe, expect, it } from "vitest";
import {
  applyPlatformProfileToDocument,
  platformHasCapability,
  platformProfileFor,
  resolveBuildPlatform,
  type PlatformDatasetTarget,
} from "./platform";

describe("platform profiles", () => {
  it("uses a desktop fallback for missing or unknown build platforms", () => {
    expect(resolveBuildPlatform(undefined)).toBe("linux");
    expect(resolveBuildPlatform("unknown", "windows")).toBe("windows");
    expect(resolveBuildPlatform("android", "macos")).toBe("android");
  });

  it("preserves every current desktop capability", () => {
    const profile = platformProfileFor("linux");

    expect(profile.shell).toBe("desktop");
    expect(platformHasCapability(profile, "view.calendar")).toBe(true);
    expect(platformHasCapability(profile, "view.projects")).toBe(true);
    expect(platformHasCapability(profile, "view.notes")).toBe(true);
    expect(platformHasCapability(profile, "view.chat")).toBe(true);
    expect(platformHasCapability(profile, "view.music")).toBe(true);
    expect(platformHasCapability(profile, "chat.local-execution")).toBe(true);
    expect(platformHasCapability(profile, "content.managed-image-downloads")).toBe(true);
    expect(platformHasCapability(profile, "notes.external-image-references")).toBe(false);
    expect(platformHasCapability(profile, "music.context-assignments")).toBe(true);
    expect(platformHasCapability(profile, "music.local-file-reveal")).toBe(true);
    expect(platformHasCapability(profile, "music.local-item-repair")).toBe(true);
    expect(platformHasCapability(profile, "music.local-root-relink-plans")).toBe(true);
    expect(platformHasCapability(profile, "music.local-root-reselection")).toBe(false);
    expect(platformHasCapability(profile, "music.soundscapes")).toBe(true);
    expect(platformHasCapability(profile, "notifications.native-scheduling")).toBe(true);
    expect(platformHasCapability(profile, "pomodoro.native-idle-detection")).toBe(true);
    expect(platformHasCapability(profile, "projects.working-folders")).toBe(true);
    expect(platformHasCapability(profile, "storage.native-file-picker")).toBe(true);
    expect(platformHasCapability(profile, "notes.file-export")).toBe(true);
    expect(platformHasCapability(profile, "runtime.desktop-pomodoro-effects")).toBe(true);
    expect(platformHasCapability(profile, "system.android-back")).toBe(false);
    expect(platformHasCapability(profile, "window.desktop-title-bar")).toBe(true);
    expect(platformHasCapability(profile, "window.detached-views")).toBe(true);
  });

  it("limits the initial Android shell to its supported views", () => {
    const profile = platformProfileFor("android");

    expect(profile.shell).toBe("mobile");
    expect(profile.capabilities).toEqual([
      "view.calendar",
      "view.projects",
      "view.notes",
      "view.chat",
      "view.music",
      "music.local-root-reselection",
      "system.android-back",
    ]);
    expect(platformHasCapability(profile, "view.chat")).toBe(true);
    expect(platformHasCapability(profile, "chat.local-execution")).toBe(false);
    expect(platformHasCapability(profile, "view.music")).toBe(true);
    expect(platformHasCapability(profile, "content.managed-image-downloads")).toBe(false);
    expect(platformHasCapability(profile, "notes.external-image-references")).toBe(false);
    expect(platformHasCapability(profile, "music.context-assignments")).toBe(false);
    expect(platformHasCapability(profile, "music.local-file-reveal")).toBe(false);
    expect(platformHasCapability(profile, "music.local-item-repair")).toBe(false);
    expect(platformHasCapability(profile, "music.local-root-relink-plans")).toBe(false);
    expect(platformHasCapability(profile, "music.local-root-reselection")).toBe(true);
    expect(platformHasCapability(profile, "music.soundscapes")).toBe(false);
    expect(platformHasCapability(profile, "notifications.native-scheduling")).toBe(false);
    expect(platformHasCapability(profile, "pomodoro.native-idle-detection")).toBe(false);
    expect(platformHasCapability(profile, "projects.working-folders")).toBe(false);
    expect(platformHasCapability(profile, "storage.native-file-picker")).toBe(false);
    expect(platformHasCapability(profile, "notes.file-export")).toBe(false);
    expect(platformHasCapability(profile, "runtime.desktop-pomodoro-effects")).toBe(false);
    expect(platformHasCapability(profile, "system.android-back")).toBe(true);
    expect(platformHasCapability(profile, "window.desktop-title-bar")).toBe(false);
    expect(platformHasCapability(profile, "window.detached-views")).toBe(false);
  });

  it("does not expose Android system Back handling to iOS", () => {
    const profile = platformProfileFor("ios");

    expect(platformHasCapability(profile, "system.android-back")).toBe(false);
  });

  it("writes the build profile to the root document dataset", () => {
    const target: PlatformDatasetTarget = { dataset: {} };

    applyPlatformProfileToDocument(platformProfileFor("android"), target);

    expect(target.dataset).toEqual({
      platform: "android",
      shell: "mobile",
    });
  });
});
