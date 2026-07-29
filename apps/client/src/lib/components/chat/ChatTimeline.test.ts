// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ChatTimelineItemRead, ChatTimelinePageRead, JsonValue } from "$lib/chat/contracts";
import { getChat } from "$lib/stores/chat.svelte";
import ChatTimeline from "./ChatTimeline.svelte";

const timestamp = "2026-07-29T12:00:00.000Z";

class ResizeObserverMock implements ResizeObserver {
  constructor(_callback: ResizeObserverCallback) {}

  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

describe("ChatTimeline", () => {
  const mounted: { target: HTMLDivElement; component: ReturnType<typeof mount> }[] = [];

  beforeEach(() => {
    vi.stubGlobal("ResizeObserver", ResizeObserverMock);
    vi.stubGlobal("CSS", { escape: (value: string) => value });
    vi.stubGlobal("matchMedia", vi.fn(() => ({
      matches: false,
      media: "",
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })));
    const chat = getChat();
    chat.activeThreads = [];
    chat.archivedThreads = [];
    chat.workingFolders = [];
    chat.settings = null;
    chat.selectedThreadId = null;
    chat.selectedWorkingFolderId = null;
    chat.timelineItems = [];
    chat.timelinePages = [];
    chat.timelineLoading = false;
    chat.timelineError = null;
    chat.pendingUserMessage = null;
  });

  afterEach(async () => {
    while (mounted.length > 0) {
      const entry = mounted.pop();
      if (!entry) continue;
      await unmount(entry.component);
      entry.target.remove();
    }
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it("offers Copy on user messages", async () => {
    const chat = getChat();
    chat.timelineItems = [message("user-message", null, "user", "Please check this")];
    const target = mountTimeline();
    await tick();

    expect(target.querySelector(".chat-user-message .chat-message-meta")?.textContent).toContain("Copy");
  });

  it("keeps turn duration on the process disclosure instead of the final answer actions", async () => {
    const chat = getChat();
    chat.timelineItems = [
      activity("command", "turn-1", "command_execution", "Run tests"),
      message("answer", "turn-1", "assistant", "Everything passed"),
    ];
    chat.timelinePages = [timelinePage()];
    const target = mountTimeline();
    await tick();

    expect(target.querySelector(".chat-process-toggle")?.textContent).toContain("Worked for 2s");
    expect(target.querySelector(".chat-process-toggle")?.textContent).not.toContain("Ran commands");
    const answerActions = target.querySelector(".chat-assistant-message .chat-message-meta");
    expect(answerActions?.textContent).toContain("Copy");
    expect(answerActions?.textContent).not.toContain("2s");
  });

  it("groups completed tools with the current live tool and preserves their nested rows", async () => {
    const chat = getChat();
    chat.timelineItems = [
      activity("command", "turn-1", "command_execution", "pnpm check", {
        sequence: 2,
        status: "completed",
        detail: "Passed",
      }),
      activity("read", "turn-1", "dynamic_tool_call", "Read", {
        sequence: 3,
        status: "active",
        detail: null,
        metadata: { toolInput: { file_path: "ChatTimeline.svelte" } },
      }),
    ];
    chat.timelinePages = [timelinePage("active")];
    const target = mountTimeline();
    await tick();

    const group = target.querySelector<HTMLButtonElement>(".chat-process-toggle");
    expect(group?.textContent).toContain("Reading ChatTimeline.svelte");
    group?.click();
    await tick();

    const history = target.querySelector(".chat-process-history");
    expect(history?.textContent).toContain("Ran pnpm check");
    expect(history?.textContent).toContain("Reading ChatTimeline.svelte");
    expect(history?.querySelectorAll(".chat-process-step")).toHaveLength(2);

    chat.timelineItems = [
      chat.timelineItems[0] as ChatTimelineItemRead,
      activity("read", "turn-1", "dynamic_tool_call", "Read", {
        sequence: 3,
        status: "completed",
        detail: null,
        metadata: { toolInput: { file_path: "ChatTimeline.svelte" } },
      }),
      message("commentary", "turn-1", "assistant", "I finished checking the file", 4),
    ];
    await tick();

    expect(target.querySelector(".chat-process-toggle")?.textContent).toContain("Ran a command and read a file");
    expect(target.querySelector(".chat-process-toggle")?.getAttribute("aria-expanded")).toBe("true");
  });

  function mountTimeline(): HTMLDivElement {
    const target = document.createElement("div");
    document.body.append(target);
    const component = mount(ChatTimeline, { target });
    mounted.push({ target, component });
    return target;
  }
});

function message(
  activityId: string,
  turnId: string | null,
  role: "user" | "assistant",
  markdown: string,
  sequence = role === "user" ? 1 : 3,
): ChatTimelineItemRead {
  return {
    activityId,
    turnId,
    sequenceAnchor: sequence,
    kind: "message",
    data: {
      schemaVersion: 1,
      value: {
        role,
        markdown,
        streamingState: "complete",
        providerItemId: activityId,
        metadata: {},
        createdAt: timestamp,
        updatedAt: timestamp,
      },
    },
  };
}

function activity(
  activityId: string,
  turnId: string,
  activityKind: "command_execution" | "dynamic_tool_call",
  title: string,
  options: {
    sequence?: number;
    status?: "active" | "completed";
    detail?: string | null;
    metadata?: { [key: string]: JsonValue };
  } = {},
): ChatTimelineItemRead {
  return {
    activityId,
    turnId,
    sequenceAnchor: options.sequence ?? 2,
    kind: "activity",
    data: {
      schemaVersion: 1,
      value: {
        activityKind,
        status: options.status ?? "completed",
        title,
        detail: options.detail === undefined ? "Passed" : options.detail,
        providerItemId: activityId,
        metadata: options.metadata ?? {},
        createdAt: timestamp,
        updatedAt: timestamp,
      },
    },
  };
}

function timelinePage(state: "active" | "completed" = "completed"): ChatTimelinePageRead {
  return {
    threadId: "thread-1",
    items: [],
    turns: [{
      turnId: "turn-1",
      state,
      startedAt: timestamp,
      completedAt: state === "completed" ? "2026-07-29T12:00:02.000Z" : null,
      stopReason: state === "completed" ? "end_turn" : null,
      modelId: "gpt-5.6-sol",
      modelOptions: [],
      modes: { safetyMode: "ask_for_approval", interactionMode: "build" },
      usage: null,
      changedFiles: [],
    }],
    previousCursor: null,
    nextCursor: null,
    threadRevision: 1,
  };
}
