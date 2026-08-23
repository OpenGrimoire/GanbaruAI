import { afterEach, describe, expect, it, vi } from "vitest";
import { getChat } from "./chat.svelte";

describe("ChatStore reply thread refresh", () => {
  const chat = getChat();

  afterEach(() => {
    vi.restoreAllMocks();
    chat.closeReplyThread();
  });

  it("bypasses the cached page whenever a thread is opened", async () => {
    const replyThreadId = crypto.randomUUID();
    const loadReplyThread = vi.spyOn(chat, "loadReplyThread").mockResolvedValue();

    await chat.openReplyThread(replyThreadId);

    expect(loadReplyThread).toHaveBeenCalledWith(replyThreadId, true);
  });
});
