// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ChatScheduledMessageRead, PostChatMessageResult } from "$lib/chat/contracts";
import { getChat } from "$lib/stores/chat.svelte";
import ChatMessageComposer from "./ChatMessageComposer.svelte";

describe("ChatMessageComposer", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
  });

  it("uses one send action and resets channel sharing after a thread reply", async () => {
    const chat = getChat();
    const destination = `reply-thread:${crypto.randomUUID()}`;
    const post = vi.spyOn(chat, "postOrganizationalMessage").mockResolvedValue(
      undefined as unknown as PostChatMessageResult,
    );
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatMessageComposer, {
      target,
      props: {
        destination,
        placeholder: "Reply",
        threadComposer: true,
      },
    });

    expect(target.querySelector(".route-note")).toBeNull();
    expect(target.querySelectorAll(".send-button")).toHaveLength(1);

    const share = target.querySelector<HTMLButtonElement>('button[title^="Share reply"]');
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

    await unmount(component);
    component = mount(ChatMessageComposer, {
      target,
      props: { destination, placeholder: "Reply", threadComposer: true },
    });
    expect(target.querySelector<HTMLTextAreaElement>("textarea")?.value).toBe("");
  });

  it("recalculates an empty composer after its mounted width settles", async () => {
    const callbacks: ResizeObserverCallback[] = [];
    class TestResizeObserver {
      constructor(callback: ResizeObserverCallback) {
        callbacks.push(callback);
      }

      observe(): void {}
      unobserve(): void {}
      disconnect(): void {}
    }
    vi.stubGlobal("ResizeObserver", TestResizeObserver);
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatMessageComposer, {
      target,
      props: {
        destination: `reply-thread:${crypto.randomUUID()}`,
        placeholder: "Reply in thread",
        threadComposer: true,
      },
    });

    const textarea = target.querySelector<HTMLTextAreaElement>("textarea");
    if (!textarea) throw new Error("Expected the organizational composer textarea");
    textarea.style.lineHeight = "20px";
    let contentHeight = 120;
    Object.defineProperty(textarea, "scrollHeight", { get: () => contentHeight });
    await vi.waitFor(() => expect(callbacks).toHaveLength(1));

    callbacks[0]?.(
      [{ contentRect: { width: 1 } } as unknown as ResizeObserverEntry],
      {} as ResizeObserver,
    );
    expect(textarea.style.height).toBe("120px");

    contentHeight = 40;
    callbacks[0]?.(
      [{ contentRect: { width: 400 } } as unknown as ResizeObserverEntry],
      {} as ResizeObserver,
    );
    expect(textarea.style.height).toBe("40px");
  });

  it("arms scheduling without a draft and waits for the send action", async () => {
    const chat = getChat();
    const scheduled = {
      id: crypto.randomUUID(),
      channelId: crypto.randomUUID(),
      replyThreadId: null,
      normalizedMarkdown: "Send this later",
      richContent: { schemaVersion: 1, value: {} },
      attachmentIds: [],
      participantMentions: [],
      resourceReferences: [],
      alsoSendToChannel: false,
      state: "scheduled",
      scheduledFor: "2026-08-04T15:00:00.000Z",
      lastError: null,
      createdAt: "2026-08-02T15:00:00.000Z",
    } as ChatScheduledMessageRead;
    vi.spyOn(chat, "listScheduledOrganizationalMessages").mockResolvedValue([]);
    const schedule = vi.spyOn(chat, "scheduleOrganizationalMessage").mockResolvedValue(scheduled);
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatMessageComposer, {
      target,
      props: {
        destination: `channel:${crypto.randomUUID()}`,
        placeholder: "Message",
      },
    });

    target.querySelector<HTMLButtonElement>('button[title="Schedule message"]')?.click();
    const quickChoice = await vi.waitFor(() => {
      const choice = target?.querySelector<HTMLButtonElement>(".schedule-choices > button");
      if (!choice) throw new Error("Expected a quick schedule choice");
      return choice;
    });
    quickChoice.click();
    await tick();

    expect(schedule).not.toHaveBeenCalled();
    expect(target.querySelector<HTMLButtonElement>('button[aria-label^="Scheduled delivery selected"]')?.getAttribute("aria-pressed")).toBe("true");

    const textarea = target.querySelector<HTMLTextAreaElement>("textarea");
    if (!textarea) throw new Error("Expected the organizational composer textarea");
    textarea.value = "Send this later";
    textarea.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText" }));
    await tick();
    target.querySelector<HTMLButtonElement>(".send-button")?.click();

    await vi.waitFor(() => {
      expect(schedule).toHaveBeenCalledWith(
        expect.stringMatching(/^channel:/),
        expect.any(String),
        { alsoSendToChannel: false },
      );
      expect(textarea.value).toBe("");
      expect(target?.querySelector(".scheduled-summary")?.textContent).toContain("1 scheduled message for");
      expect(target?.querySelector(".scheduled-summary button")?.textContent).toBe("View");
    });
  });

  it("summarizes and manages multiple scheduled messages without showing an empty state", async () => {
    const chat = getChat();
    const channelId = crypto.randomUUID();
    const scheduledMessages = Array.from({ length: 3 }, (_, index) => ({
      id: crypto.randomUUID(),
      channelId,
      replyThreadId: null,
      normalizedMarkdown: index === 0
        ? "Scheduled message 1\nwith the complete second line"
        : `Scheduled message ${index + 1}`,
      richContent: { schemaVersion: 1, value: {} },
      attachmentIds: [],
      participantMentions: [],
      resourceReferences: [],
      alsoSendToChannel: false,
      state: "scheduled",
      scheduledFor: `2026-08-0${index + 4}T15:00:00.000Z`,
      lastError: null,
      createdAt: "2026-08-02T15:00:00.000Z",
    })) as ChatScheduledMessageRead[];
    vi.spyOn(chat, "listScheduledOrganizationalMessages").mockResolvedValue(scheduledMessages);
    const sendNow = vi.spyOn(chat, "sendScheduledOrganizationalMessageNow").mockResolvedValue(
      undefined as unknown as PostChatMessageResult,
    );
    const cancel = vi.spyOn(chat, "cancelScheduledOrganizationalMessage").mockResolvedValue();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatMessageComposer, {
      target,
      props: {
        destination: `channel:${channelId}`,
        placeholder: "Message",
      },
    });

    const viewAll = await vi.waitFor(() => {
      const button = target?.querySelector<HTMLButtonElement>(".scheduled-summary button");
      if (!button) throw new Error("Expected the scheduled message summary action");
      expect(target?.querySelector(".scheduled-summary")?.textContent).toContain("3 scheduled messages");
      expect(target?.querySelector(".organizational-composer .scheduled-summary")).toBeNull();
      expect(button.textContent).toBe("View all");
      return button;
    });
    viewAll.click();

    const sendNowButton = await vi.waitFor(() => {
      const previews = target?.querySelectorAll(".scheduled-preview");
      expect(previews).toHaveLength(3);
      expect(previews?.[0]?.querySelector(".message-copy")?.textContent).toBe(
        "Scheduled message 1\nwith the complete second line",
      );
      expect(previews?.[0]?.querySelector(".profile-avatar")).not.toBeNull();
      const button = Array.from(previews?.[0]?.querySelectorAll<HTMLButtonElement>("button") ?? [])
        .find((entry) => entry.textContent === "Send now");
      if (!button) throw new Error("Expected the scheduled message send-now action");
      return button;
    });
    sendNowButton.click();

    await vi.waitFor(() => {
      expect(sendNow).toHaveBeenCalledWith(scheduledMessages[0].id);
      expect(target?.querySelectorAll(".scheduled-preview")).toHaveLength(2);
      expect(target?.querySelector(".scheduled-summary")?.textContent).toContain("2 scheduled messages");
    });

    const cancelButton = target.querySelector<HTMLButtonElement>(".scheduled-preview .cancel-action");
    if (!cancelButton) throw new Error("Expected the scheduled message cancel action");
    cancelButton.click();

    await vi.waitFor(() => {
      expect(cancel).toHaveBeenCalledWith(scheduledMessages[1].id);
      expect(target?.querySelectorAll(".scheduled-preview")).toHaveLength(1);
      expect(target?.querySelector(".scheduled-summary")?.textContent).toContain("1 scheduled message");
    });
  });

  it("opens the custom time picker as a floating panel outside the composer", async () => {
    Object.defineProperty(Element.prototype, "scrollIntoView", {
      configurable: true,
      value: vi.fn(),
    });
    const chat = getChat();
    vi.spyOn(chat, "listScheduledOrganizationalMessages").mockResolvedValue([]);
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatMessageComposer, {
      target,
      props: {
        destination: `channel:${crypto.randomUUID()}`,
        placeholder: "Message",
      },
    });

    target.querySelector<HTMLButtonElement>('button[title="Schedule message"]')?.click();
    const custom = await vi.waitFor(() => {
      const button = target?.querySelector<HTMLButtonElement>(".custom-schedule-button");
      if (!button) throw new Error("Expected the custom schedule action");
      return button;
    });
    custom.click();
    await tick();
    target.querySelector<HTMLButtonElement>(".schedule-time-button")?.click();
    await tick();

    expect(target.querySelector(".schedule-time-picker")).toBeNull();
    expect(document.body.querySelector(".schedule-time-picker .time-picker-scroll")).not.toBeNull();
  });
});
