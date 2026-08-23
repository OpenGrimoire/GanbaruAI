<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import Folder from "@lucide/svelte/icons/folder";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import * as chatApi from "$lib/api/chat";
  import {
    copyModelOptionSelections,
    copyVersionedJson,
    resolveDefaultProviderModel,
  } from "$lib/chat/composer-model";
  import type {
    ChatAccessProfileRead,
    ChatAiTeammateRead,
    ChatChannelRead,
    ChatFolderCapability,
    ChatRuntimeApprovalPolicy,
    ChatTeammatePolicyInput,
    ChatTeammateAccessRead,
    ChatTeammateAccessPreviewRead,
    ChatTeammateChannelAccessInput,
    ModelOptionSelection,
    ReplaceChatTeammateAccessRequest,
    SafetyMode,
    VersionedJson,
  } from "$lib/chat/contracts";
  import { chatErrorCode, chatErrorField, chatErrorMessage } from "$lib/chat/error-presentation";
  import { modelCompany } from "$lib/chat/model-company";
  import { preferredProjectWorkingFolder } from "$lib/chat/working-folder-selection";
  import {
    applyAccessProfileToScope,
    applyChannelPresetToScope,
    capabilitiesForPreset,
    channelCapabilityPreset,
    folderCapabilityFits,
    teammateAccessDraftErrors,
    teammateAccessDraftSnapshot,
    toggleSelectionGroup,
    type ChatChannelCapabilityPreset,
  } from "$lib/chat/teammate-access";
  import {
    teammateExecutionSummary,
    teammateProfileDraftSnapshot,
  } from "$lib/chat/teammate-draft";
  import {
    cloneChatTeammateStudioDraft,
    compareChatTeammateStudioDrafts,
    rebaseChatTeammateStudioDraft,
    type ChatTeammateStudioDraft,
  } from "$lib/chat/teammate-access-conflict";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import CustomSelect from "$lib/components/settings/CustomSelect.svelte";
  import SettingsCheckbox from "$lib/components/settings/SettingsCheckbox.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import ChatAccessControl from "$lib/components/chat/ChatAccessControl.svelte";
  import ChatControlMenu, {
    type ChatControlIcon,
    type ChatControlOption,
  } from "$lib/components/chat/ChatControlMenu.svelte";
  import ChatModelAvatar from "$lib/components/chat/ChatModelAvatar.svelte";
  import ChatModelControls from "$lib/components/chat/ChatModelControls.svelte";
  import ChatParticipantAvatar from "$lib/components/chat/ChatParticipantAvatar.svelte";
  type ChatChannelAccessBrowserComponent = typeof import("./ChatChannelAccessBrowser.svelte").default;
  type ChatChannelScopeControlsComponent = typeof import("./ChatChannelScopeControls.svelte").default;
  type ChatAccessProfilesManagerComponent = typeof import("./ChatAccessProfilesManager.svelte").default;

  let {
    initialTeammateId,
    initialChannelId,
    initialCreate = false,
    onDraftStateChange = () => {},
  }: {
    initialTeammateId?: string;
    initialChannelId?: string;
    initialCreate?: boolean;
    onDraftStateChange?: (open: boolean) => void;
  } = $props();

  type LifecycleAction = "archive" | "delete";
  type ConflictRecoveryNotice = "rebased" | "reloaded";

  interface TeammateAccessConflictState {
    teammateId: string;
    localDraft: ChatTeammateStudioDraft;
    baselineDraft: ChatTeammateStudioDraft;
    durableDraft: ChatTeammateStudioDraft;
    durableTeammate: ChatAiTeammateRead;
    durableAccess: ChatTeammateAccessRead;
  }

  const chat = getChat();
  const projects = getProjects();
  const { t } = getLocalization();

  let selectedId = $state<string | null>(null);
  let creating = $state(false);
  let showArchived = $state(false);
  let directoryQuery = $state("");
  let archivedTeammates = $state<ChatAiTeammateRead[]>([]);
  let navigationChannels = $state<ChatChannelRead[]>([]);
  let accessProfiles = $state<ChatAccessProfileRead[]>([]);
  let loadingDirectory = $state(true);
  let loadingAccess = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let errorField = $state<string | null>(null);
  let directoryScrollElement = $state<HTMLElement>();
  let detailScrollElement = $state<HTMLElement>();
  let editorDialogElement = $state<HTMLDivElement>();
  let editorReturnFocusElement: HTMLElement | null = null;
  let closeConfirmationOpen = $state(false);

  let displayName = $state("");
  let role = $state("");
  let instructions = $state("");
  let providerId = $state("");
  let safetyMode = $state<SafetyMode>("ask_for_approval");
  let modelId = $state("");
  let providerManagedModel = $state(false);
  let modelOptions = $state<ModelOptionSelection[]>([]);
  let effort = $state<string | null>(null);
  let speed = $state<string | null>(null);
  let providerOptions = $state<VersionedJson>({ schemaVersion: 1, value: {} });
  let teammateDefaultRuntimeApproval = $state<ChatRuntimeApprovalPolicy>("ask");
  let profileExpectedRevision = $state(0);
  let profileAvatar = $state<VersionedJson>({ schemaVersion: 1, value: { kind: "initials" } });
  let accessRevision = $state(0);
  let accessDraft = $state<ChatTeammateChannelAccessInput[]>([]);
  let profileBaseline = $state<string | null>(null);
  let accessBaseline = $state<string | null>(null);
  let studioDraftBaseline = $state<ChatTeammateStudioDraft | null>(null);
  let accessPreview = $state<ChatTeammateAccessPreviewRead | null>(null);
  let accessPreviewSnapshot = $state<string | null>(null);
  let accessConflict = $state<TeammateAccessConflictState | null>(null);
  let conflictPendingDraft = $state<ChatTeammateStudioDraft | null>(null);
  let conflictLoading = $state(false);
  let conflictError = $state<string | null>(null);
  let conflictRecoveryNotice = $state<ConflictRecoveryNotice | null>(null);
  let conflictPanelElement = $state<HTMLElement>();
  let recoveryNoticeElement = $state<HTMLElement>();
  let accessLoadRequest = 0;
  let conflictLoadRequest = 0;

  let profileManagerOpen = $state(false);
  let toolsMenuOpen = $state(false);
  let profileManagerLoading = $state(false);
  let ChatChannelAccessBrowser = $state<ChatChannelAccessBrowserComponent | null>(null);
  let ChatChannelScopeControls = $state<ChatChannelScopeControlsComponent | null>(null);
  let ChatAccessProfilesManager = $state<ChatAccessProfilesManagerComponent | null>(null);
  let toolsMenuElement = $state<HTMLDivElement>();
  let toolsMenuTrigger = $state<HTMLButtonElement>();
  let profileManagerLoad: Promise<void> | null = null;
  let focusedAccessChannelId = $state<string | null>(null);
  let advancedChannelIds = $state<Set<string>>(new Set());

  let lifecycleAction = $state<LifecycleAction | null>(null);
  let lifecycleTarget = $state<ChatAiTeammateRead | null>(null);
  let lifecycleBusy = $state(false);
  let lifecycleError = $state<string | null>(null);
  let initialSelectionApplied = false;
  let preserveStudioDraftForId: string | null = null;

  const allDirectoryTeammates = $derived(showArchived
    ? [...chat.teammates, ...archivedTeammates]
    : chat.teammates);
  const filteredDirectoryTeammates = $derived.by(() => {
    const query = directoryQuery.trim().toLocaleLowerCase();
    if (!query) return allDirectoryTeammates;
    return allDirectoryTeammates.filter((teammate) => (
      `${teammate.participant.displayName} ${teammate.role}`.toLocaleLowerCase().includes(query)
    ));
  });
  const allTeammates = $derived([...chat.teammates, ...archivedTeammates]);
  const selected = $derived(allDirectoryTeammates.find((entry) => entry.participant.id === selectedId) ?? null);
  const archivedMode = $derived(Boolean(selected?.participant.archivedAt));
  const providers = $derived(chat.settings?.providerInstances ?? []);
  const selectedProvider = $derived(providers.find((entry) => entry.configuration.instanceId === providerId) ?? null);
  const providerAuthoritySupport = $derived(selectedProvider?.lastProbe?.authoritySupport ?? null);
  const availableModels = $derived(selectedProvider?.modelCatalog?.models.filter((model) => model.availability !== "deprecated") ?? []);
  const selectedModel = $derived(availableModels.find((model) => model.id === modelId) ?? null);
  const permissionWorkingFolderId = $derived(
    accessDraft.flatMap((channel) => channel.folderGrants)
      .find((grant) => grant.isDefault)?.workingFolderId ?? null,
  );
  const draftCompany = $derived(selectedProvider ? modelCompany(selectedProvider.configuration.familyId, selectedModel) : null);
  const modelSelectionValid = $derived(providerManagedModel !== Boolean(modelId));
  const draftConfigurationState = $derived(
    selectedProvider?.configuration.enabled
      && selectedProvider.lastProbe?.state === "healthy"
      && modelSelectionValid
      ? "healthy"
      : "needs_setup",
  );
  const normalizedDisplayName = $derived(displayName.trim().toLocaleLowerCase());
  const nameTaken = $derived(Boolean(normalizedDisplayName && allTeammates.some((teammate) => (
    teammate.participant.id !== selectedId
      && teammate.participant.displayName.trim().toLocaleLowerCase() === normalizedDisplayName
  ))));
  const activeNavigationChannels = $derived(navigationChannels.filter((channel) => channel.archivedAt === null));
  const selectedChannelIds = $derived(new Set(accessDraft.map((channel) => channel.channelId)));
  const focusedAccessChannel = $derived(channelById(focusedAccessChannelId ?? ""));
  const focusedAccessDraft = $derived(accessDraft.find((channel) => (
    channel.channelId === focusedAccessChannelId
  )) ?? null);
  const profileCeilings = $derived(new Map(accessProfiles.map((profile) => [
    profile.id,
    profile.latestRevision.maximumFolderCapability,
  ])));
  const accessErrors = $derived(teammateAccessDraftErrors(accessDraft, profileCeilings));
  const providerResourceBlockers = $derived(accessDraft
    .flatMap(providerResourceIssuesForChannel)
    .filter((issue, index, issues) => issues.indexOf(issue) === index));
  const accessProfileOptions = $derived<ChatControlOption[]>(accessProfiles.map((profile) => ({
    value: profile.id,
    label: profile.builtinKey
      ? t(`settings.chat.teammates.profiles.${profile.builtinKey}`)
      : profile.displayName,
    description: profileDescription(profile.latestRevision.maximumFolderCapability),
    icon: profileIcon(profile.latestRevision.maximumFolderCapability),
  })));
  const currentProfileSnapshot = $derived(teammateProfileDraftSnapshot({
    displayName,
    role,
    instructions,
    providerId,
    safetyMode,
    modelId,
    providerManagedModel,
    modelOptions,
    effort,
    speed,
  }));
  const currentAccessSnapshot = $derived(JSON.stringify({
    teammateDefaultRuntimeApproval,
    channels: teammateAccessDraftSnapshot(accessDraft),
  }));
  const currentDraftSnapshot = $derived(JSON.stringify({
    profile: currentProfileSnapshot,
    providerOptions: JSON.stringify(providerOptions),
    access: currentAccessSnapshot,
  }));
  const conflictComparison = $derived(accessConflict
    ? compareChatTeammateStudioDrafts(captureStudioDraft(), accessConflict.durableDraft)
    : null);
  const profileDirty = $derived(profileBaseline !== null && currentProfileSnapshot !== profileBaseline);
  const accessDirty = $derived(accessBaseline !== null && currentAccessSnapshot !== accessBaseline);
  const dirty = $derived(profileDirty || accessDirty);
  const canSave = $derived(Boolean(
    displayName.trim()
      && role.trim()
      && providerId
      && modelSelectionValid
      && !nameTaken
      && accessErrors.length === 0
      && providerResourceBlockers.length === 0
      && dirty
      && !loadingAccess
      && !accessConflict
      && !conflictLoading
      && !conflictError
      && !archivedMode,
  ));

  const historyOptions = $derived([
    { value: "entire", label: t("settings.chat.teammates.historyEntire") },
    { value: "fromGrant", label: t("settings.chat.teammates.historyFromGrant") },
  ]);
  const channelRuntimeOptions = $derived([
    { value: "inherit", label: t("settings.chat.teammates.runtime.inherit") },
    { value: "ask", label: t("settings.chat.teammates.runtime.ask") },
    { value: "autoApprove", label: t("settings.chat.teammates.runtime.autoApprove") },
    { value: "unattended", label: t("settings.chat.teammates.runtime.unattended") },
    { value: "providerCustom", label: t("settings.chat.teammates.runtime.providerCustom") },
  ]);
  const folderRuntimeOptions = $derived([
    { value: "inherit", label: t("settings.chat.teammates.runtime.inheritChannel") },
    { value: "ask", label: t("settings.chat.teammates.runtime.ask") },
    { value: "autoApprove", label: t("settings.chat.teammates.runtime.autoApprove") },
    { value: "unattended", label: t("settings.chat.teammates.runtime.unattended") },
    { value: "providerCustom", label: t("settings.chat.teammates.runtime.providerCustom") },
  ]);

  $effect(() => {
    onDraftStateChange(creating || dirty);
    return () => onDraftStateChange(false);
  });

  $effect(() => {
    const snapshot = currentDraftSnapshot;
    if (accessPreview && accessPreviewSnapshot !== snapshot) {
      accessPreview = null;
      accessPreviewSnapshot = null;
    }
  });

  $effect(() => {
    if (initialSelectionApplied || loadingDirectory) return;
    initialSelectionApplied = true;
    if (initialCreate) {
      beginCreate(initialChannelId ?? null);
      return;
    }
    if (initialTeammateId && allDirectoryTeammates.some((entry) => entry.participant.id === initialTeammateId)) {
      selectedId = initialTeammateId;
    }
  });

  $effect(() => {
    if (!creating && !selected) return;
    void tick().then(() => editorDialogElement?.focus());
  });

  $effect(() => {
    const teammate = selected;
    if (!teammate || creating) return;
    if (preserveStudioDraftForId === teammate.participant.id) {
      preserveStudioDraftForId = null;
      return;
    }
    untrack(() => initializeSelectedTeammate(teammate));
  });

  $effect(() => {
    if (!toolsMenuOpen) return;
    function closeOnOutsideClick(event: MouseEvent): void {
      if (!(event.target instanceof Node)) return;
      if (toolsMenuElement?.contains(event.target) || toolsMenuTrigger?.contains(event.target)) return;
      toolsMenuOpen = false;
    }
    window.addEventListener("mousedown", closeOnOutsideClick, true);
    return () => window.removeEventListener("mousedown", closeOnOutsideClick, true);
  });

  onMount(() => {
    void loadChannelAccessControls();
    void loadDirectoryData();
  });

  async function loadChannelAccessControls(): Promise<void> {
    try {
      const [browserModule, scopeControlsModule] = await Promise.all([
        import("./ChatChannelAccessBrowser.svelte"),
        import("./ChatChannelScopeControls.svelte"),
      ]);
      ChatChannelAccessBrowser = browserModule.default;
      ChatChannelScopeControls = scopeControlsModule.default;
    } catch (cause: unknown) {
      errorField = null;
      error = chatErrorMessage(cause, t("settings.chat.teammates.loadFailed"));
    }
  }

  async function loadDirectoryData(): Promise<void> {
    loadingDirectory = true;
    error = null;
    const [archivedResult, channelResult, profileResult] = await Promise.allSettled([
      chatApi.listChatTeammates(true),
      chatApi.listChatNavigationChannels(),
      chatApi.listChatAccessProfiles(false),
    ]);
    if (archivedResult.status === "fulfilled") {
      archivedTeammates = archivedResult.value;
      chat.archivedTeammates = archivedResult.value;
    }
    if (channelResult.status === "fulfilled") navigationChannels = channelResult.value;
    if (profileResult.status === "fulfilled") accessProfiles = profileResult.value;
    const failure = [archivedResult, channelResult, profileResult]
      .find((result) => result.status === "rejected");
    if (failure?.status === "rejected") {
      error = chatErrorMessage(failure.reason, t("settings.chat.teammates.loadFailed"));
    }
    loadingDirectory = false;
  }

  function captureStudioDraft(): ChatTeammateStudioDraft {
    return {
      profile: {
        displayName,
        role,
        instructions,
        providerId,
        safetyMode,
        modelId,
        providerManagedModel,
        modelOptions: copyModelOptionSelections(modelOptions),
        effort,
        speed,
        providerOptions: copyVersionedJson(providerOptions),
      },
      teammateDefaultRuntimeApproval,
      channels: copyChannelAccessInputs(accessDraft),
    };
  }

  function draftFromReads(
    teammate: ChatAiTeammateRead,
    access: ChatTeammateAccessRead,
  ): ChatTeammateStudioDraft {
    const policy = teammate.latestPolicy;
    const options = copyModelOptionSelections(policy?.modelOptions ?? []);
    const provider = providers.find((entry) => (
      entry.configuration.instanceId === policy?.providerInstanceId
    )) ?? null;
    const model = provider?.modelCatalog?.models.find((entry) => entry.id === policy?.modelId) ?? null;
    const summary = teammateExecutionSummary(options, model);
    return {
      profile: {
        displayName: teammate.participant.displayName,
        role: teammate.role,
        instructions: teammate.instructions,
        providerId: policy?.providerInstanceId ?? "",
        safetyMode: policy?.safetyMode ?? "ask_for_approval",
        modelId: policy?.modelId ?? "",
        providerManagedModel: policy?.providerManagedModel ?? false,
        modelOptions: options,
        effort: summary.effort ?? policy?.effort ?? null,
        speed: summary.speed ?? policy?.speed ?? null,
        providerOptions: copyVersionedJson(
          policy?.providerOptions ?? { schemaVersion: 1, value: {} },
        ),
      },
      teammateDefaultRuntimeApproval: access.teammateDefaultRuntimeApproval,
      channels: access.channels
        .filter((channel) => channel.removedAt === null)
        .map((channel) => ({
          channelId: channel.channelId,
          accessProfileId: channel.accessProfileId,
          accessProfileRevision: channel.accessProfileRevision,
          capabilities: { ...channel.capabilities },
          historyBoundary: channel.historyBoundary.kind === "entire"
            ? { kind: "entire" as const }
            : { kind: "fromGrant" as const },
          runtimeApprovalOverride: channel.runtimeApprovalOverride,
          scratchRuntimeApprovalOverride: channel.scratchRuntimeApprovalOverride,
          folderGrants: channel.folderGrants
            .filter((grant) => grant.revokedAt === null)
            .map((grant) => ({
              workingFolderId: grant.workingFolderId,
              capability: grant.capability,
              isDefault: grant.isDefault,
              runtimeApprovalOverride: grant.runtimeApprovalOverride,
            })),
        })),
    };
  }

  function applyStudioDraft(draft: ChatTeammateStudioDraft): void {
    const copy = cloneChatTeammateStudioDraft(draft);
    displayName = copy.profile.displayName;
    role = copy.profile.role;
    instructions = copy.profile.instructions;
    providerId = copy.profile.providerId;
    safetyMode = copy.profile.safetyMode;
    modelId = copy.profile.modelId;
    providerManagedModel = copy.profile.providerManagedModel;
    modelOptions = copy.profile.modelOptions;
    effort = copy.profile.effort;
    speed = copy.profile.speed;
    providerOptions = copy.profile.providerOptions;
    teammateDefaultRuntimeApproval = copy.teammateDefaultRuntimeApproval;
    accessDraft = copy.channels;
  }

  function setStudioDraftBaseline(draft: ChatTeammateStudioDraft): void {
    const copy = cloneChatTeammateStudioDraft(draft);
    studioDraftBaseline = copy;
    profileBaseline = profileSnapshotForDraft(copy);
    accessBaseline = accessSnapshotForDraft(copy);
  }

  function profileSnapshotForDraft(draft: ChatTeammateStudioDraft): string {
    return teammateProfileDraftSnapshot(draft.profile);
  }

  function accessSnapshotForDraft(draft: ChatTeammateStudioDraft): string {
    return JSON.stringify({
      teammateDefaultRuntimeApproval: draft.teammateDefaultRuntimeApproval,
      channels: teammateAccessDraftSnapshot(draft.channels),
    });
  }

  function draftSnapshotForDraft(draft: ChatTeammateStudioDraft): string {
    return JSON.stringify({
      profile: profileSnapshotForDraft(draft),
      providerOptions: JSON.stringify(draft.profile.providerOptions),
      access: accessSnapshotForDraft(draft),
    });
  }

  function copyChannelAccessInputs(
    channels: readonly ChatTeammateChannelAccessInput[],
  ): ChatTeammateChannelAccessInput[] {
    return channels.map((channel) => ({
      ...channel,
      capabilities: { ...channel.capabilities },
      historyBoundary: channel.historyBoundary.kind === "entire"
        ? { kind: "entire" }
        : { kind: "fromGrant", lowerOrdinal: channel.historyBoundary.lowerOrdinal },
      folderGrants: channel.folderGrants.map((grant) => ({ ...grant })),
    }));
  }

  function clearAccessConflict(): void {
    conflictLoadRequest += 1;
    accessConflict = null;
    conflictPendingDraft = null;
    conflictLoading = false;
    conflictError = null;
    conflictRecoveryNotice = null;
  }

  function beginCreate(
    channelId: string | null = null,
    returnFocusElement: HTMLElement | null = null,
  ): void {
    clearAccessConflict();
    editorReturnFocusElement = returnFocusElement;
    creating = true;
    selectedId = null;
    focusedAccessChannelId = channelId;
    displayName = "";
    role = "";
    instructions = "";
    const resolved = resolveDefaultProviderModel(providers);
    providerId = resolved?.provider.configuration.instanceId ?? "";
    safetyMode = "ask_for_approval";
    modelId = resolved?.model?.id ?? "";
    providerManagedModel = resolved?.providerManaged ?? false;
    modelOptions = copyModelOptionSelections(resolved?.options ?? []);
    const summary = teammateExecutionSummary(modelOptions, resolved?.model ?? null);
    effort = summary.effort;
    speed = summary.speed;
    providerOptions = { schemaVersion: 1, value: {} };
    teammateDefaultRuntimeApproval = "ask";
    profileExpectedRevision = 0;
    profileAvatar = { schemaVersion: 1, value: { kind: "initials" } };
    accessRevision = 0;
    accessDraft = channelId ? [defaultChannelAccess(channelId)] : [];
    setStudioDraftBaseline(captureStudioDraft());
    accessPreview = null;
    accessPreviewSnapshot = null;
    error = null;
    errorField = null;
    lifecycleError = null;
  }

  function openTeammate(teammateId: string, returnFocusElement: HTMLElement): void {
    closeConfirmationOpen = false;
    editorReturnFocusElement = returnFocusElement;
    lifecycleError = null;
    creating = false;
    selectedId = teammateId;
  }

  function closeEditor(): void {
    const returnFocusElement = editorReturnFocusElement;
    editorReturnFocusElement = null;
    clearAccessConflict();
    creating = false;
    selectedId = null;
    profileBaseline = null;
    accessBaseline = null;
    studioDraftBaseline = null;
    accessPreview = null;
    accessPreviewSnapshot = null;
    void tick().then(() => returnFocusElement?.focus());
  }

  function requestCloseEditor(): void {
    if (saving || lifecycleBusy) return;
    if (dirty) {
      closeConfirmationOpen = true;
      return;
    }
    closeEditor();
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key !== "Escape" || (!creating && !selected)) return;
    if (profileManagerOpen || lifecycleAction || closeConfirmationOpen) return;
    if (toolsMenuOpen) {
      toolsMenuOpen = false;
      return;
    }
    if (event.defaultPrevented || document.querySelector("[data-app-floating-surface]")) return;
    event.preventDefault();
    event.stopPropagation();
    requestCloseEditor();
  }

  function initializeSelectedTeammate(teammate: ChatAiTeammateRead): void {
    clearAccessConflict();
    profileBaseline = null;
    accessBaseline = null;
    studioDraftBaseline = null;
    accessPreview = null;
    accessPreviewSnapshot = null;
    error = null;
    errorField = null;
    accessDraft = [];
    focusedAccessChannelId = null;
    displayName = teammate.participant.displayName;
    role = teammate.role;
    instructions = teammate.instructions;
    providerId = teammate.latestPolicy?.providerInstanceId ?? "";
    safetyMode = teammate.latestPolicy?.safetyMode ?? "ask_for_approval";
    modelId = teammate.latestPolicy?.modelId ?? "";
    providerManagedModel = teammate.latestPolicy?.providerManagedModel ?? false;
    modelOptions = copyModelOptionSelections(teammate.latestPolicy?.modelOptions ?? []);
    const summary = teammateExecutionSummary(modelOptions, selectedModel);
    effort = summary.effort ?? teammate.latestPolicy?.effort ?? null;
    speed = summary.speed ?? teammate.latestPolicy?.speed ?? null;
    providerOptions = copyVersionedJson(
      teammate.latestPolicy?.providerOptions ?? { schemaVersion: 1, value: {} },
    );
    profileExpectedRevision = teammate.participant.revision;
    profileAvatar = copyVersionedJson(teammate.participant.avatar);
    profileBaseline = currentProfileSnapshot;
    void loadTeammateAccess(teammate.participant.id);
  }

  async function loadTeammateAccess(teammateId: string): Promise<void> {
    const request = ++accessLoadRequest;
    loadingAccess = true;
    try {
      const access = await chatApi.readChatTeammateAccess(teammateId);
      if (request !== accessLoadRequest || selectedId !== teammateId) return;
      const teammate = allDirectoryTeammates.find((entry) => entry.participant.id === teammateId);
      if (!teammate) return;
      const durableDraft = draftFromReads(teammate, access);
      accessRevision = access.accessRevision;
      profileExpectedRevision = teammate.participant.revision;
      profileAvatar = copyVersionedJson(teammate.participant.avatar);
      teammateDefaultRuntimeApproval = durableDraft.teammateDefaultRuntimeApproval;
      accessDraft = copyChannelAccessInputs(durableDraft.channels);
      focusedAccessChannelId = initialChannelId
        ?? durableDraft.channels[0]?.channelId
        ?? null;
      setStudioDraftBaseline(durableDraft);
      if (initialChannelId) {
        if (!accessDraft.some((channel) => channel.channelId === initialChannelId)) {
          accessDraft = [...accessDraft, defaultChannelAccess(initialChannelId)];
        }
      }
    } catch (cause: unknown) {
      if (request === accessLoadRequest) {
        error = chatErrorMessage(cause, t("settings.chat.teammates.accessLoadFailed"));
      }
    } finally {
      if (request === accessLoadRequest) loadingAccess = false;
    }
  }

  async function loadAccessConflict(
    teammateId: string,
    localDraft: ChatTeammateStudioDraft,
  ): Promise<void> {
    const request = ++conflictLoadRequest;
    const retainedLocalDraft = cloneChatTeammateStudioDraft(localDraft);
    const retainedBaseline = cloneChatTeammateStudioDraft(
      studioDraftBaseline ?? retainedLocalDraft,
    );
    conflictPendingDraft = retainedLocalDraft;
    accessConflict = null;
    conflictError = null;
    conflictRecoveryNotice = null;
    accessPreview = null;
    accessPreviewSnapshot = null;
    conflictLoading = true;
    try {
      const [durableTeammate, durableAccess] = await Promise.all([
        chatApi.readChatTeammate(teammateId),
        chatApi.readChatTeammateAccess(teammateId),
      ]);
      if (request !== conflictLoadRequest || selectedId !== teammateId) return;
      accessConflict = {
        teammateId,
        localDraft: retainedLocalDraft,
        baselineDraft: retainedBaseline,
        durableDraft: draftFromReads(durableTeammate, durableAccess),
        durableTeammate,
        durableAccess,
      };
    } catch (cause: unknown) {
      if (request !== conflictLoadRequest) return;
      conflictError = chatErrorMessage(
        cause,
        t("settings.chat.teammates.conflict.loadFailed"),
      );
    } finally {
      if (request === conflictLoadRequest) {
        conflictLoading = false;
        await tick();
        conflictPanelElement?.focus();
      }
    }
  }

  function retryAccessConflict(): void {
    if (!selectedId || !conflictPendingDraft || conflictLoading) return;
    void loadAccessConflict(selectedId, conflictPendingDraft);
  }

  function rebaseAccessConflict(): void {
    const conflict = accessConflict;
    if (!conflict) return;
    const rebased = rebaseChatTeammateStudioDraft(
      conflict.baselineDraft,
      captureStudioDraft(),
      conflict.durableDraft,
    );
    applyStudioDraft(rebased);
    accessRevision = conflict.durableAccess.accessRevision;
    profileExpectedRevision = conflict.durableTeammate.participant.revision;
    profileAvatar = copyVersionedJson(conflict.durableTeammate.participant.avatar);
    setStudioDraftBaseline(conflict.durableDraft);
    settleAccessConflict("rebased");
  }

  async function reloadCurrentAccessConflict(): Promise<void> {
    const conflict = accessConflict;
    if (!conflict) return;
    applyStudioDraft(conflict.durableDraft);
    accessRevision = conflict.durableAccess.accessRevision;
    profileExpectedRevision = conflict.durableTeammate.participant.revision;
    profileAvatar = copyVersionedJson(conflict.durableTeammate.participant.avatar);
    setStudioDraftBaseline(conflict.durableDraft);
    preserveStudioDraftForId = conflict.teammateId;
    settleAccessConflict("reloaded");
    try {
      await chat.refreshTeammates();
    } catch (cause: unknown) {
      error = chatErrorMessage(cause, t("settings.chat.teammates.loadFailed"));
    }
  }

  function settleAccessConflict(notice: ConflictRecoveryNotice): void {
    conflictLoadRequest += 1;
    accessConflict = null;
    conflictPendingDraft = null;
    conflictLoading = false;
    conflictError = null;
    conflictRecoveryNotice = notice;
    accessPreview = null;
    accessPreviewSnapshot = null;
    error = null;
    errorField = null;
    void tick().then(() => recoveryNoticeElement?.focus());
  }

  function conversationProfile(): ChatAccessProfileRead | null {
    return accessProfiles.find((profile) => profile.builtinKey === "conversationOnly")
      ?? accessProfiles[0]
      ?? null;
  }

  function providerCapabilityIssue(capability: ChatFolderCapability): string | null {
    if (capability === "none") return null;
    const support = providerAuthoritySupport;
    if (!support) return t("settings.chat.teammates.providerAuthorityUnknown");
    if (capability === "read" && !support.internalHostTools && !support.readOnlyRoot) {
      return t("settings.chat.teammates.providerCannotRead");
    }
    if (capability === "edit" && !support.internalHostTools && !support.writableRoot) {
      return t("settings.chat.teammates.providerCannotEdit");
    }
    if (capability === "execute" && (
      !support.writableRoot
      || !support.confinedCommands
      || !support.networkBoundary
    )) {
      return t("settings.chat.teammates.providerCannotExecute");
    }
    if (capability === "publish" && (
      !support.writableRoot
      || !support.confinedCommands
      || !support.networkBoundary
      || !support.classifiedPublish
    )) {
      return t("settings.chat.teammates.providerCannotPublish");
    }
    return null;
  }

  function providerGrantIssue(
    capability: ChatFolderCapability,
    isDefault: boolean,
  ): string | null {
    const issue = providerCapabilityIssue(capability);
    if (issue) return issue;
    const support = providerAuthoritySupport;
    if (!isDefault || !support) return null;
    if (capability === "read" && (!support.readOnlyRoot || !support.denyShell)) {
      return t("settings.chat.teammates.providerCannotTargetReadOnly");
    }
    if (capability === "edit" && (!support.writableRoot || !support.denyShell)) {
      return t("settings.chat.teammates.providerCannotTargetWritable");
    }
    return null;
  }

  function providerResourceIssuesForChannel(channel: ChatTeammateChannelAccessInput): string[] {
    const grants = channel.folderGrants.filter((grant) => grant.capability !== "none");
    const issues = grants.flatMap((grant) => {
      const issue = providerGrantIssue(grant.capability, grant.isDefault);
      return issue ? [issue] : [];
    });
    if (grants.some((grant) => !grant.isDefault) && !providerAuthoritySupport?.internalHostTools) {
      issues.push(t("settings.chat.teammates.providerNeedsHostTools"));
    }
    return issues.filter((issue, index) => issues.indexOf(issue) === index);
  }

  function profileDescription(capability: ChatFolderCapability): string {
    if (capability === "read") return t("settings.chat.teammates.profileDescriptions.readOnly");
    if (capability === "edit") return t("settings.chat.teammates.profileDescriptions.editFiles");
    if (capability === "execute") return t("settings.chat.teammates.profileDescriptions.buildAndTest");
    if (capability === "publish") return t("settings.chat.teammates.profileDescriptions.publishChanges");
    return t("settings.chat.teammates.profileDescriptions.conversationOnly");
  }

  function profileIcon(capability: ChatFolderCapability): ChatControlIcon {
    if (capability === "read") return "folder";
    if (capability === "edit") return "file-pen";
    if (capability === "execute") return "pencil-ruler";
    if (capability === "publish") return "git-pull-request";
    return "messages-square";
  }

  function folderCapabilityLabel(capability: ChatFolderCapability): string {
    if (capability === "read") return t("settings.chat.teammates.folderCapabilities.read");
    if (capability === "edit") return t("settings.chat.teammates.folderCapabilities.edit");
    if (capability === "execute") return t("settings.chat.teammates.folderCapabilities.execute");
    if (capability === "publish") return t("settings.chat.teammates.folderCapabilities.publish");
    return t("settings.chat.teammates.folderCapabilities.none");
  }

  function defaultChannelAccess(channelId: string): ChatTeammateChannelAccessInput {
    const profile = conversationProfile();
    return {
      channelId,
      accessProfileId: profile?.id ?? "",
      accessProfileRevision: profile?.latestRevision.revision ?? 0,
      capabilities: capabilitiesForPreset("isolatedResponder"),
      historyBoundary: { kind: "entire" },
      runtimeApprovalOverride: null,
      scratchRuntimeApprovalOverride: null,
      folderGrants: [],
    };
  }

  function channelById(channelId: string): ChatChannelRead | null {
    return navigationChannels.find((channel) => channel.id === channelId) ?? null;
  }

  function channelAncestry(channel: ChatChannelRead): string {
    const project = projects.projects.find((entry) => entry.id === channel.projectId);
    const group = projects.groups.find((entry) => entry.id === project?.groupId);
    return `${group?.name ?? t("settings.chat.teammates.unknownGroup")} / ${project?.name ?? t("settings.chat.teammates.unknownProject")}`;
  }

  function updateAccessChannel(
    channelId: string,
    update: (channel: ChatTeammateChannelAccessInput) => ChatTeammateChannelAccessInput,
  ): void {
    accessDraft = accessDraft.map((channel) => channel.channelId === channelId ? update(channel) : channel);
    accessPreview = null;
    accessPreviewSnapshot = null;
  }

  function setBoundedSelection(channelIds: readonly string[], selectedValue: boolean): void {
    const next = toggleSelectionGroup([...channelIds], selectedChannelIds, selectedValue);
    const byId = new Map(accessDraft.map((channel) => [channel.channelId, channel]));
    accessDraft = [...next].map((channelId) => byId.get(channelId) ?? defaultChannelAccess(channelId));
    accessPreview = null;
    accessPreviewSnapshot = null;
  }

  function handleChannelSelection(channelIds: readonly string[], selectedValue: boolean): void {
    if (channelIds.length === 1) focusedAccessChannelId = channelIds[0] ?? null;
    setBoundedSelection(channelIds, selectedValue);
  }

  function setPresetForChannels(
    channelIds: readonly string[],
    preset: Exclude<ChatChannelCapabilityPreset, "custom">,
  ): void {
    accessDraft = applyChannelPresetToScope(accessDraft, new Set(channelIds), preset);
    accessPreview = null;
    accessPreviewSnapshot = null;
  }

  function folderCapabilityOptions(ceiling: ChatFolderCapability) {
    return (["read", "edit", "execute", "publish"] as const)
      .filter((capability) => folderCapabilityFits(capability, ceiling))
      .map((capability) => ({ value: capability, label: folderCapabilityLabel(capability) }));
  }

  function setAccessProfileForChannels(channelIds: readonly string[], accessProfileId: string): void {
    const profile = accessProfiles.find((entry) => entry.id === accessProfileId);
    const selectedIds = new Set(channelIds);
    const capability = profile?.latestRevision.maximumFolderCapability ?? "none";
    accessDraft = applyAccessProfileToScope(accessDraft, selectedIds, {
      id: accessProfileId,
      revision: profile?.latestRevision.revision ?? 0,
      maximumFolderCapability: capability,
    });
    if (capability !== "none") {
      accessDraft = accessDraft.map((channel) => {
        if (!selectedIds.has(channel.channelId)) return channel;
        if (channel.folderGrants.length > 0) {
          const nativeTarget = capability === "execute" || capability === "publish";
          const defaultFolderId = channel.folderGrants.find((grant) => grant.isDefault)?.workingFolderId
            ?? channel.folderGrants[0]?.workingFolderId;
          return {
            ...channel,
            folderGrants: channel.folderGrants.map((grant) => ({
              ...grant,
              capability,
              isDefault: nativeTarget && grant.workingFolderId === defaultFolderId,
            })),
          };
        }
        const projectId = channelById(channel.channelId)?.projectId;
        if (!projectId) return channel;
        const folder = preferredProjectWorkingFolder(chat.workingFolders, projectId, null);
        if (!folder) return channel;
        return {
          ...channel,
          folderGrants: [{
            workingFolderId: folder.workingFolder.id,
            capability,
            isDefault: capability === "execute" || capability === "publish",
            runtimeApprovalOverride: null,
          }],
        };
      });
    }
    accessPreview = null;
    accessPreviewSnapshot = null;
  }

  function foldersForChannel(channelId: string) {
    const projectId = channelById(channelId)?.projectId;
    return chat.workingFolders.filter((folder) => (
      folder.workingFolder.projectId === projectId && folder.workingFolder.archivedAt === null
    ));
  }

  function toggleFolderGrant(channelId: string, workingFolderId: string, selectedValue: boolean): void {
    updateAccessChannel(channelId, (channel) => {
      const grants = channel.folderGrants.filter((grant) => grant.workingFolderId !== workingFolderId);
      if (!selectedValue) return { ...channel, folderGrants: grants };
      const ceiling = profileCeilings.get(channel.accessProfileId) ?? "none";
      const capability = ceiling === "none" ? "none" : ceiling;
      return {
        ...channel,
        folderGrants: [...grants, {
          workingFolderId,
          capability,
          isDefault: false,
          runtimeApprovalOverride: null,
        }],
      };
    });
  }

  function setFolderCapability(
    channelId: string,
    workingFolderId: string,
    capability: ChatFolderCapability,
  ): void {
    updateAccessChannel(channelId, (channel) => ({
      ...channel,
      folderGrants: channel.folderGrants.map((grant) => grant.workingFolderId === workingFolderId
        ? {
            ...grant,
            capability,
            isDefault: grant.isDefault,
          }
        : grant),
    }));
  }

  function setDefaultFolder(channelId: string, workingFolderId: string): void {
    updateAccessChannel(channelId, (channel) => ({
      ...channel,
      folderGrants: channel.folderGrants.map((grant) => ({
        ...grant,
        isDefault: grant.workingFolderId === workingFolderId,
      })),
    }));
  }

  function setFolderRuntimeApproval(
    channelId: string,
    workingFolderId: string,
    policy: ChatRuntimeApprovalPolicy | null,
  ): void {
    updateAccessChannel(channelId, (channel) => ({
      ...channel,
      folderGrants: channel.folderGrants.map((grant) => grant.workingFolderId === workingFolderId
        ? { ...grant, runtimeApprovalOverride: policy }
        : grant),
    }));
  }

  async function recoverFolder(folderId: string, bindingStatus: string, managed: boolean): Promise<void> {
    try {
      if (managed && bindingStatus === "missing") {
        await chat.recreateManagedWorkingFolder(folderId);
      } else if (bindingStatus === "repository_mismatch") {
        await chat.rebindWorkingFolder(folderId, t("settings.chat.teammates.locateFolder"));
      } else {
        await chat.locateWorkingFolder(folderId, t("settings.chat.teammates.locateFolder"));
      }
    } catch (cause: unknown) {
      error = chatErrorMessage(cause, t("settings.chat.teammates.folderRecoveryFailed"));
    }
  }

  function loadProfileManager(): Promise<void> {
    if (ChatAccessProfilesManager) return Promise.resolve();
    profileManagerLoad ??= import("./ChatAccessProfilesManager.svelte")
      .then((module) => { ChatAccessProfilesManager = module.default; })
      .finally(() => { profileManagerLoad = null; });
    return profileManagerLoad;
  }

  async function openProfileManager(): Promise<void> {
    if (profileManagerLoading || profileManagerOpen) return;
    toolsMenuOpen = false;
    profileManagerLoading = true;
    try {
      await loadProfileManager();
      profileManagerOpen = true;
    } catch (cause: unknown) {
      errorField = null;
      error = chatErrorMessage(cause, t("settings.chat.teammates.loadFailed"));
    } finally {
      profileManagerLoading = false;
    }
  }

  function handleProfilesChange(nextProfiles: ChatAccessProfileRead[]): void {
    accessProfiles = nextProfiles;
    const teammateId = selected?.participant.id;
    if (teammateId) void loadTeammateAccess(teammateId);
  }

  function selectExecution(selection: {
    providerInstanceId: string | null;
    modelId: string | null;
    providerManaged: boolean;
    options: ModelOptionSelection[];
  }): void {
    providerId = selection.providerInstanceId ?? "";
    modelId = selection.modelId ?? "";
    providerManagedModel = selection.providerManaged;
    modelOptions = copyModelOptionSelections(selection.options);
    const provider = providers.find((entry) => entry.configuration.instanceId === providerId) ?? null;
    const model = provider?.modelCatalog?.models.find((entry) => entry.id === modelId) ?? null;
    const summary = teammateExecutionSummary(modelOptions, model);
    effort = summary.effort;
    speed = summary.speed;
  }

  function accessReplacementRequest(
    teammateId: string,
    policy: ChatTeammatePolicyInput,
    draft: ChatTeammateStudioDraft,
    includeProfile: boolean,
  ): ReplaceChatTeammateAccessRequest {
    return {
      teammateId,
      expectedAccessRevision: accessRevision,
      teammateDefaultRuntimeApproval: draft.teammateDefaultRuntimeApproval,
      channels: copyChannelAccessInputs(draft.channels),
      teammateProfile: includeProfile ? {
        teammateId,
        displayName: draft.profile.displayName.trim(),
        avatar: copyVersionedJson(profileAvatar),
        role: draft.profile.role.trim(),
        instructions: draft.profile.instructions.trim(),
        expectedRevision: profileExpectedRevision,
      } : null,
      policy: includeProfile ? policy : null,
    };
  }

  function policyForDraft(draft: ChatTeammateStudioDraft): ChatTeammatePolicyInput {
    return {
      providerInstanceId: draft.profile.providerId,
      safetyMode: draft.profile.safetyMode,
      providerManagedModel: draft.profile.providerManagedModel,
      modelId: draft.profile.modelId || null,
      modelOptions: copyModelOptionSelections(draft.profile.modelOptions),
      effort: draft.profile.effort,
      speed: draft.profile.speed,
      providerOptions: copyVersionedJson(draft.profile.providerOptions),
    };
  }

  function accessPreviewNeedsConfirmation(preview: ChatTeammateAccessPreviewRead): boolean {
    return preview.issues.length > 0
      || preview.isExpansion
      || preview.addedChannelIds.length > 0
      || preview.removedChannelIds.length > 0;
  }

  async function save(confirmAccess = false): Promise<void> {
    if (saving || !canSave) return;
    if (confirmAccess && (
      !accessPreview
      || accessPreview.issues.length > 0
      || accessPreviewSnapshot !== currentDraftSnapshot
    )) {
      accessPreview = null;
      accessPreviewSnapshot = null;
      return;
    }
    const creatingAtStart = creating;
    let requestDraft = captureStudioDraft();
    let includeProfile = !creatingAtStart && profileDirty;
    let includesAccessChange = !creatingAtStart && accessDirty;
    let savedDraft: ChatTeammateStudioDraft | null = null;
    let teammateId = selected?.participant.id ?? null;
    saving = true;
    error = null;
    errorField = null;
    conflictRecoveryNotice = null;
    try {
      if (creatingAtStart) {
        const creationPolicy = policyForDraft(requestDraft);
        const created = await chatApi.createChatTeammate({
          teammateId: `participant:${crypto.randomUUID()}`,
          displayName: requestDraft.profile.displayName.trim(),
          avatar: { schemaVersion: 1, value: { kind: "initials" } },
          role: requestDraft.profile.role.trim(),
          instructions: requestDraft.profile.instructions.trim(),
          policy: creationPolicy,
        });
        teammateId = created.participant.id;
        creating = false;
        selectedId = teammateId;
        profileExpectedRevision = created.participant.revision;
        profileAvatar = copyVersionedJson(created.participant.avatar);
        const inertAccess = await chatApi.readChatTeammateAccess(teammateId);
        const durableInertDraft = draftFromReads(created, inertAccess);
        requestDraft = captureStudioDraft();
        accessRevision = inertAccess.accessRevision;
        setStudioDraftBaseline(durableInertDraft);
        includeProfile = profileSnapshotForDraft(requestDraft)
          !== profileSnapshotForDraft(durableInertDraft);
        includesAccessChange = accessSnapshotForDraft(requestDraft)
          !== accessSnapshotForDraft(durableInertDraft);
        preserveStudioDraftForId = teammateId;
        await chat.refreshTeammates();
        if (!includeProfile && !includesAccessChange) savedDraft = durableInertDraft;
      }

      if (teammateId && (includeProfile || includesAccessChange)) {
        const policy = policyForDraft(requestDraft);
        const request = accessReplacementRequest(teammateId, policy, requestDraft, includeProfile);
        const requestSnapshot = draftSnapshotForDraft(requestDraft);
        if (!creatingAtStart && !confirmAccess && includesAccessChange) {
          accessPreview = await chatApi.previewChatTeammateAccess(request);
          accessPreviewSnapshot = requestSnapshot;
          if (accessPreviewNeedsConfirmation(accessPreview)) {
            return;
          }
        }
        const replaced = await chatApi.replaceChatTeammateAccess(request);
        accessRevision = replaced.accessRevision;
        savedDraft = requestDraft;
        accessPreview = null;
        accessPreviewSnapshot = null;
      }
      if (!teammateId || !savedDraft) return;
      preserveStudioDraftForId = teammateId;
      await chat.refreshTeammates();
      const refreshed = chat.teammates.find((teammate) => teammate.participant.id === teammateId);
      if (refreshed) {
        profileExpectedRevision = refreshed.participant.revision;
        profileAvatar = copyVersionedJson(refreshed.participant.avatar);
      }
      setStudioDraftBaseline(savedDraft);
    } catch (cause: unknown) {
      if (teammateId && chatErrorCode(cause) === "stale_revision") {
        error = null;
        errorField = null;
        await loadAccessConflict(teammateId, captureStudioDraft());
      } else {
        error = chatErrorMessage(cause, t("settings.chat.teammates.saveFailed"));
        errorField = chatErrorField(cause);
      }
    } finally {
      saving = false;
    }
  }

  function requestLifecycle(action: LifecycleAction): void {
    if (!selected || lifecycleBusy) return;
    if (action === "archive" && selected.activeAssignmentCount > 0) {
      lifecycleError = t("settings.chat.teammates.archiveBlocked", selected.activeAssignmentCount);
      return;
    }
    lifecycleAction = action;
    lifecycleTarget = selected;
  }

  async function confirmLifecycle(): Promise<void> {
    const action = lifecycleAction;
    const target = lifecycleTarget;
    lifecycleAction = null;
    lifecycleTarget = null;
    if (!action || !target) return;
    lifecycleBusy = true;
    try {
      if (action === "archive") {
        await chatApi.archiveChatTeammate(target.participant.id, target.participant.revision, true);
      } else {
        await chatApi.deleteUnusedChatTeammate(target.participant.id, target.participant.revision);
      }
      selectedId = null;
      await chat.refreshTeammates();
      await loadDirectoryData();
    } catch (cause: unknown) {
      lifecycleError = chatErrorMessage(cause, t("settings.chat.teammates.lifecycleFailed"));
    } finally {
      lifecycleBusy = false;
    }
  }

  async function restoreSelected(): Promise<void> {
    if (!selected || lifecycleBusy) return;
    lifecycleBusy = true;
    try {
      const restored = await chatApi.archiveChatTeammate(
        selected.participant.id,
        selected.participant.revision,
        false,
      );
      await chat.refreshTeammates();
      await loadDirectoryData();
      showArchived = false;
      selectedId = restored.participant.id;
    } catch (cause: unknown) {
      lifecycleError = chatErrorMessage(cause, t("settings.chat.teammates.restoreFailed"));
    } finally {
      lifecycleBusy = false;
    }
  }

  function clearFieldError(field: string): void {
    if (errorField !== field) return;
    error = null;
    errorField = null;
  }
