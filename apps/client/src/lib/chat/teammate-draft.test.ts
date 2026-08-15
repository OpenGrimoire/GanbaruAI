import { describe, expect, it } from "vitest";
import {
  teammateMembershipDraftSnapshot,
  teammateProfileDraftSnapshot,
  type TeammateMembershipDraftSnapshotInput,
  type TeammateProfileDraftSnapshotInput,
} from "./teammate-draft";

const PROFILE: TeammateProfileDraftSnapshotInput = {
  displayName: "Ganbaru",
  role: "Plan work",
  instructions: "Stay focused",
  providerId: "codex-default",
  modelId: "gpt-5.5",
  modelOptions: [
    { key: "speed", value: { kind: "choice", value: "standard" } },
    { key: "effort", value: { kind: "choice", value: "medium" } },
  ],
  effort: "medium",
};

const MEMBERSHIP: TeammateMembershipDraftSnapshotInput = {
  channelId: "channel-1",
  approvalPolicy: "ask_for_approval",
  folderIds: ["folder-1", "folder-2"],
  defaultFolderId: "folder-1",
};

describe("teammate draft snapshots", () => {
  it("ignores whitespace that is removed when saving a profile", () => {
    expect(teammateProfileDraftSnapshot({
      ...PROFILE,
      displayName: " Ganbaru ",
      role: " Plan work ",
    })).toBe(teammateProfileDraftSnapshot(PROFILE));
  });

  it("ignores model option order", () => {
    expect(teammateProfileDraftSnapshot({
      ...PROFILE,
      modelOptions: [...PROFILE.modelOptions].reverse(),
    })).toBe(teammateProfileDraftSnapshot(PROFILE));
  });

  it("detects a persisted profile change", () => {
    expect(teammateProfileDraftSnapshot({ ...PROFILE, effort: "high" }))
      .not.toBe(teammateProfileDraftSnapshot(PROFILE));
  });

  it("treats folder grants as an unordered set", () => {
    expect(teammateMembershipDraftSnapshot({
      ...MEMBERSHIP,
      folderIds: ["folder-2", "folder-1", "folder-2"],
    })).toBe(teammateMembershipDraftSnapshot(MEMBERSHIP));
  });

  it("includes the default folder in the saved grant set", () => {
    expect(teammateMembershipDraftSnapshot({
      ...MEMBERSHIP,
      folderIds: ["folder-2"],
    })).toBe(teammateMembershipDraftSnapshot(MEMBERSHIP));
  });
});
