import { createNotesPage } from "$lib/api/notes";
import type { NotesLoadedPage, NotesPageCreate } from "$lib/notes/types";

interface PendingPageCreation {
  request: NotesPageCreate;
  status: "pending" | "failed";
  error: string | null;
}

interface NotesPageCreationContext {
  reconcile: (loaded: NotesLoadedPage) => Promise<void>;
  afterPersisted?: (pageId: string) => Promise<void>;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** Coordinate optimistic page creation and gate mutations until persistence succeeds. */
export function createNotesPageCreationController(context: NotesPageCreationContext) {
  let pending = $state<Record<string, PendingPageCreation>>({});
  const attempts = new Map<string, Promise<void>>();
  const completions = new Map<string, {
    promise: Promise<void>;
    resolve: () => void;
  }>();

  function completionFor(pageId: string) {
    const current = completions.get(pageId);
    if (current) return current;
    let resolve: () => void = () => {};
    const promise = new Promise<void>((onResolve) => {
      resolve = onResolve;
    });
    const completion = { promise, resolve };
    completions.set(pageId, completion);
    return completion;
  }

  function persist(request: NotesPageCreate): Promise<void> {
    let succeeded = false;
    const completion = completionFor(request.id);
    pending = {
      ...pending,
      [request.id]: { request, status: "pending", error: null },
    };
    const barrier = createNotesPage(request)
      .then(async (loaded) => {
        await context.reconcile(loaded);
        const next = { ...pending };
        delete next[request.id];
        pending = next;
        succeeded = true;
        completion.resolve();
        completions.delete(request.id);
      })
      .catch((error: unknown) => {
        pending = {
          ...pending,
          [request.id]: { request, status: "failed", error: errorMessage(error) },
        };
      })
      .finally(() => {
        if (attempts.get(request.id) === barrier) attempts.delete(request.id);
        if (succeeded) {
          void context.afterPersisted?.(request.id).catch(() => undefined);
        }
      });
    attempts.set(request.id, barrier);
    return barrier;
  }

  function begin(request: NotesPageCreate): void {
    void persist(request);
  }

  async function awaitReady(pageId: string | null): Promise<void> {
    if (!pageId) return;
    const completion = completions.get(pageId);
    if (completion) await completion.promise;
  }

  function retry(pageId: string): void {
    const state = pending[pageId];
    if (!state || attempts.has(pageId)) return;
    void persist(state.request);
  }

  return {
    begin,
    awaitReady,
    retry,
    isPending(pageId: string | null): boolean {
      return Boolean(pageId && pending[pageId]);
    },
    errorFor(pageId: string | null): string | null {
      return pageId ? pending[pageId]?.error ?? null : null;
    },
  };
}
