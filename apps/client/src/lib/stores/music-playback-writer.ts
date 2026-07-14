export interface GenerationPlaybackState {
  sourceIdentity: string;
  updatedAt: number;
}

/** Serialize playback writes, coalescing queued checkpoints to the newest state. */
export function createMusicPlaybackWriter<T extends GenerationPlaybackState>(
  write: (state: T) => Promise<void>,
): {
  save(state: T, generation: number, isCurrent: (generation: number, sourceIdentity: string) => boolean): Promise<boolean>;
} {
  let running: Promise<void> | null = null;
  let pending: {
    state: T;
    generation: number;
    isCurrent: (generation: number, sourceIdentity: string) => boolean;
    resolve: (saved: boolean) => void;
    reject: (error: unknown) => void;
  } | null = null;

  async function drain(): Promise<void> {
    while (pending) {
      const next = pending;
      pending = null;
      if (!next.isCurrent(next.generation, next.state.sourceIdentity)) {
        next.resolve(false);
        continue;
      }
      try {
        await write(next.state);
        next.resolve(next.isCurrent(next.generation, next.state.sourceIdentity));
      } catch (error) {
        next.reject(error);
      }
    }
  }

  return {
    save(state, generation, isCurrent) {
      return new Promise<boolean>((resolve, reject) => {
        if (pending) pending.resolve(false);
        pending = { state, generation, isCurrent, resolve, reject };
        if (running) return;
        running = drain().finally(() => {
          running = null;
          if (pending) {
            running = drain().finally(() => {
              running = null;
            });
          }
        });
      });
    },
  };
}
