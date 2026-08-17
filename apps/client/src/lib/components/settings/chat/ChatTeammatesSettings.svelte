<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import Check from "@lucide/svelte/icons/check";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import Plus from "@lucide/svelte/icons/plus";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import * as chatApi from "$lib/api/chat";
  import {
    copyModelOptionSelections,
    copyVersionedJson,
    resolveDefaultProviderModel,
  } from "$lib/chat/composer-model";
  import type {
    ChatAiTeammateRead,
    ChatApprovalPolicy,
    ChatConversationMembershipRead,
    ModelOptionSelection,
  } from "$lib/chat/contracts";
  import { chatErrorField, chatErrorMessage } from "$lib/chat/error-presentation";
  import { modelCompany } from "$lib/chat/model-company";
  import {
    teammateExecutionSummary,
    teammateMembershipDraftSnapshot,
    teammateProfileDraftSnapshot,
  } from "$lib/chat/teammate-draft";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import CustomSelect from "$lib/components/settings/CustomSelect.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import ChatAccessControl from "$lib/components/chat/ChatAccessControl.svelte";
  import ChatModelAvatar from "$lib/components/chat/ChatModelAvatar.svelte";
  import ChatModelControls from "$lib/components/chat/ChatModelControls.svelte";
  import ChatParticipantAvatar from "$lib/components/chat/ChatParticipantAvatar.svelte";

  let {
    initialTeammateId,
    onDraftStateChange = () => {},
  }: {
    initialTeammateId?: string;
    onDraftStateChange?: (open: boolean) => void;
  } = $props();

  const chat = getChat();
  const { t } = getLocalization();
  type LifecycleAction = "archive" | "delete";

  let selectedId = $state<string | null>(null);
  let showArchived = $state(false);
  let archivedTeammates = $state<ChatAiTeammateRead[]>(chat.archivedTeammates);
  let archivedLoaded = $state(chat.loaded);
  let archivedLoading = $state(false);
  let archivedLoadError = $state<string | null>(null);
  let creating = $state(false);
  let displayName = $state("");
  let role = $state("");
  let instructions = $state("");
  let providerId = $state("");
  let modelId = $state("");
  let providerManagedModel = $state(false);
  let modelOptions = $state<ModelOptionSelection[]>([]);
  let effort = $state<string | null>(null);
  let speed = $state<string | null>(null);
  let approvalPolicy = $state<ChatApprovalPolicy>("ask_for_approval");
  let channelId = $state("");
  let folderIds = $state<string[]>([]);
  let defaultFolderId = $state("");
  let directoryScrollElement = $state<HTMLElement>();
  let draftRowElement = $state<HTMLButtonElement>();
  let detailScrollElement = $state<HTMLElement>();
  let membershipRevision = $state<number | null>(null);
  let loadingMembership = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let errorField = $state<string | null>(null);
  let savedNotice = $state(false);
  let profileBaselineSnapshot = $state<string | null>(null);
  let membershipBaselineSnapshot = $state<string | null>(null);
  let membershipLoadRequestId = 0;
  let lifecycleAction = $state<LifecycleAction | null>(null);
  let lifecycleTarget = $state<ChatAiTeammateRead | null>(null);
  let lifecycleBusy = $state(false);
  let lifecycleError = $state<string | null>(null);
  let autoStarted = false;
  let initialSelectionApplied = false;
  const directoryTeammates = $derived(showArchived
    ? [...chat.teammates, ...archivedTeammates]
    : chat.teammates);
  const allTeammates = $derived([...chat.teammates, ...archivedTeammates]);
  const selected = $derived(directoryTeammates.find((teammate) => teammate.participant.id === selectedId) ?? null);
  const archivedMode = $derived(Boolean(selected?.participant.archivedAt));
  const providers = $derived(chat.settings?.providerInstances ?? []);
  const selectedProvider = $derived(providers.find((provider) => provider.configuration.instanceId === providerId) ?? null);
  const models = $derived(selectedProvider?.modelCatalog?.models.filter((model) => model.availability !== "deprecated") ?? []);
  const selectedModel = $derived(models.find((model) => model.id === modelId) ?? null);

  $effect(() => {
    onDraftStateChange(creating);
    return () => onDraftStateChange(false);
  });
  const channel = $derived(chat.activeChannels.find((entry) => entry.id === channelId) ?? chat.selectedChannel);
  const projectFolders = $derived(chat.workingFolders.filter((entry) => (
    entry.workingFolder.projectId === channel?.projectId && entry.workingFolder.archivedAt === null
  )));
  const channelOptions = $derived(chat.activeChannels.map((entry) => ({ value: entry.id, label: `#${entry.name}` })));
  const folderOptions = $derived(projectFolders.map((entry) => ({
    value: entry.workingFolder.id,
    label: entry.workingFolder.displayName,
  })));
  const draftCompany = $derived(selectedProvider
    ? modelCompany(selectedProvider.configuration.familyId, selectedModel)
    : null);
  const modelSelectionValid = $derived(providerManagedModel !== Boolean(modelId));
  const draftConfigurationState = $derived(
    selectedProvider?.configuration.enabled
      && selectedProvider.lastProbe?.state === "healthy"
      && modelSelectionValid
      && Boolean(defaultFolderId)
      ? "healthy"
      : "needs_setup",
  );
  const dirty = $derived(
    profileBaselineSnapshot !== null
      && membershipBaselineSnapshot !== null
      && (
        currentProfileSnapshot() !== profileBaselineSnapshot
          || currentMembershipSnapshot() !== membershipBaselineSnapshot
      ),
  );
  const normalizedDisplayName = $derived(displayName.trim().toLocaleLowerCase());
  const nameTaken = $derived(Boolean(
    normalizedDisplayName
      && allTeammates.some((teammate) => (
        teammate.participant.id !== selectedId
          && teammate.participant.displayName.trim().toLocaleLowerCase() === normalizedDisplayName
      )),
  ));
  const canSave = $derived(Boolean(
    displayName.trim()
      && role.trim()
      && !nameTaken
      && providerId
      && modelSelectionValid
      && channelId
      && defaultFolderId
      && dirty
      && !loadingMembership,
  ));

  function currentProfileSnapshot(): string {
    return teammateProfileDraftSnapshot({
      displayName,
      role,
      instructions,
      providerId,
      modelId,
      providerManagedModel,
      modelOptions,
      effort,
      speed,
    });
  }

  function currentMembershipSnapshot(): string {
    return teammateMembershipDraftSnapshot({
      channelId,
      approvalPolicy,
      folderIds,
      defaultFolderId,
    });
  }

  onMount(() => {
    void refreshArchivedTeammates();
  });

  $effect(() => {
    if (initialSelectionApplied || !initialTeammateId) return;
    const activeMatch = chat.teammates.some((teammate) => teammate.participant.id === initialTeammateId);
    const archivedMatch = archivedTeammates.some((teammate) => teammate.participant.id === initialTeammateId);
    if (!activeMatch && !archivedMatch) return;
    initialSelectionApplied = true;
    creating = false;
    showArchived = archivedMatch;
    selectedId = initialTeammateId;
  });

  $effect(() => {
    if (creating) return;
    const teammates = directoryTeammates;
    if (teammates.some((teammate) => teammate.participant.id === selectedId)) return;
    selectedId = teammates[0]?.participant.id ?? null;
  });

  $effect(() => {
    if (
      autoStarted
        || creating
        || !archivedLoaded
        || chat.teammates.length > 0
        || archivedTeammates.length > 0
        || !resolveDefaultProviderModel(providers)
    ) return;
    autoStarted = true;
    beginCreate();
  });

  $effect(() => {
    const teammate = selected;
    if (!teammate || creating) return;
    // Initialization reads draft fields to capture a clean baseline. Keep those
    // reads outside dependency tracking so edits do not restart membership I/O.
    untrack(() => initializeSelectedTeammate(teammate));
  });

  function initializeSelectedTeammate(teammate: ChatAiTeammateRead): void {
    profileBaselineSnapshot = null;
    membershipBaselineSnapshot = null;
    error = null;
    errorField = null;
    savedNotice = false;
    lifecycleError = null;
    displayName = teammate.participant.displayName;
    role = teammate.role;
    instructions = teammate.instructions;
    providerId = teammate.latestPolicy?.providerInstanceId ?? "";
    modelId = teammate.latestPolicy?.modelId ?? "";
    providerManagedModel = teammate.latestPolicy?.providerManagedModel ?? false;
    modelOptions = copyModelOptionSelections(teammate.latestPolicy?.modelOptions ?? []);
    const summary = teammateExecutionSummary(modelOptions, selectedModel);
    effort = summary.effort ?? teammate.latestPolicy?.effort ?? null;
    speed = summary.speed ?? teammate.latestPolicy?.speed ?? null;
    channelId = chat.selectedChannelId ?? chat.activeChannels[0]?.id ?? "";
    profileBaselineSnapshot = currentProfileSnapshot();
    void loadMembership(teammate, channelId);
  }

  $effect(() => {
    if (dirty) savedNotice = false;
  });

  function beginCreate(): void {
    showArchived = false;
    creating = true;
    selectedId = null;
    displayName = "";
    role = "";
    instructions = "";
    approvalPolicy = "ask_for_approval";
    channelId = chat.selectedChannelId ?? chat.activeChannels[0]?.id ?? "";
    const resolved = resolveDefaultProviderModel(providers);
    providerId = resolved?.provider.configuration.instanceId ?? "";
    modelId = resolved?.model?.id ?? "";
    providerManagedModel = resolved?.providerManaged ?? false;
    modelOptions = copyModelOptionSelections(resolved?.options ?? []);
    const summary = teammateExecutionSummary(modelOptions, resolved?.model ?? null);
    effort = summary.effort;
    speed = summary.speed;
    const managed = projectFolders.find((entry) => entry.workingFolder.kind === "managed") ?? projectFolders[0];
    defaultFolderId = managed?.workingFolder.id ?? "";
    folderIds = defaultFolderId ? [defaultFolderId] : [];
    membershipRevision = null;
    error = null;
    errorField = null;
    savedNotice = false;
    profileBaselineSnapshot = currentProfileSnapshot();
    membershipBaselineSnapshot = currentMembershipSnapshot();
    void tick().then(() => draftRowElement?.scrollIntoView?.({ block: "nearest" }));
  }

  function cancelCreate(): void {
    creating = false;
    selectedId = chat.teammates[0]?.participant.id ?? null;
    error = null;
    errorField = null;
    savedNotice = false;
    profileBaselineSnapshot = null;
    membershipBaselineSnapshot = null;
  }

  async function refreshArchivedTeammates(): Promise<void> {
    archivedLoading = true;
    archivedLoadError = null;
    try {
      archivedTeammates = await chatApi.listChatTeammates(true);
      chat.archivedTeammates = archivedTeammates;
      archivedLoaded = true;
      if (archivedTeammates.length === 0) showArchived = false;
    } catch (cause: unknown) {
      archivedLoadError = chatErrorMessage(cause, t("settings.chat.teammates.archiveLoadFailed"));
    } finally {
      archivedLoading = false;
    }
  }

  function setShowArchived(value: boolean): void {
    if (creating || value === showArchived) return;
    const selectedWasArchived = Boolean(selected?.participant.archivedAt);
    showArchived = value;
    if (!value && selectedWasArchived) {
      selectedId = chat.teammates[0]?.participant.id ?? null;
    }
    error = null;
    errorField = null;
    lifecycleError = null;
  }

  async function loadMembership(teammate: ChatAiTeammateRead, requestedChannelId: string): Promise<void> {
    const requestId = ++membershipLoadRequestId;
    membershipBaselineSnapshot = null;
    if (!requestedChannelId) {
      loadingMembership = false;
      return;
    }
    loadingMembership = true;
    try {
      const memberships = await chatApi.listChatChannelMemberships(requestedChannelId);
      if (
        requestId !== membershipLoadRequestId
          || selectedId !== teammate.participant.id
          || channelId !== requestedChannelId
      ) return;
      const membership = memberships.find((entry) => entry.participant.id === teammate.participant.id) ?? null;
      applyMembership(membership);
      membershipBaselineSnapshot = currentMembershipSnapshot();
    } finally {
      if (requestId === membershipLoadRequestId) loadingMembership = false;
    }
  }

  function applyMembership(membership: ChatConversationMembershipRead | null): void {
    approvalPolicy = membership?.approvalPolicy ?? "ask_for_approval";
    folderIds = membership?.workingFolderGrants.map((grant) => grant.workingFolderId) ?? [];
    defaultFolderId = membership?.workingFolderGrants.find((grant) => grant.isDefault)?.workingFolderId
      ?? folderIds[0]
      ?? projectFolders.find((entry) => entry.workingFolder.kind === "managed")?.workingFolder.id
      ?? projectFolders[0]?.workingFolder.id
      ?? "";
    if (defaultFolderId && !folderIds.includes(defaultFolderId)) folderIds = [...folderIds, defaultFolderId];
    membershipRevision = membership?.revision ?? null;
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

  function toggleFolder(folderId: string): void {
    if (folderIds.includes(folderId)) {
      if (folderId === defaultFolderId) return;
      folderIds = folderIds.filter((id) => id !== folderId);
    } else {
      folderIds = [...folderIds, folderId];
    }
  }

  async function save(): Promise<void> {
    if (archivedMode || saving || !canSave) return;
    saving = true;
    error = null;
    errorField = null;
    savedNotice = false;
    try {
      const policy = {
        providerInstanceId: providerId,
        providerManagedModel,
        modelId: modelId || null,
        modelOptions: copyModelOptionSelections(modelOptions),
        effort,
        speed,
        providerOptions: { schemaVersion: 1, value: {} },
      };
      const membership = {
        channelId,
        addressable: true,
        approvalPolicy,
        workingFolderIds: [...new Set([...folderIds, defaultFolderId])],
        defaultWorkingFolderId: defaultFolderId,
      };
      if (creating) {
        const teammate = await chatApi.createChatTeammate({
          teammateId: `participant:${crypto.randomUUID()}`,
          displayName: displayName.trim(),
          avatar: { schemaVersion: 1, value: { kind: "initials" } },
          role: role.trim(),
          instructions: instructions.trim(),
          policy,
          memberships: [membership],
        });
        creating = false;
        selectedId = teammate.participant.id;
      } else if (selected) {
        await chatApi.updateChatTeammateProfile({
          teammateId: selected.participant.id,
          displayName: displayName.trim(),
          avatar: copyVersionedJson(selected.participant.avatar),
          role: role.trim(),
          instructions: instructions.trim(),
          expectedRevision: selected.participant.revision,
        });
        await chatApi.publishChatTeammatePolicy({ teammateId: selected.participant.id, policy });
        await chatApi.upsertChatTeammateMembership({
          teammateId: selected.participant.id,
          membership,
          expectedRevision: membershipRevision,
        });
      }
      await chat.refreshTeammates();
      profileBaselineSnapshot = currentProfileSnapshot();
      membershipBaselineSnapshot = currentMembershipSnapshot();
      savedNotice = true;
    } catch (cause: unknown) {
      error = chatErrorMessage(cause, t("settings.chat.teammates.saveFailed"));
      errorField = chatErrorField(cause);
    } finally {
      saving = false;
    }
  }

  function clearFieldError(field: string): void {
    if (errorField !== field) return;
    error = null;
    errorField = null;
  }

  function requestArchiveSelected(): void {
    if (!selected || archivedMode || lifecycleBusy) return;
    if (selected.activeAssignmentCount > 0) {
      lifecycleError = t(
        "settings.chat.teammates.archiveBlocked",
        selected.activeAssignmentCount,
      );
      return;
    }
    lifecycleError = null;
    lifecycleTarget = selected;
    lifecycleAction = "archive";
  }

  function requestDeleteSelected(): void {
    if (!selected || !archivedMode || selected.hasDurableHistory || lifecycleBusy) return;
    lifecycleError = null;
    lifecycleTarget = selected;
    lifecycleAction = "delete";
  }

  function cancelLifecycleAction(): void {
    lifecycleAction = null;
    lifecycleTarget = null;
  }

  async function confirmLifecycleAction(): Promise<void> {
    const action = lifecycleAction;
    const target = lifecycleTarget;
    if (!action || !target || lifecycleBusy) return;
    cancelLifecycleAction();
    lifecycleBusy = true;
    lifecycleError = null;
    try {
      if (action === "archive") {
        await chatApi.archiveChatTeammate(
          target.participant.id,
          target.participant.revision,
          true,
        );
      } else {
        await chatApi.deleteUnusedChatTeammate(
          target.participant.id,
          target.participant.revision,
        );
      }
      selectedId = null;
      profileBaselineSnapshot = null;
      membershipBaselineSnapshot = null;
      await chat.refreshTeammates();
      await refreshArchivedTeammates();
    } catch (cause: unknown) {
      lifecycleError = chatErrorMessage(
        cause,
        action === "archive"
          ? t("settings.chat.teammates.archiveFailed")
          : t("settings.chat.teammates.deleteFailed"),
      );
    } finally {
      lifecycleBusy = false;
    }
  }

  async function restoreSelected(): Promise<void> {
    if (!selected || !archivedMode || lifecycleBusy) return;
    const target = selected;
    lifecycleBusy = true;
    lifecycleError = null;
    try {
      const restored = await chatApi.archiveChatTeammate(
        target.participant.id,
        target.participant.revision,
        false,
      );
      profileBaselineSnapshot = null;
      membershipBaselineSnapshot = null;
      await chat.refreshTeammates();
      await refreshArchivedTeammates();
      selectedId = restored.participant.id;
    } catch (cause: unknown) {
      lifecycleError = chatErrorMessage(cause, t("settings.chat.teammates.restoreFailed"));
    } finally {
      lifecycleBusy = false;
    }
  }

</script>

<section class="teammate-settings" data-chat-settings-subsection="teammates">
  <header class="directory-header">
    <div><h2>{t("settings.chat.teammates.heading")}</h2><p>{t("settings.chat.teammates.description")}</p></div>
    <div class="header-actions">
      <button type="button" class="settings-button" disabled={creating || lifecycleBusy} onclick={beginCreate}><Plus size={13} />{creating ? t("settings.chat.teammates.draftOpen") : t("settings.chat.teammates.add")}</button>
      {#if archivedLoadError || archivedTeammates.length > 0}
        {@const archiveFilterLabel = showArchived ? t("settings.chat.teammates.hideArchived") : t("settings.chat.teammates.includeArchived")}
        <button
          type="button"
          class="settings-button archive-filter-button"
          aria-label={archiveFilterLabel}
          aria-pressed={showArchived}
          data-app-tooltip={archiveFilterLabel}
          disabled={creating || lifecycleBusy || archivedLoading}
          onclick={() => setShowArchived(!showArchived)}
        >
          <span class="archive-filter-icon" aria-hidden="true">
            <Archive size={14} />
            <span class="archive-visibility-icon">
              {#if showArchived}<Eye size={8} />{:else}<EyeOff size={8} />{/if}
            </span>
          </span>
        </button>
      {/if}
    </div>
  </header>

  <div class="directory-layout">
    <aside class="directory-panel">
      <div class="directory-list-frame">
        <div bind:this={directoryScrollElement} class="directory-scroll hide-scrollbar">
          <div>
            {#if creating || directoryTeammates.length > 0}
              <nav class="teammate-directory" aria-label={t("settings.chat.teammates.directoryLabel")}>
                {#each directoryTeammates as teammate (teammate.participant.id)}
                  {@const active = !creating && selectedId === teammate.participant.id}
                  {@const archived = teammate.participant.archivedAt !== null}
                  <button type="button" class:active class:archived-row={archived} aria-current={active ? "page" : undefined} disabled={creating || lifecycleBusy} onclick={() => { selectedId = teammate.participant.id; }}>
                    <span class="directory-avatar">
                      <ChatParticipantAvatar participant={teammate.participant} {teammate} size={32} />
                      {#if archived}<span class="archived-mark" aria-label={t("settings.chat.teammates.archived")} title={t("settings.chat.teammates.archived")}><Archive size={11} /></span>{/if}
                    </span>
                    <span class="directory-summary">
                      <strong>{teammate.participant.displayName}</strong>
                      <small>{teammate.role}</small>
                    </span>
                  </button>
                {/each}
                {#if creating}
                  <button bind:this={draftRowElement} type="button" class="active draft-row" aria-current="page">
                    {#if draftCompany}
                      <span class="directory-avatar"><ChatModelAvatar familyId={draftCompany.iconFamilyId} label={draftCompany.name} size={32} /></span>
                    {:else}<span class="draft-avatar"><Plus size={15} /></span>{/if}
                    <span class="directory-summary">
                      <strong>{displayName.trim() || t("settings.chat.teammates.name")}</strong>
                      <small>{role.trim() || t("settings.chat.teammates.role")}</small>
                    </span>
                  </button>
                {/if}
              </nav>
            {:else if showArchived && archivedLoading}
              <p class="directory-empty">{t("settings.chat.teammates.loadingArchived")}</p>
            {:else if showArchived && archivedLoadError}
              <div class="directory-empty"><p>{archivedLoadError}</p><button type="button" onclick={() => void refreshArchivedTeammates()}>{t("common.retry")}</button></div>
            {:else}
              <p class="directory-empty">{showArchived ? t("settings.chat.teammates.empty") : archivedTeammates.length > 0 ? t("settings.chat.teammates.emptyActive") : t("settings.chat.teammates.empty")}</p>
            {/if}
          </div>
        </div>
        <CalendarScrollbar scrollContainer={directoryScrollElement} wheelPassthrough />
      </div>
    </aside>

    <div class="detail-panel">
    {#if creating || selected}
    <form class="teammate-editor" onsubmit={(event) => { event.preventDefault(); void save(); }}>
      <div class="editor-scroll-frame">
        <div bind:this={detailScrollElement} class="editor-scroll hide-scrollbar">
          <div class="editor-content">
      <div class="editor-heading">
        {#if creating}
          {#if draftCompany}<ChatModelAvatar familyId={draftCompany.iconFamilyId} label={draftCompany.name} size={38} />
          {:else}<span class="draft-avatar editor-draft-avatar"><Plus size={16} /></span>{/if}
        {:else if selected}<ChatParticipantAvatar participant={selected.participant} teammate={selected} size={38} />{/if}
        <div class="editor-title">
          <div class="editor-name-line">
            <h3>{displayName.trim() || t("settings.chat.teammates.name")}</h3>
            {#if creating}
              <span class="editor-state" data-state={draftConfigurationState}><i></i>{draftConfigurationState === "healthy" ? t("settings.chat.teammates.available") : t("settings.chat.teammates.needsSetup")}</span>
            {:else if selected}
              {#if archivedMode}
                <span class="editor-state archived-state"><Archive size={12} />{t("settings.chat.teammates.archived")}</span>
              {:else}
                <span class="editor-state" data-state={selected.configurationState}><i></i>{selected.configurationState === "healthy" ? t("settings.chat.teammates.available") : t("settings.chat.teammates.needsSetup")}</span>
              {/if}
            {/if}
          </div>
          <p>{role.trim() || t("settings.chat.teammates.role")}</p>
        </div>
      </div>

      <section class="editor-section">
        <div class="section-heading"><h4>{t("settings.chat.teammates.identitySection")}</h4></div>
        <div class="field-grid">
          <div class="field full"><span id="teammate-name-label">{t("settings.chat.teammates.name")}<i class="required-marker" aria-hidden="true">*</i></span><input bind:value={displayName} aria-labelledby="teammate-name-label" aria-describedby={nameTaken || (error && errorField === "displayName") ? "teammate-name-error" : undefined} aria-invalid={nameTaken || (error && errorField === "displayName") ? "true" : undefined} placeholder={t("settings.chat.teammates.namePlaceholder")} maxlength="160" required disabled={archivedMode} oninput={() => clearFieldError("displayName")} />{#if nameTaken}<small id="teammate-name-error" class="field-error" role="alert">{t("settings.chat.teammates.nameTaken")}</small>{:else if error && errorField === "displayName"}<small id="teammate-name-error" class="field-error" role="alert">{error}</small>{/if}</div>
          <div class="field full"><span id="teammate-role-label">{t("settings.chat.teammates.role")}<i class="required-marker" aria-hidden="true">*</i></span><input bind:value={role} aria-labelledby="teammate-role-label" aria-describedby={error && errorField === "role" ? "teammate-role-error" : undefined} aria-invalid={error && errorField === "role" ? "true" : undefined} placeholder={t("settings.chat.teammates.rolePlaceholder")} maxlength="1000" required disabled={archivedMode} oninput={() => clearFieldError("role")} />{#if error && errorField === "role"}<small id="teammate-role-error" class="field-error" role="alert">{error}</small>{/if}</div>
          <div class="field full"><span id="teammate-instructions-label">{t("settings.chat.teammates.instructions")}</span><textarea bind:value={instructions} aria-labelledby="teammate-instructions-label" placeholder={t("settings.chat.teammates.instructionsPlaceholder")} maxlength="65536" rows="4" disabled={archivedMode}></textarea></div>
        </div>
      </section>

      <div class="section-divider" aria-hidden="true"></div>
      <section class="editor-section">
        <div class="section-heading"><h4>{t("settings.chat.teammates.executionSection")}</h4></div>
        <div class="field-grid">
          <div class="field execution-model-field"><span>{t("settings.chat.teammates.model")}<i class="required-marker" aria-hidden="true">*</i></span><ChatModelControls value={{ providerInstanceId: providerId || null, modelId: modelId || null, providerManaged: providerManagedModel, options: modelOptions }} disabled={archivedMode} onChange={selectExecution} /></div>
          <div class="field execution-approval-field"><span>{t("settings.chat.teammates.approval")}</span><ChatAccessControl value={approvalPolicy} providerInstanceId={providerId || null} workingFolderId={defaultFolderId || null} disabled={archivedMode} onChange={(value) => { approvalPolicy = value; }} /></div>
        </div>
      </section>

      <div class="section-divider" aria-hidden="true"></div>
      <section class="editor-section">
        <div class="section-heading"><h4>{t("settings.chat.teammates.accessSection")}</h4></div>
        <div class="field-grid">
          <div class="field"><span>{t("settings.chat.teammates.channel")}<i class="required-marker" aria-hidden="true">*</i></span><CustomSelect inline class="w-full" value={channelId} options={channelOptions} ariaLabel={t("settings.chat.teammates.channel")} disabled={archivedMode} onChange={(value) => { channelId = value; if (selected) void loadMembership(selected, value); }} /></div>
          <div class="field"><span>{t("settings.chat.teammates.defaultFolder")}<i class="required-marker" aria-hidden="true">*</i></span><CustomSelect inline class="w-full" value={defaultFolderId} options={folderOptions} ariaLabel={t("settings.chat.teammates.defaultFolder")} disabled={archivedMode} onChange={(value) => { defaultFolderId = value; if (!folderIds.includes(value)) folderIds = [...folderIds, value]; }} /></div>
        </div>
        <fieldset disabled={loadingMembership || archivedMode}>
          <legend>{t("settings.chat.teammates.allowedFolders")}</legend>
          <p>{t("settings.chat.teammates.allowedFoldersDescription")}</p>
          <div class="folder-grants">
            {#each folderOptions as folder (folder.value)}
              <button type="button" aria-pressed={folderIds.includes(folder.value)} onclick={() => toggleFolder(folder.value)}>
                <span class="grant-indicator">{#if folderIds.includes(folder.value)}<Check size={12} />{/if}</span>
                <span>{folder.label}</span>
                {#if folder.value === defaultFolderId}<small>{t("settings.chat.teammates.default")}</small>{/if}
              </button>
            {/each}
          </div>
        </fieldset>
      </section>

      {#if error && errorField !== "displayName" && errorField !== "role"}<p class="form-error" role="alert">{error}</p>{/if}
      {#if savedNotice}<p class="saved" role="status">{t("settings.chat.teammates.saved")}</p>{/if}
          </div>
        </div>
        <CalendarScrollbar scrollContainer={detailScrollElement} wheelPassthrough />
      </div>
      <footer>
        <div class="footer-leading">
          {#if creating}
            <button type="button" class="settings-button" disabled={saving} onclick={cancelCreate}><X size={13} />{t("settings.chat.teammates.cancelDraft")}</button>
          {:else if selected && archivedMode}
            {#if selected.hasDurableHistory}
              <span class="footer-note">{t("settings.chat.teammates.historyPreserved")}</span>
            {:else}
              <button type="button" class="settings-button delete-button" disabled={lifecycleBusy} onclick={requestDeleteSelected}><Trash2 size={13} />{t("settings.chat.teammates.deletePermanently")}</button>
            {/if}
          {:else if selected}
            <button
              type="button"
              class="settings-button"
              disabled={lifecycleBusy || selected.activeAssignmentCount > 0}
              title={selected.activeAssignmentCount > 0 ? t("settings.chat.teammates.archiveBlocked", selected.activeAssignmentCount) : undefined}
              onclick={requestArchiveSelected}
            ><Archive size={13} />{t("settings.chat.teammates.archive")}</button>
            {#if selected.activeAssignmentCount > 0}
              <span class="footer-note">{t("settings.chat.teammates.archiveBlocked", selected.activeAssignmentCount)}</span>
            {/if}
          {/if}
          {#if lifecycleError}<span class="footer-error" role="alert">{lifecycleError}</span>{/if}
        </div>
        <div class="footer-actions">
          {#if archivedMode && selected}
            <button type="button" class="settings-primary-button" disabled={lifecycleBusy} onclick={() => void restoreSelected()}><ArchiveRestore size={13} />{lifecycleBusy ? t("settings.chat.teammates.restoring") : t("settings.chat.teammates.restore")}</button>
          {:else}
            <button type="submit" class="settings-primary-button" disabled={saving || lifecycleBusy || !canSave}>{saving ? t("settings.chat.teammates.saving") : t("settings.chat.teammates.save")}</button>
          {/if}
        </div>
      </footer>
    </form>
    {:else}
      <p class="empty-detail">{t("settings.chat.teammates.selectPrompt")}</p>
    {/if}
    </div>
  </div>
</section>

{#if lifecycleAction && lifecycleTarget}
  <ConfirmDialog
    title={lifecycleAction === "archive"
      ? t("settings.chat.teammates.archiveTitle", lifecycleTarget.participant.displayName)
      : t("settings.chat.teammates.deleteTitle", lifecycleTarget.participant.displayName)}
    message={lifecycleAction === "archive"
      ? dirty
        ? t("settings.chat.teammates.archiveMessageWithChanges", lifecycleTarget.participant.displayName)
        : t("settings.chat.teammates.archiveMessage", lifecycleTarget.participant.displayName)
      : t("settings.chat.teammates.deleteMessage", lifecycleTarget.participant.displayName)}
    confirmLabel={lifecycleAction === "archive"
      ? t("settings.chat.teammates.archiveConfirm")
      : t("settings.chat.teammates.deletePermanently")}
    cancelLabel={t("common.cancel")}
    danger={lifecycleAction === "delete"}
    onConfirm={() => void confirmLifecycleAction()}
    onCancel={cancelLifecycleAction}
  />
{/if}

<style>
  .teammate-settings { display:grid; height:100%; min-height:0; grid-template-rows:auto minmax(0,1fr); gap:1rem; }
  .directory-header { display:flex; flex-wrap:wrap; align-items:start; justify-content:space-between; gap:0.75rem; padding-inline:0.25rem; }
  .directory-header h2 { font-size:calc(0.866667rem * var(--type-scale)); font-weight:600; }
  .directory-header p { margin-top:0.25rem; color:var(--muted-foreground); font-size:calc(0.8rem * var(--type-scale)); }
  .header-actions { display:flex; flex-wrap:wrap; align-items:center; justify-content:flex-end; gap:0.375rem; }
  .settings-button,.settings-primary-button { display:inline-flex; min-height:1.75rem; align-items:center; justify-content:center; gap:0.375rem; border:1px solid var(--border); border-radius:0.375rem; padding:0.25rem 0.625rem; font-size:calc(0.733333rem * var(--type-scale)); font-weight:500; transition:background-color 120ms ease; }
  .settings-button { background:var(--background); color:var(--foreground); }
  .settings-button:hover:not(:disabled) { background:var(--accent); }
  .archive-filter-button { width:1.75rem; padding:0; color:var(--foreground); }
  .archive-filter-icon { position:relative; display:grid; width:1rem; height:1rem; place-items:center; }
  .archive-visibility-icon { position:absolute; right:-0.22rem; bottom:-0.2rem; display:grid; width:0.7rem; height:0.7rem; place-items:center; border-radius:999px; background:var(--background); color:var(--foreground); }
  .settings-primary-button { background:var(--primary); color:var(--primary-foreground); padding-inline:0.75rem; font-size:calc(0.8rem * var(--type-scale)); }
  .settings-primary-button:hover:not(:disabled) { background:color-mix(in srgb,var(--primary) 90%,transparent); }
  .settings-button:disabled,.settings-primary-button:disabled { cursor:not-allowed; opacity:0.5; }
  .directory-layout { display:grid; min-height:0; isolation:isolate; grid-template-columns:minmax(12.5rem,0.62fr) minmax(0,1.6fr); }
  .directory-panel { position:relative; z-index:1; min-width:0; min-height:0; }
  .directory-list-frame { position:relative; height:100%; min-height:0; }
  .directory-scroll { height:100%; min-height:0; overflow-y:auto; overscroll-behavior:contain; padding-right:0.75rem; }
  .detail-panel { position:relative; z-index:2; min-width:0; min-height:0; border-left:1px solid var(--border); padding-left:1rem; }
  .teammate-directory { display:grid; align-content:start; padding-block:0.15rem; }
  .teammate-directory > button { --teammate-row-surface:var(--background); display:grid; width:auto; grid-template-columns:auto minmax(0,1fr); align-items:center; gap:0.6rem; margin-inline:0.2rem; border-radius:0.5rem; padding:0.6rem; text-align:left; }
  .teammate-directory > button:hover { --teammate-row-surface:color-mix(in srgb,var(--accent) 45%,var(--background)); background:color-mix(in srgb,var(--accent) 45%,transparent); }
  .teammate-directory > button.active { --teammate-row-surface:var(--accent); background:var(--accent); color:var(--accent-foreground); }
  .teammate-directory > button:disabled { cursor:not-allowed; opacity:0.5; }
  .teammate-directory > button.archived-row:not(.active) { color:var(--muted-foreground); }
  .directory-avatar { position:relative; display:grid; }
  .archived-mark { position:absolute; right:-0.22rem; bottom:-0.22rem; display:grid; box-sizing:border-box; width:1.05rem; height:1.05rem; place-items:center; border:2px solid var(--teammate-row-surface); border-radius:0.35rem; background:#fff; color:#000; }
  .directory-summary { display:grid; min-width:0; }
  .directory-summary > strong { overflow:hidden; font-size:calc(0.8rem * var(--type-scale)); font-weight:600; text-overflow:ellipsis; white-space:nowrap; }
  .directory-summary > small { overflow:hidden; margin-top:0.05rem; color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); line-height:0.95rem; text-overflow:ellipsis; white-space:nowrap; }
  .editor-state { display:flex; flex-shrink:0; align-items:center; gap:0.32rem; color:var(--muted-foreground); font-size:calc(0.65rem * var(--type-scale)); white-space:nowrap; }
  .editor-state i { width:0.38rem; height:0.38rem; border-radius:999px; background:var(--status-tentative); }
  .editor-state[data-state="healthy"] i { background:var(--action-confirm); }
  .archived-state { color:var(--muted-foreground); }
  .draft-avatar { display:grid; width:2rem; height:2rem; place-items:center; border:1px dashed var(--border); border-radius:22%; color:var(--muted-foreground); }
  .editor-draft-avatar { width:2.375rem; height:2.375rem; }
  .directory-empty,.empty-detail { padding:0.75rem 0.5rem; color:var(--muted-foreground); font-size:calc(0.75rem * var(--type-scale)); line-height:1.15rem; }
  .directory-empty button { margin-top:0.45rem; color:var(--foreground); text-decoration:underline; text-underline-offset:0.15rem; }
  .empty-detail { padding:1rem; }
  .section-divider { height:1px; background:var(--border); transform:scaleY(0.5); }
  .teammate-editor { display:grid; width:100%; height:100%; min-height:0; grid-template-rows:minmax(0,1fr) auto; }
  .editor-scroll-frame { position:relative; min-height:0; }
  .editor-scroll { height:100%; min-height:0; overflow-y:auto; overscroll-behavior:contain; padding-right:0.75rem; }
  .editor-content { display:grid; align-content:start; gap:1rem; padding-bottom:0.75rem; }
  .editor-heading { display:grid; grid-template-columns:auto minmax(0,1fr); align-items:center; gap:0.65rem; padding-inline:0.25rem; }
  .editor-title { min-width:0; }
  .editor-name-line { display:flex; min-width:0; align-items:center; justify-content:space-between; gap:0.75rem; }
  .editor-heading h3 { font-size:calc(0.833333rem * var(--type-scale)); font-weight:600; }
  .editor-heading p { overflow:hidden; margin-top:0.08rem; color:var(--muted-foreground); font-size:calc(0.7rem * var(--type-scale)); text-overflow:ellipsis; white-space:nowrap; }
  .editor-section { display:grid; gap:0.8rem; }
  .section-heading { padding-inline:0.25rem; }
  .section-heading h4 { font-size:calc(0.8rem * var(--type-scale)); font-weight:600; }
  .field-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:0.7rem; padding-inline:0.25rem; }
  .field-grid .field { display:grid; min-width:0; align-content:start; gap:0.3rem; color:var(--muted-foreground); font-size:calc(0.7rem * var(--type-scale)); font-weight:500; }
  .required-marker { margin-left:0.15rem; color:var(--destructive); font-style:normal; }
  .field-grid .field.full { grid-column:1/-1; }
  .execution-model-field :global(.model-control) { z-index:2; max-width:100%; justify-self:start; }
  .execution-model-field :global(.model-trigger) { min-width:12rem; }
  .execution-approval-field :global(.access-control) { justify-self:start; }
  .execution-approval-field :global(.control-trigger) { min-width:12rem; max-width:100%; justify-content:center; }
  .field-grid input,.field-grid textarea { box-sizing:border-box; width:100%; min-width:0; appearance:none; border:1px solid var(--border); border-radius:0.375rem; background:var(--background); background-clip:padding-box; padding:0.45rem 0.55rem; color:var(--foreground); outline:none; font-weight:400; }
  .field-grid input:disabled,.field-grid textarea:disabled { cursor:not-allowed; background:color-mix(in srgb,var(--muted) 35%,var(--background)); color:var(--muted-foreground); }
  .field-grid input:focus,.field-grid textarea:focus { border-color:var(--ring); }
  .field-grid textarea { resize:vertical; }
  .field-error { color:var(--destructive); font-size:calc(0.666667rem * var(--type-scale)); font-weight:400; }
  fieldset { display:grid; gap:0.3rem; padding-inline:0.25rem; }
  fieldset legend { color:var(--foreground); font-size:calc(0.766667rem * var(--type-scale)); font-weight:500; }
  fieldset > p { color:var(--muted-foreground); font-size:calc(0.7rem * var(--type-scale)); }
  .folder-grants { display:grid; margin-top:0.25rem; }
  .folder-grants button { display:grid; grid-template-columns:1.25rem minmax(0,1fr) auto; align-items:center; gap:0.55rem; border-bottom:1px solid var(--border); padding:0.55rem 0.25rem; text-align:left; font-size:calc(0.75rem * var(--type-scale)); }
  .folder-grants button:last-child { border-bottom:0; }
  .folder-grants button:hover { background:color-mix(in srgb,var(--accent) 52%,transparent); }
  .grant-indicator { display:grid; width:1rem; height:1rem; place-items:center; border:1px solid var(--border); border-radius:0.25rem; color:var(--primary-foreground); }
  .folder-grants button[aria-pressed="true"] .grant-indicator { border-color:var(--primary); background:var(--primary); }
  .folder-grants small { color:var(--muted-foreground); font-size:calc(0.666667rem * var(--type-scale)); }
  footer { display:grid; flex-shrink:0; grid-template-columns:minmax(0,1fr) auto; align-items:center; gap:0.75rem; border-top:1px solid var(--border); padding:0.5rem 0.25rem 0; }
  .footer-leading,.footer-actions { display:flex; min-width:0; align-items:center; gap:0.55rem; }
  .footer-actions { justify-content:flex-end; }
  .delete-button { border-color:var(--destructive); background:var(--destructive); color:var(--destructive-foreground); }
  .delete-button:hover:not(:disabled) { background:color-mix(in srgb,var(--destructive) 90%,transparent); }
  .footer-note,.footer-error { min-width:0; overflow:hidden; color:var(--muted-foreground); font-size:calc(0.65rem * var(--type-scale)); text-overflow:ellipsis; white-space:nowrap; }
  .footer-error { color:var(--destructive); }
  .form-error { color:var(--destructive); font-size:calc(0.7rem * var(--type-scale)); }
  .saved { color:var(--action-confirm); font-size:calc(0.7rem * var(--type-scale)); }
  @media (max-width:700px) { .directory-layout { min-height:0; grid-template-columns:1fr; grid-template-rows:minmax(4rem,30%) minmax(0,1fr); }.directory-panel { padding-bottom:0.75rem; }.detail-panel { border-top:1px solid var(--border); border-left:0; padding:1rem 0 0; }.field-grid { grid-template-columns:1fr; }.field-grid .field.full { grid-column:auto; } footer { grid-template-columns:1fr; }.footer-actions { justify-content:stretch; }.footer-actions > button { width:100%; justify-content:center; } }
</style>
