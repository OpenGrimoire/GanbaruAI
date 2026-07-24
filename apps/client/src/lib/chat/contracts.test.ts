import { describe, expect, it } from "vitest";
import {
  defaultChatVaultConfig,
  parseChatConfigRoot,
  parseChatVaultConfig,
  parseCanonicalRuntimeEvent,
  parseCanonicalStoredEvent,
  parseChatError,
  parseChatDiagnosticsRead,
  parseChatThreadShell,
  parseChatWorkspaceRead,
  parseProviderFamilyMetadata,
  parseProviderInstanceConfig,
  parseProviderModelCatalog,
  parseProviderRefreshResult,
  normalizeProviderModelCatalogForFamily,
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

  it("validates provider discovery refresh counts", () => {
    const fixture = {
      familiesScanned: 4,
      providersChecked: 2,
      providersDiscovered: 1,
      issues: 1,
    };

    expect(parseProviderRefreshResult(fixture)).toEqual(fixture);
    expect(() => parseProviderRefreshResult({ ...fixture, issues: -1 })).toThrow(
      "issues must not be negative",
    );
  });

  it("validates internal checkpoint restore audit events", () => {
    const fixture = runtimeEventFixture();
    fixture.turnId = null;
    fixture.event = {
      type: "thread_reverted",
      payload: {
        checkpointId: "checkpoint:1",
        revertedTurnIds: ["turn:2"],
        providerHistoryAction: "fork_required",
      },
    };
    expect(parseCanonicalRuntimeEvent(fixture).event).toEqual(fixture.event);
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

  it("removes deprecated models from validated catalogs", () => {
    const parsed = parseProviderModelCatalog({
      instanceId: "codex-personal",
      models: [{
        id: "gpt-old",
        displayName: "GPT Old",
        description: null,
        contextLimit: null,
        availability: "deprecated",
        capabilities: [],
        options: [],
        custom: false,
      }],
      source: "provider",
      discoveredAt: timestamp,
      stale: false,
    });

    expect(parsed.models).toEqual([]);
  });

  it("replaces cached Claude default aliases with the concrete model entry", () => {
    const catalog = parseProviderModelCatalog({
      instanceId: "claude",
      models: [
        {
          id: "default",
          displayName: "Default (Opus 4.8)",
          description: "Use the default model (currently Opus 4.8)",
          contextLimit: null,
          availability: "available",
          capabilities: [],
          options: [],
          custom: false,
        },
        {
          id: "opus",
          displayName: "Opus 4.8",
          description: null,
          contextLimit: null,
          availability: "available",
          capabilities: [],
          options: [],
          custom: false,
        },
        {
          id: "sonnet",
          displayName: "Sonnet 5",
          description: null,
          contextLimit: null,
          availability: "available",
          capabilities: [],
          options: [],
          custom: false,
        },
      ],
      source: "provider",
      discoveredAt: timestamp,
      stale: false,
    });

    expect(normalizeProviderModelCatalogForFamily(catalog, "claude").models.map((model) => ({
      id: model.id,
      displayName: model.displayName,
      optionKeys: model.options.map((option) => option.key),
    }))).toEqual([
      { id: "opus", displayName: "Opus 4.8", optionKeys: ["fastMode"] },
      { id: "sonnet", displayName: "Sonnet 5", optionKeys: [] },
    ]);
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

describe("Chat vault configuration", () => {
  it("uses explicit safe defaults when the Chat branch is absent", () => {
    const defaults = defaultChatVaultConfig();
    expect(parseChatConfigRoot({ language: "en" })).toEqual(defaults);
    expect(defaults.panels).toEqual({ railWidthPx: 320, inspectorWidthPx: 520 });
  });

  it("preserves portable unknown fields while validating known fields", () => {
    const fixture = {
      schemaVersion: 1,
      providers: [{
        schemaVersion: 1,
        instanceId: "codex-personal",
        familyId: "codex",
        label: "Personal Codex",
        accentColor: null,
        enabled: true,
        launchArguments: [],
        environment: {},
        credentialReferences: { API_TOKEN: "credential-1" },
        visibleModelIds: ["gpt-5-codex"],
        favoriteModelIds: ["gpt-5-codex"],
        providerConfig: { schemaVersion: 1, value: {} },
        futurePortableOption: { enabled: true },
      }],
      automaticProviderSetupDisabled: [],
      rememberedSelections: [{
        workspaceId: "workspace-1",
        providerInstanceId: "codex-personal",
        modelId: "gpt-5-codex",
        providerManagedModel: false,
        modelOptions: [],
        safetyMode: "ask_for_approval",
        interactionMode: "build",
      }],
      workspaceProviderPreferences: {},
      panels: { railWidthPx: 280, inspectorWidthPx: 420 },
      behavior: {
        sendKey: "enter",
        restoreLastSelectedThread: true,
        showReasoningSummaries: true,
        automaticallyFoldSettledWork: true,
        terminalScrollbackLines: 12_000,
        idleSessionTimeoutSeconds: 900,
        confirmMultilineTerminalPaste: true,
      },
      futureRootOption: "preserved",
    };

    expect(parseChatVaultConfig(fixture)).toEqual(fixture);
  });

  it("rejects machine-specific paths and implicit model selection", () => {
    expect(() => parseChatVaultConfig({
      ...defaultChatVaultConfig(),
      providers: [{
        schemaVersion: 1,
        instanceId: "codex-personal",
        familyId: "codex",
        label: "Personal Codex",
        accentColor: null,
        enabled: true,
        executable: "/usr/bin/codex",
        launchArguments: [],
        environment: {},
        credentialReferences: {},
        visibleModelIds: [],
        favoriteModelIds: [],
        providerConfig: { schemaVersion: 1, value: {} },
      }],
    })).toThrow("contains machine-specific field executable");

    expect(() => parseChatVaultConfig({
      ...defaultChatVaultConfig(),
      rememberedSelections: [{
        workspaceId: "workspace-1",
        providerInstanceId: "codex-personal",
        modelId: null,
        providerManagedModel: false,
        modelOptions: [],
        safetyMode: "ask_for_approval",
        interactionMode: "build",
      }],
    })).toThrow("modelId or provider-managed model state is required");
  });
});

describe("Chat workspace contracts", () => {
  it("parses logical workspace identity separately from the device binding", () => {
    const fixture = {
      workspace: {
        id: "workspace-1",
        projectId: "project-1",
        displayName: "Frontend",
        repositoryKind: "git",
        repositoryIdentity: "git-sha256:abc123",
        createdAt: timestamp,
        updatedAt: timestamp,
        archivedAt: null,
        revision: 2,
      },
      bindingStatus: "available",
      canonicalPath: "/home/user/project",
      lastVerifiedAt: timestamp,
      currentBranch: "feat/chat",
    };

    expect(parseChatWorkspaceRead(fixture)).toEqual(fixture);
  });

  it("rejects unknown binding states and malformed timestamps", () => {
    const fixture = {
      workspace: {
        id: "workspace-1",
        projectId: null,
        displayName: "Standalone",
        repositoryKind: "none",
        repositoryIdentity: null,
        createdAt: timestamp,
        updatedAt: timestamp,
        archivedAt: null,
        revision: 1,
      },
      bindingStatus: "trusted_forever",
      canonicalPath: null,
      lastVerifiedAt: null,
      currentBranch: null,
    };
    expect(() => parseChatWorkspaceRead(fixture)).toThrow("bindingStatus has an unsupported value");
    expect(() => parseChatWorkspaceRead({
      ...fixture,
      bindingStatus: "unbound",
      workspace: { ...fixture.workspace, updatedAt: "tomorrow" },
    })).toThrow("updatedAt must be an RFC 3339 UTC timestamp");
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
  it("parses bounded Chat diagnostics and rejects invalid retention", () => {
    const fixture = {
      preferences: { captureEnabled: false, retentionDays: 7 },
      capturedFields: ["event type"], excludedFields: ["credentials"], storageLocation: "SQLite",
      projectionHealthy: true, inconsistentProjectionCount: 0, credentialStoreAvailable: true,
      providerProbeHealthy: 1, providerProbeUnhealthy: 0, providerProbeUnknown: 1,
      liveProviderProcesses: 1, activeTurns: 0, liveTerminals: 0,
      counts: {
        retainedEvents: 0, retainedBytes: 0, attachmentCount: 1, attachmentBytes: 12,
        pendingAttachmentCleanup: 0, failedAttachmentCleanup: 0, commandOutputEvents: 2,
        commandOutputBytes: 20, checkpointFailures: 0, pendingCheckpointCleanup: 0,
        failedCheckpointCleanup: 0,
      },
    };
    expect(parseChatDiagnosticsRead(fixture)).toEqual(fixture);
    expect(() => parseChatDiagnosticsRead({ ...fixture, preferences: { captureEnabled: true, retentionDays: 31 } })).toThrow("between 1 and 30");
  });

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
      modes: { safetyMode: "ask_for_approval", interactionMode: "build" },
      state: "idle",
      latestTurnState: "completed",
      latestPreview: "Contracts compile.",
      messageCount: 2,
      revision: 4,
      lastEventSequence: 12,
      lastActivityAt: timestamp,
      unreadAt: null,
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
