import { describe, expect, it } from "vitest";
import { groupMusicIssues, musicIssueGroup } from "$lib/music/music-issue-presentation";
import type { MusicIssue } from "$lib/music/library-contracts";

function issue(issueKind: string): MusicIssue {
  return { id: issueKind, issueKind, itemId: null, playlistId: null, collectionId: null, rootId: null, relativePath: null, actionRequired: true, message: issueKind, createdAt: 1 };
}

describe("music issue presentation", () => {
  it.each([
    ["missing-local-file", "missing-local-file"],
    ["root-unavailable", "root-unavailable"],
    ["ambiguous-match", "ambiguous-match"],
    ["youtube-unavailable", "youtube-unavailable"],
    ["embedding-blocked", "embedding-blocked"],
    ["refresh-partial", "refresh-incomplete"],
  ] as const)("maps %s to %s", (kind, expected) => {
    expect(musicIssueGroup(issue(kind))).toBe(expected);
  });

  it("preserves stable issue order inside each group", () => {
    const grouped = groupMusicIssues([issue("missing-first"), issue("missing-second")]);
    expect(grouped.get("missing-local-file")?.map((entry) => entry.id)).toEqual(["missing-first", "missing-second"]);
  });
});
