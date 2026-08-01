import type {
  ChatChannelSessionRead,
  ChatChannelTimelinePageRead,
  ChatTimelineItemRead,
} from "./contracts";

export interface CachedChannelTimeline {
  pages: ChatChannelTimelinePageRead[];
  sessions: ChatChannelSessionRead[];
  items: ChatTimelineItemRead[];
}

interface CacheEntry {
  value: CachedChannelTimeline;
  bytes: number;
}

/** Keeps recently visited channel pages within entry and serialized byte limits. */
export class ChatChannelTimelineCache {
  readonly #entries = new Map<string, CacheEntry>();
  #bytes = 0;

  public constructor(
    private readonly maxEntries: number,
    private readonly maxBytes: number,
  ) {
    if (!Number.isSafeInteger(maxEntries) || maxEntries <= 0) {
      throw new Error("maxEntries must be a positive safe integer");
    }
    if (!Number.isSafeInteger(maxBytes) || maxBytes <= 0) {
      throw new Error("maxBytes must be a positive safe integer");
    }
  }

  /** Reads and promotes one retained channel timeline. */
  public get(channelId: string): CachedChannelTimeline | undefined {
    const entry = this.#entries.get(channelId);
    if (!entry) return undefined;
    this.#entries.delete(channelId);
    this.#entries.set(channelId, entry);
    return cloneTimeline(entry.value);
  }

  /** Stores one channel timeline and evicts least recently used entries as needed. */
  public set(channelId: string, value: CachedChannelTimeline): void {
    this.delete(channelId);
    const bytes = new TextEncoder().encode(JSON.stringify(value)).byteLength;
    if (bytes > this.maxBytes) return;
    this.#entries.set(channelId, { value: cloneTimeline(value), bytes });
    this.#bytes += bytes;
    while (this.#entries.size > this.maxEntries || this.#bytes > this.maxBytes) {
      const oldestId = this.#entries.keys().next().value as string | undefined;
      if (!oldestId) break;
      this.delete(oldestId);
    }
  }

  /** Removes one retained channel. */
  public delete(channelId: string): void {
    const entry = this.#entries.get(channelId);
    if (!entry) return;
    this.#entries.delete(channelId);
    this.#bytes = Math.max(0, this.#bytes - entry.bytes);
  }

  /** Removes every retained channel timeline. */
  public clear(): void {
    this.#entries.clear();
    this.#bytes = 0;
  }

  /** Reports bounded-cache accounting for diagnostics and tests. */
  public stats(): { entries: number; bytes: number } {
    return { entries: this.#entries.size, bytes: this.#bytes };
  }
}

function cloneTimeline(value: CachedChannelTimeline): CachedChannelTimeline {
  return {
    pages: value.pages.map((page) => ({
      ...page,
      sessions: page.sessions.map((session) => ({ ...session, thread: { ...session.thread } })),
      items: page.items.map((item) => ({ ...item })),
      turns: page.turns.map((turn) => ({ ...turn })),
    })),
    sessions: value.sessions.map((session) => ({ ...session, thread: { ...session.thread } })),
    items: value.items.map((item) => ({ ...item })),
  };
}
