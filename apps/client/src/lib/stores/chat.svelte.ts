import * as chatApi from "$lib/api/chat";
import type {
  ChatBehaviorPreferences,
  ChatSettingsRead,
  ChatThreadId,
  ChatThreadShellRead,
  ChatWorkspaceId,
  ChatWorkspaceRead,
  CreateChatWorkspaceRequest,
  ModelId,
  ProviderInstanceConfig,
  ProviderInstanceId,
  ProviderInstanceRead,
  ProviderProbeResult,
  ProviderSetupTestRead,
  RemoveProviderResult,
} from "$lib/chat/contracts";

class ChatStore {
  settings = $state<ChatSettingsRead | null>(null);
  workspaces = $state<ChatWorkspaceRead[]>([]);
  activeThreads = $state<ChatThreadShellRead[]>([]);
  archivedThreads = $state<ChatThreadShellRead[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  selectedWorkspaceId = $state<ChatWorkspaceId | null>(null);
  selectedThreadId = $state<ChatThreadId | null>(null);
  draftWorkspaceId = $state<ChatWorkspaceId | null>(null);
  railOpen = $state(true);
  inspectorOpen = $state(false);
  private loadRequest = 0;
  private loaded = false;

  get selectedWorkspace(): ChatWorkspaceRead | null {
    return this.workspaces.find((entry) => entry.workspace.id === this.selectedWorkspaceId) ?? null;
  }

  get selectedThread(): ChatThreadShellRead | null {
    return [...this.activeThreads, ...this.archivedThreads]
      .find((entry) => entry.id === this.selectedThreadId) ?? null;
  }

  async ensureLoaded(): Promise<void> {
    if (this.loaded) return;
    await this.reload();
  }

  async reload(): Promise<void> {
    const request = ++this.loadRequest;
    this.loading = true;
    this.error = null;
    try {
      const [settings, workspaces, activeThreads, archivedThreads] = await Promise.all([
        chatApi.readChatSettings(),
        chatApi.listChatWorkspaces(),
        chatApi.listChatThreads(null, false),
        chatApi.listChatThreads(null, true),
      ]);
      if (request !== this.loadRequest) return;
      this.settings = settings;
      this.workspaces = workspaces;
      this.activeThreads = activeThreads;
      this.archivedThreads = archivedThreads;
      this.restoreSelection(settings);
      this.loaded = true;
    } catch (error: unknown) {
      if (request !== this.loadRequest) return;
      this.error = errorMessage(error);
      throw error;
    } finally {
      if (request === this.loadRequest) this.loading = false;
    }
  }

  async refreshSettings(): Promise<void> {
    this.settings = await chatApi.readChatSettings();
  }

  async saveProvider(configuration: ProviderInstanceConfig): Promise<ProviderInstanceRead> {
    const provider = await chatApi.saveChatProvider(configuration);
    await this.refreshSettings();
    return provider;
  }

  async testProvider(configuration: ProviderInstanceConfig): Promise<ProviderSetupTestRead> {
    return chatApi.testChatProvider(configuration);
  }

  async probeProvider(instanceId: ProviderInstanceId): Promise<ProviderProbeResult> {
    const result = await chatApi.probeChatProvider(instanceId);
    await this.refreshSettings();
    return result;
  }

  async setProviderEnabled(instanceId: ProviderInstanceId, enabled: boolean): Promise<void> {
    await chatApi.setChatProviderEnabled(instanceId, enabled);
    await this.refreshSettings();
  }

  async removeProvider(instanceId: ProviderInstanceId): Promise<RemoveProviderResult> {
    const result = await chatApi.removeChatProvider(instanceId);
    await this.refreshSettings();
    return result;
  }

  async refreshModels(instanceId: ProviderInstanceId): Promise<void> {
    await chatApi.refreshChatProviderModels(instanceId);
    await this.refreshSettings();
  }

  async updateModels(instanceId: ProviderInstanceId, visible: ModelId[], favorites: ModelId[]): Promise<void> {
    await chatApi.updateChatProviderModels(instanceId, visible, favorites);
    await this.refreshSettings();
  }

  async updateBehavior(behavior: ChatBehaviorPreferences): Promise<void> {
    await chatApi.updateChatBehavior(behavior);
    await this.refreshSettings();
  }

  async createWorkspace(request: CreateChatWorkspaceRequest, pickerTitle: string, bind = true): Promise<ChatWorkspaceRead> {
    let workspace = await chatApi.createChatWorkspace(request);
    this.upsertWorkspace(workspace);
    if (bind) {
      workspace = await chatApi.bindChatWorkspace(workspace.workspace.id, pickerTitle) ?? workspace;
      this.upsertWorkspace(workspace);
    }
    this.selectWorkspace(workspace.workspace.id);
    return workspace;
  }

  async bindWorkspace(workspaceId: ChatWorkspaceId, pickerTitle: string): Promise<void> {
    const workspace = await chatApi.bindChatWorkspace(workspaceId, pickerTitle);
    if (workspace) this.upsertWorkspace(workspace);
  }

  async rebindWorkspace(workspaceId: ChatWorkspaceId, pickerTitle: string): Promise<void> {
    const workspace = await chatApi.rebindChatWorkspace(workspaceId, pickerTitle);
    if (workspace) this.upsertWorkspace(workspace);
  }

  async removeWorkspaceBinding(workspaceId: ChatWorkspaceId): Promise<void> {
    this.upsertWorkspace(await chatApi.removeChatWorkspaceBinding(workspaceId));
  }

  async openWorkspaceFolder(workspaceId: ChatWorkspaceId): Promise<void> {
    await chatApi.openChatWorkspaceFolder(workspaceId);
  }

  async renameWorkspace(workspaceId: ChatWorkspaceId, displayName: string): Promise<void> {
    const current = this.workspace(workspaceId);
    this.upsertWorkspace(await chatApi.renameChatWorkspace(workspaceId, displayName, current.workspace.revision));
  }

  async archiveWorkspace(workspaceId: ChatWorkspaceId): Promise<void> {
    const current = this.workspace(workspaceId);
    this.upsertWorkspace(await chatApi.archiveChatWorkspace(workspaceId, current.workspace.revision));
  }

  async restoreWorkspace(workspaceId: ChatWorkspaceId): Promise<void> {
    const current = this.workspace(workspaceId);
    this.upsertWorkspace(await chatApi.restoreChatWorkspace(workspaceId, current.workspace.revision));
  }

  async setWorkspaceProviderPreference(workspaceId: ChatWorkspaceId, instanceId: ProviderInstanceId | null): Promise<void> {
    await chatApi.setChatWorkspaceProviderPreference(workspaceId, instanceId);
    await this.refreshSettings();
  }

  selectWorkspace(workspaceId: ChatWorkspaceId): void {
    this.selectedWorkspaceId = workspaceId;
    if (this.selectedThread?.workspaceId !== workspaceId) this.selectThread(null);
  }

  selectThread(threadId: ChatThreadId | null): void {
    this.selectedThreadId = threadId;
    const thread = this.selectedThread;
    if (thread) this.selectedWorkspaceId = thread.workspaceId;
    void chatApi.setLastSelectedChatThread(threadId).catch((error) => {
      console.error("Failed to persist selected Chat thread", error);
    });
  }

  newDraft(workspaceId: ChatWorkspaceId): void {
    this.selectWorkspace(workspaceId);
    this.draftWorkspaceId = workspaceId;
  }

  discardDraft(): void {
    this.draftWorkspaceId = null;
  }

  async renameThread(thread: ChatThreadShellRead, title: string): Promise<void> {
    this.upsertThread(await chatApi.renameChatThread(thread.id, title, thread.revision));
  }

  async setThreadRead(thread: ChatThreadShellRead, read: boolean): Promise<void> {
    this.upsertThread(await chatApi.setChatThreadRead(thread.id, read, thread.revision));
  }

  async archiveThread(thread: ChatThreadShellRead): Promise<void> {
    const archived = await chatApi.archiveChatThread(thread.id, thread.revision);
    this.activeThreads = this.activeThreads.filter((entry) => entry.id !== thread.id);
    this.archivedThreads = [archived, ...this.archivedThreads.filter((entry) => entry.id !== thread.id)];
    if (this.selectedThreadId === thread.id) this.selectThread(archived.id);
  }

  async restoreThread(thread: ChatThreadShellRead): Promise<void> {
    const restored = await chatApi.restoreChatThread(thread.id, thread.revision);
    this.archivedThreads = this.archivedThreads.filter((entry) => entry.id !== thread.id);
    this.activeThreads = [restored, ...this.activeThreads.filter((entry) => entry.id !== thread.id)];
    this.selectThread(restored.id);
  }

  async deleteThread(thread: ChatThreadShellRead): Promise<void> {
    await chatApi.deleteChatThreadPermanently(thread.id, thread.revision, thread.title);
    this.activeThreads = this.activeThreads.filter((entry) => entry.id !== thread.id);
    this.archivedThreads = this.archivedThreads.filter((entry) => entry.id !== thread.id);
    if (this.selectedThreadId === thread.id) this.selectThread(null);
  }

  private restoreSelection(settings: ChatSettingsRead): void {
    const remembered = settings.configuration.behavior.restoreLastSelectedThread
      ? settings.lastSelectedThreadId
      : null;
    if (remembered && [...this.activeThreads, ...this.archivedThreads].some((thread) => thread.id === remembered)) {
      this.selectedThreadId = remembered;
      this.selectedWorkspaceId = this.selectedThread?.workspaceId ?? null;
      return;
    }
    if (this.selectedWorkspaceId && this.workspaces.some((entry) => entry.workspace.id === this.selectedWorkspaceId)) return;
    this.selectedThreadId = null;
    this.selectedWorkspaceId = null;
  }

  private workspace(workspaceId: ChatWorkspaceId): ChatWorkspaceRead {
    const workspace = this.workspaces.find((entry) => entry.workspace.id === workspaceId);
    if (!workspace) throw new Error("Chat workspace was not found");
    return workspace;
  }

  private upsertWorkspace(workspace: ChatWorkspaceRead): void {
    const exists = this.workspaces.some((entry) => entry.workspace.id === workspace.workspace.id);
    this.workspaces = exists
      ? this.workspaces.map((entry) => entry.workspace.id === workspace.workspace.id ? workspace : entry)
      : [...this.workspaces, workspace];
  }

  private upsertThread(thread: ChatThreadShellRead): void {
    const target = thread.archivedAt ? this.archivedThreads : this.activeThreads;
    const next = target.some((entry) => entry.id === thread.id)
      ? target.map((entry) => entry.id === thread.id ? thread : entry)
      : [thread, ...target];
    if (thread.archivedAt) this.archivedThreads = next;
    else this.activeThreads = next;
  }
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return typeof error === "string" ? error : "Chat could not be loaded";
}

let store: ChatStore | null = null;

export function getChat(): ChatStore {
  store ??= new ChatStore();
  return store;
}
