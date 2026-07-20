import { describe, expect, it } from "vitest";
import {
  parseCanonicalRuntimeEvent,
  parseCanonicalStoredEvent,
  parseChatError,
  parseChatThreadShell,
  parseProviderFamilyMetadata,
  parseProviderInstanceConfig,
  parseProviderModelCatalog,
} from "./validation";

const timestamp = "2026-07-20T12:00:00Z";

function metadataFixture(): Record<string, unknown> {
  return {
    familyId: "codex",
    displayName: "Codex",
    configurationSchemaVersion: 1,
    supportedPlatforms: ["linux", "windows", "macos"],
    minimumTestedCliVersion: null,
    defaultExecutableCandidates: ["codex"],
    implementationStatus: "metadata_only",
    potentialCapabilities: ["native_resume", "approvals", "structured_questions"],
    unavailableReason: "This provider driver has not been implemented yet.",
  };
}

function runtimeEventFixture(): Record<string, unknown> {
  return {
    schemaVersion: 1,
    eventId: "event-1",
    providerFamilyId: "codex",
    providerInstanceId: "codex-personal",
    threadId: "thread-1",
    createdAt: timestamp,
    turnId: "turn-1",
    providerTurnId: "provider-turn-1",
    providerItemId: "provider-item-1",
    providerRequestId: null,
    providerTaskId: null,
    providerReference: { schemaVersion: 1, value: { sequence: 4 } },
    event: {
      type: "content_delta",
      payload: {
        itemId: "provider-item-1",
        streamKind: "assistant_text",
        contentIndex: 0,
        delta: "Hello",
      },
    },
    redactedDiagnostic: null,
  };
}

describe("Chat provider contracts", () => {
  it("parses metadata-only provider registry entries", () => {
    expect(parseProviderFamilyMetadata(metadataFixture())).toEqual(metadataFixture());
  });

  it("preserves unknown provider instance fields and versioned configuration", () => {
    const fixture = {
      schemaVersion: 1,
      instanceId: "future-instance",
      familyId: "future-provider",
      label: "Future provider",
      accentColor: null,
      enabled: true,
      executable: "future-agent",
      providerHome: null,
      launchArguments: ["serve"],
      environment: { SAFE_MODE: "1" },
      credentialReferences: { API_TOKEN: "credential-1" },
      visibleModelIds: ["model-1"],
      favoriteModelIds: [],
      providerConfig: {
        schemaVersion: 7,
        value: { futureOption: [true, 4, "value"] },
      },
      futureCommonField: { retained: true },
    };

    expect(parseProviderInstanceConfig(fixture)).toEqual(fixture);
  });

  it("parses typed model options and provider capability data", () => {
    const fixture = {
      instanceId: "codex-personal",
      models: [{
        id: "gpt-5-codex",
        displayName: "GPT-5 Codex",
        description: null,
        contextLimit: 200_000,
        availability: "available",
        capabilities: ["images", "context_usage"],
        options: [{
          kind: "choice",
          key: "reasoning_effort",
          label: "Reasoning effort",
          description: null,
          options: [{ value: "high", label: "High", description: null }],
          defaultValue: "high",
        }],
        custom: false,
      }],
      source: "provider",
      discoveredAt: timestamp,
      stale: false,
    };

    expect(parseProviderModelCatalog(fixture)).toEqual(fixture);
  });

  it("rejects partial metadata and unsupported capability values", () => {
    const missing = metadataFixture();
    delete missing.displayName;
    expect(() => parseProviderFamilyMetadata(missing)).toThrow("displayName must be a string");

    expect(() => parseProviderFamilyMetadata({
      ...metadataFixture(),
      potentialCapabilities: ["silent_auto_approval"],
    })).toThrow("potentialCapabilities[0] has an unsupported value");
  });

  it("rejects unsafe numeric model data", () => {
    expect(() => parseProviderModelCatalog({
      instanceId: "codex-personal",
      models: [{
        id: "model-1",
        displayName: "Model",
        description: null,
        contextLimit: Number.MAX_SAFE_INTEGER + 1,
        availability: "available",
        capabilities: [],
        options: [],
        custom: false,
      }],
      source: "provider",
      discoveredAt: timestamp,
      stale: false,
    })).toThrow("contextLimit must be a safe integer");
  });
});

