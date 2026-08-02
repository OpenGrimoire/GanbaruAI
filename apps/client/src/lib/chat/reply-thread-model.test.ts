import { describe, expect, it } from "vitest";
import type { ChatAgentRunRead, VersionedJson } from "./contracts";
import {
  latestRenderableAgentRun,
  repliesWithoutRenderedRunProjection,
} from "./reply-thread-model";

function richContent(value: VersionedJson["value"]): VersionedJson {
  return { schemaVersion: 1, value };
}

function agentRun(id: string, providerExecutionThreadId: string | null): ChatAgentRunRead {
  return {
    id,
    assignmentId: "assignment-1",
    projectId: "project-1",
    workingFolderId: "folder-1",
    teammatePolicyRevisionId: "policy-1",
    effort: "medium",
    providerExecutionTurnId: `turn-${id}`,
    providerExecutionThreadId,
    state: "working",
    runOrdinal: 1,
    createdAt: "2026-08-02T08:00:00Z",
    updatedAt: "2026-08-02T08:00:01Z",
  };
}

describe("reply thread provider projection", () => {
  it("selects the newest run that has renderable provider history", () => {
    expect(latestRenderableAgentRun([
      agentRun("run-1", "thread-1"),
      agentRun("run-2", null),
      agentRun("run-3", "thread-3"),
    ])?.id).toBe("run-3");
  });

  it("hides only the materialized copy of the rendered run", () => {
    const replies = [
      { id: "human", richContent: richContent({ type: "doc" }) },
      { id: "older-run", richContent: richContent({ type: "agent_update", agentRunId: "run-1" }) },
      { id: "current-run", richContent: richContent({ type: "agent_update", agentRunId: "run-2" }) },
    ];

    expect(repliesWithoutRenderedRunProjection(replies, "run-2").map((reply) => reply.id))
      .toEqual(["human", "older-run"]);
    expect(repliesWithoutRenderedRunProjection(replies, null)).toEqual(replies);
  });
});
