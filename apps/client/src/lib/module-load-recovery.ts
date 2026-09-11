export interface LoadFailure {
  message: string;
  requiresDocumentReload: boolean;
}

const DYNAMIC_IMPORT_FAILURE_MARKERS = [
  "failed to fetch dynamically imported module",
  "error loading dynamically imported module",
  "importing a module script failed",
  "failed to load module script",
] as const;

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** Classify a load failure without discarding the original diagnostic message. */
export function classifyLoadFailure(error: unknown): LoadFailure {
  const message = errorMessage(error);
  const normalizedMessage = message.toLocaleLowerCase("en-US");
  return {
    message,
    requiresDocumentReload: DYNAMIC_IMPORT_FAILURE_MARKERS.some((marker) =>
      normalizedMessage.includes(marker)
    ),
  };
}

/** Recover in place when possible, or reload after a browser-cached import failure. */
export function recoverLoadFailure(
  failure: LoadFailure,
  retry: () => void,
  reload: () => void = () => window.location.reload(),
): void {
  if (failure.requiresDocumentReload) {
    reload();
    return;
  }
  retry();
}
