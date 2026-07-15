/**
 * Maps asynchronous Music work with a strict concurrency cap while preserving input order.
 */
export async function mapMusicWithConcurrency<T, R>(
  values: readonly T[],
  requestedLimit: number,
  task: (value: T, index: number) => Promise<R>,
): Promise<R[]> {
  if (values.length === 0) return [];
  const limit = Math.max(1, Math.min(values.length, Math.floor(requestedLimit)));
  let cursor = 0;
  const workers = Array.from({ length: limit }, async () => {
    const completed: Array<{ index: number; value: R }> = [];
    while (cursor < values.length) {
      const index = cursor;
      cursor += 1;
      const input = values[index];
      if (input === undefined) continue;
      completed.push({ index, value: await task(input, index) });
    }
    return completed;
  });
  return (await Promise.all(workers))
    .flat()
    .sort((left, right) => left.index - right.index)
    .map((entry) => entry.value);
}
