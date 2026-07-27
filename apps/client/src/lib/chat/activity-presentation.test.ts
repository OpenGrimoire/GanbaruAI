import { describe, expect, it } from "vitest";
import type { TimelineActivityRow } from "./timeline-model";
import {
  commandActivityPresentation,
  fileChangePresentation,
} from "./activity-presentation";

function activity(overrides: Partial<TimelineActivityRow> = {}): TimelineActivityRow {
  return {
    id: "command-1",
    kind: "activity",
    turnId: "turn-1",
    sequence: 1,
    createdAt: "2026-07-27T23:00:00.000Z",
    activityKind: "command_execution",
    status: "completed",
    title: "/usr/bin/bash -lc 'printf 4'",
    detail: "4\n",
    metadata: {
      schemaVersion: 1,
      value: { cwd: "/workspace", exitCode: 0, durationMs: 18 },
    },
    ...overrides,
  };
}

describe("command activity presentation", () => {
  it("keeps command lifecycle data separate from streamed output", () => {
    expect(commandActivityPresentation(activity())).toEqual({
      command: "/usr/bin/bash -lc 'printf 4'",
      cwd: "/workspace",
      output: "4",
      exitCode: 0,
      durationMs: 18,
      running: false,
    });
  });

  it("represents an active command without fabricating output", () => {
    expect(commandActivityPresentation(activity({ status: "active", detail: null }))).toMatchObject({
      output: null,
      running: true,
    });
  });
});

describe("file change presentation", () => {
  it("parses file paths, kinds, diffs, and line counts", () => {
    expect(fileChangePresentation(activity({
      activityKind: "file_change",
      metadata: {
        schemaVersion: 1,
        value: {
          changes: [{
            path: "src/app.ts",
            kind: "update",
            diff: "--- a/src/app.ts\n+++ b/src/app.ts\n-old\n+new",
          }],
        },
      },
    }))).toEqual([{
      path: "src/app.ts",
      kind: "update",
      diff: "--- a/src/app.ts\n+++ b/src/app.ts\n-old\n+new",
      additions: 1,
      deletions: 1,
    }]);
  });
});
