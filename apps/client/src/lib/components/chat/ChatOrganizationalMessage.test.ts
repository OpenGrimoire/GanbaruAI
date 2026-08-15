// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it } from "vitest";
import type { ChatMessageRead, ChatParticipantRead } from "$lib/chat/contracts";
import ChatOrganizationalMessage from "./ChatOrganizationalMessage.svelte";

const localParticipant: ChatParticipantRead = {
  id: "participant:local-owner",
  kind: "local_user",
  displayName: "You",
  avatar: { schemaVersion: 1, value: {} },
  revision: 1,
  archivedAt: null,
};

const message: ChatMessageRead = {
  itemId: "message:mention",
  conversationId: "conversation:general",
  replyThreadId: null,
  revisionId: "revision:mention",
  revision: 1,
  author: localParticipant,
  normalizedMarkdown: "Ask @Ganbaru now",
  richContent: { schemaVersion: 1, value: {} },
  mentions: [{
    participantId: "participant:ganbaru",
    participantKind: "ai_teammate",
    labelSnapshot: "Ganbaru",
    startOffset: 4,
    endOffset: 12,
  }],
  attachmentIds: [],
  resourceReferences: [],
  replyThread: null,
  ordinal: 1,
  editedAt: null,
  createdAt: "2026-08-12T20:00:00.000Z",
};

describe("ChatOrganizationalMessage", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
  });

  it("renders structured mentions as identity tags without changing surrounding text", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatOrganizationalMessage, { target, props: { message } });

    const copy = target.querySelector<HTMLElement>(".message-copy");
    const mention = copy?.querySelector<HTMLButtonElement>(".mention-trigger");
    expect(mention?.textContent).toBe("@Ganbaru");
    expect(copy?.textContent).toBe("Ask @Ganbaru now");

    mention?.click();
    await tick();
    expect(document.body.querySelector(".identity-card")?.textContent).toContain("Ganbaru");
  });
});
