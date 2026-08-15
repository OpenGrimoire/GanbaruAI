import { describe, expect, it } from "vitest";
import type { ChatConversationMembershipRead, ChatParticipantRead } from "./contracts";
import {
  copyParticipantMentionSlice,
  expandEditRangeToParticipantMentions,
  insertParticipantMention,
  mentionAfterCaret,
  mentionBeforeCaret,
  mentionQueryAtCaret,
  participantMentionRichContent,
  participantMentionCandidates,
  participantMentionTextSegments,
  pasteParticipantMentionSlice,
  rebaseMentionsAfterInput,
} from "./participant-mentions";

const teammate: ChatParticipantRead = {
  id: "participant:ganbaru",
  kind: "ai_teammate",
  displayName: "Ganbaru",
  avatar: { schemaVersion: 1, value: {} },
  revision: 1,
  archivedAt: null,
};

function membership(participant = teammate): ChatConversationMembershipRead {
  return {
    conversationId: "conversation:general",
    participant,
    addressable: true,
    approvalPolicy: "ask_for_approval",
    workingFolderGrants: [],
    revision: 1,
    removedAt: null,
  };
}

describe("participant mentions", () => {
  it("finds the caret query and filters current memberships by role", () => {
    expect(mentionQueryAtCaret("Please ask @gan", 15)).toEqual({ start: 11, query: "gan" });
    expect(mentionQueryAtCaret("Please ask @Ganbaru St", 22)).toEqual({
      start: 11,
      query: "Ganbaru St",
    });
    const candidates = participantMentionCandidates(
      [membership()],
      new Map([[teammate.id, { role: "Frontend work", configurationState: "healthy" }]]),
      "front",
    );
    expect(candidates.map((entry) => entry.participant.id)).toEqual([teammate.id]);
  });

  it("keeps archived teammates out of new mention candidates", () => {
    const archived = { ...teammate, archivedAt: "2026-08-14T12:00:00.000Z" };
    expect(participantMentionCandidates(
      [membership(archived)],
      new Map([[archived.id, { role: "Frontend work", configurationState: "healthy" }]]),
      "",
    )).toEqual([]);
  });

  it("serializes UTF-8 ranges and deletes a mention atomically", () => {
    const inserted = insertParticipantMention("Hola á @gan", [], 7, 11, teammate);
    expect(inserted.text).toBe("Hola á @Ganbaru ");
    expect(inserted.mentions[0]).toMatchObject({ startOffset: 8, endOffset: 16 });
    expect(mentionBeforeCaret(inserted.text, inserted.mentions, inserted.selection - 1)).toEqual({ start: 7, end: 15 });
    expect(rebaseMentionsAfterInput(inserted.text, "Hola á ", inserted.mentions)).toEqual([]);
  });

  it("adds typing space at the end and preserves existing spacing in prose", () => {
    expect(insertParticipantMention("@gan", [], 0, 4, teammate).text).toBe("@Ganbaru ");
    expect(insertParticipantMention("Ask @gan later", [], 4, 8, teammate).text).toBe("Ask @Ganbaru later");
  });

  it("keeps historical labels when the teammate is renamed", () => {
    const inserted = insertParticipantMention("@g", [], 0, 2, teammate);
    const renamed = { ...teammate, displayName: "Ganbaru Studio" };
    expect(inserted.mentions[0].labelSnapshot).toBe("Ganbaru");
    expect(renamed.displayName).toBe("Ganbaru Studio");
    expect(inserted.text).toBe("@Ganbaru ");
  });

  it("uses a multi-word display name as one atomic mention label", () => {
    const participant = { ...teammate, displayName: "Ganbaru Studio" };
    const inserted = insertParticipantMention("Ask @gan", [], 4, 8, participant);
    expect(inserted.text).toBe("Ask @Ganbaru Studio ");
    expect(inserted.mentions[0]).toMatchObject({
      labelSnapshot: "Ganbaru Studio",
      startOffset: 4,
      endOffset: 19,
    });
  });

  it("drops a structured mention when paste or IME editing touches its token", () => {
    const inserted = insertParticipantMention("Ask @g now", [], 4, 6, teammate);
    const pasted = inserted.text.replace("@Ganbaru", "@someone");
    expect(rebaseMentionsAfterInput(inserted.text, pasted, inserted.mentions)).toEqual([]);
  });

  it("preserves a structured mention while typing the request after it", () => {
    const inserted = insertParticipantMention("@ganb", [], 0, 5, teammate);
    const request = `${inserted.text}Please create hello.py`;

    expect(rebaseMentionsAfterInput(inserted.text, request, inserted.mentions)).toEqual(inserted.mentions);
  });

  it("expands partial edits and forward deletion to the complete mention", () => {
    const inserted = insertParticipantMention("Ask @g now", [], 4, 6, teammate);
    expect(expandEditRangeToParticipantMentions(inserted.text, inserted.mentions, 7, 9)).toEqual({
      start: 4,
      end: 12,
    });
    expect(mentionAfterCaret(inserted.text, inserted.mentions, 4)).toEqual({ start: 4, end: 12 });
  });

  it("copies and pastes atomic mention identity snapshots", () => {
    const inserted = insertParticipantMention("Ask @g", [], 4, 6, teammate);
    const copied = copyParticipantMentionSlice(inserted.text, inserted.mentions, 4, 12);
    const pasted = pasteParticipantMentionSlice("Tell ", [], 5, 5, copied);
    expect(pasted.text).toBe("Tell @Ganbaru");
    expect(pasted.mentions).toEqual([expect.objectContaining({
      participantId: teammate.id,
      labelSnapshot: "Ganbaru",
      startOffset: 5,
      endOffset: 13,
    })]);
  });

  it("serializes mentions as atomic rich-content nodes", () => {
    const inserted = insertParticipantMention("Ask @g now", [], 4, 6, teammate);
    expect(participantMentionRichContent(inserted.text, inserted.mentions)).toEqual({
      type: "message",
      content: [
        { type: "text", text: "Ask " },
        {
          type: "participantMention",
          attrs: {
            participantId: teammate.id,
            participantKind: teammate.kind,
            labelSnapshot: teammate.displayName,
          },
        },
        { type: "text", text: " now" },
      ],
    });
  });

  it("splits only validated structured mentions for interactive rendering", () => {
    const inserted = insertParticipantMention("Ask @g now", [], 4, 6, teammate);
    expect(participantMentionTextSegments(inserted.text, inserted.mentions)).toEqual([
      { kind: "text", text: "Ask " },
      { kind: "mention", text: "@Ganbaru", mention: inserted.mentions[0] },
      { kind: "text", text: " now" },
    ]);
    expect(participantMentionTextSegments("Ask @someone", inserted.mentions)).toEqual([
      { kind: "text", text: "Ask @someone" },
    ]);
  });
});
