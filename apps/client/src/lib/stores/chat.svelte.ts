import * as chatApi from "$lib/api/chat";
import type {
  ChatBehaviorPreferences,
  ChatAttachmentRead,
  ChatDraftMention,
  ChatInteractionStateRead,
  ChatSettingsRead,
  ChatThreadId,
  ChatThreadShellRead,
  ChatTimelineItemRead,
  ChatTimelinePageRead,
  ChatWorkspaceId,
  ChatWorkspaceRead,
  CreateChatWorkspaceRequest,
  InteractionMode,
  ModelId,
  ProviderInstanceConfig,
  ProviderInstanceId,
  ProviderInstanceRead,
  ProviderProbeResult,
  ProviderSetupTestRead,
  RemoveProviderResult,
  SafetyMode,
  UserInputAnswer,
  VersionedJson,
} from "$lib/chat/contracts";
import { evictTimelinePages, mergeTimelineItems } from "$lib/chat/timeline-virtualization";
import { ChatComposerController, parseDraftMentions, type ChatComposerSnapshot } from "$lib/chat/composer-controller";
import { queuedFollowupDispatchReady, readComposerModelSelection } from "$lib/chat/composer-model";
import { AsyncFrameCoalescer } from "$lib/chat/frame-coalescer";

class ChatStore {
  private readonly composerController = new ChatComposerController();
  composer = $state<ChatComposerSnapshot>(this.composerController.snapshot());
  composerAttachments = $state<ChatAttachmentRead[]>([]);
  interaction = $state<ChatInteractionStateRead | null>(null);
  interactionLoading = $state(false);
  sendError = $state<string | null>(null);
  settings = $state<ChatSettingsRead | null>(null);
  workspaces = $state<ChatWorkspaceRead[]>([]);
  activeThreads = $state<ChatThreadShellRead[]>([]);
  archivedThreads = $state<ChatThreadShellRead[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  selectedWorkspaceId = $state<ChatWorkspaceId | null>(null);
  selectedThreadId = $state<ChatThreadId | null>(null);
  draftWorkspaceId = $state<ChatWorkspaceId | null>(null);
  timelinePages = $state<ChatTimelinePageRead[]>([]);
  timelineItems = $state<ChatTimelineItemRead[]>([]);
  timelineLoading = $state(false);
  timelineError = $state<string | null>(null);
  railOpen = $state(true);
  inspectorOpen = $state(false);
  private loadRequest = 0;
  private timelineRequest = 0;
  private attachmentRequest = 0;
  private interactionRequest = 0;
  private attachmentKey = "";
  private readonly queuedDispatches = new Set<string>();
  private readonly nativeChanges = new AsyncFrameCoalescer<string>(
    (threadId) => this.refreshNativeChange(threadId),
  );
  private loaded = false;

  constructor() {
    this.composerController.subscribe((snapshot) => {
      this.composer = snapshot;
      const key = `${snapshot.workspaceId ?? ""}:${snapshot.attachmentIds.join("\0")}`;
      if (key !== this.attachmentKey) {
        this.attachmentKey = key;
        void this.loadComposerAttachments(snapshot).catch(() => undefined);
      }
    });
  }

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
      const workspaceId = this.selectedWorkspaceId;
      if (workspaceId) await this.composerController.bind(workspaceId, this.selectedThreadId);
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
    else void this.composerController.bind(workspaceId, this.selectedThreadId).catch(() => undefined);
  }

  selectThread(threadId: ChatThreadId | null): void {
    if (threadId !== this.selectedThreadId) {
      this.timelinePages = [];
      this.timelineItems = [];
      this.timelineError = null;
    }
    this.selectedThreadId = threadId;
    const thread = this.selectedThread;
    if (thread) this.selectedWorkspaceId = thread.workspaceId;
    if (this.selectedWorkspaceId) {
      void this.composerController.bind(this.selectedWorkspaceId, threadId).catch(() => undefined);
    }
    void chatApi.setLastSelectedChatThread(threadId).catch((error) => {
      console.error("Failed to persist selected Chat thread", error);
    });
    if (threadId) void this.loadTimeline(threadId).catch(() => undefined);
    if (threadId) void this.refreshInteraction(threadId).catch(() => undefined);
    else this.interaction = null;
  }

