import type * as ReviewDiffRuntimeModule from "$lib/chat/review-diff-runtime";

let runtimeModulePromise: Promise<typeof ReviewDiffRuntimeModule> | null = null;

/** Loads the enhanced Review renderer once and shares it across intent and panel activation. */
export function loadChatReviewDiffRuntime(): Promise<typeof ReviewDiffRuntimeModule> {
  runtimeModulePromise ??= import("$lib/chat/review-diff-runtime");
  return runtimeModulePromise;
}
