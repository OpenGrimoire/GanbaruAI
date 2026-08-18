<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import Bot from "@lucide/svelte/icons/bot";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import Folder from "@lucide/svelte/icons/folder";
  import Hash from "@lucide/svelte/icons/hash";
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
  import {
    capabilitiesForPreset,
    channelCapabilityPreset,
    folderCapabilityFits,
    selectionState,
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
  import ChatModelAvatar from "$lib/components/chat/ChatModelAvatar.svelte";
  import ChatModelControls from "$lib/components/chat/ChatModelControls.svelte";
  import ChatParticipantAvatar from "$lib/components/chat/ChatParticipantAvatar.svelte";

  type ChatAccessProfilesManagerComponent = typeof import("./ChatAccessProfilesManager.svelte").default;
  type ChatScratchManagerComponent = typeof import("./ChatScratchManager.svelte").default;

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
  type CompactPickerLevel = "groups" | "projects" | "channels";
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
  let savedNotice = $state(false);
  let directoryScrollElement = $state<HTMLElement>();
  let detailScrollElement = $state<HTMLElement>();

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

  let channelPickerOpen = $state(false);
  let profileManagerOpen = $state(false);
  let scratchManagerOpen = $state(false);
  let profileManagerLoading = $state(false);
  let scratchManagerLoading = $state(false);
  let ChatAccessProfilesManager = $state<ChatAccessProfilesManagerComponent | null>(null);
  let ChatScratchManager = $state<ChatScratchManagerComponent | null>(null);
  let profileManagerLoad: Promise<void> | null = null;
  let scratchManagerLoad: Promise<void> | null = null;
  let channelPickerQuery = $state("");
  let pickerGroupId = $state<string | null>(null);
  let pickerProjectId = $state<string | null>(null);
  let compactPickerLevel = $state<CompactPickerLevel>("groups");
  let expandedChannelIds = $state<Set<string>>(new Set());
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
  const normalizedDisplayName = $derived(displayName.trim().toLocaleLowerCase());
  const nameTaken = $derived(Boolean(normalizedDisplayName && allTeammates.some((teammate) => (
    teammate.participant.id !== selectedId
      && teammate.participant.displayName.trim().toLocaleLowerCase() === normalizedDisplayName
  ))));
  const activeNavigationChannels = $derived(navigationChannels.filter((channel) => channel.archivedAt === null));
  const selectedChannelIds = $derived(new Set(accessDraft.map((channel) => channel.channelId)));
  const profileCeilings = $derived(new Map(accessProfiles.map((profile) => [
    profile.id,
    profile.latestRevision.maximumFolderCapability,
  ])));
  const accessErrors = $derived(teammateAccessDraftErrors(accessDraft, profileCeilings));
  const providerAccessBlockers = $derived(accessDraft.flatMap((channel) => {
    if (!channel.capabilities.participate) return [];
    const grants = channel.folderGrants.filter((grant) => grant.capability !== "none");
    const hasDefaultTarget = grants.some((grant) => grant.isDefault);
    const issues = grants.flatMap((grant) => {
      const issue = providerGrantIssue(grant.capability, grant.isDefault);
      return issue ? [issue] : [];
    });
    if (!hasDefaultTarget) {
      const scratchIssue = providerScratchIssue();
      if (scratchIssue) issues.push(scratchIssue);
    }
    if (grants.some((grant) => !grant.isDefault) && !providerAuthoritySupport?.internalHostTools) {
      issues.push(t("settings.chat.teammates.providerNeedsHostTools"));
    }
    return issues;
  }).filter((issue, index, issues) => issues.indexOf(issue) === index));
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
      && providerAccessBlockers.length === 0
      && dirty
      && !loadingAccess
      && !accessConflict
      && !conflictLoading
      && !conflictError
      && !archivedMode,
  ));

  const pickerGroups = $derived(projects.groups.filter((group) => !group.archivedAt));
  const pickerProjects = $derived(projects.projects.filter((project) => (
    project.status === "active" && (!pickerGroupId || project.groupId === pickerGroupId)
  )));
  const pickerGroupName = $derived(projects.groups.find((group) => group.id === pickerGroupId)?.name ?? null);
  const pickerProjectName = $derived(projects.projects.find((project) => project.id === pickerProjectId)?.name ?? null);
  const visibleCompactPickerLevel = $derived(channelPickerQuery.trim() ? "channels" : compactPickerLevel);
  const pickerChannels = $derived.by(() => {
    const query = channelPickerQuery.trim().toLocaleLowerCase();
    return activeNavigationChannels.filter((channel) => {
      if (!query) {
        if (pickerProjectId && channel.projectId !== pickerProjectId) return false;
        if (!pickerProjectId && pickerGroupId) {
          const project = projects.projects.find((entry) => entry.id === channel.projectId);
          if (project?.groupId !== pickerGroupId) return false;
        }
        return true;
      }
      const project = projects.projects.find((entry) => entry.id === channel.projectId);
      const group = projects.groups.find((entry) => entry.id === project?.groupId);
      return `${group?.name ?? ""} ${project?.name ?? ""} ${channel.name}`
        .toLocaleLowerCase()
        .includes(query);
    });
  });
  const channelPresetOptions = $derived([
    { value: "contextSource", label: t("settings.chat.teammates.presets.contextSource") },
    { value: "isolatedResponder", label: t("settings.chat.teammates.presets.isolatedResponder") },
    { value: "collaborator", label: t("settings.chat.teammates.presets.collaborator") },
    { value: "custom", label: t("settings.chat.teammates.presets.custom") },
  ]);
  const accessProfileOptions = $derived(accessProfiles.map((profile) => ({
    value: profile.id,
    label: profileLabel(profile),
  })));
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
      return;
    }
    if (chat.teammates.length === 0) beginCreate(initialChannelId ?? null);
    else selectedId = chat.teammates[0]?.participant.id ?? null;
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

  onMount(() => {
    void loadDirectoryData();
  });

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

  function beginCreate(channelId: string | null = null): void {
    clearAccessConflict();
    creating = true;
    selectedId = null;
    channelPickerOpen = false;
    expandedChannelIds = channelId ? new Set([channelId]) : new Set();
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
    savedNotice = false;
  }

  function cancelCreate(): void {
    clearAccessConflict();
    creating = false;
    selectedId = chat.teammates[0]?.participant.id ?? null;
    profileBaseline = null;
    accessBaseline = null;
    studioDraftBaseline = null;
    accessPreview = null;
    accessPreviewSnapshot = null;
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
    savedNotice = false;
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
      setStudioDraftBaseline(durableDraft);
      if (initialChannelId) {
        expandedChannelIds = new Set([initialChannelId]);
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
    savedNotice = false;
    void tick().then(() => recoveryNoticeElement?.focus());
  }

  function profileLabel(profile: ChatAccessProfileRead): string {
    if (!profile.builtinKey) return profile.displayName;
    return t(`settings.chat.teammates.profiles.${profile.builtinKey}`);
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

  function providerScratchIssue(): string | null {
    const support = providerAuthoritySupport;
    if (!support) return t("settings.chat.teammates.providerAuthorityUnknown");
    if (!support.writableRoot || !support.confinedCommands || !support.networkBoundary) {
      return t("settings.chat.teammates.providerCannotUseScratch");
    }
    return null;
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
    return activeNavigationChannels.find((channel) => channel.id === channelId) ?? null;
  }

  function ancestry(channelId: string): { group: string; project: string; channel: string } {
    const channel = channelById(channelId);
    const project = projects.projects.find((entry) => entry.id === channel?.projectId);
    const group = projects.groups.find((entry) => entry.id === project?.groupId);
    return {
      group: group?.name ?? t("settings.chat.teammates.unknownGroup"),
      project: project?.name ?? t("settings.chat.teammates.unknownProject"),
      channel: channel?.name ?? t("settings.chat.teammates.unavailableChannel"),
    };
  }

  function updateAccessChannel(
    channelId: string,
    update: (channel: ChatTeammateChannelAccessInput) => ChatTeammateChannelAccessInput,
  ): void {
    accessDraft = accessDraft.map((channel) => channel.channelId === channelId ? update(channel) : channel);
    accessPreview = null;
    accessPreviewSnapshot = null;
  }

  function setChannelSelected(channelId: string, selectedValue: boolean): void {
    if (selectedValue && !selectedChannelIds.has(channelId)) {
      accessDraft = [...accessDraft, defaultChannelAccess(channelId)];
    } else if (!selectedValue) {
      accessDraft = accessDraft.filter((channel) => channel.channelId !== channelId);
    }
    accessPreview = null;
    accessPreviewSnapshot = null;
  }

  function setBoundedSelection(channelIds: string[], selectedValue: boolean): void {
    const next = toggleSelectionGroup(channelIds, selectedChannelIds, selectedValue);
    const byId = new Map(accessDraft.map((channel) => [channel.channelId, channel]));
    accessDraft = [...next].map((channelId) => byId.get(channelId) ?? defaultChannelAccess(channelId));
    accessPreview = null;
    accessPreviewSnapshot = null;
  }

  function openCompactGroup(groupId: string): void {
    pickerGroupId = groupId;
    pickerProjectId = null;
    compactPickerLevel = "projects";
  }

  function openCompactProject(projectId: string, groupId: string): void {
    pickerGroupId = groupId;
    pickerProjectId = projectId;
    compactPickerLevel = "channels";
  }

  function compactPickerBack(): void {
    if (channelPickerQuery.trim()) {
      channelPickerQuery = "";
      return;
    }
    if (compactPickerLevel === "channels") {
      pickerProjectId = null;
      compactPickerLevel = "projects";
      return;
    }
    pickerGroupId = null;
    compactPickerLevel = "groups";
  }

  function setPreset(channelId: string, preset: ChatChannelCapabilityPreset): void {
    if (preset === "custom") return;
    updateAccessChannel(channelId, (channel) => ({
      ...channel,
      capabilities: capabilitiesForPreset(preset),
      historyBoundary: { kind: "entire" },
    }));
  }

  function openChannelPicker(): void {
    channelPickerOpen = true;
    channelPickerQuery = "";
    pickerGroupId = null;
    pickerProjectId = null;
    compactPickerLevel = "groups";
  }

  function leaveAccessSubview(): void {
    channelPickerOpen = false;
    channelPickerQuery = "";
  }

  function folderCapabilityOptions(ceiling: ChatFolderCapability) {
    return (["read", "edit", "execute", "publish"] as const)
      .filter((capability) => folderCapabilityFits(capability, ceiling))
      .map((capability) => ({ value: capability, label: folderCapabilityLabel(capability) }));
  }

  function setAccessProfile(channelId: string, accessProfileId: string): void {
    const profile = accessProfiles.find((entry) => entry.id === accessProfileId);
    updateAccessChannel(channelId, (channel) => ({
      ...channel,
      accessProfileId,
      accessProfileRevision: profile?.latestRevision.revision ?? 0,
      folderGrants: channel.folderGrants.filter((grant) => folderCapabilityFits(
        grant.capability,
        profile?.latestRevision.maximumFolderCapability ?? "none",
      )),
    }));
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

  function teammateDirectoryStatus(teammate: ChatAiTeammateRead): string {
    const health = teammate.configurationState === "healthy"
      ? t("settings.chat.teammates.available")
      : t("settings.chat.teammates.needsSetup");
    return teammate.participant.archivedAt
      ? `${health} · ${t("settings.chat.teammates.archived")}`
      : health;
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

  function loadScratchManager(): Promise<void> {
    if (ChatScratchManager) return Promise.resolve();
    scratchManagerLoad ??= import("./ChatScratchManager.svelte")
      .then((module) => { ChatScratchManager = module.default; })
      .finally(() => { scratchManagerLoad = null; });
    return scratchManagerLoad;
  }

  async function openProfileManager(): Promise<void> {
    if (profileManagerLoading || profileManagerOpen) return;
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

  async function openScratchManager(): Promise<void> {
    if (scratchManagerLoading || scratchManagerOpen) return;
    scratchManagerLoading = true;
    try {
      await loadScratchManager();
      scratchManagerOpen = true;
    } catch (cause: unknown) {
      errorField = null;
      error = chatErrorMessage(cause, t("settings.chat.teammates.loadFailed"));
    } finally {
      scratchManagerLoading = false;
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
    savedNotice = false;
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
        if (!confirmAccess && includesAccessChange) {
          accessPreview = await chatApi.previewChatTeammateAccess(request);
          accessPreviewSnapshot = requestSnapshot;
          if (accessPreviewNeedsConfirmation(accessPreview)) {
            channelPickerOpen = false;
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
      savedNotice = currentDraftSnapshot === draftSnapshotForDraft(savedDraft);
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

<section class="teammate-settings" data-chat-settings-subsection="teammates">
  <header class="directory-header">
    <div><h2>{t("settings.chat.teammates.heading")}</h2><p>{t("settings.chat.teammates.description")}</p></div>
    <div class="header-actions">
      <button type="button" class="settings-button" disabled={creating || dirty} onclick={() => beginCreate(null)}><Plus size={13} />{t("settings.chat.teammates.add")}</button>
      {#if archivedTeammates.length > 0}
        {@const archiveFilterLabel = showArchived ? t("settings.chat.teammates.hideArchived") : t("settings.chat.teammates.includeArchived")}
        <button type="button" class="archive-filter" aria-label={archiveFilterLabel} aria-pressed={showArchived} data-app-tooltip={archiveFilterLabel} disabled={creating || dirty || lifecycleBusy} onclick={() => { showArchived = !showArchived; }}><Archive size={14} /><span aria-hidden="true">{#if showArchived}<Eye size={8} />{:else}<EyeOff size={8} />{/if}</span></button>
      {/if}
    </div>
  </header>

  <div class="directory-layout">
    <aside class="directory-panel">
      {#if allDirectoryTeammates.length > 8}<label class="directory-search"><Search size={13} /><input bind:value={directoryQuery} aria-label={t("settings.chat.teammates.searchDirectory")} placeholder={t("settings.chat.teammates.searchDirectory")} /></label>{/if}
      <div class="scroll-frame">
        <div bind:this={directoryScrollElement} class="directory-scroll hide-scrollbar">
          <nav aria-label={t("settings.chat.teammates.directoryLabel")} class="directory-list teammate-directory">
            {#if creating}
              <button type="button" class="directory-row active" aria-current="page"><span class="draft-avatar">{#if draftCompany}<ChatModelAvatar familyId={draftCompany.iconFamilyId} label={draftCompany.name} size={30} />{:else}<Plus size={15} />{/if}</span><span><strong>{displayName || t("settings.chat.teammates.newTeammate")}</strong><small>{role || t("settings.chat.teammates.inertUntilAdded")}</small></span></button>
            {/if}
            {#each filteredDirectoryTeammates as teammate (teammate.participant.id)}
              <button type="button" class:active={!creating && selectedId === teammate.participant.id} class="directory-row" aria-current={!creating && selectedId === teammate.participant.id ? "page" : undefined} disabled={dirty || lifecycleBusy} onclick={() => { creating = false; selectedId = teammate.participant.id; }}>
                <span class="directory-avatar"><ChatParticipantAvatar participant={teammate.participant} {teammate} size={32} /></span>
                <span class="directory-summary"><strong>{teammate.participant.displayName}</strong><small>{teammate.role}</small></span>
              </button>
            {/each}
            {#if loadingDirectory}<p class="empty-copy" role="status">{t("common.loading")}</p>{:else if !creating && filteredDirectoryTeammates.length === 0}<p class="empty-copy">{t("settings.chat.teammates.empty")}</p>{/if}
          </nav>
        </div>
        <CalendarScrollbar scrollContainer={directoryScrollElement} wheelPassthrough />
      </div>
    </aside>

    <div class="detail-panel">
      {#if creating || selected}
        <form class="editor teammate-editor" aria-busy={saving || conflictLoading} onsubmit={(event) => { event.preventDefault(); void save(); }}>
          <header class="editor-header">
            <div class="identity-heading">
              {#if creating}{#if draftCompany}<ChatModelAvatar familyId={draftCompany.iconFamilyId} label={draftCompany.name} size={38} />{:else}<span class="draft-avatar large"><Plus size={16} /></span>{/if}{:else if selected}<ChatParticipantAvatar participant={selected.participant} teammate={selected} size={38} />{/if}
              <div><h3>{displayName || t("settings.chat.teammates.newTeammate")}</h3><p>{role || t("settings.chat.teammates.inertUntilAdded")}</p></div>
            </div>
          </header>

          <div class="editor-scroll-frame">
            <div bind:this={detailScrollElement} class="editor-scroll hide-scrollbar">
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

              <div class="overview-and-access">
                <div class="overview-content">
                  <section class="editor-section"><div class="section-heading"><h4>{t("settings.chat.teammates.identitySection")}</h4></div><div class="field-grid">
                    <label class="field"><span>{t("settings.chat.teammates.name")}</span><input bind:value={displayName} maxlength="160" disabled={archivedMode} aria-invalid={nameTaken} oninput={() => clearFieldError("displayName")} />{#if nameTaken}<small class="field-error">{t("settings.chat.teammates.nameTaken")}</small>{/if}</label>
                    <label class="field"><span>{t("settings.chat.teammates.role")}</span><input bind:value={role} maxlength="1000" disabled={archivedMode} oninput={() => clearFieldError("role")} /></label>
                    <label class="field full"><span>{t("settings.chat.teammates.instructions")}</span><textarea bind:value={instructions} rows="6" maxlength="65536" disabled={archivedMode} placeholder={t("settings.chat.teammates.instructionsPlaceholder")}></textarea></label>
                  </div></section>
                  <section class="editor-section"><div class="section-heading"><h4>{t("settings.chat.teammates.executionSection")}</h4></div><div class="field-grid execution-fields">
                    <div class="field execution-model-field"><span>{t("settings.chat.teammates.model")}<i class="required-marker" aria-hidden="true">*</i></span><ChatModelControls value={{ providerInstanceId: providerId || null, modelId: modelId || null, providerManaged: providerManagedModel, options: modelOptions }} disabled={archivedMode} onChange={selectExecution} /></div>
                    <div class="field execution-approval-field"><span>{t("settings.chat.teammates.approval")}</span><ChatAccessControl value={safetyMode} providerInstanceId={providerId || null} workingFolderId={permissionWorkingFolderId} disabled={archivedMode} onChange={(value) => { safetyMode = value; }} /></div>
                  </div></section>
                </div>
                <div class="section-divider" aria-hidden="true"></div>
                <div class="access-content">
                  <div class="access-toolbar"><div><h4>{t("settings.chat.teammates.accessSection")}</h4></div><div class="access-toolbar-actions"><button type="button" class="text-button" disabled={dirty || profileManagerLoading} aria-busy={profileManagerLoading} onclick={() => void openProfileManager()}>{profileManagerLoading ? t("common.loading") : t("settings.chat.teammates.profileManager.manage")}</button><button type="button" class="text-button" disabled={dirty || scratchManagerLoading} aria-busy={scratchManagerLoading} onclick={() => void openScratchManager()}>{scratchManagerLoading ? t("common.loading") : t("settings.chat.teammates.scratchManager.manage")}</button><button type="button" class="secondary-button" disabled={archivedMode || accessProfiles.length === 0} aria-expanded={channelPickerOpen} onclick={openChannelPicker}><Plus size={14} />{t("settings.chat.teammates.addChannel")}</button></div></div>
                  {#if providerAccessBlockers.length}<aside class="impact-preview" role="alert"><div><strong>{t("settings.chat.teammates.providerAuthorityBlocked")}</strong>{#each providerAccessBlockers as blocker}<small>{blocker}</small>{/each}</div></aside>{/if}

                  {#if channelPickerOpen}
                    <section class="channel-picker" aria-label={t("settings.chat.teammates.channelPickerLabel")}>
                      <header><label class="search-field"><Search size={14} /><input bind:value={channelPickerQuery} placeholder={t("settings.chat.teammates.searchChannels")} aria-label={t("settings.chat.teammates.searchChannels")} /></label><button type="button" aria-label={t("common.close")} onclick={leaveAccessSubview}><X size={15} /></button></header>
                      <div class="compact-picker">
                        <header>
                          {#if visibleCompactPickerLevel !== "groups"}<button type="button" aria-label={t("settings.chat.teammates.back")} onclick={compactPickerBack}><ArrowLeft size={14} /></button>{/if}
                          <span><strong>{visibleCompactPickerLevel === "groups" ? t("settings.chat.teammates.groups") : visibleCompactPickerLevel === "projects" ? t("settings.chat.teammates.projects") : t("settings.chat.teammates.channels")}</strong>{#if pickerGroupName && !channelPickerQuery.trim()}<small>{pickerGroupName}{#if pickerProjectName} / {pickerProjectName}{/if}</small>{/if}</span>
                        </header>
                        <div class="compact-picker-list">
                          {#if visibleCompactPickerLevel === "groups"}
                            {#each pickerGroups as group (group.id)}
                              {@const groupChannelIds = activeNavigationChannels.filter((channel) => projects.projects.find((project) => project.id === channel.projectId)?.groupId === group.id).map((channel) => channel.id)}
                              {@const state = selectionState(groupChannelIds, selectedChannelIds)}
                              <div class="compact-picker-row"><SettingsCheckbox checked={state === "all"} mixed={state === "some"} label={group.name} onChange={(checked) => setBoundedSelection(groupChannelIds, checked)} /><button type="button" onclick={() => openCompactGroup(group.id)}><span>{group.name}</span><ChevronRight size={14} /></button></div>
                            {/each}
                          {:else if visibleCompactPickerLevel === "projects"}
                            {#each pickerProjects as project (project.id)}
                              {@const projectChannelIds = activeNavigationChannels.filter((channel) => channel.projectId === project.id).map((channel) => channel.id)}
                              {@const state = selectionState(projectChannelIds, selectedChannelIds)}
                              <div class="compact-picker-row"><SettingsCheckbox checked={state === "all"} mixed={state === "some"} label={project.name} onChange={(checked) => setBoundedSelection(projectChannelIds, checked)} /><button type="button" onclick={() => openCompactProject(project.id, project.groupId)}><span>{project.name}</span><ChevronRight size={14} /></button></div>
                            {/each}
                          {:else}
                            {#each pickerChannels as channel (channel.id)}<div class="channel-option"><SettingsCheckbox checked={selectedChannelIds.has(channel.id)} label={t("settings.chat.teammates.channelAccessTitle", channel.name)} onChange={(checked) => setChannelSelected(channel.id, checked)} /><Hash size={13} /><button type="button" onclick={() => setChannelSelected(channel.id, !selectedChannelIds.has(channel.id))}><b>{channel.name}</b><small>{ancestry(channel.id).group} / {ancestry(channel.id).project}</small></button></div>{/each}
                            {#if pickerChannels.length === 0}<p class="empty-copy">{t("settings.chat.teammates.noChannels")}</p>{/if}
                          {/if}
                        </div>
                      </div>
                      <footer><span>{t("settings.chat.teammates.selectedChannels", accessDraft.length)}</span><button type="button" class="primary-button" onclick={leaveAccessSubview}>{t("common.done")}</button></footer>
                    </section>
                  {/if}

                  {#if loadingAccess}<p class="empty-copy" role="status">{t("common.loading")}</p>{:else if accessDraft.length === 0}<section class="empty-access"><Bot size={20} /><h5>{t("settings.chat.teammates.noAccessTitle")}</h5><p>{t("settings.chat.teammates.inertUntilAdded")}</p><button type="button" class="text-button" onclick={openChannelPicker}>{t("settings.chat.teammates.addChannel")}</button></section>{:else}
                    <div class="access-list">
                      {#each accessDraft as channelAccess (channelAccess.channelId)}
                        {@const labels = ancestry(channelAccess.channelId)}
                        {@const expanded = expandedChannelIds.has(channelAccess.channelId)}
                        {@const selectedProfile = accessProfiles.find((profile) => profile.id === channelAccess.accessProfileId) ?? null}
                        {@const preset = channelCapabilityPreset(channelAccess.capabilities)}
                        <article class="access-card">
                          <header>
                            <button type="button" class="access-summary" aria-expanded={expanded} onclick={() => { const next = new Set(expandedChannelIds); if (expanded) next.delete(channelAccess.channelId); else next.add(channelAccess.channelId); expandedChannelIds = next; }}>
                              <ChevronRight size={15} class={expanded ? "expanded" : undefined} /><Hash size={14} />
                              <span><strong>{labels.channel}</strong><small>{labels.group} / {labels.project}</small><small>{t("settings.chat.teammates.channelAccessSummary", t(`settings.chat.teammates.presets.${preset}`), selectedProfile ? profileLabel(selectedProfile) : t("settings.chat.teammates.profileUnavailable"), channelAccess.folderGrants.length ? t("settings.chat.teammates.folderCount", channelAccess.folderGrants.length) : t("settings.chat.teammates.scratchFallback"))}</small></span>
                            </button>
                            <button type="button" class="icon-button danger" disabled={archivedMode} aria-label={t("settings.chat.teammates.removeChannel", labels.channel)} onclick={() => setChannelSelected(channelAccess.channelId, false)}><Trash2 size={14} /></button>
                          </header>
                          {#if expanded}
                            <div class="access-details">
                              <div class="field-grid compact">
                                <div class="field"><span>{t("settings.chat.teammates.channelPreset")}</span><CustomSelect inline class="w-full" value={preset} options={preset === "custom" ? channelPresetOptions : channelPresetOptions.filter((option) => option.value !== "custom")} ariaLabel={t("settings.chat.teammates.channelPreset")} disabled={archivedMode} onChange={(value) => setPreset(channelAccess.channelId, value as ChatChannelCapabilityPreset)} /></div>
                                <div class="field"><span>{t("settings.chat.teammates.accessProfile")}</span><CustomSelect inline class="w-full" value={channelAccess.accessProfileId} options={accessProfileOptions} ariaLabel={t("settings.chat.teammates.accessProfile")} disabled={archivedMode} onChange={(value) => setAccessProfile(channelAccess.channelId, value)} /></div>
                                {#if channelAccess.capabilities.readHistory}<div class="field"><span>{t("settings.chat.teammates.history")}</span><CustomSelect inline class="w-full" value={channelAccess.historyBoundary.kind} options={historyOptions} ariaLabel={t("settings.chat.teammates.history")} disabled={archivedMode} onChange={(value) => updateAccessChannel(channelAccess.channelId, (channel) => ({ ...channel, historyBoundary: value === "fromGrant" ? { kind: "fromGrant" } : { kind: "entire" } }))} /></div>{/if}
                              </div>
                              {#if preset === "custom"}<fieldset class="capability-switches" disabled={archivedMode}><legend>{t("settings.chat.teammates.channelCapabilities")}</legend><div><SettingsCheckbox checked={channelAccess.capabilities.readHistory} label={t("settings.chat.teammates.readHistory")} disabled={archivedMode} onChange={(checked) => updateAccessChannel(channelAccess.channelId, (channel) => ({ ...channel, capabilities: { ...channel.capabilities, readHistory: checked }, historyBoundary: { kind: "entire" } }))} /><span><strong>{t("settings.chat.teammates.readHistory")}</strong><small>{t("settings.chat.teammates.readHistoryDescription")}</small></span></div><div><SettingsCheckbox checked={channelAccess.capabilities.participate} label={t("settings.chat.teammates.participate")} disabled={archivedMode} onChange={(checked) => updateAccessChannel(channelAccess.channelId, (channel) => ({ ...channel, capabilities: { ...channel.capabilities, participate: checked } }))} /><span><strong>{t("settings.chat.teammates.participate")}</strong><small>{t("settings.chat.teammates.participateDescription")}</small></span></div></fieldset>{/if}
                              <fieldset class="folder-access" disabled={archivedMode}>
                                <legend>{t("settings.chat.teammates.foldersForChannel")}</legend>
                                <p>{t("settings.chat.teammates.foldersForChannelDescription")}</p>
                                {#if selectedProfile?.latestRevision.maximumFolderCapability !== "none"}
                                  {#each foldersForChannel(channelAccess.channelId) as folderRead (folderRead.workingFolder.id)}
                                    {@const grant = channelAccess.folderGrants.find((entry) => entry.workingFolderId === folderRead.workingFolder.id)}
                                    <div class="folder-row">
                                      <SettingsCheckbox checked={Boolean(grant)} label={t("settings.chat.teammates.allowFolder", folderRead.workingFolder.displayName)} disabled={archivedMode} onChange={(checked) => toggleFolderGrant(channelAccess.channelId, folderRead.workingFolder.id, checked)} />
                                      <Folder size={14} />
                                      <span class="folder-name"><strong>{folderRead.workingFolder.displayName}</strong><small data-status={folderRead.bindingStatus}>{t(`settings.chat.teammates.binding.${folderRead.bindingStatus}`)}</small></span>
                                      {#if grant}
                                        <CustomSelect inline class="folder-select" value={grant.capability} options={folderCapabilityOptions(selectedProfile?.latestRevision.maximumFolderCapability ?? "none")} ariaLabel={t("settings.chat.teammates.folderCapabilityFor", folderRead.workingFolder.displayName)} onChange={(value) => setFolderCapability(channelAccess.channelId, folderRead.workingFolder.id, value as ChatFolderCapability)} />
                                        {#if advancedChannelIds.has(channelAccess.channelId)}<CustomSelect inline class="approval-select" value={grant.runtimeApprovalOverride ?? "inherit"} options={folderRuntimeOptions} ariaLabel={t("settings.chat.teammates.folderRuntimeApprovalFor", folderRead.workingFolder.displayName)} onChange={(value) => setFolderRuntimeApproval(channelAccess.channelId, folderRead.workingFolder.id, value === "inherit" ? null : value as ChatRuntimeApprovalPolicy)} />{/if}
                                        <label class="default-target"><input type="radio" name={`default-${channelAccess.channelId}`} checked={grant.isDefault} onchange={() => setDefaultFolder(channelAccess.channelId, folderRead.workingFolder.id)} />{t("settings.chat.teammates.defaultTarget")}</label>
                                      {/if}
                                      {#if folderRead.bindingStatus !== "available"}<button type="button" class="link-button" onclick={() => void recoverFolder(folderRead.workingFolder.id, folderRead.bindingStatus, folderRead.workingFolder.kind === "managed")}>{folderRead.workingFolder.kind === "managed" && folderRead.bindingStatus === "missing" ? t("settings.chat.teammates.recreate") : folderRead.bindingStatus === "repository_mismatch" ? t("settings.chat.teammates.relink") : t("settings.chat.teammates.locate")}</button>{/if}
                                    </div>
                                  {/each}
                                {/if}
                                {#if !channelAccess.folderGrants.some((grant) => grant.isDefault)}
                                  <div class="scratch-row">
                                    <Bot size={15} />
                                    <span><strong>{selectedProfile?.latestRevision.maximumFolderCapability === "none" ? t("settings.chat.teammates.privateScratch") : t("settings.chat.teammates.scratchFallback")}</strong><small>{selectedProfile?.latestRevision.maximumFolderCapability === "none" ? t("settings.chat.teammates.privateScratchDescription") : t("settings.chat.teammates.scratchFallbackDescription")}</small></span>
                                    {#if advancedChannelIds.has(channelAccess.channelId)}<div class="scratch-approval"><span>{t("settings.chat.teammates.scratchRuntimeApproval")}</span><CustomSelect inline class="w-full" value={channelAccess.scratchRuntimeApprovalOverride ?? "inherit"} options={folderRuntimeOptions} ariaLabel={t("settings.chat.teammates.scratchRuntimeApproval")} onChange={(value) => updateAccessChannel(channelAccess.channelId, (channel) => ({ ...channel, scratchRuntimeApprovalOverride: value === "inherit" ? null : value as ChatRuntimeApprovalPolicy }))} /></div>{/if}
                                  </div>
                                {/if}
                              </fieldset>
                              <button type="button" class="disclosure-button" aria-expanded={advancedChannelIds.has(channelAccess.channelId)} onclick={() => { const next = new Set(advancedChannelIds); if (next.has(channelAccess.channelId)) next.delete(channelAccess.channelId); else next.add(channelAccess.channelId); advancedChannelIds = next; }}><ChevronRight size={13} class={advancedChannelIds.has(channelAccess.channelId) ? "expanded" : undefined} />{advancedChannelIds.has(channelAccess.channelId) ? t("settings.chat.teammates.hideAdvancedAccess") : t("settings.chat.teammates.advancedAccess")}</button>
                              {#if advancedChannelIds.has(channelAccess.channelId)}<div class="field advanced-runtime"><span>{t("settings.chat.teammates.channelRuntimeApproval")}</span><CustomSelect inline class="w-full" value={channelAccess.runtimeApprovalOverride ?? "inherit"} options={channelRuntimeOptions} ariaLabel={t("settings.chat.teammates.channelRuntimeApproval")} disabled={archivedMode} onChange={(value) => updateAccessChannel(channelAccess.channelId, (channel) => ({ ...channel, runtimeApprovalOverride: value === "inherit" ? null : value as ChatRuntimeApprovalPolicy }))} /></div>{/if}
                            </div>
                          {/if}
                        </article>
                      {/each}
                    </div>
                  {/if}

                  {#if accessPreview}
                    <aside class="impact-preview" role={accessPreview.issues.length ? "alert" : "status"}><div><strong>{accessPreview.issues.length ? t("settings.chat.teammates.accessIssuesTitle") : accessPreview.isExpansion ? t("settings.chat.teammates.expansionTitle") : t("settings.chat.teammates.reductionTitle")}</strong><p>{t("settings.chat.teammates.impactSummary", accessPreview.addedChannelIds.length, accessPreview.removedChannelIds.length)}</p>{#each accessPreview.issues as issue}<small>{issue.message}</small>{/each}</div>{#if accessPreview.issues.length === 0}<button type="button" class="primary-button" disabled={saving || !canSave || accessPreviewSnapshot !== currentDraftSnapshot} onclick={() => void save(true)}>{t("settings.chat.teammates.applyAccessChanges")}</button>{/if}</aside>
                  {/if}
                </div>
              </div>
            </div>
            <CalendarScrollbar scrollContainer={detailScrollElement} wheelPassthrough />
          </div>

          <footer class="editor-footer"><div>{#if creating}<button type="button" class="secondary-button" onclick={cancelCreate}><X size={14} />{t("common.cancel")}</button>{:else if selected && archivedMode}<button type="button" class="danger-button" disabled={selected.hasDurableHistory} onclick={() => requestLifecycle("delete")}><Trash2 size={14} />{t("settings.chat.teammates.deletePermanently")}</button>{:else if selected}<button type="button" class="secondary-button" disabled={selected.activeAssignmentCount > 0} onclick={() => requestLifecycle("archive")}><Archive size={14} />{t("settings.chat.teammates.archive")}</button>{/if}{#if lifecycleError}<span class="field-error" role="alert">{lifecycleError}</span>{/if}</div><div class="save-area">{#if error}<span class="field-error" role="alert">{error}</span>{/if}{#if savedNotice}<span class="saved" role="status">{t("settings.chat.teammates.saved")}</span>{/if}{#if archivedMode}<button type="button" class="primary-button" onclick={() => void restoreSelected()}><ArchiveRestore size={14} />{t("settings.chat.teammates.restore")}</button>{:else}<button type="submit" class="primary-button" disabled={saving || !canSave}>{saving ? t("settings.chat.teammates.saving") : creating ? t("settings.chat.teammates.createInert") : t("settings.chat.teammates.save")}</button>{/if}</div></footer>
        </form>
      {:else}
        <div class="empty-detail"><Bot size={24} /><p>{t("settings.chat.teammates.selectPrompt")}</p></div>
      {/if}
    </div>
  </div>
</section>

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

{#if profileManagerOpen && ChatAccessProfilesManager}
  {@const Manager = ChatAccessProfilesManager}
  <Manager
    profiles={accessProfiles}
    onProfilesChange={handleProfilesChange}
    onClose={() => { profileManagerOpen = false; }}
  />
{/if}

{#if scratchManagerOpen && ChatScratchManager}
  {@const Manager = ChatScratchManager}
  <Manager onClose={() => { scratchManagerOpen = false; }} />
{/if}

<style>
  .header-actions,.editor-header,.identity-heading,.access-toolbar,.access-toolbar-actions,.editor-footer,.save-area { display:flex; align-items:center; }
  .header-actions,.save-area { gap:0.45rem; }
  .primary-button,.secondary-button,.danger-button { display:inline-flex; min-height:2rem; align-items:center; justify-content:center; gap:0.38rem; border-radius:0.42rem; padding:0.35rem 0.7rem; font-size:calc(0.733333rem * var(--type-scale)); font-weight:600; }
  .primary-button { background:var(--primary); color:var(--primary-foreground); }
  .secondary-button { border:1px solid var(--border); background:var(--background); color:var(--foreground); }
  .danger-button { background:var(--destructive); color:var(--destructive-foreground); }
  button:hover:not(:disabled) { filter:brightness(0.96); }
  button:disabled { cursor:not-allowed; opacity:0.5; }
  .search-field { display:flex; min-height:2rem; align-items:center; gap:0.4rem; border:1px solid var(--border); border-radius:0.45rem; background:var(--background); padding-inline:0.55rem; color:var(--muted-foreground); }
  .search-field input { width:100%; min-width:0; border:0; background:transparent; color:var(--foreground); outline:0; font-size:calc(0.733333rem * var(--type-scale)); }
  .scroll-frame,.editor-scroll-frame { position:relative; min-height:0; }
  .directory-scroll,.editor-scroll { height:100%; overflow-y:auto; overscroll-behavior:contain; }
  .directory-list { display:grid; align-content:start; gap:0.15rem; padding-right:0.35rem; }
  .directory-row { display:grid; min-width:0; grid-template-columns:auto minmax(0,1fr) auto; align-items:center; gap:0.55rem; border-radius:0.5rem; padding:0.55rem; text-align:left; }
  .directory-row:hover,.directory-row.active { background:var(--accent); color:var(--accent-foreground); }
  .directory-row > span:not(.channel-count) { display:grid; min-width:0; }
  .directory-row strong,.directory-row small { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .directory-row strong { font-size:calc(0.766667rem * var(--type-scale)); }
  .directory-row small { color:var(--muted-foreground); font-size:calc(0.65rem * var(--type-scale)); }
  .draft-avatar { display:grid; width:1.9rem; height:1.9rem; place-items:center; border:1px dashed var(--border); border-radius:0.45rem; color:var(--muted-foreground); }
  .draft-avatar.large { width:2.4rem; height:2.4rem; }
  .empty-copy { padding:0.7rem; color:var(--muted-foreground); font-size:calc(0.7rem * var(--type-scale)); }
  .detail-panel { min-width:0; min-height:0; }
  .editor { display:grid; height:100%; min-height:0; grid-template-rows:auto minmax(0,1fr) auto; }
  .editor-header { flex-wrap:wrap; justify-content:space-between; gap:0.7rem; border-bottom:1px solid var(--border); padding:0.75rem 0.9rem 0; }
  .identity-heading { min-width:0; gap:0.6rem; padding-bottom:0.7rem; }
  .identity-heading > div { min-width:0; }
  .identity-heading h3 { overflow:hidden; font-size:calc(0.866667rem * var(--type-scale)); font-weight:650; text-overflow:ellipsis; white-space:nowrap; }
  .identity-heading p { overflow:hidden; margin-top:0.08rem; color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); text-overflow:ellipsis; white-space:nowrap; }
  .overview-content,.access-content { display:grid; align-content:start; gap:1rem; }
  .section-heading h4,.access-toolbar h4 { font-size:calc(0.8rem * var(--type-scale)); font-weight:650; }
  .field-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:0.75rem; }
  .field-grid.compact { grid-template-columns:repeat(4,minmax(0,1fr)); }
  .field { display:grid; min-width:0; align-content:start; gap:0.3rem; color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); font-weight:550; }
  .execution-fields { gap:0.7rem; padding-inline:0.25rem; }
  .required-marker { margin-left:0.15rem; color:var(--destructive); font-style:normal; }
  .execution-model-field :global(.model-control) { z-index:2; max-width:100%; justify-self:start; }
  .execution-model-field :global(.model-trigger) { min-width:12rem; }
  .execution-approval-field :global(.access-control) { justify-self:start; }
  .execution-approval-field :global(.control-trigger) { min-width:12rem; max-width:100%; justify-content:center; }
  .field.full { grid-column:1/-1; }
  .field input,.field textarea { min-width:0; border:1px solid var(--border); border-radius:0.42rem; background:var(--background); padding:0.47rem 0.55rem; color:var(--foreground); outline:0; font-weight:400; }
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
  .access-toolbar { justify-content:space-between; gap:1rem; }
  .access-toolbar-actions { flex-wrap:wrap; justify-content:flex-end; gap:0.4rem; }
  .channel-picker > header,.channel-picker > footer { display:flex; align-items:center; justify-content:space-between; gap:0.6rem; padding:0.55rem; }
  .channel-picker > header .search-field { width:min(24rem,100%); }
  .channel-picker > header > button,.icon-button { display:grid; width:1.8rem; height:1.8rem; place-items:center; border-radius:0.4rem; color:var(--muted-foreground); }
  .channel-option { display:grid; min-height:2.25rem; grid-template-columns:auto auto minmax(0,1fr); align-items:center; gap:0.4rem; border-radius:0.42rem; padding:0.3rem 0.45rem; }
  .channel-option:hover { background:var(--accent); }
  .channel-option > button:last-child { display:grid; min-width:0; text-align:left; }
  .channel-option b { font-size:calc(0.7rem * var(--type-scale)); }
  .channel-option small { overflow:hidden; color:var(--muted-foreground); font-size:calc(0.6rem * var(--type-scale)); text-overflow:ellipsis; white-space:nowrap; }
  .channel-picker > footer { color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); }
  .compact-picker { display:none; }
  .compact-picker > header { display:flex; min-height:2.7rem; align-items:center; gap:0.45rem; border-bottom:1px solid var(--border); padding:0.4rem 0.55rem; }
  .compact-picker > header > button { display:grid; width:2rem; height:2rem; place-items:center; border-radius:0.4rem; }
  .compact-picker > header > span { display:grid; min-width:0; }
  .compact-picker > header strong { font-size:calc(0.7rem * var(--type-scale)); }
  .compact-picker > header small { overflow:hidden; color:var(--muted-foreground); font-size:calc(0.6rem * var(--type-scale)); text-overflow:ellipsis; white-space:nowrap; }
  .compact-picker-list { min-height:12rem; max-height:25rem; padding:0.45rem; overflow-y:auto; }
  .compact-picker-row { display:grid; min-height:2.75rem; grid-template-columns:auto minmax(0,1fr); align-items:center; gap:0.45rem; border-bottom:1px solid color-mix(in srgb,var(--border) 55%,transparent); padding-inline:0.45rem; }
  .compact-picker-row > button { display:flex; min-height:2.75rem; min-width:0; align-items:center; justify-content:space-between; gap:0.45rem; text-align:left; }
  .compact-picker-row > button > span { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .empty-access { display:grid; min-height:13rem; place-items:center; align-content:center; gap:0.5rem; border:1px dashed var(--border); border-radius:0.65rem; color:var(--muted-foreground); text-align:center; }
  .empty-access h5 { color:var(--foreground); font-size:calc(0.78rem * var(--type-scale)); }
  .empty-access p { max-width:27rem; font-size:calc(0.7rem * var(--type-scale)); line-height:1.05rem; }
  .access-list { display:grid; gap:0.55rem; }
  .access-summary { display:grid; min-width:0; grid-template-columns:auto auto minmax(0,1fr); align-items:center; gap:0.4rem; text-align:left; }
  .access-summary > span { display:grid; min-width:0; }
  .access-summary strong { font-size:calc(0.733333rem * var(--type-scale)); }
  .access-summary small { overflow:hidden; color:var(--muted-foreground); font-size:calc(0.62rem * var(--type-scale)); text-overflow:ellipsis; white-space:nowrap; }
  .icon-button.danger:hover { background:color-mix(in srgb,var(--destructive) 12%,transparent); color:var(--destructive); }
  .access-details { display:grid; gap:0.9rem; border-top:1px solid var(--border); padding:0.8rem; }
  fieldset { display:grid; gap:0.4rem; }
  fieldset legend { margin-bottom:0.15rem; font-size:calc(0.7rem * var(--type-scale)); font-weight:650; }
  fieldset > p { color:var(--muted-foreground); font-size:calc(0.65rem * var(--type-scale)); }
  .capability-switches { grid-template-columns:repeat(2,minmax(0,1fr)); }
  .capability-switches legend { grid-column:1/-1; }
  .capability-switches span,.scratch-row span { display:grid; }
  .capability-switches strong,.scratch-row strong { font-size:calc(0.68rem * var(--type-scale)); }
  .capability-switches small,.scratch-row small { margin-top:0.12rem; color:var(--muted-foreground); font-size:calc(0.62rem * var(--type-scale)); line-height:0.9rem; }
  .folder-row { display:grid; min-height:2.65rem; grid-template-columns:auto auto minmax(8rem,1fr) auto auto auto auto; align-items:center; gap:0.5rem; border-bottom:1px solid var(--border); padding:0.35rem 0.2rem; }
  .folder-row:last-child { border-bottom:0; }
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
  .saved { color:var(--action-confirm); font-size:calc(0.65rem * var(--type-scale)); }
  .empty-detail { display:grid; height:100%; place-items:center; align-content:center; gap:0.5rem; color:var(--muted-foreground); font-size:calc(0.73rem * var(--type-scale)); }
  @media (max-width:900px) { .field-grid.compact { grid-template-columns:repeat(2,minmax(0,1fr)); }.folder-row { grid-template-columns:auto auto minmax(7rem,1fr) auto auto; }.folder-row .default-target,.folder-row .link-button { grid-column:3/-1; justify-self:start; }.scratch-row { grid-template-columns:auto minmax(0,1fr); }.scratch-approval { grid-column:2; } }
  @media (max-width:700px) { .directory-panel { border-right:0; border-bottom:1px solid var(--border); }.directory-list { grid-template-columns:repeat(auto-fill,minmax(11rem,1fr)); }.editor-header { align-items:flex-end; }.field-grid,.field-grid.compact,.capability-switches { grid-template-columns:1fr; }.capability-switches legend { grid-column:auto; }.conflict-panel { grid-template-columns:1fr; }.conflict-actions { justify-content:stretch; }.conflict-actions button { flex:1; }.access-card > header { grid-template-columns:minmax(0,1fr) auto; }.editor-footer { align-items:stretch; flex-direction:column; }.editor-footer > div,.save-area { justify-content:space-between; }.save-area .primary-button { flex:1; } }
  @media (pointer:coarse) { .conflict-actions button,.conflict-panel > button { min-height:44px; } }

  .teammate-settings { display:grid; height:100%; min-height:0; grid-template-rows:auto minmax(0,1fr); gap:0.8rem; }
  .directory-header { display:flex; flex-wrap:wrap; align-items:start; justify-content:space-between; gap:0.75rem; padding-inline:0.25rem; }
  .directory-header h2 { font-size:calc(0.866667rem * var(--type-scale)); font-weight:600; }
  .directory-header p { margin-top:0.25rem; color:var(--muted-foreground); font-size:calc(0.8rem * var(--type-scale)); }
  .header-actions { display:flex; align-items:center; gap:0.35rem; }
  .settings-button { display:inline-flex; min-height:1.9rem; align-items:center; justify-content:center; gap:0.35rem; border-radius:0.42rem; padding:0.3rem 0.65rem; font-size:calc(0.733333rem * var(--type-scale)); font-weight:600; line-height:1; white-space:nowrap; }
  .settings-button { border:1px solid var(--border); background:var(--background); color:var(--foreground); }
  .archive-filter { position:relative; display:grid; width:1.9rem; height:1.9rem; place-items:center; border-radius:0.42rem; color:var(--muted-foreground); }
  .archive-filter:hover,.archive-filter[aria-pressed="true"] { background:var(--accent); color:var(--foreground); }
  .archive-filter > span { position:absolute; right:0.08rem; bottom:0.08rem; display:grid; width:0.72rem; height:0.72rem; place-items:center; border-radius:50%; background:var(--background); }
  .directory-layout { display:grid; min-height:0; isolation:isolate; grid-template-columns:minmax(12.5rem,0.62fr) minmax(0,1.6fr); overflow:visible; border:0; border-radius:0; background:transparent; }
  .directory-panel { position:relative; z-index:1; display:grid; min-width:0; min-height:0; grid-template-rows:auto minmax(0,1fr); gap:0.45rem; border:0; padding:0 0.75rem 0 0; }
  .directory-search { display:flex; height:1.9rem; align-items:center; gap:0.35rem; border-bottom:1px solid var(--border); padding-inline:0.35rem; color:var(--muted-foreground); }
  .directory-search input { width:100%; min-width:0; background:transparent; color:var(--foreground); outline:0; font-size:calc(0.733333rem * var(--type-scale)); }
  .directory-list { gap:0; padding:0.15rem 0.2rem 0.15rem 0; }
  .directory-row { grid-template-columns:auto minmax(0,1fr); margin-inline:0.2rem; padding:0.6rem; }
  .directory-row:hover,.directory-row.active { background:var(--accent); color:var(--accent-foreground); }
  .directory-avatar { position:relative; display:grid; }
  .directory-summary { display:grid; min-width:0; }
  .directory-summary strong,.directory-summary small { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .directory-summary strong { font-size:calc(0.8rem * var(--type-scale)); font-weight:600; }
  .directory-summary small { margin-top:0.05rem; color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); }
  .detail-panel { position:relative; z-index:2; min-width:0; min-height:0; border-left:1px solid var(--border); padding-left:1rem; }
  .teammate-editor { grid-template-rows:auto minmax(0,1fr) auto; }
  .editor-header { border:0; padding:0.2rem 0.25rem 0.75rem; }
  .identity-heading { padding:0; }
  .editor-scroll { padding:0 0.75rem 1rem 0; }
  .overview-and-access,.overview-content,.access-content { display:grid; align-content:start; gap:0.9rem; }
  .editor-section { display:grid; gap:0.75rem; border:0; padding:0; }
  .section-divider { height:1px; background:var(--border); }
  .text-button { color:var(--primary); font-size:calc(0.68rem * var(--type-scale)); font-weight:600; white-space:nowrap; }
  .text-button:hover { text-decoration:underline; text-underline-offset:0.15rem; }
  .access-toolbar { align-items:center; }
  .access-toolbar-actions { flex-wrap:nowrap; }
  .impact-preview { display:flex; align-items:flex-start; justify-content:space-between; gap:0.75rem; border:0; border-left:2px solid var(--destructive); border-radius:0; background:transparent; padding:0.15rem 0 0.15rem 0.65rem; }
  .channel-picker { display:grid; border:0; border-radius:0; background:transparent; box-shadow:none; overflow:visible; }
  .channel-picker > header { border:0; padding:0 0 0.6rem; }
  .channel-picker > footer { border:0; padding:0.6rem 0 0; }
  .channel-picker > footer > span { display:none; }
  .channel-picker .compact-picker { display:grid; }
  .compact-picker > header { border:0; border-bottom:1px solid var(--border); padding-inline:0; }
  .compact-picker-list { min-height:8rem; max-height:18rem; padding:0; }
  .compact-picker-row { border-bottom:1px solid color-mix(in srgb,var(--border) 55%,transparent); padding-inline:0.25rem; }
  .channel-option { border-bottom:1px solid color-mix(in srgb,var(--border) 55%,transparent); border-radius:0; padding-inline:0.25rem; }
  .channel-option:hover,.compact-picker-row:hover { background:color-mix(in srgb,var(--accent) 55%,transparent); }
  .empty-access { min-height:5rem; place-items:start; align-content:center; border:0; border-radius:0; text-align:left; }
  .empty-access h5 { font-size:calc(0.733333rem * var(--type-scale)); }
  .empty-access p { max-width:none; }
  .access-list { gap:0; border-top:1px solid var(--border); }
  .access-card { border:0; border-bottom:1px solid var(--border); border-radius:0; background:transparent; overflow:visible; }
  .access-card > header { display:grid; grid-template-columns:minmax(0,1fr) auto; min-height:3.2rem; align-items:center; gap:0.4rem; padding:0.45rem 0.2rem; }
  .access-summary { grid-template-columns:auto auto minmax(0,1fr); }
  .access-summary > :global(svg:first-child) { transition:transform 120ms ease; }
  .access-summary > :global(svg.expanded:first-child) { transform:rotate(90deg); }
  .access-summary small:last-child { margin-top:0.08rem; }
  .access-details { gap:0.85rem; border-top:1px solid color-mix(in srgb,var(--border) 65%,transparent); padding:0.8rem 0.2rem 1rem 1.45rem; }
  .capability-switches { gap:0; }
  .capability-switches > div { display:grid; grid-template-columns:auto minmax(0,1fr); gap:0.5rem; border:0; border-bottom:1px solid color-mix(in srgb,var(--border) 55%,transparent); border-radius:0; padding:0.55rem 0.15rem; }
  .folder-row { grid-template-columns:auto auto minmax(7rem,1fr) minmax(7rem,9rem) minmax(8rem,10rem) auto auto; }
  .scratch-row { display:grid; grid-template-columns:auto minmax(0,1fr) minmax(9rem,auto); align-items:center; gap:0.7rem; border-top:1px solid var(--border); border-radius:0; background:transparent; padding:0.7rem 0.2rem; }
  .scratch-approval { display:grid; min-width:9rem; gap:0.2rem; color:var(--muted-foreground); font-size:calc(0.6rem * var(--type-scale)); }
  .disclosure-button { display:inline-flex; width:max-content; align-items:center; gap:0.3rem; color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); font-weight:600; }
  .disclosure-button:hover { color:var(--foreground); }
  .disclosure-button :global(svg) { transition:transform 120ms ease; }
  .disclosure-button :global(svg.expanded) { transform:rotate(90deg); }
  .advanced-runtime { width:min(18rem,100%); }
  .editor-footer { padding:0.65rem 0.75rem 0.65rem 0; }
  .editor-footer button { white-space:nowrap; }
  .primary-button,.secondary-button,.danger-button { white-space:nowrap; }
  .teammate-settings input[type="radio"] { appearance:none; width:0.9rem; height:0.9rem; border:1px solid var(--border); border-radius:50%; background:var(--background); }
  .teammate-settings input[type="radio"]:checked { border:0.25rem solid var(--primary); }
  @media (max-width:700px) { .teammate-settings { grid-template-rows:auto minmax(0,1fr); }.directory-layout { grid-template-columns:1fr; grid-template-rows:minmax(7rem,28%) minmax(0,1fr); }.directory-panel { border-bottom:1px solid var(--border); padding:0 0 0.75rem; }.detail-panel { border-top:0; border-left:0; padding:0.8rem 0 0; }.directory-list { grid-template-columns:repeat(auto-fill,minmax(11rem,1fr)); }.field-grid,.field-grid.compact,.capability-switches { grid-template-columns:1fr; }.folder-row { grid-template-columns:auto auto minmax(7rem,1fr) minmax(7rem,9rem); }.folder-row .default-target,.folder-row .link-button { grid-column:3/-1; justify-self:start; }.editor-footer { padding-right:0; } }
</style>