  async loadOlderTimeline(selectedSequence: number | null = null): Promise<void> {
    const threadId = this.selectedThreadId;
    const cursor = this.timelinePages[0]?.previousCursor;
    if (!threadId || !cursor || this.timelineLoading) return;
    await this.loadTimeline(threadId, cursor, true, selectedSequence);
  }

  newDraft(workspaceId: ChatWorkspaceId): void {
    this.selectWorkspace(workspaceId);
    this.draftWorkspaceId = workspaceId;
  }

  discardDraft(): void {
    this.draftWorkspaceId = null;
  }

  setComposerText(text: string): void { this.composerController.setText(text); }
  setComposerAttachments(attachmentIds: string[]): void { this.composerController.setAttachments(attachmentIds); }
  setComposerMentions(mentions: ChatDraftMention[]): void { this.composerController.setMentions(mentions); }
  setComposerProvider(instanceId: ProviderInstanceId | null): void { this.composerController.setProvider(instanceId); }
  setComposerModel(selection: VersionedJson | null): void { this.composerController.setModelSelection(selection); }
  setComposerModes(safety: SafetyMode | null, interaction: InteractionMode | null): void { this.composerController.setModes(safety, interaction); }
  markComposerSent(): void { this.composerController.markSent(); }
  flushComposer(): Promise<void> { return this.composerController.flush(); }

  async refreshInteraction(threadId = this.selectedThreadId): Promise<void> {
    const request = ++this.interactionRequest;
    if (!threadId) { this.interaction = null; return; }
    this.interactionLoading = true;
    try {
      const interaction = await chatApi.readChatInteractionState(threadId);
      if (request === this.interactionRequest && threadId === this.selectedThreadId) {
        this.interaction = interaction;
        const latestTurnState = this.selectedThread?.latestTurnState ?? null;
        if (queuedFollowupDispatchReady(interaction.sessionState, latestTurnState) && interaction.queuedFollowup) {
          void this.dispatchQueuedFollowup(interaction.queuedFollowup).catch((error: unknown) => {
            this.sendError = errorMessage(error);
          });
        }
      }
    } finally {
      if (request === this.interactionRequest) this.interactionLoading = false;
    }
  }

  async sendComposer(): Promise<void> {
    const workspaceId = this.composer.workspaceId;
    const providerInstanceId = this.composer.providerInstanceId;
    const safetyMode = this.composer.safetyMode;
    const interactionMode = this.composer.interactionMode;
    const model = readComposerModelSelection(this.composer.modelSelection);
    if (!workspaceId || !providerInstanceId || !safetyMode || !interactionMode) {
      throw new Error("Complete every Chat composer selection before sending");
    }
    const prompt = this.composer.text;
    const attachmentIds = [...this.composer.attachmentIds];
    const mentions = this.composer.mentions.map((mention) => ({ relativePath: mention.relativePath, kind: mention.kind }));
    await chatApi.validateChatWorkspaceMentions(workspaceId, mentions.map((mention) => mention.relativePath));
    await this.composerController.flush();
    this.sendError = null;
    const current = this.selectedThread;
    const result = await chatApi.sendChatTurn({
      command: { clientCommandId: crypto.randomUUID(), expectedThreadRevision: current?.revision ?? null },
      workspaceId,
      threadId: current?.id ?? null,
      newThreadId: current ? null : crypto.randomUUID(),
      turnId: crypto.randomUUID(),
      messageId: crypto.randomUUID(),
      providerInstanceId,
      providerManagedModel: model.providerManaged,
      modelId: model.modelId,
      modelOptions: model.options,
      modes: { safetyMode, interactionMode },
      prompt,
      attachmentIds,
      mentions,
    });
    this.composerController.markSent();
    await this.composerController.flush();
    this.upsertThread(result.thread);
    await chatApi.rememberChatComposerSelection({
      workspaceId,
      providerInstanceId,
      modelId: model.modelId,
      providerManagedModel: model.providerManaged,
      modelOptions: model.options,
      safetyMode,
      interactionMode,
    });
    this.settings = await chatApi.readChatSettings();
    this.selectThread(result.thread.id);
    this.sendError = result.launchError?.message ?? null;
    await this.loadTimeline(result.thread.id);
    await this.refreshInteraction(result.thread.id);
  }

