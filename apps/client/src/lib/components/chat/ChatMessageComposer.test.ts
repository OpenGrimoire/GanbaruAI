// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { PostChatMessageResult } from "$lib/chat/contracts";
import { getChat } from "$lib/stores/chat.svelte";
import ChatMessageComposer from "./ChatMessageComposer.svelte";

describe("ChatMessageComposer", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    vi.restoreAllMocks();
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
  });

  it("uses one send action and resets channel sharing after a thread reply", async () => {
    const chat = getChat();
    const post = vi.spyOn(chat, "postOrganizationalMessage").mockResolvedValue(
      undefined as unknown as PostChatMessageResult,
    );
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatMessageComposer, {
      target,
      props: {
        destination: `reply-thread:${crypto.randomUUID()}`,
        placeholder: "Reply",
        threadComposer: true,
      },
    });

    expect(target.querySelector(".route-note")).toBeNull();
    expect(target.querySelectorAll(".send-button")).toHaveLength(1);

    const share = target.querySelector<HTMLButtonElement>("button[aria-pressed]");
    expect(share?.getAttribute("aria-pressed")).toBe("false");
    share?.click();
    await tick();
    expect(share?.getAttribute("aria-pressed")).toBe("true");

    const textarea = target.querySelector<HTMLTextAreaElement>("textarea");
    if (!textarea) throw new Error("Expected the organizational composer textarea");
    textarea.value = "Share this update";
    textarea.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText" }));
    await tick();
    target.querySelector<HTMLButtonElement>(".send-button")?.click();

    await vi.waitFor(() => {
      expect(post).toHaveBeenCalledWith(expect.any(String), { alsoSendToChannel: true });
      expect(share?.getAttribute("aria-pressed")).toBe("false");
    });
  });
});
