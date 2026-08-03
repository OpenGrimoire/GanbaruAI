// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ChatParticipantRead } from "$lib/chat/contracts";
import { getPreferences } from "$lib/stores/preferences.svelte";
import ChatParticipantAvatar from "./ChatParticipantAvatar.svelte";

const profileImage = vi.hoisted(() => ({
  load: vi.fn(async (relativePath: string) => `https://profile.invalid/${relativePath}`),
}));

vi.mock("$lib/api/profile-image", () => ({
  profileImageAssetUrl: profileImage.load,
}));

const localParticipant: ChatParticipantRead = {
  id: "participant:local-owner",
  kind: "local_user",
  displayName: "You",
  handle: null,
  avatar: { schemaVersion: 1, value: {} },
  revision: 1,
  archivedAt: null,
};

describe("ChatParticipantAvatar", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;
  const preferences = getPreferences();
  const originalDisplayName = preferences.profileDisplayName;
  const originalImagePath = preferences.profileImagePath;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
    preferences.setProfileDisplayName(originalDisplayName);
    preferences.setProfileImagePath(originalImagePath);
    profileImage.load.mockClear();
  });

  it("reacts to current local profile names and images without changing the participant", async () => {
    preferences.setProfileDisplayName("Victor Benito");
    preferences.setProfileImagePath(null);
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatParticipantAvatar, {
      target,
      props: { participant: localParticipant, size: 32 },
    });

    expect(target.textContent?.trim()).toBe("VB");

    preferences.setProfileDisplayName("Renamed Person");
    const imagePath = `profile/${"a".repeat(64)}.png`;
    preferences.setProfileImagePath(imagePath);
    await tick();
    await vi.waitFor(() => {
      expect(target?.querySelector("img")?.src).toBe(`https://profile.invalid/${imagePath}`);
    });
    expect(profileImage.load).toHaveBeenCalledWith(imagePath);
    expect(localParticipant.displayName).toBe("You");
  });
});
