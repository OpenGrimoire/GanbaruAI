// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type {
  ChatParticipantRead,
  ChatReplyThreadPageRead,
  ChatScheduledMessageRead,
  PostChatMessageResult,
} from "$lib/chat/contracts";
import { getChat } from "$lib/stores/chat.svelte";
import ChatMessageComposer from "./ChatMessageComposer.svelte";

const teammate: ChatParticipantRead = {
  id: "participant:ganbaru",
  kind: "ai_teammate",
  displayName: "Ganbaru",
  avatar: { schemaVersion: 1, value: {} },
  revision: 1,
  archivedAt: null,
};

function workingReplyThread(): ChatReplyThreadPageRead {
  const timestamp = "2026-08-05T16:00:00.000Z";
  return {
    thread: {
      id: "reply-thread:working",
      replyCount: 0,
      lastActivityAt: timestamp,
      participants: [teammate],
      unread: false,
      workState: "working",
    },
    rootMessage: {
      itemId: "item:root",
      conversationId: "conversation:test",
      replyThreadId: null,
      revisionId: "revision:root",
      revision: 1,
      author: teammate,
      normalizedMarkdown: "Work on this",
      richContent: { schemaVersion: 1, value: {} },
      mentions: [],
      attachmentIds: [],
      resourceReferences: [],
      replyThread: null,
      ordinal: 1,
      editedAt: null,
      createdAt: timestamp,
    },
    replies: [],
    assignment: {
      id: "assignment:working",
      replyThreadId: "reply-thread:working",
      teammate,
      triggeringMessageItemId: "item:root",
      previousAssignmentId: null,
      state: "working",
      stateReason: null,
      revision: 1,
      settledAt: null,
      createdAt: timestamp,
      updatedAt: timestamp,
    },
    agentRuns: [],
    previousCursor: null,
    revision: 1,
  };
}

describe("ChatMessageComposer", () => {
  const chat = getChat();
  const initialReplyThread = chat.replyThread;
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
    chat.replyThread = initialReplyThread;
  });

  it("uses the primary button and shortcut to stop active work while Enter still sends", async () => {
    const page = workingReplyThread();
    chat.replyThread = page;
    vi.spyOn(chat, "listScheduledOrganizationalMessages").mockResolvedValue([]);
    const cancel = vi.spyOn(chat, "cancelAssignment").mockResolvedValue();
    const post = vi.spyOn(chat, "postOrganizationalMessage").mockResolvedValue(
      undefined as unknown as PostChatMessageResult,
    );
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatMessageComposer, {
      target,
      props: {
        destination: `reply-thread:${page.thread.id}`,
        placeholder: "Reply",
        threadComposer: true,
      },
    });

    const primary = target.querySelector<HTMLButtonElement>('.send-button[data-action="stop"]');
    expect(primary?.disabled).toBe(false);
    expect(primary?.querySelector(".lucide-square")).not.toBeNull();
    primary?.click();
    await vi.waitFor(() => expect(cancel).toHaveBeenCalledWith(page.assignment?.id));
    expect(post).not.toHaveBeenCalled();

    const textarea = target.querySelector<HTMLTextAreaElement>("textarea");
    if (!textarea) throw new Error("Expected the organizational composer textarea");
    textarea.value = "One more detail";
    textarea.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText" }));
    textarea.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "Enter" }));
    await vi.waitFor(() => expect(post).toHaveBeenCalledWith(
      expect.any(String),
      { alsoSendToChannel: false },
    ));

    window.dispatchEvent(new Event("ganbaru-ai:chat-stop-requested"));
    await vi.waitFor(() => expect(cancel).toHaveBeenCalledTimes(2));
  });

  it("shows one failed-work recovery above the thread composer and retries in place", async () => {
    const page = workingReplyThread();
    if (!page.assignment) throw new Error("Expected a work assignment");
    page.assignment.state = "failed";
    page.assignment.stateReason = "Codex executable could not be resolved";
    page.thread.workState = "failed";
    chat.replyThread = page;
    vi.spyOn(chat, "listScheduledOrganizationalMessages").mockResolvedValue([]);
    const retry = vi.spyOn(chat, "retryAssignment").mockResolvedValue();
    const requestScrollToBottom = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatMessageComposer, {
      target,
      props: {
        destination: `reply-thread:${page.thread.id}`,
        placeholder: "Reply",
        threadComposer: true,
        onRequestScrollToBottom: requestScrollToBottom,
      },
    });

    const summary = target.querySelector<HTMLElement>(".assignment-error-summary");
    expect(summary?.textContent).toContain("The agent stopped because of an error.");
    const retryButton = summary?.querySelector<HTMLButtonElement>("button");
    expect(retryButton?.textContent).toBe("Retry work");
    retryButton?.click();

    await vi.waitFor(() => {
      expect(retry).toHaveBeenCalledWith(page.assignment?.id);
      expect(requestScrollToBottom).toHaveBeenCalledOnce();
    });
  });

  it("uses one send action and resets channel sharing after a thread reply", async () => {
    const chat = getChat();
    const destination = `reply-thread:${crypto.randomUUID()}`;
    const post = vi.spyOn(chat, "postOrganizationalMessage").mockResolvedValue(
      undefined as unknown as PostChatMessageResult,
    );
    const requestScrollToBottom = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatMessageComposer, {
      target,
      props: {
        destination,
        placeholder: "Reply",
        threadComposer: true,
        onRequestScrollToBottom: requestScrollToBottom,
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
      expect(requestScrollToBottom).toHaveBeenCalledOnce();
    });

    await unmount(component);
    component = mount(ChatMessageComposer, {
      target,
      props: {
        destination,
        placeholder: "Reply",
        threadComposer: true,
        onRequestScrollToBottom: requestScrollToBottom,
      },
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
    const liveHeightsDuringMeasurement: string[] = [];
    vi.spyOn(HTMLTextAreaElement.prototype, "scrollHeight", "get").mockImplementation(function (this: HTMLTextAreaElement) {
      if (this !== textarea) liveHeightsDuringMeasurement.push(textarea.style.height);
      return contentHeight;
    });
    await vi.waitFor(() => expect(callbacks).toHaveLength(1));

    callbacks[0]?.(
      [{ contentRect: { width: 1 } } as unknown as ResizeObserverEntry],
      {} as ResizeObserver,
    );
    expect(textarea.style.height).toBe("120px");
    expect(liveHeightsDuringMeasurement).not.toContain("0px");

    contentHeight = 40;
    callbacks[0]?.(
      [{ contentRect: { width: 400 } } as unknown as ResizeObserverEntry],
      {} as ResizeObserver,
    );
    expect(textarea.style.height).toBe("40px");
    expect(liveHeightsDuringMeasurement).not.toContain("0px");
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
