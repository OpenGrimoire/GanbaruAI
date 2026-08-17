import { describe, expect, it } from "vitest";
import type { ChatTeammateChannelAccessInput } from "$lib/chat/contracts";
import {
  capabilitiesForPreset,
  channelCapabilityPreset,
  folderCapabilityFits,
  resolveRuntimeApproval,
  selectionState,
  teammateAccessDraftErrors,
  teammateAccessDraftSnapshot,
  toggleSelectionGroup,
} from "./teammate-access";

function access(channelId: string): ChatTeammateChannelAccessInput {
  return {
    channelId,
    accessProfileId: "profile:build",
    accessProfileRevision: 1,
    capabilities: { readHistory: true, participate: true },
    historyBoundary: { kind: "entire" },
    runtimeApprovalOverride: null,
    scratchRuntimeApprovalOverride: null,
    folderGrants: [],
  };
}

describe("teammate access", () => {
  it("maps independent channel capabilities to presets", () => {
    expect(channelCapabilityPreset(capabilitiesForPreset("contextSource"))).toBe("contextSource");
    expect(channelCapabilityPreset(capabilitiesForPreset("isolatedResponder"))).toBe("isolatedResponder");
    expect(channelCapabilityPreset(capabilitiesForPreset("collaborator"))).toBe("collaborator");
    expect(channelCapabilityPreset({ readHistory: false, participate: false })).toBe("custom");
  });

  it("keeps folder capabilities inside the profile ceiling", () => {
    expect(folderCapabilityFits("read", "execute")).toBe(true);
    expect(folderCapabilityFits("publish", "execute")).toBe(false);
  });

  it("resolves the most local runtime approval", () => {
    expect(resolveRuntimeApproval("ask", null, null)).toEqual({ policy: "ask", source: "teammate" });
    expect(resolveRuntimeApproval("ask", "autoApprove", null)).toEqual({
      policy: "autoApprove", source: "channel",
    });
    expect(resolveRuntimeApproval("ask", "autoApprove", "unattended")).toEqual({
      policy: "unattended", source: "folder",
    });
  });

  it("bulk-selects only the exact provided channel set", () => {
    const selected = new Set(["channel:elsewhere"]);
    const next = toggleSelectionGroup(["channel:a", "channel:b"], selected, true);
    expect([...next].sort()).toEqual(["channel:a", "channel:b", "channel:elsewhere"]);
    expect(selectionState(["channel:a", "channel:b"], next)).toBe("all");
    expect(selectionState(["channel:a", "channel:c"], next)).toBe("some");
  });

  it("creates order-independent atomic snapshots", () => {
    expect(teammateAccessDraftSnapshot([access("channel:b"), access("channel:a")]))
      .toBe(teammateAccessDraftSnapshot([access("channel:a"), access("channel:b")]));
  });

  it("keeps scratch approval overrides in atomic drafts", () => {
    const first = access("channel:a");
    const second = access("channel:a");
    second.scratchRuntimeApprovalOverride = "unattended";

    expect(teammateAccessDraftSnapshot([first]))
      .not.toBe(teammateAccessDraftSnapshot([second]));
  });

  it("keeps the reviewed profile revision in atomic drafts", () => {
    const first = access("channel:a");
    const second = access("channel:a");
    second.accessProfileRevision = 2;

    expect(teammateAccessDraftSnapshot([first]))
      .not.toBe(teammateAccessDraftSnapshot([second]));
  });

  it("rejects duplicated channels and grants above the profile ceiling", () => {
    const first = access("channel:a");
    first.folderGrants = [{
      workingFolderId: "folder:a",
      capability: "publish",
      isDefault: true,
      runtimeApprovalOverride: null,
    }];
    const errors = teammateAccessDraftErrors(
      [first, access("channel:a")],
      new Map([["profile:build", "execute"]]),
    );
    expect(errors).toContain("Duplicate channel channel:a");
    expect(errors).toContain("Folder folder:a exceeds its access profile");
  });

  it("allows executable access to fall back to private scratch", () => {
    const draft = access("channel:a");
    draft.folderGrants = [{
      workingFolderId: "folder:a",
      capability: "execute",
      isDefault: false,
      runtimeApprovalOverride: null,
    }];
    expect(teammateAccessDraftErrors(
      [draft],
      new Map([["profile:build", "execute"]]),
    )).toEqual([]);
  });
});
