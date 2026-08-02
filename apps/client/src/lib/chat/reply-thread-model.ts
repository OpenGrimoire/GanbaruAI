import type { ChatAgentRunRead, VersionedJson } from "./contracts";

interface RichReply {
  richContent: VersionedJson;
}

/** Returns the newest agent run whose provider execution can be rendered. */
export function latestRenderableAgentRun(
  agentRuns: readonly ChatAgentRunRead[],
): ChatAgentRunRead | null {
  return agentRuns.findLast((agentRun) => agentRun.providerExecutionThreadId !== null) ?? null;
}

/** Removes only the projected copies owned by the provider run rendered in the thread. */
export function repliesWithoutRenderedRunProjection<T extends RichReply>(
  replies: readonly T[],
  renderedRunId: string | null,
): T[] {
  if (renderedRunId === null) return [...replies];
  return replies.filter((reply) => projectedAgentRunId(reply) !== renderedRunId);
}

export function projectedAgentRunId(message: RichReply): string | null {
  const value = message.richContent.value;
  if (typeof value !== "object" || value === null || Array.isArray(value)) return null;
  return value.type === "agent_update" && typeof value.agentRunId === "string"
    ? value.agentRunId
    : null;
}
