import { describe, expect, it, vi } from "vitest";
import type { ChatWorkspaceRead } from "./contracts";
import { ChatWorkspaceController, type ChatWorkspaceApi } from "./workspace-controller";

const timestamp = "2026-07-20T12:00:00Z";

function workspace(id: string, displayName = "Frontend"): ChatWorkspaceRead {
  return {
    workspace: {
      id,
      projectId: "project-1",
      displayName,
      repositoryKind: "none",
      repositoryIdentity: null,
      createdAt: timestamp,
      updatedAt: timestamp,
      archivedAt: null,
      revision: 1,
    },
    bindingStatus: "unbound",
    canonicalPath: null,
    lastVerifiedAt: null,
    currentBranch: null,
  };
}

function fakeApi(overrides: Partial<ChatWorkspaceApi> = {}): ChatWorkspaceApi {
  return {
    list: vi.fn(async () => []),
    create: vi.fn(async (request) => workspace(request.id, request.displayName)),
    rename: vi.fn(async (id, name) => workspace(id, name)),
    bind: vi.fn(async (id): Promise<ChatWorkspaceRead | null> => ({
      ...workspace(id),
      bindingStatus: "available",
    })),
    rebind: vi.fn(async (id): Promise<ChatWorkspaceRead | null> => ({
      ...workspace(id),
      bindingStatus: "available",
    })),
    removeBinding: vi.fn(async (id) => workspace(id)),
    archive: vi.fn(async (id) => ({
      ...workspace(id),
      workspace: { ...workspace(id).workspace, archivedAt: timestamp },
    })),
    restore: vi.fn(async (id) => workspace(id)),
    openFolder: vi.fn(async () => undefined),
    ...overrides,
  };
}

describe("ChatWorkspaceController", () => {
  it("loads workspace rows and applies narrow mutations", async () => {
    const api = fakeApi({ list: vi.fn(async () => [workspace("workspace-1")]) });
    const controller = new ChatWorkspaceController(api);

    await controller.refresh();
    await controller.rename("workspace-1", "Application");

    expect(controller.snapshot()).toMatchObject({
      loading: false,
      error: null,
      workspaces: [{ workspace: { id: "workspace-1", displayName: "Application" } }],
    });
  });

  it("keeps existing state when the native folder picker is cancelled", async () => {
    const api = fakeApi({
      list: vi.fn(async () => [workspace("workspace-1")]),
      bind: vi.fn(async () => null),
    });
    const controller = new ChatWorkspaceController(api);
    await controller.refresh();

    expect(await controller.bind("workspace-1", "Choose workspace folder")).toBeNull();
    expect(controller.snapshot().workspaces[0].bindingStatus).toBe("unbound");
  });

  it("ignores stale refresh results", async () => {
    let resolveFirst: ((rows: ChatWorkspaceRead[]) => void) | undefined;
    const first = new Promise<ChatWorkspaceRead[]>((resolve) => {
      resolveFirst = resolve;
    });
    const api = fakeApi({
      list: vi.fn()
        .mockReturnValueOnce(first)
        .mockResolvedValueOnce([workspace("workspace-new")]),
    });
    const controller = new ChatWorkspaceController(api);

    const staleRefresh = controller.refresh();
    await controller.refresh();
    resolveFirst?.([workspace("workspace-stale")]);
    await staleRefresh;

    expect(controller.snapshot().workspaces.map((row) => row.workspace.id)).toEqual(["workspace-new"]);
  });
});
