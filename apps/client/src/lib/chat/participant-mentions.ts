import type {
  ChatConversationMembershipRead,
  ChatParticipantMentionInput,
  ChatParticipantRead,
  JsonValue,
} from "./contracts";

export interface ParticipantMentionCandidate {
  participant: ChatParticipantRead;
  role: string;
  configurationState: string | null;
}

export interface MentionInsertionResult {
  text: string;
  mentions: ChatParticipantMentionInput[];
  selection: number;
}

export interface ParticipantMentionClipboardSlice {
  text: string;
  mentions: ChatParticipantMentionInput[];
}

export type ParticipantMentionTextSegment =
  | { kind: "text"; text: string }
  | { kind: "mention"; text: string; mention: ChatParticipantMentionInput };

/** Lists current addressable channel members matching a mention query. */
export function participantMentionCandidates(
  memberships: readonly ChatConversationMembershipRead[],
  roles: ReadonlyMap<string, { role: string; configurationState: string }>,
  query: string,
): ParticipantMentionCandidate[] {
  const normalized = query.trim().toLocaleLowerCase();
  return memberships
    .filter((membership) => (
      membership.removedAt === null
        && membership.addressable
        && membership.participant.archivedAt === null
    ))
    .map((membership) => {
      const teammate = roles.get(membership.participant.id);
      return {
        participant: membership.participant,
        role: teammate?.role ?? "",
        configurationState: teammate?.configurationState ?? null,
      };
    })
    .filter((candidate) => !normalized || [
      candidate.participant.displayName,
      candidate.role,
    ].some((value) => value.toLocaleLowerCase().includes(normalized)))
    .sort((left, right) => (
      Number(right.participant.kind === "ai_teammate") - Number(left.participant.kind === "ai_teammate")
      || left.participant.displayName.localeCompare(right.participant.displayName)
    ));
}

/** Finds an incomplete mention immediately before a text caret. */
export function mentionQueryAtCaret(text: string, caret: number): { start: number; query: string } | null {
  const before = text.slice(0, caret);
  const match = /(?:^|\s)@([\p{L}\p{N}_. -]*)$/u.exec(before);
  if (!match) return null;
  return { start: caret - match[1].length - 1, query: match[1] };
}

/** Inserts one structured participant mention and rebases existing ranges. */
export function insertParticipantMention(
  text: string,
  mentions: readonly ChatParticipantMentionInput[],
  replaceStart: number,
  replaceEnd: number,
  participant: ChatParticipantRead,
): MentionInsertionResult {
  const label = participant.displayName.trim();
  if (!label) throw new Error("The participant does not have an addressable name");
  const token = `@${label}`;
  const withSpace = replaceEnd === text.length || !/^\s/u.test(text.slice(replaceEnd))
    ? `${token} `
    : token;
  const nextText = `${text.slice(0, replaceStart)}${withSpace}${text.slice(replaceEnd)}`;
  const rebased = rebaseParticipantMentions(text, nextText, mentions, replaceStart, replaceEnd);
  const tokenEnd = replaceStart + token.length;
  const inserted: ChatParticipantMentionInput = {
    participantId: participant.id,
    participantKind: participant.kind,
    labelSnapshot: label,
    startOffset: utf8Offset(nextText, replaceStart),
    endOffset: utf8Offset(nextText, tokenEnd),
  };
  return {
    text: nextText,
    mentions: [...rebased, inserted].sort((left, right) => left.startOffset - right.startOffset),
    selection: replaceStart + withSpace.length,
  };
}

