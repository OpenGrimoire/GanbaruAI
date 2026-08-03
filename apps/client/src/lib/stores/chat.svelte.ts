import * as chatApi from "$lib/api/chat";
import * as workingFolderApi from "$lib/api/project-working-folders";
import type {
  ChatBehaviorPreferences,
  ChatChannelId,
  ChatChannelRead,
  ChatChannelPageRead,
  ChatProjectPrimaryWorkingFolderRead,
  ChatReplyThreadId,
  ChatReplyThreadPageRead,
  ChatMessageRead,
  ChatMessageSearchResultRead,
  ChatParticipantMentionInput,
  ChatResourceReferenceInput,
  ChatAiTeammateRead,
  ChatWorkAssignmentId,
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
import { ChatConversationPageCache } from "$lib/chat/conversation-page-cache";
import { ChatComposerController, parseDraftMentions, type ChatComposerSeed, type ChatComposerSnapshot } from "$lib/chat/composer-controller";
import {
  composerModelSelection,
  queuedFollowupDispatchReady,
  readComposerModelSelection,
  resolveDefaultProviderModel,
  providerAvailable,
} from "$lib/chat/composer-model";
import { chatErrorMessage } from "$lib/chat/error-presentation";
import { AsyncFrameCoalescer } from "$lib/chat/frame-coalescer";
import type { TimelineMessageRow } from "$lib/chat/timeline-model";
import { getProjects } from "$lib/stores/projects.svelte";
import { preferredProjectWorkingFolder } from "$lib/chat/working-folder-selection";
import { readLastChatChannelId, saveLastChatChannelId } from "$lib/chat/channel-sections";

const projects = getProjects();
const CHAT_RECENT_THREAD_WINDOW = 200;
const CHAT_TIMELINE_CACHE_MAX_ENTRIES = 6;
const CHAT_TIMELINE_CACHE_MAX_BYTES = 8 * 1024 * 1024;
const CHAT_TIMELINE_MAX_PAGES_PER_CHANNEL = 8;
type ChatTimelineStorePage = ChatTimelinePageRead;

export interface ChatOrganizationalDraft {
  normalizedMarkdown: string;
  richContent: VersionedJson;
  attachmentIds: string[];
  participantMentions: ChatParticipantMentionInput[];
  resourceReferences: ChatResourceReferenceInput[];
  selectionStart: number;
  selectionEnd: number;
}

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
  providerDiscoveryLoading = $state(false);
  sendError = $state<string | null>(null);
  settings = $state<ChatSettingsRead | null>(null);
  workingFolders = $state<ProjectWorkingFolderRead[]>([]);
  activeChannels = $state<ChatChannelRead[]>([]);
  archivedChannels = $state<ChatChannelRead[]>([]);
  archivedChannelsLoading = $state(false);
  archivedChannelsError = $state<string | null>(null);
  channelsLoading = $state(true);
  selectedChannelId = $state<ChatChannelId | null>(null);
  teammates = $state<ChatAiTeammateRead[]>([]);
  channelPages = $state<ChatChannelPageRead[]>([]);
  channelMessages = $state<ChatMessageRead[]>([]);
  channelMessagesLoading = $state(false);
  channelMessagesError = $state<string | null>(null);
  openReplyThreadId = $state<ChatReplyThreadId | null>(null);
  replyThreadPages = $state<ChatReplyThreadPageRead[]>([]);
  replyThread = $state<ChatReplyThreadPageRead | null>(null);
  replyThreadLoading = $state(false);
  replyThreadError = $state<string | null>(null);
  organizationalDrafts = $state<Record<string, ChatOrganizationalDraft>>({});
  organizationalScrollPositions = $state<Record<string, number>>({});
  selectedExecutionRunId = $state<string | null>(null);
  messageAnchorId = $state<string | null>(null);
  primaryWorkingFolder = $state<ChatProjectPrimaryWorkingFolderRead | null>(null);
  channelArchiveOpen = $state(false);
  activeThreads = $state<ChatThreadShellRead[]>([]);
  archivedThreads = $state<ChatThreadShellRead[]>([]);
  archivedThreadsLoading = $state(false);
  loading = $state(false);
  error = $state<string | null>(null);
  selectedWorkingFolderId = $state<ProjectWorkingFolderId | null>(null);
  selectedThreadId = $state<ChatThreadId | null>(null);
  draftWorkingFolderId = $state<ProjectWorkingFolderId | null>(null);
  draftThreadId = $state<ChatThreadId | null>(null);
  selectedExecutionEnvironmentId = $state<string | null>(null);
  timelinePages = $state<ChatTimelineStorePage[]>([]);
  timelineItems = $state<ChatTimelineItemRead[]>([]);
  pendingUserMessage = $state<{ threadId: ChatThreadId; row: TimelineMessageRow } | null>(null);
  timelineLoading = $state(false);
  timelineError = $state<string | null>(null);
  railOpen = $state(true);
  inspectorOpen = $state(false);
  private loadRequest = 0;
  private channelLoadRequest = 0;
  private projectSelectionProjectId: string | null = null;
  private projectSelectionPromise: Promise<void> | null = null;
  private archivedChannelsProjectId: string | null = null;
  private navigationChannels: ChatChannelRead[] = [];
  private channelSelectionRequest = 0;
  private loadPromise: Promise<void> | null = null;
  private providerDiscoveryPromise: Promise<void> | null = null;
  private archivedThreadsPromise: Promise<void> | null = null;
  private workingFolderRefreshPromise: Promise<void> | null = null;
  private archivedThreadsLoaded = false;
  private timelineRequest = 0;
  private channelMessagesRequest = 0;
  private replyThreadRequest = 0;
  private attachmentRequest = 0;
  private interactionRequest = 0;
  private attachmentKey = "";
  private failedSendOptions: ChatComposerSendOptions | null = null;
  private readonly channelPageCache = new ChatConversationPageCache<ChatChannelPageRead>(
    CHAT_TIMELINE_CACHE_MAX_ENTRIES,
    CHAT_TIMELINE_CACHE_MAX_BYTES,
    CHAT_TIMELINE_MAX_PAGES_PER_CHANNEL,
  );
  private readonly replyThreadPageCache = new ChatConversationPageCache<ChatReplyThreadPageRead>(
    CHAT_TIMELINE_CACHE_MAX_ENTRIES,
    CHAT_TIMELINE_CACHE_MAX_BYTES,
    CHAT_TIMELINE_MAX_PAGES_PER_CHANNEL,
  );
  private readonly queuedDispatches = new Set<string>();
  private readonly nativeChanges = new AsyncFrameCoalescer<string>(
    (threadId) => this.refreshNativeChange(threadId),
  );
  private loaded = false;
  private vaultGeneration = 0;

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

  get selectedChannel(): ChatChannelRead | null {
    return [...this.activeChannels, ...this.archivedChannels]
      .find((entry) => entry.id === this.selectedChannelId) ?? null;
  }

  get selectedThread(): ChatThreadShellRead | null {
    return [...this.activeThreads, ...this.archivedThreads]
      .find((entry) => entry.id === this.selectedThreadId) ?? null;
  }

  async ensureLoaded(): Promise<void> {
    if (this.loaded) return;
    this.loadPromise ??= this.reload().finally(() => {
      this.loadPromise = null;
    });
    await this.loadPromise;
  }

  /** Starts the bounded Chat working set before the route is opened. */
  async prewarmForProject(projectId: string | null): Promise<void> {
    await this.ensureLoaded();
    if (projectId && this.selectedChannel?.projectId !== projectId) {
      await this.syncProjectSelection(projectId);
    }
  }

  /** Invalidates vault-scoped Chat state before prewarming another vault. */
  resetForVault(): void {
    this.vaultGeneration += 1;
    this.loadRequest += 1;
    this.channelLoadRequest += 1;
    this.channelSelectionRequest += 1;
    this.timelineRequest += 1;
    this.loaded = false;
    this.loadPromise = null;
    this.navigationChannels = [];
    this.channelPageCache.clear();
    this.replyThreadPageCache.clear();
    this.projectSelectionProjectId = null;
    this.projectSelectionPromise = null;
    this.settings = null;
    this.providerDiscoveryPromise = null;
    this.providerDiscoveryLoading = false;
    this.workingFolders = [];
    this.activeChannels = [];
    this.archivedChannels = [];
    this.archivedChannelsProjectId = null;
    this.archivedChannelsLoading = false;
    this.archivedChannelsError = null;
    this.channelsLoading = true;
    this.selectedChannelId = null;
    this.teammates = [];
    this.channelPages = [];
    this.channelMessages = [];
    this.channelMessagesError = null;
    this.openReplyThreadId = null;
    this.replyThreadPages = [];
    this.replyThread = null;
    this.replyThreadError = null;
    this.organizationalDrafts = {};
    this.organizationalScrollPositions = {};
    this.selectedExecutionRunId = null;
    this.messageAnchorId = null;
    this.primaryWorkingFolder = null;
    this.activeThreads = [];
    this.archivedThreads = [];
    this.selectedWorkingFolderId = null;
    this.selectedThreadId = null;
    this.timelinePages = [];
    this.timelineItems = [];
    this.timelineError = null;
    this.error = null;
    this.loading = false;
    this.composerController.reset();
  }

  async reload(): Promise<void> {
    const request = ++this.loadRequest;
    const vaultGeneration = this.vaultGeneration;
    this.loading = true;
    this.error = null;
    try {
      const [settings, workingFolders, navigationChannels, teammates] = await Promise.all([
        chatApi.readChatSettings(),
        workingFolderApi.listCachedProjectWorkingFolders(),
        chatApi.listChatNavigationChannels(),
        chatApi.listChatTeammates(false),
      ]);
      if (request !== this.loadRequest || vaultGeneration !== this.vaultGeneration) return;
      this.settings = settings;
      this.workingFolders = workingFolders;
      this.teammates = teammates;
      this.activeThreads = [];
      this.navigationChannels = navigationChannels;
      this.archivedThreads = [];
      this.archivedThreadsLoaded = false;
      void this.discoverProviders().catch((error: unknown) => {
        console.error("Automatic Chat provider discovery failed", error);
      });
      void chatApi.recoverChatAssignmentDispatchJobs().catch((error: unknown) => {
        console.error("Chat assignment recovery failed", error);
      });
      await projects.ensureLoaded();
      if (request !== this.loadRequest || vaultGeneration !== this.vaultGeneration) return;
      const projectId = projects.selectedProjectId;
      if (projectId) {
        await this.loadProjectChannels(projectId);
        const rememberedChannelId = readLastChatChannelId(projectId);
        const initial = this.activeChannels.find((channel) => channel.id === rememberedChannelId)
          ?? this.activeChannels.find((channel) => channel.isDefault)
          ?? this.activeChannels[0]
          ?? null;
        if (initial) await this.selectChannel(initial.id);
        else await this.restoreSelection(settings);
      } else {
        this.channelsLoading = false;
        await this.restoreSelection(settings);
      }
      const workingFolderId = this.selectedWorkingFolderId;
      if (workingFolderId && !this.selectedChannel) await this.composerController.bind(
        workingFolderId,
        this.selectedThreadId,
        this.composerSeed(this.selectedThread),
      );
      if (request !== this.loadRequest || vaultGeneration !== this.vaultGeneration) return;
      this.loaded = true;
      void this.prefetchRememberedProjectChannels(projectId, vaultGeneration);
      void this.refreshWorkingFolders().catch((error: unknown) => {
        console.error("Chat working folder reconciliation failed", error);
      });
    } catch (error: unknown) {
      if (request !== this.loadRequest) return;
      this.error = chatErrorMessage(error, "Chat could not be loaded");
      throw error;
    } finally {
      if (request === this.loadRequest) this.loading = false;
    }
  }

  private async prefetchRememberedProjectChannels(
    selectedProjectId: string | null,
    vaultGeneration: number,
  ): Promise<void> {
    for (const project of projects.projects) {
      if (project.id === selectedProjectId || project.status !== "active") continue;
      const rememberedChannelId = readLastChatChannelId(project.id);
      const projectChannels = this.navigationChannels.filter((channel) => channel.projectId === project.id);
      const channel = projectChannels.find((entry) => entry.id === rememberedChannelId)
        ?? projectChannels.find((entry) => entry.isDefault)
        ?? projectChannels[0]
        ?? null;
      if (!channel || this.channelPageCache.get(channel.id)) continue;
      try {
        const page = await chatApi.readChatChannelPage(channel.id);
        if (vaultGeneration !== this.vaultGeneration) return;
        this.channelPageCache.set(channel.id, [page]);
      } catch (error: unknown) {
        console.error(`Chat prefetch failed for project ${project.id}`, error);
      }
    }
  }

  async refreshSettings(): Promise<void> {
    this.settings = await chatApi.readChatSettings();
  }

  async ensureArchivedThreads(): Promise<void> {
    if (this.archivedThreadsLoaded) return;
    this.archivedThreadsPromise ??= (async () => {
      this.archivedThreadsLoading = true;
      try {
        const threads = await chatApi.listChatThreadWindow(
          null,
          true,
          CHAT_RECENT_THREAD_WINDOW,
        );
        const existing = new Map(this.archivedThreads.map((thread) => [thread.id, thread]));
        for (const thread of threads) existing.set(thread.id, thread);
        this.archivedThreads = [...existing.values()].sort((left, right) => (
          right.lastActivityAt.localeCompare(left.lastActivityAt)
        ));
        this.archivedThreadsLoaded = true;
      } finally {
        this.archivedThreadsLoading = false;
        this.archivedThreadsPromise = null;
      }
    })();
    await this.archivedThreadsPromise;
  }

  private async discoverProviders(): Promise<void> {
    if (this.providerDiscoveryPromise) return this.providerDiscoveryPromise;
    const vaultGeneration = this.vaultGeneration;
    this.providerDiscoveryLoading = true;
    const discovery = chatApi.discoverDefaultChatProviders()
      .then((settings) => {
        if (vaultGeneration !== this.vaultGeneration) return;
        this.settings = settings;
        this.applyDiscoveredComposerDefaults(settings);
      })
      .finally(() => {
        if (this.providerDiscoveryPromise === discovery) {
          this.providerDiscoveryLoading = false;
          this.providerDiscoveryPromise = null;
        }
      });
    this.providerDiscoveryPromise = discovery;
    return discovery;
  }

  private applyDiscoveredComposerDefaults(settings: ChatSettingsRead): void {
    if (!this.composer.workingFolderId) return;
    const currentProviderId = this.composer.providerInstanceId;
    if (currentProviderId) {
      const currentProvider = settings.providerInstances.find((provider) => (
        provider.configuration.instanceId === currentProviderId
      ));
      if (!currentProvider || !providerAvailable(currentProvider)) return;
      const currentModel = readComposerModelSelection(this.composer.modelSelection);
      const catalog = currentProvider.modelCatalog?.models ?? [];
      const explicitModelValid = currentModel.modelId !== null && catalog.some((model) => (
        model.id === currentModel.modelId && model.availability !== "deprecated"
      ));
      const providerManagedValid = currentModel.providerManaged && catalog.length === 0;
      if (explicitModelValid || providerManagedValid) return;
    }
    const preferredProviderId = settings.configuration.workingFolderProviderPreferences[
      this.composer.workingFolderId
    ] ?? null;
    const resolved = resolveDefaultProviderModel(settings.providerInstances, preferredProviderId);
    if (!resolved) return;
    this.composerController.setProvider(resolved.provider.configuration.instanceId);
    this.composerController.setModelSelection(composerModelSelection(
      resolved.model?.id ?? null,
      resolved.providerManaged,
      resolved.options,
    ));
    this.composerController.setModes(
      this.composer.safetyMode ?? "ask_for_approval",
      this.composer.interactionMode ?? "build",
    );
    void this.composerController.flush().catch((error: unknown) => {
      console.error("Chat composer defaults could not be saved", error);
    });
  }

  private async refreshWorkingFolders(): Promise<void> {
    if (this.workingFolderRefreshPromise) return this.workingFolderRefreshPromise;
    this.workingFolderRefreshPromise = workingFolderApi.listProjectWorkingFolders()
      .then((workingFolders) => {
        for (const workingFolder of workingFolders) this.upsertWorkingFolder(workingFolder);
      })
      .finally(() => {
        this.workingFolderRefreshPromise = null;
      });
    return this.workingFolderRefreshPromise;
  }

  syncProjectSelection(projectId: string | null): Promise<void> {
    if (!projectId || !this.loaded) return Promise.resolve();
    if (this.selectedChannel?.projectId === projectId) return Promise.resolve();
    if (this.projectSelectionProjectId === projectId && this.projectSelectionPromise) {
      return this.projectSelectionPromise;
    }
    const selection = this.performProjectSelection(projectId);
    const tracked = selection.finally(() => {
      if (this.projectSelectionPromise !== tracked) return;
      this.projectSelectionProjectId = null;
      this.projectSelectionPromise = null;
    });
    this.projectSelectionProjectId = projectId;
    this.projectSelectionPromise = tracked;
    return tracked;
  }

  private async performProjectSelection(projectId: string): Promise<void> {
    if (!await this.loadProjectChannels(projectId)) return;
    if (projects.selectedProjectId !== projectId) return;
    const rememberedChannelId = readLastChatChannelId(projectId);
    const selected = this.activeChannels.find((channel) => channel.id === rememberedChannelId)
      ?? this.activeChannels.find((channel) => channel.isDefault)
      ?? this.activeChannels[0]
      ?? null;
    if (selected) await this.selectChannel(selected.id);
  }

  async loadProjectChannels(projectId: string): Promise<boolean> {
    const request = ++this.channelLoadRequest;
    this.channelsLoading = true;
    this.activeChannels = [];
    this.archivedChannels = [];
    this.archivedChannelsProjectId = null;
    this.archivedChannelsError = null;
    try {
      const cached = this.navigationChannels.filter((channel) => channel.projectId === projectId);
      const active = cached.length > 0
        ? cached
        : await chatApi.listChatChannels(projectId, false);
      if (request !== this.channelLoadRequest) return false;
      this.activeChannels = active;
      if (cached.length === 0) this.mergeNavigationChannels(active);
      return true;
    } finally {
      if (request === this.channelLoadRequest) {
        this.channelsLoading = false;
      }
    }
  }

  private async loadArchivedChannels(projectId: string): Promise<void> {
    if (this.archivedChannelsProjectId === projectId) return;
    this.archivedChannelsLoading = true;
    this.archivedChannelsError = null;
    try {
      const archived = await chatApi.listChatChannels(projectId, true);
      if (projects.selectedProjectId !== projectId) return;
      this.archivedChannels = archived;
      this.archivedChannelsProjectId = projectId;
    } catch (error: unknown) {
      if (projects.selectedProjectId === projectId) {
        this.archivedChannelsError = chatErrorMessage(
          error,
          "Archived Chat channels could not be loaded",
        );
      }
      throw error;
    } finally {
      if (projects.selectedProjectId === projectId) this.archivedChannelsLoading = false;
    }
  }

  private mergeNavigationChannels(channels: readonly ChatChannelRead[]): void {
    const byId = new Map(this.navigationChannels.map((channel) => [channel.id, channel]));
    for (const channel of channels) {
      if (channel.archivedAt) byId.delete(channel.id);
      else byId.set(channel.id, channel);
    }
    this.navigationChannels = [...byId.values()];
  }

  async selectChannel(channelId: ChatChannelId): Promise<void> {
    const request = ++this.channelSelectionRequest;
    const channel = [...this.activeChannels, ...this.archivedChannels]
      .find((entry) => entry.id === channelId)
      ?? await chatApi.readChatChannel(channelId);
    if (request !== this.channelSelectionRequest) return;
    this.upsertChannel(channel);
    const changedChannel = this.selectedChannelId !== channelId;
    this.selectedChannelId = channelId;
    saveLastChatChannelId(channel.projectId, channelId);
    this.channelArchiveOpen = false;
    await projects.selectProject(channel.projectId);
    if (request !== this.channelSelectionRequest) return;
    if (changedChannel) this.closeReplyThread();
    try {
      this.primaryWorkingFolder = await chatApi.readChatProjectPrimaryWorkingFolder(channel.projectId);
    } catch {
      this.primaryWorkingFolder = null;
    }
    const rememberedFolderId = await workingFolderApi.lastProjectWorkingFolder(channel.projectId);
    const fallbackFolder = preferredProjectWorkingFolder(this.workingFolders, channel.projectId, rememberedFolderId);
    this.selectedWorkingFolderId = this.primaryWorkingFolder?.workingFolderId
      ?? fallbackFolder?.workingFolder.id
      ?? null;
    this.selectedThreadId = null;
    this.selectedExecutionRunId = null;
    this.selectedExecutionEnvironmentId = null;
    this.draftWorkingFolderId = null;
    this.draftThreadId = null;
    await this.loadChannelMessages(channel.id);
    if (request !== this.channelSelectionRequest) return;
    this.interaction = null;
    if (channel.unreadCount > 0) {
      void chatApi.setChatChannelRead(channel.id, true)
        .then((updated) => this.upsertChannel(updated))
        .catch(() => undefined);
    }
  }

  async createChannel(request: import("$lib/chat/contracts").CreateChatChannelRequest): Promise<ChatChannelRead> {
    const channel = await chatApi.createChatChannel(request);
    this.activeChannels = [...this.activeChannels, channel].sort(channelSort);
    await this.selectChannel(channel.id);
    return channel;
  }

  async updateChannelDetails(channel: ChatChannelRead, name: string, topic: string): Promise<ChatChannelRead> {
    const updated = await chatApi.updateChatChannelDetails({
      channelId: channel.id,
      name,
      topic,
      expectedRevision: channel.revision,
    });
    this.upsertChannel(updated);
    return updated;
  }

  async refreshTeammates(): Promise<void> {
    this.teammates = await chatApi.listChatTeammates(false);
    if (this.selectedChannelId) {
      this.upsertChannel(await chatApi.readChatChannel(this.selectedChannelId));
    }
  }

  organizationalDraft(destination: string): ChatOrganizationalDraft {
    return this.organizationalDrafts[destination] ?? emptyOrganizationalDraft();
  }

  setOrganizationalDraft(destination: string, draft: ChatOrganizationalDraft): void {
    this.organizationalDrafts = { ...this.organizationalDrafts, [destination]: cloneOrganizationalDraft(draft) };
  }

  setOrganizationalScrollPosition(destination: string, scrollTop: number): void {
    this.organizationalScrollPositions = { ...this.organizationalScrollPositions, [destination]: scrollTop };
  }

  async postOrganizationalMessage(
    destination: string,
    options: { alsoSendToChannel?: boolean } = {},
  ): Promise<import("$lib/chat/contracts").PostChatMessageResult> {
    const channel = this.selectedChannel;
    if (!channel) throw new Error("Choose a channel before posting");
    const draft = this.organizationalDraft(destination);
    if (!draft.normalizedMarkdown.trim() && draft.attachmentIds.length === 0 && draft.resourceReferences.length === 0) {
      throw new Error("Write a message or attach context before posting");
    }
    const replyThreadId = destination.startsWith("reply-thread:")
      ? destination.slice("reply-thread:".length)
      : null;
    const result = await chatApi.postChatMessage({
      clientCommandId: crypto.randomUUID(),
      channelId: channel.id,
      replyThreadId,
      normalizedMarkdown: draft.normalizedMarkdown,
      richContent: draft.richContent,
      attachmentIds: [...draft.attachmentIds],
      participantMentions: draft.participantMentions.map((mention) => ({ ...mention })),
      resourceReferences: draft.resourceReferences.map((reference) => ({ ...reference })),
      alsoSendToChannel: options.alsoSendToChannel ?? false,
    });
    this.setOrganizationalDraft(destination, emptyOrganizationalDraft());
    await this.loadChannelMessages(channel.id, true);
    if (replyThreadId || result.assignment) await this.openReplyThread(result.replyThreadId);
    this.upsertChannel(await chatApi.readChatChannel(channel.id));
    return result;
  }

  async openReplyThread(replyThreadId: ChatReplyThreadId): Promise<void> {
    this.openReplyThreadId = replyThreadId;
    this.selectedExecutionRunId = null;
    await this.loadReplyThread(replyThreadId);
  }

  async searchOrganizationalMessages(query: string): Promise<ChatMessageSearchResultRead[]> {
    const normalized = query.trim();
    if (normalized.length < 2) return [];
    return chatApi.searchChatMessages(normalized);
  }

  async openMessageSearchResult(result: ChatMessageSearchResultRead): Promise<void> {
    this.messageAnchorId = null;
    await projects.selectProject(result.projectId);
    await this.loadProjectChannels(result.projectId);
    await this.selectChannel(result.channelId);
    const anchorCursor = String(result.ordinal + 1);
    if (result.replyThreadId) {
      this.openReplyThreadId = result.replyThreadId;
      this.selectedExecutionRunId = null;
      this.replyThreadLoading = true;
      this.replyThreadError = null;
      try {
        const page = await chatApi.readChatReplyThreadPage(result.replyThreadId, anchorCursor);
        if (this.openReplyThreadId !== result.replyThreadId) return;
        this.replyThreadPages = [page];
        this.replyThread = page;
      } catch (error: unknown) {
        this.replyThreadError = chatErrorMessage(error, "Search result thread could not be loaded");
        throw error;
      } finally {
        if (this.openReplyThreadId === result.replyThreadId) this.replyThreadLoading = false;
      }
    } else {
      this.channelMessagesLoading = true;
      this.channelMessagesError = null;
      try {
        const page = await chatApi.readChatChannelPage(result.channelId, anchorCursor);
        if (this.selectedChannelId !== result.channelId) return;
        this.channelPages = [page];
        this.channelMessages = page.messages;
      } catch (error: unknown) {
        this.channelMessagesError = chatErrorMessage(error, "Search result could not be loaded");
        throw error;
      } finally {
        if (this.selectedChannelId === result.channelId) this.channelMessagesLoading = false;
      }
    }
    this.messageAnchorId = result.messageItemId;
  }

  clearMessageAnchor(messageItemId: string): void {
    if (this.messageAnchorId === messageItemId) this.messageAnchorId = null;
  }

  closeReplyThread(): void {
    this.replyThreadRequest += 1;
    this.openReplyThreadId = null;
    this.replyThreadPages = [];
    this.replyThread = null;
    this.replyThreadError = null;
    this.selectedExecutionRunId = null;
  }

  async selectAssignmentExecution(runId: string): Promise<void> {
    const replyThreadId = this.openReplyThreadId;
    const run = this.replyThread?.agentRuns.find((entry) => entry.id === runId) ?? null;
    if (!run?.providerExecutionThreadId) throw new Error("This execution has not started yet");
    let thread = [...this.activeThreads, ...this.archivedThreads]
      .find((entry) => entry.id === run.providerExecutionThreadId) ?? null;
    thread ??= await chatApi.readChatThreadShell(run.providerExecutionThreadId);
    if (this.openReplyThreadId !== replyThreadId
      || !this.replyThread?.agentRuns.some((entry) => entry.id === runId)) return;
    this.upsertThread(thread);
    this.selectedExecutionRunId = run.id;
    this.selectThread(thread.id);
  }

  async cancelAssignment(assignmentId: ChatWorkAssignmentId): Promise<void> {
    const expectedRevision = this.replyThread?.assignment?.id === assignmentId
      ? this.replyThread.assignment.revision
      : null;
    if (expectedRevision === null) throw new Error("Work assignment was not loaded");
    await chatApi.cancelChatAssignment(assignmentId, expectedRevision);
    if (this.openReplyThreadId) await this.loadReplyThread(this.openReplyThreadId, true);
    if (this.selectedChannelId) await this.loadChannelMessages(this.selectedChannelId, true);
  }

  async retryAssignment(assignmentId: ChatWorkAssignmentId): Promise<void> {
    const expectedRevision = this.replyThread?.assignment?.id === assignmentId
      ? this.replyThread.assignment.revision
      : null;
    if (expectedRevision === null) throw new Error("Work assignment was not loaded");
    await chatApi.retryChatAssignment(assignmentId, expectedRevision);
    if (this.openReplyThreadId) await this.loadReplyThread(this.openReplyThreadId, true);
  }

  async archiveChannel(channel: ChatChannelRead): Promise<void> {
    const archived = await chatApi.archiveChatChannel(channel.id, channel.revision);
    this.mergeNavigationChannels([archived]);
    this.activeChannels = this.activeChannels.filter((entry) => entry.id !== channel.id);
    this.archivedChannels = [archived, ...this.archivedChannels.filter((entry) => entry.id !== channel.id)];
    if (this.selectedChannelId === channel.id) {
      const fallback = this.activeChannels.find((entry) => entry.isDefault) ?? this.activeChannels[0];
      if (fallback) await this.selectChannel(fallback.id);
    }
  }

  async restoreChannel(channel: ChatChannelRead): Promise<void> {
    const restored = await chatApi.restoreChatChannel(channel.id, channel.revision);
    this.archivedChannels = this.archivedChannels.filter((entry) => entry.id !== channel.id);
    this.activeChannels = [...this.activeChannels, restored].sort(channelSort);
    this.mergeNavigationChannels([restored]);
  }

  openChannelArchive(): void {
    this.channelArchiveOpen = true;
    const projectId = projects.selectedProjectId;
    if (projectId) void this.loadArchivedChannels(projectId).catch(() => undefined);
  }

  closeChannelArchive(): void {
    this.channelArchiveOpen = false;
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

  async setProjectPrimaryWorkingFolder(workingFolderId: ProjectWorkingFolderId): Promise<void> {
    const projectId = this.selectedChannel?.projectId ?? projects.selectedProjectId;
    if (!projectId || !this.primaryWorkingFolder) throw new Error("Project primary working folder was not loaded");
    this.primaryWorkingFolder = await chatApi.setChatProjectPrimaryWorkingFolder(
      projectId,
      workingFolderId,
      this.primaryWorkingFolder.revision,
    );
    this.selectedWorkingFolderId = workingFolderId;
  }

  selectWorkingFolder(workingFolderId: ProjectWorkingFolderId): void {
    this.selectedWorkingFolderId = workingFolderId;
    this.selectedExecutionEnvironmentId = null;
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
    if (threadId) {
      void chatApi.readChatThreadExecutionEnvironment(threadId)
        .then((environmentId) => {
          if (this.selectedThreadId === threadId) this.selectedExecutionEnvironmentId = environmentId;
        })
        .catch(() => undefined);
    } else {
      this.selectedExecutionEnvironmentId = null;
    }
    const thread = this.selectedThread;
    if (thread) {
      this.selectedWorkingFolderId = thread.workingFolderId;
      void projects.selectProject(thread.projectId);
      void workingFolderApi.rememberProjectWorkingFolder(thread.projectId, thread.workingFolderId);
    }
    if (this.selectedWorkingFolderId) {
      void this.composerController.bind(
        this.selectedWorkingFolderId,
        threadId,
        this.composerSeed(thread),
      ).catch(() => undefined);
    }
    void chatApi.setLastSelectedChatThread(threadId).catch((error) => {
      console.error("Failed to persist selected Chat thread", error);
    });
    if (threadId) void this.loadTimeline(threadId).catch(() => undefined);
    if (threadId) void this.refreshInteraction(threadId).catch(() => undefined);
    else this.interaction = null;
  }

  selectThreadShell(thread: ChatThreadShellRead): void {
    this.upsertThread(thread);
    this.selectThread(thread.id);
  }

  async loadOlderTimeline(selectedSequence: number | null = null): Promise<void> {
    const threadId = this.selectedThreadId;
    const cursor = this.timelinePages[0]?.previousCursor;
    if (!threadId || !cursor || this.timelineLoading) return;
    await this.loadTimeline(threadId, cursor, true, selectedSequence);
  }

  async startNewChannelSession(): Promise<void> {
    if (!this.selectedWorkingFolderId) return;
    this.selectedThreadId = null;
    this.selectedExecutionEnvironmentId = null;
    this.ensureDraftThread(this.selectedWorkingFolderId);
    this.interaction = null;
    await this.composerController.bind(
      this.selectedWorkingFolderId,
      null,
      null,
    );
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
  setExecutionEnvironment(environmentId: string | null): void { this.selectedExecutionEnvironmentId = environmentId; }
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
            this.sendError = chatErrorMessage(error);
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
    const selectedCurrent = this.selectedThread;
    const current = selectedCurrent && !threadMatchesExecutionConfiguration(
      selectedCurrent,
      workingFolderId,
      providerInstanceId,
      model.modelId,
      model.providerManaged,
      model.options,
    )
      ? null
      : selectedCurrent;
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
        sourceThreadId: threadId,
      },
    };
    this.composerController.markSent();
    let result: Awaited<ReturnType<typeof chatApi.sendChatTurn>>;
    try {
      if (mentions.length > 0) {
        await chatApi.validateChatWorkingFolderMentions(
          workingFolderId,
          mentions.map((mention) => mention.relativePath),
          this.selectedExecutionEnvironmentId,
        );
      }
      await this.composerController.flush();
      result = await chatApi.sendChatTurn({
        command: { clientCommandId: crypto.randomUUID(), expectedThreadRevision: current?.revision ?? null },
        workingFolderId,
        threadId: current?.id ?? null,
        newThreadId,
        executionEnvironmentId: current ? null : this.selectedExecutionEnvironmentId,
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
    this.selectedThreadId = result.thread.id;
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
    await this.nativeChanges.push(threadId);
  }

  private async refreshNativeChange(threadId: string): Promise<void> {
    const channelId = this.selectedChannelId;
    if (channelId) await this.loadChannelMessages(channelId, true);
    if (this.openReplyThreadId) await this.loadReplyThread(this.openReplyThreadId, true);
    if (threadId !== this.selectedThreadId) return;
    await this.loadTimeline(threadId);
    await this.refreshInteraction(threadId);
  }

  async renameThread(thread: ChatThreadShellRead, title: string): Promise<void> {
    this.upsertThread(await chatApi.renameChatThread(thread.id, title, thread.revision));
  }

  async setThreadRead(thread: ChatThreadShellRead, read: boolean): Promise<void> {
    this.upsertThread(await chatApi.setChatThreadRead(thread.id, read, thread.revision));
  }

  async forkThread(thread: ChatThreadShellRead, title: string): Promise<void> {
    const forked = await chatApi.forkChatThread(thread.id, crypto.randomUUID(), title);
    this.activeThreads = [forked, ...this.activeThreads.filter((entry) => entry.id !== forked.id)];
    this.selectThread(forked.id);
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
    if (remembered && ![...this.activeThreads, ...this.archivedThreads].some((thread) => thread.id === remembered)) {
      try {
        this.upsertThread(await chatApi.readChatThreadShell(remembered));
      } catch (error: unknown) {
        console.warn("Remembered Chat thread could not be restored", error);
      }
    }
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
        this.sendError = chatErrorMessage(error);
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
        executionEnvironmentId: null,
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

  private composerSeed(thread: ChatThreadShellRead | null): ChatComposerSeed | null {
    if (!thread) return null;
    return {
      providerInstanceId: thread.providerInstanceId,
      modelSelection: composerModelSelection(thread.modelId, thread.modelId === null, thread.modelOptions),
      safetyMode: thread.modes.safetyMode,
      interactionMode: thread.modes.interactionMode,
    };
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

  private upsertChannel(channel: ChatChannelRead): void {
    this.mergeNavigationChannels([channel]);
    const target = channel.archivedAt ? this.archivedChannels : this.activeChannels;
    const next = target.some((entry) => entry.id === channel.id)
      ? target.map((entry) => entry.id === channel.id ? channel : entry)
      : [...target, channel];
    if (channel.archivedAt) {
      this.activeChannels = this.activeChannels.filter((entry) => entry.id !== channel.id);
      this.archivedChannels = next.sort(channelSort);
    } else {
      this.archivedChannels = this.archivedChannels.filter((entry) => entry.id !== channel.id);
      this.activeChannels = next.sort(channelSort);
    }
  }

  async loadChannelMessages(channelId: ChatChannelId, force = false): Promise<void> {
    if (!force) {
      const cached = this.channelPageCache.get(channelId);
      if (cached) {
        this.channelMessagesLoading = false;
        this.channelMessagesError = null;
        this.channelPages = cached;
        this.channelMessages = mergeCommunicationMessages(cached.flatMap((page) => page.messages));
        return;
      }
    }
    const request = ++this.channelMessagesRequest;
    this.channelMessagesLoading = true;
    this.channelMessagesError = null;
    try {
      const page = await chatApi.readChatChannelPage(channelId);
      if (request !== this.channelMessagesRequest || this.selectedChannelId !== channelId) return;
      this.channelPages = [page];
      this.channelMessages = page.messages;
      this.channelPageCache.set(channelId, this.channelPages);
    } catch (error: unknown) {
      if (request !== this.channelMessagesRequest) return;
      this.channelMessagesError = chatErrorMessage(error, "Chat channel could not be loaded");
      throw error;
    } finally {
      if (request === this.channelMessagesRequest) this.channelMessagesLoading = false;
    }
  }

  async loadOlderChannelMessages(): Promise<void> {
    const channelId = this.selectedChannelId;
    if (!channelId || this.channelMessagesLoading) return;
    const currentPage = this.channelPages[0];
    if (!currentPage?.previousCursor) return;
    const request = ++this.channelMessagesRequest;
    this.channelMessagesLoading = true;
    this.channelMessagesError = null;
    try {
      const page = await chatApi.readChatChannelPage(channelId, currentPage.previousCursor);
      if (request !== this.channelMessagesRequest || this.selectedChannelId !== channelId) return;
      this.channelPages = [page, ...this.channelPages].slice(0, CHAT_TIMELINE_MAX_PAGES_PER_CHANNEL);
      this.channelMessages = mergeCommunicationMessages(this.channelPages.flatMap((entry) => entry.messages));
      this.channelPageCache.set(channelId, this.channelPages);
    } catch (error: unknown) {
      if (request !== this.channelMessagesRequest) return;
      this.channelMessagesError = chatErrorMessage(error, "Older channel history could not be loaded");
      throw error;
    } finally {
      if (request === this.channelMessagesRequest) this.channelMessagesLoading = false;
    }
  }

  async loadReplyThread(replyThreadId: ChatReplyThreadId, force = false): Promise<void> {
    if (!force) {
      const cached = this.replyThreadPageCache.get(replyThreadId);
      if (cached) {
        this.replyThreadPages = cached;
        this.replyThread = mergeReplyThreadPages(cached);
        this.replyThreadLoading = false;
        this.replyThreadError = null;
        return;
      }
    }
    const request = ++this.replyThreadRequest;
    this.replyThreadLoading = true;
    this.replyThreadError = null;
    try {
      const page = await chatApi.readChatReplyThreadPage(replyThreadId);
      if (request !== this.replyThreadRequest || this.openReplyThreadId !== replyThreadId) return;
      this.replyThreadPages = [page];
      this.replyThread = page;
      this.replyThreadPageCache.set(replyThreadId, [page]);
    } catch (error: unknown) {
      if (request !== this.replyThreadRequest) return;
      this.replyThreadError = chatErrorMessage(error, "Reply thread could not be loaded");
      throw error;
    } finally {
      if (request === this.replyThreadRequest) this.replyThreadLoading = false;
    }
  }

  async loadOlderReplyThreadMessages(): Promise<void> {
    const replyThreadId = this.openReplyThreadId;
    const currentPage = this.replyThreadPages[0];
    if (!replyThreadId || !currentPage?.previousCursor || this.replyThreadLoading) return;
    const request = ++this.replyThreadRequest;
    this.replyThreadLoading = true;
    try {
      const page = await chatApi.readChatReplyThreadPage(replyThreadId, currentPage.previousCursor);
      if (request !== this.replyThreadRequest || this.openReplyThreadId !== replyThreadId) return;
      this.replyThreadPages = [page, ...this.replyThreadPages].slice(0, CHAT_TIMELINE_MAX_PAGES_PER_CHANNEL);
      this.replyThread = mergeReplyThreadPages(this.replyThreadPages);
      this.replyThreadPageCache.set(replyThreadId, this.replyThreadPages);
    } finally {
      if (request === this.replyThreadRequest) this.replyThreadLoading = false;
    }
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
      this.timelineError = chatErrorMessage(error, "Chat could not be loaded");
      throw error;
    } finally {
      if (request === this.timelineRequest) this.timelineLoading = false;
    }
  }
}

function channelSort(left: ChatChannelRead, right: ChatChannelRead): number {
  if (left.isDefault !== right.isDefault) return left.isDefault ? -1 : 1;
  return left.name.localeCompare(right.name) || left.id.localeCompare(right.id);
}

function threadMatchesExecutionConfiguration(
  thread: ChatThreadShellRead,
  workingFolderId: ProjectWorkingFolderId,
  providerInstanceId: ProviderInstanceId,
  modelId: ModelId | null,
  providerManagedModel: boolean,
  modelOptions: readonly import("$lib/chat/contracts").ModelOptionSelection[],
): boolean {
  return thread.workingFolderId === workingFolderId
    && thread.providerInstanceId === providerInstanceId
    && thread.modelId === modelId
    && (thread.modelId === null) === providerManagedModel
    && JSON.stringify(thread.modelOptions) === JSON.stringify(modelOptions);
}

function emptyOrganizationalDraft(): ChatOrganizationalDraft {
  return {
    normalizedMarkdown: "",
    richContent: { schemaVersion: 1, value: { type: "document", children: [] } },
    attachmentIds: [],
    participantMentions: [],
    resourceReferences: [],
    selectionStart: 0,
    selectionEnd: 0,
  };
}

function cloneOrganizationalDraft(draft: ChatOrganizationalDraft): ChatOrganizationalDraft {
  return structuredClone(draft);
}

function mergeCommunicationMessages(messages: readonly ChatMessageRead[]): ChatMessageRead[] {
  const byId = new Map(messages.map((message) => [message.itemId, message]));
  return [...byId.values()].sort((left, right) => (
    left.ordinal - right.ordinal || left.itemId.localeCompare(right.itemId)
  ));
}

function mergeReplyThreadPages(pages: readonly ChatReplyThreadPageRead[]): ChatReplyThreadPageRead | null {
  const latest = pages.at(-1);
  if (!latest) return null;
  return {
    ...latest,
    replies: mergeCommunicationMessages(pages.flatMap((page) => page.replies)),
    previousCursor: pages[0]?.previousCursor ?? null,
    agentRuns: pages.flatMap((page) => page.agentRuns).filter((run, index, runs) => (
      runs.findIndex((candidate) => candidate.id === run.id) === index
    )),
  };
}

let store: ChatStore | null = null;

export function getChat(): ChatStore {
  store ??= new ChatStore();
  return store;
}
