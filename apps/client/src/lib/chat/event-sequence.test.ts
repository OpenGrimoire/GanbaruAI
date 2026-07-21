import { describe, expect, it } from "vitest";
import type { ChatChangeNotification } from "./contracts";
import { applyChatChangeNotification, reconcileChatSequence } from "./event-sequence";

function notification(sequence: number, revision = sequence + 1): ChatChangeNotification {
  return {
    threadId: "thread-1",
    sequence,
    revision,
    changedProjectionKeys: ["messages", "messages", "thread"],
  };
}

describe("Chat event sequence reconciliation", () => {
  it("applies only the next sequence and deduplicates projection keys", () => {
    expect(
      applyChatChangeNotification({ lastSequence: 3, revision: 4 }, notification(4, 5)),
    ).toEqual({
      kind: "applied",
      state: { lastSequence: 4, revision: 5 },
      changedProjectionKeys: ["messages", "thread"],
    });
  });

  it("ignores duplicate and late notifications", () => {
    expect(
      applyChatChangeNotification({ lastSequence: 4, revision: 5 }, notification(4, 5)).kind,
    ).toBe("duplicate");
    expect(
      applyChatChangeNotification({ lastSequence: 4, revision: 5 }, notification(2, 3)).kind,
    ).toBe("duplicate");
  });

  it("requests replay after a gap and rejects a stale revision", () => {
    expect(
      applyChatChangeNotification({ lastSequence: 4, revision: 5 }, notification(7, 8)),
    ).toMatchObject({ kind: "gap", replayAfterSequence: 4 });
    expect(
      applyChatChangeNotification({ lastSequence: 4, revision: 8 }, notification(5, 8)).kind,
    ).toBe("stale_revision");
  });

  it("reconciles detached-window wakeups against durable shell state", () => {
    expect(
      reconcileChatSequence(
        { lastSequence: 10, revision: 11 },
        { lastSequence: 13, revision: 14 },
      ),
    ).toMatchObject({ kind: "gap", replayAfterSequence: 10 });
    expect(
      reconcileChatSequence(
        { lastSequence: 10, revision: 11 },
        { lastSequence: 10, revision: 11 },
      ).kind,
    ).toBe("duplicate");
  });
});
