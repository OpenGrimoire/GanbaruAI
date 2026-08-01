import { describe, expect, it } from "vitest";
import type { CachedChannelTimeline } from "./channel-timeline-cache";
import { ChatChannelTimelineCache } from "./channel-timeline-cache";

function timeline(channelId: string, markdown = channelId): CachedChannelTimeline {
  return {
    sessions: [],
    items: [{
      activityId: `message:${channelId}`,
      turnId: null,
      sequenceAnchor: 1,
      kind: "message",
      data: { schemaVersion: 1, value: { markdown } },
    }],
    pages: [{
      channelId,
      sessions: [],
      items: [],
      turns: [],
      previousCursor: null,
      revision: 1,
    }],
  };
}

describe("ChatChannelTimelineCache", () => {
  it("evicts the least recently used channel at the entry limit", () => {
    const cache = new ChatChannelTimelineCache(2, 100_000);
    cache.set("first", timeline("first"));
    cache.set("second", timeline("second"));
    expect(cache.get("first")).toBeDefined();
    cache.set("third", timeline("third"));
    expect(cache.get("second")).toBeUndefined();
    expect(cache.get("first")).toBeDefined();
    expect(cache.get("third")).toBeDefined();
  });

  it("rejects one oversized timeline without evicting retained entries", () => {
    const retained = timeline("retained");
    const retainedBytes = new TextEncoder().encode(JSON.stringify(retained)).byteLength;
    const cache = new ChatChannelTimelineCache(2, retainedBytes + 32);
    cache.set("retained", retained);
    cache.set("oversized", timeline("oversized", "x".repeat(retainedBytes * 2)));
    expect(cache.get("retained")).toBeDefined();
    expect(cache.get("oversized")).toBeUndefined();
  });

  it("returns copies so active state cannot mutate a retained timeline", () => {
    const cache = new ChatChannelTimelineCache(2, 100_000);
    cache.set("channel", timeline("channel"));
    const restored = cache.get("channel");
    restored?.items.splice(0);
    expect(cache.get("channel")?.items).toHaveLength(1);
  });
});
