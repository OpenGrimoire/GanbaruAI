import * as chatApi from "$lib/api/chat";
import type {
  ChatWorkspaceId,
  ChatWorkspaceRead,
  CreateChatWorkspaceRequest,
} from "$lib/chat/contracts";

export interface ChatWorkspaceApi {
  list(): Promise<ChatWorkspaceRead[]>;
  create(request: CreateChatWorkspaceRequest): Promise<ChatWorkspaceRead>;
  rename(workspaceId: ChatWorkspaceId, displayName: string): Promise<ChatWorkspaceRead>;
  bind(workspaceId: ChatWorkspaceId): Promise<ChatWorkspaceRead | null>;
  rebind(workspaceId: ChatWorkspaceId): Promise<ChatWorkspaceRead | null>;
  removeBinding(workspaceId: ChatWorkspaceId): Promise<ChatWorkspaceRead>;
  archive(workspaceId: ChatWorkspaceId): Promise<ChatWorkspaceRead>;
  restore(workspaceId: ChatWorkspaceId): Promise<ChatWorkspaceRead>;
  openFolder(workspaceId: ChatWorkspaceId): Promise<void>;
}

export interface ChatWorkspaceControllerSnapshot {
  workspaces: ChatWorkspaceRead[];
  loading: boolean;
  error: string | null;
}

const nativeApi: ChatWorkspaceApi = {
  list: chatApi.listChatWorkspaces,
  create: chatApi.createChatWorkspace,
  rename: chatApi.renameChatWorkspace,
  bind: chatApi.bindChatWorkspace,
  rebind: chatApi.rebindChatWorkspace,
  removeBinding: chatApi.removeChatWorkspaceBinding,
  archive: chatApi.archiveChatWorkspace,
  restore: chatApi.restoreChatWorkspace,
  openFolder: chatApi.openChatWorkspaceFolder,
};

export class ChatWorkspaceController {
  private workspaces: ChatWorkspaceRead[] = [];
  private loading = false;
  private error: string | null = null;
  private requestGeneration = 0;

  public constructor(private readonly api: ChatWorkspaceApi = nativeApi) {}

  public snapshot(): ChatWorkspaceControllerSnapshot {
    return {
      workspaces: [...this.workspaces],
      loading: this.loading,
      error: this.error,
    };
  }

  public async refresh(): Promise<void> {
    const generation = ++this.requestGeneration;
    this.loading = true;
    this.error = null;
    try {
      const workspaces = await this.api.list();
      if (generation !== this.requestGeneration) return;
      this.workspaces = workspaces;
    } catch (error: unknown) {
      if (generation !== this.requestGeneration) return;
      this.error = errorMessage(error);
      throw error;
    } finally {
      if (generation === this.requestGeneration) this.loading = false;
    }
  }

  public async create(request: CreateChatWorkspaceRequest): Promise<ChatWorkspaceRead> {
    return this.apply(await this.api.create(request));
  }

  public async rename(workspaceId: ChatWorkspaceId, displayName: string): Promise<ChatWorkspaceRead> {
    return this.apply(await this.api.rename(workspaceId, displayName));
  }

  public async bind(workspaceId: ChatWorkspaceId): Promise<ChatWorkspaceRead | null> {
    const workspace = await this.api.bind(workspaceId);
    return workspace === null ? null : this.apply(workspace);
  }

  public async rebind(workspaceId: ChatWorkspaceId): Promise<ChatWorkspaceRead | null> {
    const workspace = await this.api.rebind(workspaceId);
    return workspace === null ? null : this.apply(workspace);
  }

  public async removeBinding(workspaceId: ChatWorkspaceId): Promise<ChatWorkspaceRead> {
    return this.apply(await this.api.removeBinding(workspaceId));
  }

  public async archive(workspaceId: ChatWorkspaceId): Promise<ChatWorkspaceRead> {
    return this.apply(await this.api.archive(workspaceId));
  }

  public async restore(workspaceId: ChatWorkspaceId): Promise<ChatWorkspaceRead> {
    return this.apply(await this.api.restore(workspaceId));
  }

  public async openFolder(workspaceId: ChatWorkspaceId): Promise<void> {
    await this.api.openFolder(workspaceId);
  }

  private apply(workspace: ChatWorkspaceRead): ChatWorkspaceRead {
    const index = this.workspaces.findIndex((candidate) => candidate.workspace.id === workspace.workspace.id);
    this.workspaces = index === -1
      ? [...this.workspaces, workspace]
      : this.workspaces.map((candidate, candidateIndex) => candidateIndex === index ? workspace : candidate);
    this.error = null;
    return workspace;
  }
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "Chat workspaces could not be loaded";
}
