// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type {
  ChatAgentRunRead,
  ChatMessageRead,
  ChatParticipantRead,
  ChatReplyThreadPageRead,
} from "$lib/chat/contracts";
import { getChat } from "$lib/stores/chat.svelte";
import ChatReplyThreadPanel from "./ChatReplyThreadPanel.svelte";

const localParticipant: ChatParticipantRead = {
  id: "participant:local-owner",
  kind: "local_user",
  displayName: "You",
  handle: null,
  avatar: { schemaVersion: 1, value: {} },
  revision: 1,
  archivedAt: null,
};

const agentParticipant: ChatParticipantRead = {
  id: "participant:ganbaru",
  kind: "ai_teammate",
  displayName: "Ganbaru",
  handle: "ganbaru",
  avatar: { schemaVersion: 1, value: {} },
  revision: 1,
  archivedAt: null,
};

function message(
  itemId: string,
  author: ChatParticipantRead,
  markdown: string,
  ordinal: number,
): ChatMessageRead {
  return {
    itemId,
    conversationId: "conversation:test",
    replyThreadId: ordinal === 0 ? null : "reply-thread:test",
    revisionId: `revision:${itemId}`,
    revision: 1,
    author,
    normalizedMarkdown: markdown,
    richContent: {
      schemaVersion: 1,
      value: ordinal === 0
        ? { type: "doc" }
        : { type: "agent_update", agentRunId: "run:test", updateKind: "result" },
    },
    mentions: [],
    attachmentIds: [],
    resourceReferences: [],
    replyThread: null,
    ordinal,
    editedAt: null,
    createdAt: `2026-08-04T17:0${ordinal}:00.000Z`,
  };
}

const run: ChatAgentRunRead = {
  id: "run:test",
  assignmentId: "assignment:test",
  projectId: "project:test",
  workingFolderId: "folder:test",
  teammatePolicyRevisionId: "policy:test",
  effort: "medium",
  providerExecutionTurnId: "turn:test",
  providerExecutionThreadId: "provider-thread:test",
  state: "completed",
  runOrdinal: 1,
  createdAt: "2026-08-04T17:00:30.000Z",
  updatedAt: "2026-08-04T17:01:00.000Z",
};

const page: ChatReplyThreadPageRead = {
  thread: {
    id: "reply-thread:test",
    replyCount: 1,
    lastActivityAt: "2026-08-04T17:01:00.000Z",
    participants: [agentParticipant],
    unread: false,
    workState: "ready_for_review",
  },
  rootMessage: message("root:test", localParticipant, "Original request", 0),
  replies: [message("reply:test", agentParticipant, "Projected answer", 1)],
  assignment: null,
  agentRuns: [run],
  previousCursor: null,
  revision: 1,
};

describe("ChatReplyThreadPanel", () => {
  const chat = getChat();
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;
  const originalOpenReplyThreadId = chat.openReplyThreadId;
  const originalReplyThread = chat.replyThread;
  const originalReplyThreadPages = chat.replyThreadPages;
  const originalSelectedExecutionRunId = chat.selectedExecutionRunId;
  const originalSelectedThreadId = chat.selectedThreadId;

  afterEach(async () => {
    vi.restoreAllMocks();
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
    chat.openReplyThreadId = originalOpenReplyThreadId;
    chat.replyThread = originalReplyThread;
    chat.replyThreadPages = originalReplyThreadPages;
    chat.selectedExecutionRunId = originalSelectedExecutionRunId;
    chat.selectedThreadId = originalSelectedThreadId;
  });

  it("never paints a projected agent answer while the exact run is loading", async () => {
    chat.openReplyThreadId = page.thread.id;
    chat.replyThread = page;
    chat.replyThreadPages = [page];
    chat.selectedExecutionRunId = null;
    chat.selectedThreadId = null;
    vi.spyOn(chat, "selectAssignmentExecution").mockReturnValue(new Promise(() => undefined));
    vi.spyOn(chat, "listScheduledOrganizationalMessages").mockResolvedValue([]);
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatReplyThreadPanel, {
      target,
      props: { onClose: vi.fn() },
    });
    await tick();

    expect(target.textContent).not.toContain("Projected answer");
    expect(target.querySelector('.thread-scroll[aria-busy="true"]')).not.toBeNull();
    expect(target.querySelector('[data-message-item-id="reply:test"]')).toBeNull();
  });
});
