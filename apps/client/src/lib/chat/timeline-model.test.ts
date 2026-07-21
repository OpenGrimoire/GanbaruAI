import { describe, expect, it } from "vitest";
import type { CanonicalEvent, CanonicalStoredEvent, ChatTimelineItemRead, ChatTimelineTurnRead } from "./contracts";
import { buildTimelineDisplayRows, parseTimelineUserContext, projectCanonicalTimeline, projectTimelineReadModel } from "./timeline-model";

const start = "2026-07-21T14:00:00.000Z";

function stored(sequence: number, event: CanonicalEvent, options: { eventId?: string; turnId?: string | null; createdAt?: string } = {}): CanonicalStoredEvent {
  return {
    schemaVersion: 1,
    eventId: options.eventId ?? `event-${sequence}`,
    providerFamilyId: "codex",
    providerInstanceId: "codex-work",
    threadId: "thread-1",
    createdAt: options.createdAt ?? start,
    turnId: options.turnId ?? "turn-1",
    providerTurnId: "provider-turn-1",
    providerItemId: null,
    providerRequestId: null,
    providerTaskId: null,
    providerReference: null,
    event,
    redactedDiagnostic: null,
    sequence,
    ingestedAt: options.createdAt ?? start,
  };
}

describe("canonical timeline projection", () => {
  it("projects durable checkpoint restores as thread-level notices", () => {
    const projection = projectCanonicalTimeline([
      stored(1, {
        type: "thread_reverted",
        payload: {
          checkpointId: "checkpoint:1",
          revertedTurnIds: ["turn:2"],
          providerHistoryAction: "fork_required",
        },
      }, { turnId: null }),
    ]);
    expect(projection.rows).toMatchObject([{
      kind: "activity",
      title: "thread_reverted",
      status: "completed",
      metadata: { value: { revertedTurnIds: ["turn:2"] } },
    }]);
  });

  it("keeps multiple assistant items and orders content parts by provider content index", () => {
    const events = [
      stored(4, { type: "content_delta", payload: { itemId: "assistant-a", streamKind: "assistant_text", contentIndex: 1, delta: "world" } }),
      stored(2, { type: "content_delta", payload: { itemId: "assistant-a", streamKind: "assistant_text", contentIndex: 0, delta: "Hello " } }),
      stored(3, { type: "content_delta", payload: { itemId: "assistant-b", streamKind: "assistant_text", contentIndex: 0, delta: "Second answer" } }),
      stored(5, { type: "content_delta", payload: { itemId: "assistant-a", streamKind: "assistant_text", contentIndex: 0, delta: "there " } }),
      stored(6, { type: "content_delta", payload: { itemId: "assistant-a", streamKind: "assistant_text", contentIndex: 0, delta: "ignored" } }, { eventId: "event-5" }),
    ];

    const projection = projectCanonicalTimeline(events);
    expect(projection.rows.filter((row) => row.kind === "message")).toMatchObject([
      { id: "message:assistant-a", markdown: "Hello there world" },
      { id: "message:assistant-b", markdown: "Second answer" },
    ]);
    expect(projection.ignoredDuplicateEventIds).toEqual(["event-5"]);
  });

  it("attaches duration, effective model, usage, and deduplicated changed files to the final answer", () => {
    const changed = { relativePath: "src/app.ts", previousRelativePath: null, additions: 4, deletions: 1, binary: false, status: "modified" };
    const events = [
      stored(1, { type: "turn_started", payload: { providerTurnId: "provider-turn-1", state: "active", modes: { safetyMode: "supervised", interactionMode: "build" }, modelId: "gpt-5", modelOptions: [] } }),
      stored(2, { type: "content_delta", payload: { itemId: "answer", streamKind: "assistant_text", contentIndex: 0, delta: "Done" } }),
      stored(3, { type: "diff_updated", payload: { source: "provider", files: [changed], providerDiff: null } }),
      stored(4, { type: "model_rerouted", payload: { requestedModelId: "gpt-5", effectiveModelId: "gpt-5.1", reason: "Capacity" } }),
      stored(5, { type: "turn_completed", payload: { state: "completed", stopReason: "end_turn", usage: { inputTokens: 10, outputTokens: 4, cachedInputTokens: 2, contextTokens: 16, contextLimit: 100, cost: null }, changedFiles: [changed] } }, { createdAt: "2026-07-21T14:00:12.500Z" }),
    ];

    const projection = projectCanonicalTimeline(events);
    const answer = projection.rows.find((row) => row.id === "message:answer");
    expect(answer).toMatchObject({
      state: "complete",
      metadata: {
        durationMs: 12_500,
        modelId: "gpt-5.1",
        changedFiles: [changed],
        diffSources: ["provider"],
        usage: { inputTokens: 10, outputTokens: 4 },
      },
    });
  });

  it("projects lifecycle activity, plans, and pending request resolution", () => {
    const events = [
      stored(1, { type: "item_started", payload: { itemId: "command", kind: "command_execution", status: "active", title: "Run tests", detail: null, safeMetadata: null } }),
      stored(2, { type: "content_delta", payload: { itemId: "command", streamKind: "command_output", contentIndex: 0, delta: "Passed" } }),
      stored(3, { type: "item_completed", payload: { itemId: "command", kind: "command_execution", status: "completed", title: "Run tests", detail: "Passed", safeMetadata: null } }),
      stored(4, { type: "plan_updated", payload: { markdown: "Plan", steps: [{ id: "one", text: "Test", status: "completed" }] } }),
      stored(5, { type: "request_opened", payload: { requestId: "request-1", kind: "command_execution", title: "Run command?", detail: null, allowedDecisions: [], safePayload: { schemaVersion: 1, value: {} } } }),
      stored(6, { type: "request_resolved", payload: { requestId: "request-1", state: "resolved", decision: null } }),
    ];

    const projection = projectCanonicalTimeline(events);
    expect(projection.rows).toEqual(expect.arrayContaining([
      expect.objectContaining({ id: "activity:command", status: "completed", detail: "Passed" }),
      expect.objectContaining({ id: "plan:structured:turn-1", markdown: "Plan" }),
    ]));
    expect(projection.pendingRequests).toMatchObject([{ id: "request-1", state: "resolved" }]);
  });

  it("shows pending work only until meaningful provider output arrives", () => {
    const started = stored(1, { type: "turn_started", payload: { providerTurnId: "provider-turn-1", state: "active", modes: { safetyMode: "supervised", interactionMode: "build" }, modelId: "gpt-5", modelOptions: [] } });
    const user = stored(2, { type: "item_completed", payload: { itemId: "user", kind: "user_message", status: "completed", title: null, detail: "Please continue", safeMetadata: null } });
    expect(projectCanonicalTimeline([started, user]).rows).toMatchObject([
      { id: "message:user", role: "user" },
      { id: "turn-pending:turn-1", status: "active" },
    ]);
    expect(projectCanonicalTimeline([
      started,
      user,
      stored(3, { type: "content_delta", payload: { itemId: "answer", streamKind: "assistant_text", contentIndex: 0, delta: "Starting" } }),
    ]).rows.some((row) => row.id === "turn-pending:turn-1")).toBe(false);
  });

  it("folds settled work while leaving the final assistant answer visible", () => {
    const events = [
      stored(1, { type: "turn_started", payload: { providerTurnId: "provider-turn-1", state: "active", modes: { safetyMode: "supervised", interactionMode: "build" }, modelId: "gpt-5", modelOptions: [] } }),
      stored(2, { type: "item_completed", payload: { itemId: "command", kind: "command_execution", status: "completed", title: "Run tests", detail: "Passed", safeMetadata: null } }),
      stored(3, { type: "item_completed", payload: { itemId: "file", kind: "file_change", status: "completed", title: "Edit file", detail: "Changed", safeMetadata: null } }),
      stored(4, { type: "content_delta", payload: { itemId: "answer", streamKind: "assistant_text", contentIndex: 0, delta: "Finished" } }),
      stored(5, { type: "turn_aborted", payload: { state: "interrupted", reason: "User stopped", recoverable: true } }, { createdAt: "2026-07-21T14:00:05.000Z" }),
    ];
    const projection = projectCanonicalTimeline(events);

    const folded = buildTimelineDisplayRows(projection.rows, projection.turns);
    expect(folded).toMatchObject([
      { id: "turn-fold:turn-1", state: "interrupted", durationMs: 5_000, hiddenRows: [{ id: "activity:command" }, { id: "activity:file" }] },
      { id: "message:answer", state: "interrupted", markdown: "Finished" },
    ]);

    const expanded = buildTimelineDisplayRows(projection.rows, projection.turns, new Set(["turn-1"]));
    expect(expanded.map((row) => row.id)).toEqual([
      "turn-fold:turn-1",
      "activity:command",
      "activity:file",
      "message:answer",
    ]);
  });

  it("groups consecutive settled activities with stable row IDs", () => {
    const events = [
      stored(1, { type: "item_completed", payload: { itemId: "one", kind: "command_execution", status: "completed", title: "One", detail: null, safeMetadata: null } }, { turnId: null }),
      stored(2, { type: "item_completed", payload: { itemId: "two", kind: "file_change", status: "completed", title: "Two", detail: null, safeMetadata: null } }, { turnId: null }),
    ];
    const projection = projectCanonicalTimeline(events);
    expect(buildTimelineDisplayRows(projection.rows, projection.turns)).toMatchObject([
      {
        id: "activity-group:activity:one:activity:two",
        latest: { id: "activity:two" },
        earlierRows: [{ id: "activity:one" }],
      },
    ]);
    const groupId = "activity-group:activity:one:activity:two";
    expect(buildTimelineDisplayRows(projection.rows, projection.turns, new Set(), new Set([groupId]))).toMatchObject([
      { id: groupId, expanded: true, earlierRows: [{ id: "activity:one" }], latest: { id: "activity:two" } },
    ]);
  });

  it("keeps a matching late assistant event visible after turn settlement", () => {
    const projection = projectCanonicalTimeline([
      stored(1, { type: "turn_started", payload: { providerTurnId: "provider-turn-1", state: "active", modes: { safetyMode: "supervised", interactionMode: "build" }, modelId: "gpt-5", modelOptions: [] } }),
      stored(2, { type: "turn_completed", payload: { state: "completed", stopReason: "end_turn", usage: null, changedFiles: [] } }, { createdAt: "2026-07-21T14:00:02.000Z" }),
      stored(3, { type: "content_delta", payload: { itemId: "late-answer", streamKind: "assistant_text", contentIndex: 0, delta: "Recovered final answer" } }, { createdAt: "2026-07-21T14:00:03.000Z" }),
    ]);

    expect(projection.rows).toMatchObject([
      { id: "message:late-answer", markdown: "Recovered final answer", state: "complete", metadata: { durationMs: 2_000 } },
    ]);
  });

  it("hydrates the same display model from validated paged SQLite projections", () => {
    const items: ChatTimelineItemRead[] = [{
      activityId: "message-1",
      turnId: "turn-1",
      sequenceAnchor: 2,
      kind: "message",
      data: { schemaVersion: 1, value: { role: "assistant", markdown: "Persisted", streamingState: "complete", providerItemId: "provider-item", metadata: {}, createdAt: start, updatedAt: start } },
    }];
    const turns: ChatTimelineTurnRead[] = [{
      turnId: "turn-1",
      state: "completed",
      startedAt: start,
      completedAt: "2026-07-21T14:00:02.000Z",
      stopReason: "end_turn",
      modelId: "gpt-5",
      modelOptions: [],
      modes: { safetyMode: "supervised", interactionMode: "build" },
      usage: null,
      changedFiles: [],
    }];

    expect(projectTimelineReadModel(items, turns).rows).toMatchObject([
      { id: "message-1", markdown: "Persisted", state: "complete", metadata: { durationMs: 2_000, modelId: "gpt-5" } },
    ]);
  });

  it("bounds and validates durable user context metadata", () => {
    const context = parseTimelineUserContext({
      attachments: [{ attachmentId: "attachment-1", displayName: "diagram.png", kind: "image", byteSize: 1024, status: "persisted" }, { filename: 42 }],
      mentions: [{ relativePath: "src/main.ts", kind: "file" }, { relativePath: null }],
      terminalContext: ["pnpm test", { label: "Focused terminal selection" }, { label: 7 }],
      preCheckpointId: "checkpoint-1",
    });

    expect(context).toEqual({
      attachments: [{ id: "attachment-1", displayName: "diagram.png", kind: "image", byteSize: 1024, status: "persisted" }],
      mentions: [{ relativePath: "src/main.ts", kind: "file" }],
      terminalContext: ["pnpm test", "Focused terminal selection"],
      preCheckpointId: "checkpoint-1",
    });
  });
});
