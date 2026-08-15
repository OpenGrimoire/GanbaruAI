// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ChatParticipantRead } from "$lib/chat/contracts";
import type { ChatModelParticipant } from "$lib/chat/participant-identity";
import { getPreferences } from "$lib/stores/preferences.svelte";
import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
import ChatIdentityButton from "./ChatIdentityButton.svelte";

const localParticipant: ChatParticipantRead = {
  id: "participant:local-owner",
  kind: "local_user",
  displayName: "You",
  avatar: { schemaVersion: 1, value: {} },
  revision: 1,
  archivedAt: null,
};

const model: ChatModelParticipant = {
  displayName: "GPT 5.6 Sol",
  company: { id: "openai", name: "OpenAI", iconFamilyId: "codex", order: 0 },
  modelId: "gpt-5.6-sol",
  description: "Frontier coding model",
  contextLimit: 258_000,
  defaultReasoning: "Medium",
};

describe("ChatIdentityButton", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;
  const preferences = getPreferences();
  const settings = getSettingsLauncher();
  const originalDisplayName = preferences.profileDisplayName;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
    preferences.setProfileDisplayName(originalDisplayName);
    settings.close();
    vi.useRealTimers();
  });

  it("opens local identity details and links to profile settings", async () => {
    preferences.setProfileDisplayName("Victor Benito");
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatIdentityButton, {
      target,
      props: { participant: localParticipant, presentation: "name" },
    });

    target.querySelector<HTMLButtonElement>(".identity-trigger")?.click();
    await tick();
    const card = document.body.querySelector<HTMLElement>(".identity-card");
    expect(card?.textContent).toContain("Victor Benito");
    expect(card?.textContent).toContain("Private local identity");

    card?.querySelector<HTMLButtonElement>("footer button")?.click();
    expect(settings.isOpen).toBe(true);
    expect(settings.targetSection).toBe("profile");
  });

  it("shows exact model provenance instead of only repeating its name", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatIdentityButton, {
      target,
      props: {
        model,
        presentation: "avatar",
        providerLabel: "Codex local",
      },
    });

    target.querySelector<HTMLButtonElement>(".identity-trigger")?.click();
    await tick();
    const card = document.body.querySelector<HTMLElement>(".identity-card");
    expect(card?.textContent).not.toContain("Execution model");
    expect(card?.textContent).toContain("Frontier coding model");
    expect(card?.textContent).toContain("Default reasoning");
    expect(card?.textContent).toContain("Medium");
    expect(card?.textContent).toContain("Codex local");
    expect(card?.textContent).toContain("gpt-5.6-sol");
    expect(card?.textContent).toContain("258,000 tokens");
  });

  it("previews identity on deliberate hover and closes after leaving", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatIdentityButton, {
      target,
      props: { participant: localParticipant, presentation: "mention", triggerLabel: "@you" },
    });
    await tick();
    const trigger = target.querySelector<HTMLButtonElement>(".identity-trigger");

    trigger?.dispatchEvent(new MouseEvent("pointerenter", { bubbles: true }));
    await new Promise((resolve) => setTimeout(resolve, 200));
    await tick();
    expect(document.body.querySelector(".identity-card")).not.toBeNull();

    trigger?.dispatchEvent(new MouseEvent("pointerleave", { bubbles: true }));
    await new Promise((resolve) => setTimeout(resolve, 160));
    await tick();
    expect(document.body.querySelector(".identity-card")).toBeNull();
  });
});
