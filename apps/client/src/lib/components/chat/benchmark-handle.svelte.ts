/**
 * Narrow bridge used by the isolated benchmark harness to drive the mounted
 * Project Chat surface through the same store and DOM paths as user interaction.
 */
export interface ChatBenchmarkMeasurements {
  threadIds(): string[];
  waitUntilUsable(): Promise<void>;
  switchThread(threadId: string): Promise<void>;
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

  threadIds(): string[] {
    return this.#measurements?.threadIds() ?? [];
  }

  waitUntilUsable(): Promise<void> {
    return this.#measurements?.waitUntilUsable()
      ?? Promise.reject(new Error("Project Chat is not mounted"));
  }

  switchThread(threadId: string): Promise<void> {
    return this.#measurements?.switchThread(threadId)
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
