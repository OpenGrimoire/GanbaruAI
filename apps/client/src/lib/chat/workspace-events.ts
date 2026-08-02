export const CHAT_OPEN_BOTTOM_WORKSPACE_EVENT = "ganbaru-ai:chat-open-bottom-workspace";

export type ChatWorkspaceRequestSource = "changes" | "review" | "file";

export interface ChatBottomWorkspaceRequest {
  source: ChatWorkspaceRequestSource;
  detail: unknown;
}

const WORKSPACE_EVENT_SOURCES: Readonly<Record<string, ChatWorkspaceRequestSource>> = {
  "ganbaru-ai:chat-open-changes": "changes",
  "ganbaru-ai:chat-open-review": "review",
  "ganbaru-ai:chat-open-file": "file",
};

/** Converts a public Chat workspace event into a request for the bottom dock. */
export function chatBottomWorkspaceRequest(
  eventType: string,
  detail: unknown,
): ChatBottomWorkspaceRequest | null {
  const source = WORKSPACE_EVENT_SOURCES[eventType];
  if (!source) return null;
  const request = { source, detail };
  return isChatBottomWorkspaceRequest(request) ? request : null;
}

/** Validates the internal event used to route workspace tools into the bottom dock. */
export function isChatBottomWorkspaceRequest(value: unknown): value is ChatBottomWorkspaceRequest {
  if (!isRecord(value)) return false;
  return (value.source === "changes" || value.source === "review" || value.source === "file")
    && "detail" in value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