/** Splits message text around validated structured participant mentions. */
export function participantMentionTextSegments(
  text: string,
  mentions: readonly ChatParticipantMentionInput[],
): ParticipantMentionTextSegment[] {
  const segments: ParticipantMentionTextSegment[] = [];
  let cursor = 0;
  for (const mention of [...mentions].sort((left, right) => left.startOffset - right.startOffset)) {
    const start = jsOffsetFromUtf8(text, mention.startOffset);
    const end = jsOffsetFromUtf8(text, mention.endOffset);
    const expected = `@${mention.labelSnapshot}`;
    if (start < cursor || end <= start || end > text.length || text.slice(start, end) !== expected) continue;
    if (start > cursor) segments.push({ kind: "text", text: text.slice(cursor, start) });
    segments.push({ kind: "mention", text: text.slice(start, end), mention: { ...mention } });
    cursor = end;
  }
  if (cursor < text.length) segments.push({ kind: "text", text: text.slice(cursor) });
  return segments;
}

/** Rebases ranges after an edit and drops any mention touched by that edit. */
export function rebaseParticipantMentions(
  previousText: string,
  nextText: string,
  mentions: readonly ChatParticipantMentionInput[],
  editStart: number,
  editEnd: number,
): ChatParticipantMentionInput[] {
  const delta = nextText.length - (previousText.length - (editEnd - editStart));
  return mentions.flatMap((mention) => {
    const start = jsOffsetFromUtf8(previousText, mention.startOffset);
    const end = jsOffsetFromUtf8(previousText, mention.endOffset);
    if (start < editEnd && end > editStart) return [];
    const mentionFollowsEdit = start >= editEnd;
    const nextStart = mentionFollowsEdit ? start + delta : start;
    const nextEnd = mentionFollowsEdit ? end + delta : end;
    const expected = `@${mention.labelSnapshot}`;
    if (nextText.slice(nextStart, nextEnd) !== expected) return [];
    return [{
      ...mention,
      startOffset: utf8Offset(nextText, nextStart),
      endOffset: utf8Offset(nextText, nextEnd),
    }];
  });
}

/** Derives the single replacement represented by a textarea input. */
export function rebaseMentionsAfterInput(
  previousText: string,
  nextText: string,
  mentions: readonly ChatParticipantMentionInput[],
): ChatParticipantMentionInput[] {
  let prefix = 0;
  while (prefix < previousText.length && prefix < nextText.length && previousText[prefix] === nextText[prefix]) {
    prefix += 1;
  }
  let suffix = 0;
  while (
    suffix < previousText.length - prefix
    && suffix < nextText.length - prefix
    && previousText[previousText.length - 1 - suffix] === nextText[nextText.length - 1 - suffix]
  ) {
    suffix += 1;
  }
  return rebaseParticipantMentions(
    previousText,
    nextText,
    mentions,
    prefix,
    previousText.length - suffix,
  );
}

/** Returns the complete mention range before a backspace caret, if present. */
export function mentionBeforeCaret(
  text: string,
  mentions: readonly ChatParticipantMentionInput[],
  caret: number,
): { start: number; end: number } | null {
  for (const mention of mentions) {
    const start = jsOffsetFromUtf8(text, mention.startOffset);
    const end = jsOffsetFromUtf8(text, mention.endOffset);
    if (end === caret) return { start, end };
  }
  return null;
}

/** Expands a textarea edit so a participant mention is replaced as one atomic unit. */
export function expandEditRangeToParticipantMentions(
  text: string,
  mentions: readonly ChatParticipantMentionInput[],
  editStart: number,
  editEnd: number,
): { start: number; end: number } {
  let start = editStart;
  let end = editEnd;
  for (const mention of mentions) {
    const mentionStart = jsOffsetFromUtf8(text, mention.startOffset);
    const mentionEnd = jsOffsetFromUtf8(text, mention.endOffset);
    const caretInside = editStart === editEnd && editStart > mentionStart && editStart < mentionEnd;
    const rangesOverlap = editStart < mentionEnd && editEnd > mentionStart;
    if (!caretInside && !rangesOverlap) continue;
    start = Math.min(start, mentionStart);
    end = Math.max(end, mentionEnd);
  }
  return { start, end };
}

