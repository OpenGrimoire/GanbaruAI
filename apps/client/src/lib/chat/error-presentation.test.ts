import { describe, expect, it } from "vitest";
import { chatErrorCode, chatErrorField, chatErrorMessage } from "./error-presentation";

describe("chatErrorMessage", () => {
  it("reads typed Tauri Chat errors without object stringification", () => {
    expect(chatErrorMessage({
      code: "invalid_state_transition",
      message: "Chat provider session is not ready",
      recoverable: true,
    })).toBe("Chat provider session is not ready");
  });

  it("reads nested transport errors and retains a useful fallback", () => {
    expect(chatErrorMessage({ error: { message: "Provider process stopped" } })).toBe("Provider process stopped");
    expect(chatErrorMessage({ code: "unknown" }, "Could not send message")).toBe("Could not send message");
  });

  it("reads the field from direct and nested validation errors", () => {
    expect(chatErrorField({ field: "handle", message: "Already used" })).toBe("handle");
    expect(chatErrorField({ error: { field: "displayName", message: "Required" } })).toBe("displayName");
    expect(chatErrorField("Failed")).toBeNull();
  });

  it("reads only recognized direct and nested error codes", () => {
    expect(chatErrorCode({ code: "stale_revision" })).toBe("stale_revision");
    expect(chatErrorCode({ error: { code: "conflict" } })).toBe("conflict");
    expect(chatErrorCode({ cause: { code: "permission" } })).toBe("permission");
    expect(chatErrorCode({ code: "invented" })).toBeNull();
    expect(chatErrorCode("stale_revision")).toBeNull();
  });
});
