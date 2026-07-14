const flushers = new Set<() => Promise<void>>();

export function registerQuickNotesFlusher(flush: () => Promise<void>): () => void {
  flushers.add(flush);
  return () => flushers.delete(flush);
}

export async function flushQuickNoteEditors(): Promise<void> {
  await Promise.all([...flushers].map((flush) => flush()));
}
