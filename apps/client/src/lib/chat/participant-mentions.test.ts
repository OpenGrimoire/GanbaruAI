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
  pasteParticipantMentionSlice,
  rebaseMentionsAfterInput,
} from "./participant-mentions";

const teammate: ChatParticipantRead = {
  id: "participant:ganbaru",
  kind: "ai_teammate",
  displayName: "Ganbaru",
  handle: "ganbaru",
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
  it("finds the caret query and filters current memberships by purpose", () => {
    expect(mentionQueryAtCaret("Please ask @gan", 15)).toEqual({ start: 11, query: "gan" });
    const candidates = participantMentionCandidates(
      [membership()],
      new Map([[teammate.id, { purpose: "Frontend work", configurationState: "healthy" }]]),
      "front",
    );
    expect(candidates.map((entry) => entry.participant.id)).toEqual([teammate.id]);
  });

  it("serializes UTF-8 ranges and deletes a mention atomically", () => {
    const inserted = insertParticipantMention("Hola á @gan", [], 7, 11, teammate);
    expect(inserted.text).toBe("Hola á @ganbaru");
    expect(inserted.mentions[0]).toMatchObject({ startOffset: 8, endOffset: 16 });
    expect(mentionBeforeCaret(inserted.text, inserted.mentions, inserted.selection)).toEqual({ start: 7, end: 15 });
    expect(rebaseMentionsAfterInput(inserted.text, "Hola á ", inserted.mentions)).toEqual([]);
  });

  it("keeps historical labels when the teammate is renamed", () => {
    const inserted = insertParticipantMention("@g", [], 0, 2, teammate);
    const renamed = { ...teammate, displayName: "Ganbaru Studio", handle: "studio" };
    expect(inserted.mentions[0].labelSnapshot).toBe("Ganbaru");
    expect(renamed.displayName).toBe("Ganbaru Studio");
  });

  it("drops a structured mention when paste or IME editing touches its token", () => {
    const inserted = insertParticipantMention("Ask @g now", [], 4, 6, teammate);
    const pasted = inserted.text.replace("@ganbaru", "@someone");
    expect(rebaseMentionsAfterInput(inserted.text, pasted, inserted.mentions)).toEqual([]);
  });

  it("preserves a structured mention while typing the request after it", () => {
    const inserted = insertParticipantMention("@ganb", [], 0, 5, teammate);
    const request = `${inserted.text} Please create hello.py`;

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
    expect(pasted.text).toBe("Tell @ganbaru");
    expect(pasted.mentions).toEqual([expect.objectContaining({
      participantId: teammate.id,
      handleSnapshot: "ganbaru",
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
            handle: teammate.handle,
            labelSnapshot: teammate.displayName,
          },
        },
        { type: "text", text: " now" },
      ],
    });
  });
});