/** Returns the complete mention range after a forward-delete caret, if present. */
export function mentionAfterCaret(
  text: string,
  mentions: readonly ChatParticipantMentionInput[],
  caret: number,
): { start: number; end: number } | null {
  for (const mention of mentions) {
    const start = jsOffsetFromUtf8(text, mention.startOffset);
    const end = jsOffsetFromUtf8(text, mention.endOffset);
    if (start === caret) return { start, end };
  }
  return null;
}

/** Serializes a selected text slice with complete atomic mention snapshots. */
export function copyParticipantMentionSlice(
  text: string,
  mentions: readonly ChatParticipantMentionInput[],
  start: number,
  end: number,
): ParticipantMentionClipboardSlice {
  const byteStart = utf8Offset(text, start);
  const byteEnd = utf8Offset(text, end);
  return {
    text: text.slice(start, end),
    mentions: mentions
      .filter((mention) => mention.startOffset >= byteStart && mention.endOffset <= byteEnd)
      .map((mention) => ({
        ...mention,
        startOffset: mention.startOffset - byteStart,
        endOffset: mention.endOffset - byteStart,
      })),
  };
}

/** Inserts a copied text slice and preserves its validated mention snapshots. */
export function pasteParticipantMentionSlice(
  text: string,
  mentions: readonly ChatParticipantMentionInput[],
  start: number,
  end: number,
  slice: ParticipantMentionClipboardSlice,
): MentionInsertionResult {
  const expanded = expandEditRangeToParticipantMentions(text, mentions, start, end);
  const nextText = `${text.slice(0, expanded.start)}${slice.text}${text.slice(expanded.end)}`;
  const rebased = rebaseParticipantMentions(text, nextText, mentions, expanded.start, expanded.end);
  const insertedByteStart = utf8Offset(nextText, expanded.start);
  const inserted = slice.mentions.flatMap((mention) => {
    if (mention.startOffset >= mention.endOffset) return [];
    const relativeStart = jsOffsetFromUtf8(slice.text, mention.startOffset);
    const relativeEnd = jsOffsetFromUtf8(slice.text, mention.endOffset);
    const expected = `@${mention.labelSnapshot}`;
    if (slice.text.slice(relativeStart, relativeEnd) !== expected) return [];
    return [{
      ...mention,
      startOffset: insertedByteStart + mention.startOffset,
      endOffset: insertedByteStart + mention.endOffset,
    }];
  });
  return {
    text: nextText,
    mentions: [...rebased, ...inserted].sort((left, right) => left.startOffset - right.startOffset),
    selection: expanded.start + slice.text.length,
  };
}

/** Builds rich content with participant mentions represented as atomic identity nodes. */
export function participantMentionRichContent(
  text: string,
  mentions: readonly ChatParticipantMentionInput[],
): JsonValue {
  const content: JsonValue[] = [];
  let cursor = 0;
  for (const mention of [...mentions].sort((left, right) => left.startOffset - right.startOffset)) {
    const start = jsOffsetFromUtf8(text, mention.startOffset);
    const end = jsOffsetFromUtf8(text, mention.endOffset);
    if (start < cursor || end <= start || end > text.length) continue;
    if (start > cursor) content.push({ type: "text", text: text.slice(cursor, start) });
    content.push({
      type: "participantMention",
      attrs: {
        participantId: mention.participantId,
        participantKind: mention.participantKind,
        labelSnapshot: mention.labelSnapshot,
      },
    });
    cursor = end;
  }
  if (cursor < text.length) content.push({ type: "text", text: text.slice(cursor) });
  return { type: "message", content };
}

function utf8Offset(text: string, offset: number): number {
  return new TextEncoder().encode(text.slice(0, offset)).byteLength;
}

function jsOffsetFromUtf8(text: string, byteOffset: number): number {
  if (byteOffset <= 0) return 0;
  let bytes = 0;
  let offset = 0;
  for (const character of text) {
    const length = new TextEncoder().encode(character).byteLength;
    if (bytes + length > byteOffset) break;
    bytes += length;
    offset += character.length;
  }
  return offset;
}
