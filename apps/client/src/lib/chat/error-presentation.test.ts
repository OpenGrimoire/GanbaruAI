import { describe, expect, it } from "vitest";
import { chatErrorField, chatErrorMessage } from "./error-presentation";

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
});