  async retryFailedSend(): Promise<void> {
    if (!this.composerController.restoreSentSnapshot()) return;
    await this.sendComposer();
  }

  async editFailedSend(): Promise<void> {
    if (!this.composerController.restoreSentSnapshot()) return;
    await this.forkCurrentComposer(null);
    this.sendError = null;
  }

  async changeProviderAfterFailure(): Promise<void> {
    this.composerController.restoreSentSnapshot();
    await this.forkCurrentComposer(null);
    this.composerController.setProvider(null);
    this.composerController.setModelSelection(null);
    this.composerController.setModes(null, null);
    this.sendError = null;
  }

  async forkComposerWithProvider(providerInstanceId: ProviderInstanceId): Promise<void> {
    await this.forkCurrentComposer(providerInstanceId);
  }

  async steerComposer(): Promise<void> {
    const thread = this.selectedThread;
    const prompt = this.composer.text.trim();
    if (!thread || !prompt) return;
    await chatApi.steerChatTurn({
      command: { clientCommandId: crypto.randomUUID(), expectedThreadRevision: thread.revision },
      threadId: thread.id,
      messageId: crypto.randomUUID(),
      prompt,
    });
    this.composerController.markSent();
    await this.composerController.flush();
    await this.loadTimeline(thread.id);
  }

  async queueComposer(): Promise<void> {
    const thread = this.selectedThread;
    const prompt = this.composer.text;
    if (!thread || !prompt.trim() || !this.composer.providerInstanceId || !this.composer.modelSelection
      || !this.composer.safetyMode || !this.composer.interactionMode) return;
    await chatApi.saveChatQueuedFollowup({
      id: this.interaction?.queuedFollowup?.id ?? crypto.randomUUID(),
      threadId: thread.id,
      text: prompt,
      providerInstanceId: this.composer.providerInstanceId,
      modelSelection: this.composer.modelSelection,
      safetyMode: this.composer.safetyMode,
      interactionMode: this.composer.interactionMode,
      attachmentIds: [...this.composer.attachmentIds],
      mentions: {
        schemaVersion: 1,
        value: this.composer.mentions.map((mention) => ({
          relativePath: mention.relativePath,
          kind: mention.kind,
          ignored: mention.ignored,
        })),
      },
    });
    this.composerController.markSent();
    await this.composerController.flush();
    await this.refreshInteraction(thread.id);
  }

  async cancelQueuedFollowup(): Promise<void> {
    if (!this.selectedThreadId) return;
    await chatApi.cancelChatQueuedFollowup(this.selectedThreadId);
    await this.refreshInteraction(this.selectedThreadId);
  }

  async editQueuedFollowup(): Promise<void> {
    const queued = this.interaction?.queuedFollowup;
    if (!queued) return;
    this.composerController.setText(queued.text);
    this.composerController.setAttachments(queued.attachmentIds);
    this.composerController.setMentions(parseDraftMentions(queued.mentions));
    this.composerController.setProvider(queued.providerInstanceId);
    this.composerController.setModelSelection(queued.modelSelection);
    this.composerController.setModes(queued.safetyMode, queued.interactionMode);
    await this.composerController.flush();
    await this.cancelQueuedFollowup();
  }

