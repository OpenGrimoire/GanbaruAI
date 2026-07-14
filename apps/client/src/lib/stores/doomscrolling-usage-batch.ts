export function createDoomscrollingUsageBatch<T>(write: (samples: T[]) => Promise<void>): {
  enqueue(sample: T): void;
  flush(): Promise<void>;
  readonly size: number;
} {
  let pending: T[] = [];
  let flushInFlight: Promise<void> | null = null;

  async function drain(): Promise<void> {
    while (pending.length > 0) {
      const batch = pending;
      pending = [];
      try {
        await write(batch);
      } catch (error) {
        pending = [...batch, ...pending];
        throw error;
      }
    }
  }

  return {
    enqueue(sample) {
      pending.push(sample);
    },
    flush() {
      if (!flushInFlight) {
        flushInFlight = drain().finally(() => {
          flushInFlight = null;
        });
      }
      return flushInFlight;
    },
    get size() {
      return pending.length;
    },
  };
}
