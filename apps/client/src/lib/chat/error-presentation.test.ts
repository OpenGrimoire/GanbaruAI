import { describe, expect, it } from "vitest";
import { chatErrorMessage } from "./error-presentation";

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
});
