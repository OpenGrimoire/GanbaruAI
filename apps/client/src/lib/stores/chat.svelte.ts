import * as chatApi from "$lib/api/chat";
import * as workingFolderApi from "$lib/api/project-working-folders";
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
  ProjectWorkingFolderId,
  ProjectWorkingFolderRead,
  CreateProjectWorkingFolderRequest,
  InteractionMode,
  ModelId,
  ProviderInstanceConfig,
  ProviderInstanceId,
  ProviderInstanceRead,
  ProviderProbeResult,
  ProviderRefreshResult,
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
import type { TimelineMessageRow } from "$lib/chat/timeline-model";
import { getProjects } from "$lib/stores/projects.svelte";
import { preferredProjectWorkingFolder } from "$lib/chat/working-folder-selection";

const projects = getProjects();

export interface ChatComposerSendOptions {
  promptOverride?: string;
  omitComposerContext?: boolean;
}

class ChatStore {
  private readonly composerController = new ChatComposerController();
  composer = $state<ChatComposerSnapshot>(this.composerController.snapshot());
  composerAttachments = $state<ChatAttachmentRead[]>([]);
  interaction = $state<ChatInteractionStateRead | null>(null);
  interactionLoading = $state(false);
  sendError = $state<string | null>(null);
  settings = $state<ChatSettingsRead | null>(null);
  workingFolders = $state<ProjectWorkingFolderRead[]>([]);
  activeThreads = $state<ChatThreadShellRead[]>([]);
  archivedThreads = $state<ChatThreadShellRead[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  selectedWorkingFolderId = $state<ProjectWorkingFolderId | null>(null);
  selectedThreadId = $state<ChatThreadId | null>(null);
  draftWorkingFolderId = $state<ProjectWorkingFolderId | null>(null);
  draftThreadId = $state<ChatThreadId | null>(null);
  timelinePages = $state<ChatTimelinePageRead[]>([]);
  timelineItems = $state<ChatTimelineItemRead[]>([]);
  pendingUserMessage = $state<{ threadId: ChatThreadId; row: TimelineMessageRow } | null>(null);
  timelineLoading = $state(false);
  timelineError = $state<string | null>(null);
  railOpen = $state(true);
  inspectorOpen = $state(false);
  private loadRequest = 0;
  private timelineRequest = 0;
  private attachmentRequest = 0;
  private interactionRequest = 0;
  private attachmentKey = "";
  private failedSendOptions: ChatComposerSendOptions | null = null;
  private readonly queuedDispatches = new Set<string>();
  private readonly nativeChanges = new AsyncFrameCoalescer<string>(
    (threadId) => this.refreshNativeChange(threadId),
  );
  private loaded = false;

  constructor() {
    this.composerController.subscribe((snapshot) => {
      this.composer = snapshot;
      const key = `${snapshot.workingFolderId ?? ""}:${snapshot.attachmentIds.join("\0")}`;
      if (key !== this.attachmentKey) {
        this.attachmentKey = key;
        void this.loadComposerAttachments(snapshot).catch(() => undefined);
      }
    });
  }

  get selectedWorkingFolder(): ProjectWorkingFolderRead | null {
    return this.workingFolders.find((entry) => entry.workingFolder.id === this.selectedWorkingFolderId) ?? null;
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
      const [settings, workingFolders, activeThreads, archivedThreads] = await Promise.all([
        chatApi.readChatSettings(),
        workingFolderApi.listProjectWorkingFolders(),
        chatApi.listChatThreads(null, false),
        chatApi.listChatThreads(null, true),
      ]);
      if (request !== this.loadRequest) return;
      this.settings = settings;
      this.workingFolders = workingFolders;
      this.activeThreads = activeThreads;
      this.archivedThreads = archivedThreads;
      await projects.ensureLoaded();
      await this.restoreSelection(settings);
      const workingFolderId = this.selectedWorkingFolderId;
      if (workingFolderId) await this.composerController.bind(workingFolderId, this.selectedThreadId);
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

  async syncProjectSelection(projectId: string | null): Promise<void> {
    if (!projectId || !this.loaded) return;
    if (this.selectedThread?.projectId === projectId) return;
    const remembered = await workingFolderApi.lastProjectWorkingFolder(projectId);
    const selected = preferredProjectWorkingFolder(this.workingFolders, projectId, remembered);
    if (selected) this.selectWorkingFolder(selected.workingFolder.id);
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

  async refreshAllProviders(): Promise<ProviderRefreshResult> {
    const result = await chatApi.refreshAllChatProviders();
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

  async addExternalWorkingFolder(
    request: CreateProjectWorkingFolderRequest,
    pickerTitle: string,
  ): Promise<ProjectWorkingFolderRead | null> {
    const workingFolder = await workingFolderApi.addExternalProjectWorkingFolder(request, pickerTitle);
    if (!workingFolder) return null;
    this.upsertWorkingFolder(workingFolder);
    this.selectWorkingFolder(workingFolder.workingFolder.id);
    return workingFolder;
  }

  async locateWorkingFolder(workingFolderId: ProjectWorkingFolderId, pickerTitle: string): Promise<void> {
    const workingFolder = await workingFolderApi.locateProjectWorkingFolder(workingFolderId, pickerTitle);
    if (workingFolder) this.upsertWorkingFolder(workingFolder);
  }

  async rebindWorkingFolder(workingFolderId: ProjectWorkingFolderId, pickerTitle: string): Promise<void> {
    const workingFolder = await workingFolderApi.rebindProjectWorkingFolder(workingFolderId, pickerTitle);
    if (workingFolder) this.upsertWorkingFolder(workingFolder);
  }

  async unbindWorkingFolder(workingFolderId: ProjectWorkingFolderId): Promise<void> {
    this.upsertWorkingFolder(await workingFolderApi.unbindProjectWorkingFolder(workingFolderId));
  }

  async openWorkingFolder(workingFolderId: ProjectWorkingFolderId): Promise<void> {
    await workingFolderApi.openProjectWorkingFolder(workingFolderId);
  }

  async recreateManagedWorkingFolder(workingFolderId: ProjectWorkingFolderId): Promise<void> {
    this.upsertWorkingFolder(await workingFolderApi.recreateManagedProjectWorkingFolder(workingFolderId));
  }

  async renameWorkingFolder(workingFolderId: ProjectWorkingFolderId, displayName: string): Promise<void> {
    const current = this.workingFolder(workingFolderId);
    this.upsertWorkingFolder(await workingFolderApi.renameProjectWorkingFolder(workingFolderId, displayName, current.workingFolder.revision));
  }

  async archiveWorkingFolder(workingFolderId: ProjectWorkingFolderId): Promise<void> {
    const current = this.workingFolder(workingFolderId);
    this.upsertWorkingFolder(await workingFolderApi.archiveProjectWorkingFolder(workingFolderId, current.workingFolder.revision));
  }

  async restoreWorkingFolder(workingFolderId: ProjectWorkingFolderId): Promise<void> {
    const current = this.workingFolder(workingFolderId);
    this.upsertWorkingFolder(await workingFolderApi.restoreProjectWorkingFolder(workingFolderId, current.workingFolder.revision));
  }

  async removeWorkingFolder(workingFolderId: ProjectWorkingFolderId): Promise<void> {
    await workingFolderApi.removeProjectWorkingFolder(workingFolderId);
    this.workingFolders = this.workingFolders.filter(
      (entry) => entry.workingFolder.id !== workingFolderId,
    );
    if (this.selectedWorkingFolderId === workingFolderId) {
      this.selectedWorkingFolderId = null;
      await this.syncProjectSelection(projects.selectedProjectId);
    }
  }

  async setWorkingFolderProviderPreference(workingFolderId: ProjectWorkingFolderId, instanceId: ProviderInstanceId | null): Promise<void> {
    await chatApi.setChatWorkingFolderProviderPreference(workingFolderId, instanceId);
    await this.refreshSettings();
  }

  selectWorkingFolder(workingFolderId: ProjectWorkingFolderId): void {
    this.selectedWorkingFolderId = workingFolderId;
    const projectId = this.selectedWorkingFolder?.workingFolder.projectId;
    if (projectId) {
      void projects.selectProject(projectId);
      void workingFolderApi.rememberProjectWorkingFolder(projectId, workingFolderId);
    }
    this.selectThread(null);
    this.ensureDraftThread(workingFolderId);
  }

  selectThread(threadId: ChatThreadId | null): void {
    if (threadId !== this.selectedThreadId) {
      this.timelinePages = [];
      this.timelineItems = [];
      this.timelineError = null;
    }
    this.selectedThreadId = threadId;
    const thread = this.selectedThread;
    if (thread) {
      this.selectedWorkingFolderId = thread.workingFolderId;
      void projects.selectProject(thread.projectId);
      void workingFolderApi.rememberProjectWorkingFolder(thread.projectId, thread.workingFolderId);
    }
    if (this.selectedWorkingFolderId) {
      void this.composerController.bind(this.selectedWorkingFolderId, threadId).catch(() => undefined);
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

  newDraft(workingFolderId: ProjectWorkingFolderId): void {
    this.selectWorkingFolder(workingFolderId);
  }

  discardDraft(): void {
    this.draftWorkingFolderId = null;
    this.draftThreadId = null;
  }

  setComposerText(text: string): void { this.composerController.setText(text); }
  setComposerRichContent(text: string, richContent: VersionedJson): void {
    this.composerController.setRichContent(text, richContent);
  }
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

  async sendComposer(options: ChatComposerSendOptions = {}): Promise<void> {
    const workingFolderId = this.composer.workingFolderId;
    const providerInstanceId = this.composer.providerInstanceId;
    const safetyMode = this.composer.safetyMode;
    const interactionMode = this.composer.interactionMode;
    const model = readComposerModelSelection(this.composer.modelSelection);
    if (!workingFolderId || !providerInstanceId || !safetyMode || !interactionMode) {
      throw new Error("Complete every Chat composer selection before sending");
    }
    const prompt = options.promptOverride ?? this.composer.text;
    const attachmentIds = options.omitComposerContext ? [] : [...this.composer.attachmentIds];
    const mentions = options.omitComposerContext
      ? []
      : this.composer.mentions.map((mention) => ({ relativePath: mention.relativePath, kind: mention.kind }));
    this.sendError = null;
    this.failedSendOptions = null;
    const current = this.selectedThread;
    const newThreadId = current ? null : this.ensureDraftThread(workingFolderId);
    const threadId = current?.id ?? newThreadId;
    if (!threadId) throw new Error("A Chat thread ID is required before sending");
    const turnId = crypto.randomUUID();
    const messageId = crypto.randomUUID();
    this.pendingUserMessage = {
      threadId,
      row: {
        id: messageId,
        kind: "message",
        role: "user",
        turnId,
        sequence: Number.MAX_SAFE_INTEGER,
        createdAt: new Date().toISOString(),
        markdown: prompt,
        state: "pending",
        phase: null,
        userContext: {
          attachments: this.composerAttachments
            .filter((attachment) => attachmentIds.includes(attachment.id))
            .map((attachment) => ({
              id: attachment.id,
              displayName: attachment.originalDisplayName,
              kind: attachment.kind,
              byteSize: attachment.byteSize,
              status: "pending",
            })),
          mentions: mentions.map((mention) => ({ ...mention })),
          terminalContext: [],
          preCheckpointId: null,
        },
        metadata: null,
      },
    };
    this.composerController.markSent();
    let result: Awaited<ReturnType<typeof chatApi.sendChatTurn>>;
    try {
      if (mentions.length > 0) {
        await chatApi.validateChatWorkingFolderMentions(
          workingFolderId,
          mentions.map((mention) => mention.relativePath),
        );
      }
      await this.composerController.flush();
      result = await chatApi.sendChatTurn({
        command: { clientCommandId: crypto.randomUUID(), expectedThreadRevision: current?.revision ?? null },
        workingFolderId,
        threadId: current?.id ?? null,
        newThreadId,
        turnId,
        messageId,
        providerInstanceId,
        providerManagedModel: model.providerManaged,
        modelId: model.modelId,
        modelOptions: model.options,
        modes: { safetyMode, interactionMode },
        prompt,
        attachmentIds,
        mentions,
      });
    } catch (error: unknown) {
      this.pendingUserMessage = null;
      this.composerController.restoreSentSnapshot();
      await this.composerController.flush();
      throw error;
    }
    this.upsertThread(result.thread);
    if (!current) {
      this.draftWorkingFolderId = null;
      this.draftThreadId = null;
    }
    this.selectThread(result.thread.id);
    this.sendError = result.launchError?.message ?? null;
    this.failedSendOptions = result.launchError ? { ...options } : null;
    try {
      await this.loadTimeline(result.thread.id);
    } finally {
      this.pendingUserMessage = null;
    }
    await chatApi.rememberChatComposerSelection({
      workingFolderId,
      providerInstanceId,
      modelId: model.modelId,
      providerManagedModel: model.providerManaged,
      modelOptions: model.options,
      safetyMode,
      interactionMode,
    });
    this.settings = await chatApi.readChatSettings();
    await this.refreshInteraction(result.thread.id);
  }

  async retryFailedSend(): Promise<void> {
    const options = this.failedSendOptions ?? {};
    if (!this.composerController.restoreSentSnapshot()) return;
    await this.sendComposer(options);
  }

  async editFailedSend(): Promise<void> {
    if (!this.composerController.restoreSentSnapshot()) return;
    await this.forkCurrentComposer(null);
    this.sendError = null;
    this.failedSendOptions = null;
  }

  async changeProviderAfterFailure(): Promise<void> {
    this.composerController.restoreSentSnapshot();
    await this.forkCurrentComposer(null);
    this.composerController.setProvider(null);
    this.composerController.setModelSelection(null);
    this.composerController.setModes(null, null);
    this.sendError = null;
    this.failedSendOptions = null;
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
    const workingFolderId = this.composer.workingFolderId;
    if (!workingFolderId) throw new Error("Choose a project working folder before attaching images");
    for (const file of files) {
      const attachment = await chatApi.importChatImage(
        workingFolderId,
        crypto.randomUUID(),
        file.name,
        [...new Uint8Array(await file.arrayBuffer())],
      );
      this.composerAttachments = [...this.composerAttachments, attachment];
      this.composerController.setAttachments(this.composerAttachments.map((entry) => entry.id));
    }
  }

  async pickComposerImages(title: string): Promise<void> {
    const workingFolderId = this.composer.workingFolderId;
    if (!workingFolderId) throw new Error("Choose a project working folder before attaching images");
    const available = Math.max(0, 8 - this.composerAttachments.length);
    if (available === 0) throw new Error("Attach up to eight Chat images");
    const imported = await chatApi.pickChatImages(
      workingFolderId,
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

  private async restoreSelection(settings: ChatSettingsRead): Promise<void> {
    const remembered = settings.configuration.behavior.restoreLastSelectedThread
      ? settings.lastSelectedThreadId
      : null;
    if (remembered && [...this.activeThreads, ...this.archivedThreads].some((thread) => thread.id === remembered)) {
      this.selectedThreadId = remembered;
      this.selectedWorkingFolderId = this.selectedThread?.workingFolderId ?? null;
      const selectedThread = this.selectedThread;
      if (selectedThread) await projects.selectProject(selectedThread.projectId);
      void this.loadTimeline(remembered).catch(() => undefined);
      return;
    }
    if (this.selectedWorkingFolderId && this.workingFolders.some((entry) => (
      entry.workingFolder.id === this.selectedWorkingFolderId
        && entry.workingFolder.projectId === projects.selectedProjectId
    ))) return;
    this.selectedThreadId = null;
    const projectId = projects.selectedProjectId;
    const rememberedFolderId = projectId
      ? await workingFolderApi.lastProjectWorkingFolder(projectId)
      : null;
    const selected = projectId
      ? preferredProjectWorkingFolder(this.workingFolders, projectId, rememberedFolderId)
      : null;
    this.selectedWorkingFolderId = selected?.workingFolder.id ?? null;
  }

  private ensureDraftThread(workingFolderId: ProjectWorkingFolderId): ChatThreadId {
    if (this.draftWorkingFolderId !== workingFolderId || !this.draftThreadId) {
      this.draftWorkingFolderId = workingFolderId;
      this.draftThreadId = crypto.randomUUID();
    }
    return this.draftThreadId;
  }

  private async loadComposerAttachments(snapshot: ChatComposerSnapshot): Promise<void> {
    const request = ++this.attachmentRequest;
    if (!snapshot.workingFolderId || snapshot.attachmentIds.length === 0) {
      this.composerAttachments = [];
      return;
    }
    try {
      const attachments = await chatApi.readChatAttachments(snapshot.workingFolderId, snapshot.attachmentIds);
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
    const workingFolderId = this.selectedWorkingFolderId;
    const providerInstanceId = queued.providerInstanceId;
    const safetyMode = queued.safetyMode;
    const interactionMode = queued.interactionMode;
    const model = readComposerModelSelection(queued.modelSelection);
    if (!thread || !workingFolderId || !providerInstanceId || !safetyMode || !interactionMode) return;
    this.queuedDispatches.add(queued.id);
    try {
      const mentions = parseDraftMentions(queued.mentions);
      const result = await chatApi.sendChatTurn({
        command: { clientCommandId: `queue-dispatch:${queued.id}`, expectedThreadRevision: null },
        workingFolderId,
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
    if (!snapshot.workingFolderId) return;
    await this.composerController.flush();
    this.selectedThreadId = null;
    this.draftWorkingFolderId = snapshot.workingFolderId;
    this.draftThreadId = crypto.randomUUID();
    this.timelinePages = [];
    this.timelineItems = [];
    this.interaction = null;
    await this.composerController.bind(snapshot.workingFolderId, null);
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

  private workingFolder(workingFolderId: ProjectWorkingFolderId): ProjectWorkingFolderRead {
    const workingFolder = this.workingFolders.find((entry) => entry.workingFolder.id === workingFolderId);
    if (!workingFolder) throw new Error("Project working folder was not found");
    return workingFolder;
  }

  private upsertWorkingFolder(workingFolder: ProjectWorkingFolderRead): void {
    const exists = this.workingFolders.some((entry) => (
      entry.workingFolder.id === workingFolder.workingFolder.id
    ));
    this.workingFolders = exists
      ? this.workingFolders.map((entry) => (
        entry.workingFolder.id === workingFolder.workingFolder.id ? workingFolder : entry
      ))
      : [...this.workingFolders, workingFolder];
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
