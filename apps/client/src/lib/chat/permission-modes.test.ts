import { describe, expect, it } from "vitest";
import { providerSupportsPermissionMode } from "./permission-modes";

describe("Chat permission modes", () => {
  it("exposes manual approval and Full access for every provider transport", () => {
    for (const family of ["codex", "claude", "cursor", "grok", "opencode"]) {
      expect(providerSupportsPermissionMode(family, "ask_for_approval")).toBe(true);
      expect(providerSupportsPermissionMode(family, "full_access")).toBe(true);
    }
  });

  it("limits automatic review to classifier-backed providers", () => {
    expect(providerSupportsPermissionMode("codex", "approve_for_me")).toBe(true);
    expect(providerSupportsPermissionMode("claude", "approve_for_me")).toBe(true);
    expect(providerSupportsPermissionMode("cursor", "approve_for_me")).toBe(false);
    expect(providerSupportsPermissionMode("opencode", "approve_for_me")).toBe(false);
  });

  it("limits config.toml permissions to Codex", () => {
    expect(providerSupportsPermissionMode("codex", "custom")).toBe(true);
    expect(providerSupportsPermissionMode("claude", "custom")).toBe(false);
    expect(providerSupportsPermissionMode(null, "custom")).toBe(false);
  });
});
