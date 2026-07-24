import { describe, expect, it } from "vitest";
import type { ProviderCapabilities } from "./contracts";
import {
  autosizeComposerHeight,
  composerActionState,
  composerTokenTrigger,
  contextMeter,
  filterPromptCatalog,
  filterWorkspacePaths,
  interactionModeForPrompt,
  parseApprovalChoices,
  parseUserInputQuestions,
  queuedFollowupDispatchReady,
  rankedModels,
  shouldSendComposerKey,
  validateComposerSelections,
  validateImageFiles,
  validateModelOptions,
  validateUserInputAnswers,
} from "./composer-model";

function capabilities(...values: ProviderCapabilities["entries"][number]["capability"][]): ProviderCapabilities {
  return { entries: values.map((capability) => ({ capability, supported: true, explanation: null })) };
}

describe("Chat composer model", () => {
  it("prioritizes requests and Stop while selecting capability-driven follow-up behavior", () => {
    expect(composerActionState("waiting_for_approval", capabilities("steering"), true, true).primary).toBe("resolve_request");
    expect(composerActionState("active", capabilities("steering"), true, false)).toMatchObject({ primary: "stop", followup: "steer" });
    expect(composerActionState("active", capabilities("queued_follow_up"), true, false).followup).toBe("queue");
    expect(composerActionState("active", capabilities(), true, false).followup).toBe("retain");
    expect(composerActionState("ready", capabilities(), true, false)).toMatchObject({ primary: "send", sendEnabled: true });
    expect(queuedFollowupDispatchReady("ready", "completed")).toBe(true);
    expect(queuedFollowupDispatchReady("stopped", "failed")).toBe(true);
    expect(queuedFollowupDispatchReady("stopped", "active")).toBe(false);
  });

  it("requires every explicit first-use choice and workspace-specific Full access trust", () => {
    expect(validateComposerSelections({
      workspaceId: null,
      providerInstanceId: null,
      modelId: null,
      providerManagedModel: false,
      safetyMode: null,
      interactionMode: null,
      fullAccessTrusted: false,
    }, capabilities())).toHaveLength(5);
    expect(validateComposerSelections({
      workspaceId: "workspace-1",
      providerInstanceId: "codex",
      modelId: "gpt-5",
      providerManagedModel: false,
      safetyMode: "full_access",
      interactionMode: "plan",
      fullAccessTrusted: false,
    }, capabilities())).toEqual([
      { field: "interaction", message: "This provider does not support native Plan mode" },
      { field: "trust", message: "Confirm Full access for this provider and workspace" },
    ]);
  });

  it("detects mention, skill, and command tokens at the caret", () => {
    expect(composerTokenTrigger("Inspect @src/cal", 16)).toMatchObject({ kind: "mention", query: "src/cal" });
    expect(composerTokenTrigger("Use $doc", 8)).toMatchObject({ kind: "skill", query: "doc" });
    expect(composerTokenTrigger("/review", 7)).toMatchObject({ kind: "command", query: "review" });
  });

  it("maps the universal plan command to Plan mode and otherwise preserves the current mode", () => {
    expect(interactionModeForPrompt("  /plan inspect the architecture", "build")).toBe("plan");
    expect(interactionModeForPrompt("/PLAN", null)).toBe("plan");
    expect(interactionModeForPrompt("Implement the plan", "plan")).toBe("plan");
    expect(interactionModeForPrompt("Implement the change", null)).toBe("build");
  });

  it("fuzzy-ranks provider menus and workspace paths while preserving stale entries", () => {
    const catalog = [
      { value: "/review", label: "Review changes", description: null, kind: "command" as const, stale: true },
      { value: "$docs", label: "Documentation", description: null, kind: "skill" as const, stale: false },
    ];
    expect(filterPromptCatalog(catalog, "command", "rvw")).toMatchObject([{ value: "/review", stale: true }]);
    expect(filterPromptCatalog(catalog, "skill", "doc")).toMatchObject([{ value: "$docs" }]);
    expect(filterWorkspacePaths([
      { relativePath: "src/calendar/view.ts", displayName: "view.ts", kind: "file", ignored: false },
      { relativePath: "docs/calendar.md", displayName: "calendar.md", kind: "file", ignored: true },
    ], "cal").map((entry) => entry.relativePath)).toEqual(["src/calendar/view.ts", "docs/calendar.md"]);
  });

  it("ranks favorite and recent models before the remaining catalog without replacing raw IDs", () => {
    const models = ["z-model", "favorite", "recent"].map((id) => ({
      id, displayName: id, description: null, contextLimit: null, availability: "available" as const,
      capabilities: [], options: [], custom: false,
    }));
    expect(rankedModels(models, ["favorite"], ["recent"], "").map((model) => model.id)).toEqual(["favorite", "recent", "z-model"]);
  });

  it("parses safe approval and structured-question payloads", () => {
    expect(parseApprovalChoices({ schemaVersion: 1, value: [{ id: "allow", label: "Allow once", decisionKind: "allow_once", description: null }] })).toHaveLength(1);
    expect(parseUserInputQuestions({ schemaVersion: 1, value: [{ id: "q1", question: "Choose", options: [{ id: "a", label: "A" }], multiple: false, freeFormAllowed: true, required: true }] })).toMatchObject([{ id: "q1", options: [{ id: "a" }] }]);
  });

  it("validates typed traits and structured answers without accepting unknown options", () => {
    expect(validateModelOptions([
      { kind: "choice", key: "effort", label: "Effort", description: null, options: [{ value: "high", label: "High", description: null }], defaultValue: "high" },
    ], [{ key: "effort", value: { kind: "choice", value: "invalid" } }])).toEqual(["effort"]);
    const questions = parseUserInputQuestions({
      schemaVersion: 1,
      value: [{ id: "q1", question: "Choose", options: [{ id: "a", label: "A" }], multiple: false, freeFormAllowed: false, required: true }],
    });
    expect(validateUserInputAnswers(questions, [{ questionId: "q1", selectedOptionIds: [], freeFormText: null }])).toEqual(["q1"]);
    expect(validateUserInputAnswers(questions, [{ questionId: "q1", selectedOptionIds: ["a"], freeFormText: null }])).toEqual([]);
    expect(validateUserInputAnswers(questions, [{ questionId: "q1", selectedOptionIds: ["missing"], freeFormText: null }])).toEqual(["q1"]);
  });

  it("shows only provider-reported context facts and caps visual ratios", () => {
    expect(contextMeter(null, 100)).toBeNull();
    expect(contextMeter(90, 100)).toMatchObject({ ratio: 0.9, warning: true });
    expect(contextMeter(120, 100)?.ratio).toBe(1);
    expect(contextMeter(40, null)).toMatchObject({ maximumTokens: null, ratio: null });
  });

  it("implements send keys, bounded autosize, and image limits", () => {
    expect(shouldSendComposerKey({ key: "Enter", shiftKey: false, ctrlKey: false, metaKey: false, isComposing: false }, "enter")).toBe(true);
    expect(shouldSendComposerKey({ key: "Enter", shiftKey: true, ctrlKey: false, metaKey: false, isComposing: false }, "enter")).toBe(false);
    expect(autosizeComposerHeight(1, 20)).toBe(76);
    expect(autosizeComposerHeight(1_000, 20)).toBe(216);
    expect(validateImageFiles(Array.from({ length: 2 }, (_, index) => ({ name: `${index}.png`, size: 10, type: "image/png" })) as File[], 7)).toContain("8 images");
    expect(validateImageFiles([{ name: "large.png", size: 2 * 1024 * 1024, type: "image/png" }] as File[], 1, 49 * 1024 * 1024)).toContain("50 MiB");
  });
});
