export const CHAT_OPEN_WORKSPACE_PANEL_EVENT = "ganbaru-ai:chat-open-workspace-panel";

export type ChatWorkspaceRequestSource = "changes" | "review" | "file";

export interface ChatWorkspaceRequest {
  source: ChatWorkspaceRequestSource;
  detail: unknown;
}

const WORKSPACE_EVENT_SOURCES: Readonly<Record<string, ChatWorkspaceRequestSource>> = {
  "ganbaru-ai:chat-open-changes": "changes",
  "ganbaru-ai:chat-open-review": "review",
  "ganbaru-ai:chat-open-file": "file",
};

/** Converts a public Chat workspace event into a workspace panel request. */
export function chatWorkspaceRequest(
  eventType: string,
  detail: unknown,
): ChatWorkspaceRequest | null {
  const source = WORKSPACE_EVENT_SOURCES[eventType];
  if (!source) return null;
  const request = { source, detail };
  return isChatWorkspaceRequest(request) ? request : null;
}

/** Validates the internal event used to route workspace tools into a panel. */
export function isChatWorkspaceRequest(value: unknown): value is ChatWorkspaceRequest {
  if (!isRecord(value)) return false;
  return (value.source === "changes" || value.source === "review" || value.source === "file")
    && "detail" in value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