  async stop(force = false): Promise<void> {
    if (!this.selectedThreadId) return;
    await chatApi.stopChatSession(this.selectedThreadId, force);
    await this.refreshInteraction(this.selectedThreadId);
  }

  async resolveApproval(decision: import("$lib/chat/contracts").ApprovalDecision): Promise<void> {
    const pending = this.interaction?.pendingRequest;
    const thread = this.selectedThread;
    if (!pending || !thread) return;
    await chatApi.resolveChatApproval({
      command: { clientCommandId: crypto.randomUUID(), expectedThreadRevision: thread.revision },
      threadId: thread.id,
      requestId: pending.id,
      providerRequestId: pending.providerRequestId,
      decision,
    });
    await this.refreshInteraction(thread.id);
  }

  async resolveUserInput(answers: UserInputAnswer[]): Promise<void> {
    const pending = this.interaction?.pendingRequest;
    const thread = this.selectedThread;
    if (!pending || !thread) return;
    await chatApi.resolveChatUserInput({
      command: { clientCommandId: crypto.randomUUID(), expectedThreadRevision: thread.revision },
      threadId: thread.id,
      requestId: pending.id,
      providerRequestId: pending.providerRequestId,
      answers,
    });
    await this.refreshInteraction(thread.id);
  }

  async importComposerImages(files: File[]): Promise<void> {
    const workspaceId = this.composer.workspaceId;
    if (!workspaceId) throw new Error("Choose a Chat workspace before attaching images");
    for (const file of files) {
      const attachment = await chatApi.importChatImage(
        workspaceId,
        crypto.randomUUID(),
        file.name,
        [...new Uint8Array(await file.arrayBuffer())],
      );
      this.composerAttachments = [...this.composerAttachments, attachment];
      this.composerController.setAttachments(this.composerAttachments.map((entry) => entry.id));
    }
  }

  async pickComposerImages(title: string): Promise<void> {
    const workspaceId = this.composer.workspaceId;
    if (!workspaceId) throw new Error("Choose a Chat workspace before attaching images");
    const available = Math.max(0, 8 - this.composerAttachments.length);
    if (available === 0) throw new Error("Attach up to eight Chat images");
    const imported = await chatApi.pickChatImages(
      workspaceId,
      Array.from({ length: available }, () => crypto.randomUUID()),
      title,
    );
    this.composerAttachments = [...this.composerAttachments, ...imported];
    this.composerController.setAttachments(this.composerAttachments.map((attachment) => attachment.id));
  }

  removeComposerAttachment(attachmentId: string): void {
    this.composerAttachments = this.composerAttachments.filter((attachment) => attachment.id !== attachmentId);
    this.composerController.setAttachments(this.composerAttachments.map((attachment) => attachment.id));
  }

  async handleNativeChange(threadId: string): Promise<void> {
    if (threadId !== this.selectedThreadId) return;
    await this.nativeChanges.push(threadId);
  }

