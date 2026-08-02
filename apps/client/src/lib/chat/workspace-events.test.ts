import { describe, expect, it } from "vitest";
import {
  chatBottomWorkspaceRequest,
  isChatBottomWorkspaceRequest,
} from "./workspace-events";

describe("Chat workspace event routing", () => {
  it("routes existing file and review events without changing their payload", () => {
    const fileDetail = { relativePath: "src/main.ts" };
    expect(chatBottomWorkspaceRequest("ganbaru-ai:chat-open-file", fileDetail)).toEqual({
      source: "file",
      detail: fileDetail,
    });
    expect(chatBottomWorkspaceRequest("ganbaru-ai:chat-open-review", null)).toEqual({
      source: "review",
      detail: null,
    });
  });

  it("rejects unknown or malformed internal requests", () => {
    expect(chatBottomWorkspaceRequest("ganbaru-ai:unknown", {})).toBeNull();
    expect(isChatBottomWorkspaceRequest({ source: "panel", detail: null })).toBe(false);
    expect(isChatBottomWorkspaceRequest({ source: "files", detail: {} })).toBe(false);
  });
});
