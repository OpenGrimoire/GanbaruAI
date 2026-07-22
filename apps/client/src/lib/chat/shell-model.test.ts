import { describe, expect, it } from "vitest";
import type { ChatThreadShellRead, ChatWorkspaceRead, ProviderInstanceRead } from "./contracts";
import type { Project, ProjectGroup } from "$lib/projects/types";
import { buildChatRailModel, filterArchivedThreads, filterThreadTitles, nextThreadIndex, partitionThreadSearchResults, resolveChatFirstUseState, threadStatus } from "./shell-model";

const timestamp = "2026-07-20T12:00:00.000Z";

function workspace(id = "workspace", projectId: string | null = "project"): ChatWorkspaceRead {
  return {
    workspace: {
      id,
      projectId,
      displayName: id,
      repositoryKind: "git",
      repositoryIdentity: "git-sha256:test",
      createdAt: timestamp,
      updatedAt: timestamp,
      archivedAt: null,
      revision: 1,
    },
    bindingStatus: "available",
    canonicalPath: "/workspace",
    lastVerifiedAt: timestamp,
    currentBranch: "feat/chat",
  };
}

function provider(state: ProviderInstanceRead["lastProbe"] extends infer _Probe ? "healthy" | "authentication_required" : never = "healthy"): ProviderInstanceRead {
  return {
    configuration: {
      schemaVersion: 1,
      instanceId: "codex",
      familyId: "codex",
      label: "Codex",
      accentColor: "#2563eb",
      enabled: true,
      executable: "codex",
      providerHome: null,
      launchArguments: [],
      environment: {},
      credentialReferences: {},
      visibleModelIds: [],
      favoriteModelIds: [],
      providerConfig: { schemaVersion: 1, value: {} },
    },
    lastProbe: {
      instanceId: "codex",
      state,
      version: "1.0",
      accountLabel: null,
      capabilities: { entries: [] },
      checkedAt: timestamp,
      detail: null,
    },
    lastSuccessfulProbeAt: timestamp,
    modelCatalog: null,
  };
}

function thread(id = "thread", overrides: Partial<ChatThreadShellRead> = {}): ChatThreadShellRead {
  return {
    id,
    workspaceId: "workspace",
    projectId: "project",
    title: "Fix calendar",
    providerFamilyId: "codex",
    providerInstanceId: "codex",
    providerThreadId: null,
    modelId: null,
    modelOptions: [],
    modes: { safetyMode: "supervised", interactionMode: "build" },
    state: "idle",
    latestTurnState: null,
    latestPreview: null,
    messageCount: 0,
    revision: 1,
    lastEventSequence: 0,
    lastActivityAt: timestamp,
    unreadAt: null,
    archivedAt: null,
    ...overrides,
  };
}

describe("Chat shell model", () => {
  it("routes every first-use and unavailable state precisely", () => {
    const base = { providers: [provider()], workspaces: [workspace()], selectedWorkspaceId: "workspace", selectedThreadId: null, threads: [] };
    expect(resolveChatFirstUseState({ ...base, providers: [] }).kind).toBe("no_provider");
    expect(resolveChatFirstUseState({ ...base, workspaces: [] }).kind).toBe("no_workspace");
    expect(resolveChatFirstUseState({ ...base, selectedWorkspaceId: null }).kind).toBe("select_workspace");
    expect(resolveChatFirstUseState({ ...base, workspaces: [{ ...workspace(), bindingStatus: "missing" }] }).kind).toBe("missing_binding");
    expect(resolveChatFirstUseState(base).kind).toBe("no_thread");
    expect(resolveChatFirstUseState({ ...base, providers: [provider("authentication_required")] }).kind).toBe("provider_unavailable");
    expect(resolveChatFirstUseState({ ...base, providers: [provider("authentication_required"), { ...provider(), configuration: { ...provider().configuration, instanceId: "healthy" }, lastProbe: { ...provider().lastProbe!, instanceId: "healthy" } }] }).kind).toBe("no_thread");
    expect(resolveChatFirstUseState({ ...base, selectedThreadId: "thread", threads: [thread()] }).kind).toBe("conversation");
    expect(resolveChatFirstUseState({ ...base, providers: [], selectedThreadId: "thread", threads: [thread()] }).kind).toBe("provider_unavailable");
    expect(resolveChatFirstUseState({ ...base, providers: [provider("authentication_required")], selectedThreadId: "thread", threads: [thread()] }).kind).toBe("provider_unavailable");
    expect(resolveChatFirstUseState({ ...base, providers: [provider("authentication_required")], selectedThreadId: "thread", threads: [thread("thread", { archivedAt: timestamp })] }).kind).toBe("archived_thread");
  });

  it("prioritizes actionable and active statuses over unread", () => {
    expect(threadStatus(thread("answer", { latestTurnState: "waiting_for_user_input", unreadAt: timestamp }))).toBe("waiting_answer");
    expect(threadStatus(thread("approval", { latestTurnState: "waiting_for_approval" }))).toBe("waiting_approval");
    expect(threadStatus(thread("active", { latestTurnState: "active" }))).toBe("working");
    expect(threadStatus(thread("failed", { latestTurnState: "failed" }))).toBe("error");
    expect(threadStatus(thread("unread", { unreadAt: timestamp }))).toBe("unread");
  });

  it("groups Projects without copying their identity and subdivides multiple workspaces", () => {
    const groups: ProjectGroup[] = [{ id: "group", name: "Work", icon: "lucide:folder", sortOrder: 1, collapsed: true, createdAt: timestamp, updatedAt: timestamp }];
    const projects: Project[] = [{ id: "project", groupId: "group", name: "Ganbaru", icon: "lucide:folder", sortOrder: 1, status: "active", defaultEventName: null, defaultEventTimeMode: "timed", defaultEventDurationMinutes: null, defaultPomodoroMode: "preset", defaultIdleSettingsSource: "global", defaultIdlePauseEnabled: true, defaultIdleThresholdMinutes: 5, createdAt: timestamp, updatedAt: timestamp }];
    const model = buildChatRailModel(groups, projects, [workspace("workspace"), workspace("second"), workspace("standalone", null)], [thread()], "thread");
    expect(model.groups[0].projects[0].project).toBe(projects[0]);
    expect(model.groups[0].projects[0].workspaces.every((entry) => entry.showSubdivision)).toBe(true);
    expect(model.standalone).toHaveLength(1);
    expect(model.retainedThreadId).toBe("thread");
  });

  it("filters titles locally and wraps keyboard traversal", () => {
    expect(filterThreadTitles([thread(), thread("other", { title: "Añadir notas" })], "anadir").map((entry) => entry.id)).toEqual(["other"]);
    expect(nextThreadIndex(1, 2, "next")).toBe(0);
    expect(nextThreadIndex(0, 2, "previous")).toBe(1);
    expect(nextThreadIndex(-1, 0, "next")).toBe(-1);
  });

  it("partitions search results and filters the archive by project", () => {
    const active = thread("active");
    const archived = thread("archived", { archivedAt: timestamp, projectId: "project-2", title: "Archived calendar" });
    expect(partitionThreadSearchResults([archived, active])).toEqual({ active: [active], archived: [archived] });
    expect(filterArchivedThreads([active, archived], "calendar", "project-2")).toEqual([archived]);
    expect(filterArchivedThreads([active, archived], "calendar", "project")).toEqual([]);
  });
});