describe("Chat event contracts", () => {
  it("parses a canonical content delta envelope", () => {
    expect(parseCanonicalRuntimeEvent(runtimeEventFixture())).toEqual(runtimeEventFixture());
  });

  it("parses the explicit bounded unknown-event fallback", () => {
    const fixture = runtimeEventFixture();
    fixture.event = {
      type: "unknown",
      payload: {
        sourceType: "provider.future_event",
        summary: "A future provider event was received.",
        safePayload: { schemaVersion: 1, value: { safeType: "future_event" } },
      },
    };

    expect(parseCanonicalRuntimeEvent(fixture)).toEqual(fixture);
  });

  it("rejects unknown canonical event names instead of trusting their payload", () => {
    const fixture = runtimeEventFixture();
    fixture.event = { type: "provider_raw_event", payload: { secret: "must not cross" } };
    expect(() => parseCanonicalRuntimeEvent(fixture)).toThrow("event.type has an unsupported value");
  });

  it("rejects malformed nested event payloads", () => {
    const fixture = runtimeEventFixture();
    fixture.event = {
      type: "content_delta",
      payload: {
        itemId: "provider-item-1",
        streamKind: "assistant_text",
        contentIndex: -1,
        delta: "Hello",
      },
    };
    expect(() => parseCanonicalRuntimeEvent(fixture)).toThrow("contentIndex must not be negative");
  });

  it("validates flattened stored-event sequence fields", () => {
    const fixture = { ...runtimeEventFixture(), sequence: 5, ingestedAt: timestamp };
    expect(parseCanonicalStoredEvent(fixture)).toEqual(fixture);
    expect(() => parseCanonicalStoredEvent({ ...fixture, sequence: 1.5 })).toThrow("sequence must be a safe integer");
  });

  it("rejects invalid UTC timestamps and provider identifiers", () => {
    expect(() => parseCanonicalRuntimeEvent({
      ...runtimeEventFixture(),
      createdAt: "2026-07-20T07:00:00-05:00",
    })).toThrow("createdAt must be an RFC 3339 UTC timestamp");

    expect(() => parseCanonicalRuntimeEvent({
      ...runtimeEventFixture(),
      providerFamilyId: "codex\u0000hidden",
    })).toThrow("providerFamilyId contains a control character");
  });
});

describe("Chat read and error contracts", () => {
  it("parses lightweight thread shells without message history", () => {
    const fixture = {
      id: "thread-1",
      workspaceId: "workspace-1",
      projectId: "project-1",
      title: "Implement Chat contracts",
      providerFamilyId: "codex",
      providerInstanceId: "codex-personal",
      providerThreadId: null,
      modelId: "gpt-5-codex",
      modelOptions: [],
      modes: { safetyMode: "supervised", interactionMode: "build" },
      state: "idle",
      latestTurnState: "completed",
      latestPreview: "Contracts compile.",
      messageCount: 2,
      revision: 4,
      lastEventSequence: 12,
      lastActivityAt: timestamp,
      archivedAt: null,
    };
    expect(parseChatThreadShell(fixture)).toEqual(fixture);
  });

  it("parses structured errors and rejects unbounded unknown detail types", () => {
    const fixture = {
      code: "driver_unavailable",
      message: "The provider driver is not implemented.",
      field: null,
      recoverable: true,
      details: { providerFamilyId: "codex" },
    };
    expect(parseChatError(fixture)).toEqual(fixture);
    expect(() => parseChatError({ ...fixture, details: undefined })).toThrow("details is not valid JSON");
  });
});