  private async refreshNativeChange(threadId: string): Promise<void> {
    if (threadId !== this.selectedThreadId) return;
    await this.loadTimeline(threadId);
    if (threadId !== this.selectedThreadId) return;
    const active = await chatApi.listChatThreads(null, false);
    this.activeThreads = active;
    await this.refreshInteraction(threadId);
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
      void this.loadTimeline(remembered).catch(() => undefined);
      return;
    }
    if (this.selectedWorkspaceId && this.workspaces.some((entry) => entry.workspace.id === this.selectedWorkspaceId)) return;
    this.selectedThreadId = null;
    this.selectedWorkspaceId = null;
  }

  private async loadComposerAttachments(snapshot: ChatComposerSnapshot): Promise<void> {
    const request = ++this.attachmentRequest;
    if (!snapshot.workspaceId || snapshot.attachmentIds.length === 0) {
      this.composerAttachments = [];
      return;
    }
    try {
      const attachments = await chatApi.readChatAttachments(snapshot.workspaceId, snapshot.attachmentIds);
      if (request === this.attachmentRequest) this.composerAttachments = attachments;
    } catch (error: unknown) {
      if (request === this.attachmentRequest) {
        this.composerAttachments = [];
        this.sendError = errorMessage(error);
      }
    }
  }

  private async dispatchQueuedFollowup(queued: import("$lib/chat/contracts").ChatQueuedFollowupRead): Promise<void> {
    if (this.queuedDispatches.has(queued.id)) return;
    const thread = this.selectedThread;
    const workspaceId = this.selectedWorkspaceId;
    const providerInstanceId = queued.providerInstanceId;
    const safetyMode = queued.safetyMode;
    const interactionMode = queued.interactionMode;
    const model = readComposerModelSelection(queued.modelSelection);
    if (!thread || !workspaceId || !providerInstanceId || !safetyMode || !interactionMode) return;
    this.queuedDispatches.add(queued.id);
    try {
      const mentions = parseDraftMentions(queued.mentions);
      const result = await chatApi.sendChatTurn({
        command: { clientCommandId: `queue-dispatch:${queued.id}`, expectedThreadRevision: null },
        workspaceId,
        threadId: thread.id,
        newThreadId: null,
        turnId: `queue-turn:${queued.id}`,
        messageId: `queue-message:${queued.id}`,
        providerInstanceId,
        providerManagedModel: model.providerManaged,
        modelId: model.modelId,
        modelOptions: model.options,
        modes: { safetyMode, interactionMode },
        prompt: queued.text,
        attachmentIds: [...queued.attachmentIds],
        mentions: mentions.map((mention) => ({ relativePath: mention.relativePath, kind: mention.kind })),
      });
      await chatApi.markChatQueuedFollowupDispatched(thread.id, queued.id);
      this.upsertThread(result.thread);
      this.sendError = result.launchError?.message ?? null;
      await this.loadTimeline(thread.id);
      await this.refreshInteraction(thread.id);
    } finally {
      this.queuedDispatches.delete(queued.id);
    }
  }

  private async forkCurrentComposer(providerInstanceId: ProviderInstanceId | null): Promise<void> {
    const snapshot = this.composerController.snapshot();
    if (!snapshot.workspaceId) return;
    await this.composerController.flush();
    this.selectedThreadId = null;
    this.draftWorkspaceId = snapshot.workspaceId;
    this.timelinePages = [];
    this.timelineItems = [];
    this.interaction = null;
    await this.composerController.bind(snapshot.workspaceId, null);
    this.composerController.setText(snapshot.text);
    this.composerController.setAttachments(snapshot.attachmentIds);
    this.composerController.setMentions(snapshot.mentions);
    this.composerController.setProvider(providerInstanceId ?? snapshot.providerInstanceId);
    this.composerController.setModelSelection(providerInstanceId === null ? snapshot.modelSelection : null);
    this.composerController.setModes(
      providerInstanceId === null ? snapshot.safetyMode : null,
      providerInstanceId === null ? snapshot.interactionMode : null,
    );
    await this.composerController.flush();
    await chatApi.setLastSelectedChatThread(null);
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

  private async loadTimeline(
    threadId: ChatThreadId,
    cursor: string | null = null,
    prepend = false,
    selectedSequence: number | null = null,
  ): Promise<void> {
    const request = ++this.timelineRequest;
    this.timelineLoading = true;
    this.timelineError = null;
    try {
      const page = await chatApi.readChatTimelinePage(threadId, cursor);
      if (request !== this.timelineRequest || this.selectedThreadId !== threadId) return;
      const pages = prepend
        ? evictTimelinePages([page, ...this.timelinePages], selectedSequence, 8)
        : [page];
      this.timelinePages = pages;
      this.timelineItems = mergeTimelineItems([], pages.flatMap((entry) => entry.items));
    } catch (error: unknown) {
      if (request !== this.timelineRequest) return;
      this.timelineError = errorMessage(error);
      throw error;
    } finally {
      if (request === this.timelineRequest) this.timelineLoading = false;
    }
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