</script>

{#snippet channelDetails(channelAccess: ChatTeammateChannelAccessInput)}
  {@const selectedProfile = accessProfiles.find((profile) => profile.id === channelAccess.accessProfileId) ?? null}
  {@const preset = channelCapabilityPreset(channelAccess.capabilities)}
  <div class="access-details">
    <div class="membership-controls">
      <div class="field">
        <span>{t("settings.chat.teammates.channelBehavior")}</span>
        {#if ChatChannelScopeControls}
          {@const ScopeControls = ChatChannelScopeControls}
          <ScopeControls channels={[channelAccess]} disabled={archivedMode} onPresetChange={(nextPreset) => setPresetForChannels([channelAccess.channelId], nextPreset)} />
        {/if}
      </div>
      <div class="field work-access-field">
        <span>{t("settings.chat.teammates.workAccess")}</span>
        <ChatControlMenu
          value={channelAccess.accessProfileId}
          options={accessProfileOptions}
          ariaLabel={t("settings.chat.teammates.workAccess")}
          onChange={(profileId) => setAccessProfileForChannels([channelAccess.channelId], profileId)}
          disabled={archivedMode}
          showTooltip={false}
        />
      </div>
    </div>
    {#if channelAccess.capabilities.readHistory}
      <div class="field history-field"><span>{t("settings.chat.teammates.history")}</span><CustomSelect inline class="w-full" value={channelAccess.historyBoundary.kind} options={historyOptions} ariaLabel={t("settings.chat.teammates.history")} disabled={archivedMode} onChange={(value) => updateAccessChannel(channelAccess.channelId, (channel) => ({ ...channel, historyBoundary: value === "fromGrant" ? { kind: "fromGrant" } : { kind: "entire" } }))} /></div>
    {/if}
    {#if preset === "custom"}
      <fieldset class="capability-switches" disabled={archivedMode}>
        <legend>{t("settings.chat.teammates.channelCapabilities")}</legend>
        <div><SettingsCheckbox checked={channelAccess.capabilities.readHistory} label={t("settings.chat.teammates.readHistory")} disabled={archivedMode} onChange={(checked) => updateAccessChannel(channelAccess.channelId, (channel) => ({ ...channel, capabilities: { ...channel.capabilities, readHistory: checked }, historyBoundary: { kind: "entire" } }))} /><span><strong>{t("settings.chat.teammates.readHistory")}</strong><small>{t("settings.chat.teammates.readHistoryDescription")}</small></span></div>
        <div><SettingsCheckbox checked={channelAccess.capabilities.participate} label={t("settings.chat.teammates.participate")} disabled={archivedMode} onChange={(checked) => updateAccessChannel(channelAccess.channelId, (channel) => ({ ...channel, capabilities: { ...channel.capabilities, participate: checked } }))} /><span><strong>{t("settings.chat.teammates.participate")}</strong><small>{t("settings.chat.teammates.participateDescription")}</small></span></div>
      </fieldset>
    {/if}
    {#if selectedProfile?.latestRevision.maximumFolderCapability !== "none"}
      <fieldset class="folder-access" disabled={archivedMode}>
        <legend>{t("settings.chat.teammates.foldersForChannel")}</legend>
        {#each foldersForChannel(channelAccess.channelId) as folderRead (folderRead.workingFolder.id)}
          {@const grant = channelAccess.folderGrants.find((entry) => entry.workingFolderId === folderRead.workingFolder.id)}
          <div class="folder-row">
            <SettingsCheckbox checked={Boolean(grant)} label={t("settings.chat.teammates.allowFolder", folderRead.workingFolder.displayName)} disabled={archivedMode} onChange={(checked) => toggleFolderGrant(channelAccess.channelId, folderRead.workingFolder.id, checked)} />
            <Folder size={14} />
            <span class="folder-name"><strong>{folderRead.workingFolder.displayName}</strong><small data-status={folderRead.bindingStatus}>{t(`settings.chat.teammates.binding.${folderRead.bindingStatus}`)}</small></span>
            <div class="folder-controls">
              {#if grant}
                <CustomSelect inline class="folder-select" value={grant.capability} options={folderCapabilityOptions(selectedProfile?.latestRevision.maximumFolderCapability ?? "none")} ariaLabel={t("settings.chat.teammates.folderCapabilityFor", folderRead.workingFolder.displayName)} onChange={(value) => setFolderCapability(channelAccess.channelId, folderRead.workingFolder.id, value as ChatFolderCapability)} />
                {#if advancedChannelIds.has(channelAccess.channelId)}<CustomSelect inline class="approval-select" value={grant.runtimeApprovalOverride ?? "inherit"} options={folderRuntimeOptions} ariaLabel={t("settings.chat.teammates.folderRuntimeApprovalFor", folderRead.workingFolder.displayName)} onChange={(value) => setFolderRuntimeApproval(channelAccess.channelId, folderRead.workingFolder.id, value === "inherit" ? null : value as ChatRuntimeApprovalPolicy)} />{/if}
                <label class="default-target"><input type="radio" name={`default-${channelAccess.channelId}`} checked={grant.isDefault} onchange={() => setDefaultFolder(channelAccess.channelId, folderRead.workingFolder.id)} />{t("settings.chat.teammates.defaultTarget")}</label>
              {/if}
              {#if folderRead.bindingStatus !== "available"}<button type="button" class="link-button" onclick={() => void recoverFolder(folderRead.workingFolder.id, folderRead.bindingStatus, folderRead.workingFolder.kind === "managed")}>{folderRead.workingFolder.kind === "managed" && folderRead.bindingStatus === "missing" ? t("settings.chat.teammates.recreate") : folderRead.bindingStatus === "repository_mismatch" ? t("settings.chat.teammates.relink") : t("settings.chat.teammates.locate")}</button>{/if}
            </div>
          </div>
        {/each}
      </fieldset>
    {/if}
    {#if channelAccess.folderGrants.length > 0}
      <button type="button" class="disclosure-button" aria-expanded={advancedChannelIds.has(channelAccess.channelId)} onclick={() => { const next = new Set(advancedChannelIds); if (next.has(channelAccess.channelId)) next.delete(channelAccess.channelId); else next.add(channelAccess.channelId); advancedChannelIds = next; }}><ChevronRight size={13} class={advancedChannelIds.has(channelAccess.channelId) ? "expanded" : undefined} />{advancedChannelIds.has(channelAccess.channelId) ? t("settings.chat.teammates.hideAdvancedAccess") : t("settings.chat.teammates.advancedAccess")}</button>
    {/if}
    {#if channelAccess.folderGrants.length > 0 && advancedChannelIds.has(channelAccess.channelId)}
      <div class="field-grid compact advanced-fields">
        <div class="field"><span>{t("settings.chat.teammates.channelRuntimeApproval")}</span><CustomSelect inline class="w-full" value={channelAccess.runtimeApprovalOverride ?? "inherit"} options={channelRuntimeOptions} ariaLabel={t("settings.chat.teammates.channelRuntimeApproval")} disabled={archivedMode} onChange={(value) => updateAccessChannel(channelAccess.channelId, (channel) => ({ ...channel, runtimeApprovalOverride: value === "inherit" ? null : value as ChatRuntimeApprovalPolicy }))} /></div>
      </div>
    {/if}
  </div>
{/snippet}

<svelte:window onkeydown={handleKeydown} />

<section class="teammate-settings" data-chat-settings-subsection="teammates">
  <header class="directory-header">
    <div><h2>{t("settings.chat.teammates.heading")}</h2><p>{t("settings.chat.teammates.description")}</p></div>
    <div class="header-actions">
      <button type="button" class="settings-button" disabled={creating || dirty} onclick={(event) => beginCreate(null, event.currentTarget)}><Plus size={13} />{t("settings.chat.teammates.add")}</button>
      {#if archivedTeammates.length > 0}
        {@const archiveFilterLabel = showArchived ? t("settings.chat.teammates.hideArchived") : t("settings.chat.teammates.includeArchived")}
        <button type="button" class="archive-filter" aria-label={archiveFilterLabel} aria-pressed={showArchived} data-app-tooltip={archiveFilterLabel} disabled={creating || dirty || lifecycleBusy} onclick={() => { showArchived = !showArchived; }}><Archive size={14} /><span aria-hidden="true">{#if showArchived}<Eye size={8} />{:else}<EyeOff size={8} />{/if}</span></button>
      {/if}
      <div class="tools-menu-anchor">
        <button bind:this={toolsMenuTrigger} type="button" class="archive-filter" aria-label={t("settings.chat.teammates.accessTools")} aria-haspopup="menu" aria-expanded={toolsMenuOpen} disabled={dirty} onclick={() => { toolsMenuOpen = !toolsMenuOpen; }}><Ellipsis size={15} /></button>
        {#if toolsMenuOpen}
          <div bind:this={toolsMenuElement} role="menu" class="tools-menu" data-app-floating-surface>
            <button type="button" role="menuitem" disabled={profileManagerLoading} onclick={() => void openProfileManager()}>{profileManagerLoading ? t("common.loading") : t("settings.chat.teammates.profileManager.heading")}</button>
          </div>
        {/if}
      </div>
    </div>
  </header>

  <div class="directory-panel">
      {#if allDirectoryTeammates.length > 8}<label class="directory-search"><Search size={13} /><input bind:value={directoryQuery} aria-label={t("settings.chat.teammates.searchDirectory")} placeholder={t("settings.chat.teammates.searchDirectory")} /></label>{/if}
      <div class="scroll-frame">
        <div bind:this={directoryScrollElement} class="directory-scroll hide-scrollbar">
          <nav aria-label={t("settings.chat.teammates.directoryLabel")} class="directory-list teammate-directory">
            {#each filteredDirectoryTeammates as teammate (teammate.participant.id)}
              <button type="button" class="directory-row" disabled={lifecycleBusy} onclick={(event) => openTeammate(teammate.participant.id, event.currentTarget)}>
                <span class="directory-avatar"><ChatParticipantAvatar participant={teammate.participant} {teammate} size={32} /></span>
                <span class="directory-summary">
                  <span class="directory-name-line">
                    <strong>{teammate.participant.displayName}</strong>
                    <span
                      class="health-dot"
                      data-state={teammate.configurationState}
                      data-app-tooltip={teammate.configurationState === "healthy" ? t("settings.chat.teammates.available") : t("settings.chat.teammates.needsSetup")}
                      role="img"
                      aria-label={teammate.configurationState === "healthy" ? t("settings.chat.teammates.available") : t("settings.chat.teammates.needsSetup")}
                    ></span>
                  </span>
                  <small>{teammate.role}</small>
                </span>
              </button>
            {/each}
            {#if loadingDirectory}<p class="empty-copy" role="status">{t("common.loading")}</p>{:else if filteredDirectoryTeammates.length === 0}<p class="empty-copy">{t("settings.chat.teammates.empty")}</p>{/if}
          </nav>
        </div>
        <CalendarScrollbar scrollContainer={directoryScrollElement} wheelPassthrough />
      </div>
  </div>
</section>

{#if creating || selected}
  <div class="editor-backdrop" role="presentation" onclick={(event) => { if (event.target === event.currentTarget) requestCloseEditor(); }}>
    <div bind:this={editorDialogElement} class="teammate-editor" role="dialog" aria-modal="true" aria-labelledby="teammate-editor-title" tabindex="-1">
        <form class="editor" aria-busy={saving || conflictLoading} onsubmit={(event) => { event.preventDefault(); void save(); }}>
          <div class="editor-scroll-frame">
            <div bind:this={detailScrollElement} class="editor-scroll hide-scrollbar">
              <div class="editor-heading">
                {#if creating}
                  {#if draftCompany}<ChatModelAvatar familyId={draftCompany.iconFamilyId} label={draftCompany.name} size={38} />{:else}<span class="draft-avatar large"><Plus size={16} /></span>{/if}
                {:else if selected}<ChatParticipantAvatar participant={selected.participant} teammate={selected} size={38} />{/if}
                <div class="editor-title">
                  <div class="editor-name-line">
                    <h3 id="teammate-editor-title">{displayName.trim() || t("settings.chat.teammates.name")}</h3>
                    {#if creating}
                      <span class="health-dot" data-state={draftConfigurationState} data-app-tooltip={draftConfigurationState === "healthy" ? t("settings.chat.teammates.available") : t("settings.chat.teammates.needsSetup")} role="img" aria-label={draftConfigurationState === "healthy" ? t("settings.chat.teammates.available") : t("settings.chat.teammates.needsSetup")}></span>
                    {:else if selected}
                      <span class="health-dot" data-state={selected.configurationState} data-app-tooltip={selected.configurationState === "healthy" ? t("settings.chat.teammates.available") : t("settings.chat.teammates.needsSetup")} role="img" aria-label={selected.configurationState === "healthy" ? t("settings.chat.teammates.available") : t("settings.chat.teammates.needsSetup")}></span>
                      {#if archivedMode}<span class="archived-state" data-app-tooltip={t("settings.chat.teammates.archived")} role="img" aria-label={t("settings.chat.teammates.archived")}><Archive size={12} /></span>{/if}
                    {/if}
                  </div>
                  <p>{role.trim() || t("settings.chat.teammates.role")}</p>
                </div>
                <button type="button" class="editor-close" aria-label={t("common.close")} onclick={requestCloseEditor}><X size={16} /></button>
              </div>

              {#if conflictLoading}
                <aside bind:this={conflictPanelElement} class="conflict-panel" role="status" tabindex="-1">
                  <div>
                    <strong>{t("settings.chat.teammates.conflict.loadingTitle")}</strong>
                    <p>{t("settings.chat.teammates.conflict.loadingDescription")}</p>
                  </div>
                </aside>
              {:else if accessConflict && conflictComparison}
                <aside bind:this={conflictPanelElement} class="conflict-panel" role="alert" tabindex="-1">
                  <div class="conflict-copy">
                    <strong>{t("settings.chat.teammates.conflict.title")}</strong>
                    <p>{t("settings.chat.teammates.conflict.description")}</p>
                    <ul>
                      <li>{conflictComparison.identityChanged ? t("settings.chat.teammates.conflict.identityChanged") : t("settings.chat.teammates.conflict.identityUnchanged")}</li>
                      <li>{conflictComparison.policyChanged ? t("settings.chat.teammates.conflict.policyChanged") : t("settings.chat.teammates.conflict.policyUnchanged")}</li>
                      <li>{conflictComparison.runtimeApprovalChanged ? t("settings.chat.teammates.conflict.runtimeChanged") : t("settings.chat.teammates.conflict.runtimeUnchanged")}</li>
                      <li>{t("settings.chat.teammates.conflict.channelSummary", conflictComparison.addedChannelCount, conflictComparison.removedChannelCount, conflictComparison.changedChannelCount)}</li>
                    </ul>
                  </div>
                  <div class="conflict-actions">
                    <button type="button" class="primary-button" onclick={rebaseAccessConflict}>{t("settings.chat.teammates.conflict.keepAndRebase")}</button>
                    <button type="button" class="secondary-button" onclick={() => void reloadCurrentAccessConflict()}>{t("settings.chat.teammates.conflict.reloadCurrent")}</button>
                  </div>
                </aside>
              {:else if conflictError}
                <aside bind:this={conflictPanelElement} class="conflict-panel" role="alert" tabindex="-1">
                  <div>
                    <strong>{t("settings.chat.teammates.conflict.loadFailedTitle")}</strong>
                    <p>{conflictError}</p>
                  </div>
                  <button type="button" class="secondary-button" onclick={retryAccessConflict}>{t("common.retry")}</button>
                </aside>
              {/if}

              {#if conflictRecoveryNotice}
                <aside bind:this={recoveryNoticeElement} class="conflict-recovery-notice" role="status" tabindex="-1">
                  {conflictRecoveryNotice === "rebased"
                    ? t("settings.chat.teammates.conflict.rebasedNotice")
                    : t("settings.chat.teammates.conflict.reloadedNotice")}
                </aside>
              {/if}

              <div class="editor-content">
                <section class="editor-section"><div class="section-heading"><h4>{t("settings.chat.teammates.identitySection")}</h4></div><div class="field-grid">
                  <div class="field full"><span id="teammate-name-label">{t("settings.chat.teammates.name")}<i class="required-marker" aria-hidden="true">*</i></span><input bind:value={displayName} aria-labelledby="teammate-name-label" aria-describedby={nameTaken || (error && errorField === "displayName") ? "teammate-name-error" : undefined} aria-invalid={nameTaken || (error && errorField === "displayName") ? "true" : undefined} placeholder={t("settings.chat.teammates.namePlaceholder")} maxlength="160" required disabled={archivedMode} oninput={() => clearFieldError("displayName")} />{#if nameTaken}<small id="teammate-name-error" class="field-error" role="alert">{t("settings.chat.teammates.nameTaken")}</small>{:else if error && errorField === "displayName"}<small id="teammate-name-error" class="field-error" role="alert">{error}</small>{/if}</div>
                  <div class="field full"><span id="teammate-role-label">{t("settings.chat.teammates.role")}<i class="required-marker" aria-hidden="true">*</i></span><input bind:value={role} aria-labelledby="teammate-role-label" aria-describedby={error && errorField === "role" ? "teammate-role-error" : undefined} aria-invalid={error && errorField === "role" ? "true" : undefined} placeholder={t("settings.chat.teammates.rolePlaceholder")} maxlength="1000" required disabled={archivedMode} oninput={() => clearFieldError("role")} />{#if error && errorField === "role"}<small id="teammate-role-error" class="field-error" role="alert">{error}</small>{/if}</div>
                  <div class="field full"><span id="teammate-instructions-label">{t("settings.chat.teammates.instructions")}</span><textarea bind:value={instructions} aria-labelledby="teammate-instructions-label" rows="4" maxlength="65536" disabled={archivedMode} placeholder={t("settings.chat.teammates.instructionsPlaceholder")}></textarea></div>
                </div></section>

                <section class="editor-section"><div class="section-heading"><h4>{t("settings.chat.teammates.executionSection")}</h4></div><div class="field-grid execution-fields">
                  <div class="field execution-model-field"><span>{t("settings.chat.teammates.model")}<i class="required-marker" aria-hidden="true">*</i></span><ChatModelControls value={{ providerInstanceId: providerId || null, modelId: modelId || null, providerManaged: providerManagedModel, options: modelOptions }} disabled={archivedMode} onChange={selectExecution} /></div>
                  <div class="field execution-approval-field"><span>{t("settings.chat.teammates.approval")}</span><ChatAccessControl value={safetyMode} providerInstanceId={providerId || null} workingFolderId={permissionWorkingFolderId} disabled={archivedMode} onChange={(value) => { safetyMode = value; }} /></div>
                </div></section>

                <section class="editor-section access-section">
                  <div class="section-heading"><h4>{t("settings.chat.teammates.accessSection")}</h4></div>
                  {#if ChatChannelAccessBrowser && ChatChannelScopeControls}
                    {@const AccessBrowser = ChatChannelAccessBrowser}
                    <AccessBrowser
                      channels={activeNavigationChannels}
                      {selectedChannelIds}
                      focusedChannelId={focusedAccessChannelId}
                      disabled={archivedMode || accessProfiles.length === 0}
                      onSelectionChange={handleChannelSelection}
                      onFocusChange={(channelId) => { focusedAccessChannelId = channelId; }}
                    />
                    {#if loadingAccess}
                      <p class="empty-copy" role="status">{t("common.loading")}</p>
                    {:else if focusedAccessChannel}
                      <div class="focused-membership">
                        <div class="focused-membership-heading">
                          <strong>{focusedAccessChannel.name}</strong>
                          <small>{channelAncestry(focusedAccessChannel)}</small>
                        </div>
                        {#if focusedAccessDraft}
                          {@render channelDetails(focusedAccessDraft)}
                          {#if providerResourceIssuesForChannel(focusedAccessDraft)[0]}
                            <p class="resource-summary-error" role="alert">{providerResourceIssuesForChannel(focusedAccessDraft)[0]}</p>
                          {/if}
                        {:else}
                          <p class="empty-access">{t("settings.chat.teammates.selectChannelAccess")}</p>
                        {/if}
                      </div>
                    {:else}
                      <p class="empty-access">{t("settings.chat.teammates.noChannels")}</p>
                    {/if}
                  {:else}
                    <p class="empty-copy" role="status">{t("common.loading")}</p>
                  {/if}

                  {#if accessPreview}
                    <aside class="impact-preview" role={accessPreview.issues.length ? "alert" : "status"}><div><strong>{accessPreview.issues.length ? t("settings.chat.teammates.accessIssuesTitle") : accessPreview.isExpansion ? t("settings.chat.teammates.expansionTitle") : t("settings.chat.teammates.reductionTitle")}</strong><p>{t("settings.chat.teammates.impactSummary", accessPreview.addedChannelIds.length, accessPreview.removedChannelIds.length)}</p>{#each accessPreview.issues as issue}<small>{issue.message}</small>{/each}</div>{#if accessPreview.issues.length === 0}<button type="button" class="primary-button" disabled={saving || !canSave || accessPreviewSnapshot !== currentDraftSnapshot} onclick={() => void save(true)}>{t("settings.chat.teammates.applyAccessChanges")}</button>{/if}</aside>
                  {/if}
                </section>
              </div>
            </div>
            <CalendarScrollbar scrollContainer={detailScrollElement} wheelPassthrough />
          </div>

          <footer class="editor-footer"><div>{#if creating}<button type="button" class="secondary-button" onclick={requestCloseEditor}><X size={14} />{t("common.cancel")}</button>{:else if selected && archivedMode}<button type="button" class="danger-button" disabled={selected.hasDurableHistory} onclick={() => requestLifecycle("delete")}><Trash2 size={14} />{t("settings.chat.teammates.deletePermanently")}</button>{:else if selected}<button type="button" class="secondary-button" disabled={selected.activeAssignmentCount > 0} onclick={() => requestLifecycle("archive")}><Archive size={14} />{t("settings.chat.teammates.archive")}</button>{/if}{#if lifecycleError}<span class="field-error" role="alert">{lifecycleError}</span>{/if}</div><div class="save-area">{#if error && errorField !== "displayName" && errorField !== "role"}<span class="field-error" role="alert">{error}</span>{/if}{#if archivedMode}<button type="button" class="primary-button" onclick={() => void restoreSelected()}><ArchiveRestore size={14} />{t("settings.chat.teammates.restore")}</button>{:else}<button type="submit" class="primary-button" disabled={saving || !canSave}>{saving ? t("settings.chat.teammates.saving") : creating ? t("settings.chat.teammates.createInert") : t("settings.chat.teammates.save")}</button>{/if}</div></footer>
        </form>
    </div>
  </div>
{/if}

{#if lifecycleAction && lifecycleTarget}
  <ConfirmDialog
    title={lifecycleAction === "archive" ? t("settings.chat.teammates.archiveTitle", lifecycleTarget.participant.displayName) : t("settings.chat.teammates.deleteTitle", lifecycleTarget.participant.displayName)}
    message={lifecycleAction === "archive" ? t("settings.chat.teammates.archiveMessage", lifecycleTarget.participant.displayName) : t("settings.chat.teammates.deleteMessage", lifecycleTarget.participant.displayName)}
    confirmLabel={lifecycleAction === "archive" ? t("settings.chat.teammates.archiveConfirm") : t("settings.chat.teammates.deletePermanently")}
    cancelLabel={t("common.cancel")}
    danger={lifecycleAction === "delete"}
    onConfirm={() => void confirmLifecycle()}
    onCancel={() => { lifecycleAction = null; lifecycleTarget = null; }}
  />
{/if}

{#if closeConfirmationOpen}
  <ConfirmDialog
    title={t("settings.chat.teammates.discardDraftTitle")}
    message={t("settings.chat.teammates.discardDraftMessage")}
    confirmLabel={t("settings.chat.teammates.discardDraft")}
    cancelLabel={t("settings.chat.teammates.keepEditing")}
    onConfirm={() => { closeConfirmationOpen = false; closeEditor(); }}
    onCancel={() => { closeConfirmationOpen = false; }}
  />
{/if}

{#if profileManagerOpen && ChatAccessProfilesManager}
  {@const Manager = ChatAccessProfilesManager}
  <Manager
    profiles={accessProfiles}
    onProfilesChange={handleProfilesChange}
    onClose={() => { profileManagerOpen = false; }}
  />
{/if}


<style>
  .header-actions,.editor-footer,.save-area { display:flex; align-items:center; }
  .header-actions,.save-area { gap:0.45rem; }
  .primary-button,.secondary-button,.danger-button { display:inline-flex; min-height:2rem; align-items:center; justify-content:center; gap:0.38rem; border-radius:0.42rem; padding:0.35rem 0.7rem; font-size:calc(0.733333rem * var(--type-scale)); font-weight:600; }
  .primary-button { background:var(--primary); color:var(--primary-foreground); }
  .secondary-button { border:1px solid var(--border); background:var(--background); color:var(--foreground); }
  .danger-button { background:var(--destructive); color:var(--destructive-foreground); }
  button:hover:not(:disabled) { filter:brightness(0.96); }
  button:disabled { cursor:not-allowed; opacity:0.5; }
  .scroll-frame,.editor-scroll-frame { position:relative; min-height:0; }
  .directory-scroll,.editor-scroll { height:100%; overflow-y:auto; overscroll-behavior:contain; }
  .directory-list { display:grid; align-content:start; gap:0.15rem; padding-right:0.35rem; }
  .directory-row { display:grid; min-width:0; grid-template-columns:auto minmax(0,1fr) auto; align-items:center; gap:0.55rem; border-radius:0.5rem; padding:0.55rem; text-align:left; }
  .directory-row:hover { background:var(--accent); color:var(--accent-foreground); }
  .directory-row > span:not(.channel-count) { display:grid; min-width:0; }
  .directory-row strong,.directory-row small { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .directory-row strong { font-size:calc(0.766667rem * var(--type-scale)); }
  .directory-row small { color:var(--muted-foreground); font-size:calc(0.65rem * var(--type-scale)); }
  .draft-avatar { display:grid; width:1.9rem; height:1.9rem; place-items:center; border:1px dashed var(--border); border-radius:0.45rem; color:var(--muted-foreground); }
  .draft-avatar.large { width:2.4rem; height:2.4rem; }
  .empty-copy { padding:0.7rem; color:var(--muted-foreground); font-size:calc(0.7rem * var(--type-scale)); }
  .editor { display:grid; height:100%; min-height:0; grid-template-rows:minmax(0,1fr) auto; }
  .editor-heading { display:grid; min-width:0; grid-template-columns:auto minmax(0,1fr) auto; align-items:center; gap:0.65rem; padding-inline:0.25rem; }
  .editor-title { min-width:0; }
  .editor-name-line { display:flex; min-width:0; align-items:center; gap:0.38rem; }
  .editor-heading h3 { overflow:hidden; font-size:calc(0.833333rem * var(--type-scale)); font-weight:600; text-overflow:ellipsis; white-space:nowrap; }
  .editor-heading p { overflow:hidden; margin-top:0.08rem; color:var(--muted-foreground); font-size:calc(0.7rem * var(--type-scale)); text-overflow:ellipsis; white-space:nowrap; }
  .health-dot { width:0.4rem; height:0.4rem; flex:0 0 auto; border-radius:50%; background:var(--status-tentative); }
  .health-dot[data-state="healthy"] { background:var(--action-confirm); }
  .archived-state { display:grid; width:0.9rem; height:0.9rem; flex:0 0 auto; place-items:center; color:var(--muted-foreground); }
  .editor-close { display:grid; width:2rem; height:2rem; place-items:center; border-radius:0.42rem; color:var(--muted-foreground); }
  .editor-close:hover { background:var(--accent); color:var(--foreground); }
  .editor-content { display:grid; align-content:start; gap:1rem; }
  .section-heading h4 { font-size:calc(0.8rem * var(--type-scale)); font-weight:600; }
  .field-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:0.75rem; }
  .field-grid.compact { grid-template-columns:repeat(2,minmax(0,1fr)); }
  .field { display:grid; min-width:0; align-content:start; gap:0.3rem; color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); font-weight:550; }
  .execution-fields { gap:0.7rem; padding-inline:0.25rem; }
  .required-marker { margin-left:0.15rem; color:var(--destructive); font-style:normal; }
  .execution-model-field :global(.model-control) { z-index:2; max-width:100%; justify-self:start; }
  .execution-model-field :global(.model-trigger) { min-width:12rem; }
  .execution-approval-field :global(.access-control) { justify-self:start; }
  .execution-approval-field :global(.control-trigger) { min-width:12rem; max-width:100%; justify-content:center; }
  .field.full { grid-column:1/-1; }
  .field input,.field textarea { box-sizing:border-box; width:100%; min-width:0; appearance:none; border:1px solid var(--border); border-radius:0.375rem; background:var(--background); padding:0.47rem 0.55rem; color:var(--foreground); outline:0; font-weight:400; }
  .field input:focus,.field textarea:focus { border-color:var(--ring); }
  .field textarea { resize:vertical; }
  .field small { color:var(--muted-foreground); font-weight:400; line-height:1rem; }
  .impact-preview { display:flex; align-items:flex-start; gap:0.6rem; }
  .impact-preview strong { font-size:calc(0.733333rem * var(--type-scale)); }
  .impact-preview p,.impact-preview small { display:block; margin-top:0.12rem; color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); line-height:1rem; }
  .conflict-panel { display:grid; grid-template-columns:minmax(0,1fr) auto; align-items:start; gap:0.8rem; margin-bottom:1rem; border:1px solid color-mix(in srgb,var(--destructive) 45%,var(--border)); border-radius:0.6rem; background:color-mix(in srgb,var(--destructive) 7%,var(--background)); padding:0.8rem; outline:0; }
  .conflict-panel:focus-visible,.conflict-recovery-notice:focus-visible { box-shadow:0 0 0 2px color-mix(in srgb,var(--primary) 55%,transparent); }
  .conflict-copy { min-width:0; }
  .conflict-panel strong { font-size:calc(0.75rem * var(--type-scale)); }
  .conflict-panel p,.conflict-panel li { color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); line-height:1rem; }
  .conflict-panel p { margin-top:0.18rem; }
  .conflict-panel ul { display:grid; gap:0.1rem; margin-top:0.45rem; padding-left:1rem; }
  .conflict-actions { display:flex; flex-wrap:wrap; justify-content:flex-end; gap:0.4rem; }
  .conflict-recovery-notice { margin-bottom:1rem; border:1px solid color-mix(in srgb,var(--action-confirm) 45%,var(--border)); border-radius:0.5rem; background:color-mix(in srgb,var(--action-confirm) 8%,transparent); padding:0.6rem 0.7rem; color:var(--foreground); font-size:calc(0.68rem * var(--type-scale)); outline:0; }
  .empty-access { padding-inline:0.25rem; color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); }
  .focused-membership { display:grid; min-width:0; gap:0.65rem; padding:0.05rem 0.25rem 0; }
  .focused-membership-heading { display:grid; min-width:0; }
  .focused-membership-heading strong { overflow:hidden; font-size:calc(0.75rem * var(--type-scale)); font-weight:600; text-overflow:ellipsis; white-space:nowrap; }
  .focused-membership-heading small { overflow:hidden; margin-top:0.05rem; color:var(--muted-foreground); font-size:calc(0.63rem * var(--type-scale)); text-overflow:ellipsis; white-space:nowrap; }
  .access-details { display:grid; gap:0.75rem; padding:0; }
  .membership-controls { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:0.7rem; }
  .membership-controls :global(.scope-controls) { justify-content:flex-start; }
  .membership-controls :global(.control-trigger) { width:100%; max-width:11.5rem; justify-content:flex-start; border:1px solid var(--border); border-radius:0.45rem; background:var(--background); color:var(--foreground); }
  .history-field { width:min(14rem,100%); }
  .resource-summary-error { max-width:34rem; color:var(--destructive); font-size:calc(0.64rem * var(--type-scale)); line-height:0.95rem; }
  fieldset { display:grid; gap:0.4rem; }
  fieldset legend { margin-bottom:0.15rem; font-size:calc(0.7rem * var(--type-scale)); font-weight:650; }
  .capability-switches { grid-template-columns:repeat(2,minmax(0,1fr)); }
  .capability-switches legend { grid-column:1/-1; }
  .capability-switches span { display:grid; }
  .capability-switches strong { font-size:calc(0.68rem * var(--type-scale)); }
  .capability-switches small { margin-top:0.12rem; color:var(--muted-foreground); font-size:calc(0.62rem * var(--type-scale)); line-height:0.9rem; }
  .folder-row { display:grid; min-height:2.65rem; grid-template-columns:auto auto minmax(8rem,1fr) auto; align-items:center; gap:0.5rem; padding:0.35rem 0.2rem; }
  .folder-controls { display:flex; flex-wrap:wrap; align-items:center; justify-content:flex-end; gap:0.4rem; }
  .folder-name { display:grid; min-width:0; }
  .folder-name strong { overflow:hidden; font-size:calc(0.68rem * var(--type-scale)); text-overflow:ellipsis; white-space:nowrap; }
  .folder-name small { color:var(--muted-foreground); font-size:calc(0.6rem * var(--type-scale)); }
  .folder-name small[data-status="available"] { color:var(--action-confirm); }
  .default-target { display:flex; align-items:center; gap:0.25rem; color:var(--muted-foreground); font-size:calc(0.6rem * var(--type-scale)); }
  .link-button { color:var(--primary); font-size:calc(0.64rem * var(--type-scale)); font-weight:600; }
  .impact-preview { justify-content:space-between; }
  .editor-footer { justify-content:space-between; gap:0.75rem; border-top:1px solid var(--border); padding:0.65rem 0.85rem; }
  .editor-footer > div { display:flex; min-width:0; align-items:center; gap:0.5rem; }
  .field-error { color:var(--destructive); font-size:calc(0.65rem * var(--type-scale)); }
  @media (max-width:900px) { .field-grid.compact { grid-template-columns:repeat(2,minmax(0,1fr)); }.folder-row { grid-template-columns:auto auto minmax(7rem,1fr); }.folder-controls { grid-column:3; justify-content:flex-start; } }
  @media (max-width:700px) { .field-grid,.field-grid.compact,.capability-switches,.membership-controls { grid-template-columns:1fr; }.capability-switches legend { grid-column:auto; }.conflict-panel { grid-template-columns:1fr; }.conflict-actions { justify-content:stretch; }.conflict-actions button { flex:1; }.editor-footer { align-items:stretch; flex-direction:column; }.editor-footer > div,.save-area { justify-content:space-between; }.save-area .primary-button { flex:1; } }
  @media (pointer:coarse) { .conflict-actions button,.conflict-panel > button { min-height:44px; } }

  .teammate-settings { display:grid; height:100%; min-height:0; grid-template-rows:auto minmax(0,1fr); gap:0.8rem; }
  .directory-header { display:flex; flex-wrap:wrap; align-items:start; justify-content:space-between; gap:0.75rem; padding-inline:0.25rem; }
  .directory-header h2 { font-size:calc(0.866667rem * var(--type-scale)); font-weight:600; }
  .directory-header p { margin-top:0.25rem; color:var(--muted-foreground); font-size:calc(0.8rem * var(--type-scale)); }
  .header-actions { display:flex; align-items:center; gap:0.35rem; }
  .tools-menu-anchor { position:relative; }
  .tools-menu { position:absolute; z-index:20; top:calc(100% + 0.25rem); right:0; display:grid; min-width:10rem; border:1px solid var(--border); border-radius:0.42rem; background:var(--popover); padding:0.2rem; color:var(--popover-foreground); box-shadow:0 0.35rem 1rem color-mix(in srgb,var(--foreground) 12%,transparent); }
  .tools-menu button { min-height:1.9rem; border-radius:0.32rem; padding:0.35rem 0.55rem; text-align:left; font-size:calc(0.7rem * var(--type-scale)); }
  .tools-menu button:hover:not(:disabled) { background:var(--accent); color:var(--accent-foreground); }
  .settings-button { display:inline-flex; min-height:1.9rem; align-items:center; justify-content:center; gap:0.35rem; border-radius:0.42rem; padding:0.3rem 0.65rem; font-size:calc(0.733333rem * var(--type-scale)); font-weight:600; line-height:1; white-space:nowrap; }
  .settings-button { border:1px solid var(--border); background:var(--background); color:var(--foreground); }
  .archive-filter { position:relative; display:grid; width:1.9rem; height:1.9rem; place-items:center; border-radius:0.42rem; color:var(--muted-foreground); }
  .archive-filter:hover,.archive-filter[aria-pressed="true"] { background:var(--accent); color:var(--foreground); }
  .archive-filter > span { position:absolute; right:0.08rem; bottom:0.08rem; display:grid; width:0.72rem; height:0.72rem; place-items:center; border-radius:50%; background:var(--background); }
  .directory-panel { position:relative; display:grid; min-width:0; min-height:0; grid-template-rows:auto minmax(0,1fr); gap:0.45rem; }
  .directory-search { display:flex; height:1.9rem; align-items:center; gap:0.35rem; border-bottom:1px solid var(--border); padding-inline:0.35rem; color:var(--muted-foreground); }
  .directory-search input { width:100%; min-width:0; background:transparent; color:var(--foreground); outline:0; font-size:calc(0.733333rem * var(--type-scale)); }
  .directory-list { grid-template-columns:repeat(3,minmax(0,1fr)); gap:0.4rem; padding:0.15rem 0.2rem; }
  .directory-row { min-height:3.25rem; grid-template-columns:auto minmax(0,1fr); margin:0; background:color-mix(in srgb,var(--accent) 42%,transparent); padding:0.6rem; }
  .directory-row:hover { background:color-mix(in srgb,var(--accent) 72%,transparent); color:var(--accent-foreground); }
  .directory-avatar { position:relative; display:grid; }
  .directory-summary { display:grid; min-width:0; }
  .directory-name-line { display:flex; min-width:0; align-items:center; gap:0.35rem; }
  .directory-summary strong,.directory-summary small { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .directory-name-line strong { min-width:0; }
  .directory-summary strong { font-size:calc(0.8rem * var(--type-scale)); font-weight:600; }
  .directory-summary small { margin-top:0.05rem; color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); }
  .editor-backdrop { position:fixed; z-index:80; inset:0; display:grid; place-items:center; background:color-mix(in srgb,#000 48%,transparent); padding:1rem; }
  .teammate-editor { width:min(56rem,90vw); height:min(40rem,80dvh); grid-template-rows:minmax(0,1fr) auto; overflow:hidden; border:1px solid var(--border); border-radius:0.75rem; background:var(--card); color:var(--card-foreground); box-shadow:0 24px 70px color-mix(in srgb,#000 35%,transparent); outline:0; }
  .editor-scroll { display:grid; align-content:start; gap:1rem; padding:1rem; }
  .editor-section { display:grid; gap:0.75rem; border:0; padding:0; }
  .impact-preview { display:flex; align-items:flex-start; justify-content:space-between; gap:0.75rem; border:0; border-left:2px solid var(--destructive); border-radius:0; background:transparent; padding:0.15rem 0 0.15rem 0.65rem; }
  .access-details { border:0; }
  .capability-switches { gap:0; }
  .capability-switches > div { display:grid; grid-template-columns:auto minmax(0,1fr); gap:0.5rem; border:0; border-bottom:1px solid color-mix(in srgb,var(--border) 55%,transparent); border-radius:0; padding:0.55rem 0.15rem; }
  .folder-row { grid-template-columns:auto auto minmax(7rem,1fr) auto; }
  .disclosure-button { display:inline-flex; width:max-content; align-items:center; gap:0.3rem; color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); font-weight:600; }
  .disclosure-button:hover { color:var(--foreground); }
  .disclosure-button :global(svg) { transition:transform 120ms ease; }
  .disclosure-button :global(svg.expanded) { transform:rotate(90deg); }
  .advanced-fields { width:min(30rem,100%); }
  .editor-footer { padding:0.65rem 0.8rem; }
  .editor-footer button { white-space:nowrap; }
  .primary-button,.secondary-button,.danger-button { white-space:nowrap; }
  .teammate-editor input[type="radio"] { appearance:none; width:0.9rem; height:0.9rem; border:1px solid var(--border); border-radius:50%; background:var(--background); }
  .teammate-editor input[type="radio"]:checked { border:0.25rem solid var(--primary); }
  @media (max-width:900px) { .directory-list { grid-template-columns:repeat(2,minmax(0,1fr)); } }
  @media (max-width:700px) { .teammate-editor { width:100%; height:94dvh; }.field-grid,.field-grid.compact,.capability-switches { grid-template-columns:1fr; }.folder-row { grid-template-columns:auto auto minmax(7rem,1fr); }.folder-controls { grid-column:3; justify-content:flex-start; } }
  @media (max-width:520px) { .directory-list { grid-template-columns:1fr; } }
</style>
