/**
 * Narrow bridge used by the isolated benchmark harness to drive the mounted
 * Project Chat surface through the same store and DOM paths as user interaction.
 */
export interface ChatBenchmarkMeasurements {
  channelIds(): string[];
  waitUntilUsable(): Promise<void>;
  switchChannel(channelId: string): Promise<void>;
  localSearch(query: string): number;
  streamFrames(frameCount: number): Promise<number[]>;
}

class ChatBenchmarkHandle {
  #measurements: ChatBenchmarkMeasurements | null = null;

  register(measurements: ChatBenchmarkMeasurements): () => void {
    this.#measurements = measurements;
    return () => {
      if (this.#measurements === measurements) this.#measurements = null;
    };
  }

  get available(): boolean {
    return this.#measurements !== null;
  }

  channelIds(): string[] {
    return this.#measurements?.channelIds() ?? [];
  }

  waitUntilUsable(): Promise<void> {
    return this.#measurements?.waitUntilUsable()
      ?? Promise.reject(new Error("Project Chat is not mounted"));
  }

  switchChannel(channelId: string): Promise<void> {
    return this.#measurements?.switchChannel(channelId)
      ?? Promise.reject(new Error("Project Chat is not mounted"));
  }

  localSearch(query: string): number {
    return this.#measurements?.localSearch(query) ?? 0;
  }

  streamFrames(frameCount: number): Promise<number[]> {
    return this.#measurements?.streamFrames(frameCount)
      ?? Promise.reject(new Error("Project Chat is not mounted"));
  }
}

let handle: ChatBenchmarkHandle | null = null;

export function getChatBenchmarkHandle(): ChatBenchmarkHandle {
  handle ??= new ChatBenchmarkHandle();
  return handle;
}

/** Registers measurements for the currently mounted Chat workspace. */
export function registerMountedChatBenchmark(): () => void {
  const chat = getChat();
  return getChatBenchmarkHandle().register({
    channelIds: () => chat.activeChannels.map((channel) => channel.id),
    waitUntilUsable: () => waitForBenchmarkState(() => (
      !chat.loading && chat.activeChannels.length > 0
    )),
    switchChannel: async (channelId) => {
      await chat.selectChannel(channelId);
      await waitForBenchmarkState(() => (
        chat.selectedChannelId === channelId
        && !chat.channelMessagesLoading
        && chat.channelMessages.length > 0
      ));
      await nextAnimationFrame();
    },
    localSearch: (query) => {
      const normalized = query.trim().toLocaleLowerCase();
      return [...chat.activeChannels, ...chat.archivedChannels].filter((channel) => (
        channel.name.toLocaleLowerCase().includes(normalized)
        || channel.topic.toLocaleLowerCase().includes(normalized)
      )).length;
    },
    streamFrames: async (frameCount) => {
      const targetIndex = chat.channelMessages.length - 1;
      const original = chat.channelMessages[targetIndex];
      if (!original || targetIndex < 0) {
        throw new Error("Chat benchmark requires a loaded message");
      }
      const markdown = original.normalizedMarkdown;
      const samples: number[] = [];
      let previous = await nextAnimationFrame();
      try {
        for (let index = 0; index < frameCount; index += 1) {
          chat.channelMessages = chat.channelMessages.map((message, messageIndex) => (
            messageIndex === targetIndex
              ? { ...message, normalizedMarkdown: `${markdown}\nstream-${index}` }
              : message
          ));
          await tick();
          const painted = await nextAnimationFrame();
          samples.push(painted - previous);
          previous = painted;
        }
      } finally {
        chat.channelMessages = chat.channelMessages.map((message, messageIndex) => (
          messageIndex === targetIndex ? original : message
        ));
        await tick();
      }
      return samples;
    },
  });
}

async function waitForBenchmarkState(
  predicate: () => boolean,
  timeoutMs = 10_000,
): Promise<void> {
  const deadline = performance.now() + timeoutMs;
  while (!predicate()) {
    if (performance.now() >= deadline) throw new Error("Chat benchmark state timed out");
    await nextAnimationFrame();
  }
}

function nextAnimationFrame(): Promise<number> {
  return new Promise((resolve) => requestAnimationFrame(resolve));
}
import { tick } from "svelte";
import { getChat } from "$lib/stores/chat.svelte";
