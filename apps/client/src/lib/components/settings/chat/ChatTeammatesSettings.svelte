<script lang="ts">
  import Archive from "@lucide/svelte/icons/archive";
  import Bot from "@lucide/svelte/icons/bot";
  import Check from "@lucide/svelte/icons/check";
  import Plus from "@lucide/svelte/icons/plus";
  import Save from "@lucide/svelte/icons/save";
  import * as chatApi from "$lib/api/chat";
  import {
    copyModelOptionSelections,
    copyVersionedJson,
    defaultModelOptions,
    resolveDefaultProviderModel,
  } from "$lib/chat/composer-model";
  import type {
    ChatAiTeammateRead,
    ChatApprovalPolicy,
    ChatConversationMembershipRead,
    ModelOptionSelection,
    ProviderInstanceRead,
    ProviderModel,
  } from "$lib/chat/contracts";
  import { recommendedProviderModel } from "$lib/chat/composer-model";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import CustomSelect from "$lib/components/settings/CustomSelect.svelte";
  import ChatParticipantAvatar from "$lib/components/chat/ChatParticipantAvatar.svelte";

  const chat = getChat();
  const { t } = getLocalization();
  let selectedId = $state<string | null>(null);
  let creating = $state(false);
  let displayName = $state("Ganbaru");
  let handle = $state("ganbaru");
  let purpose = $state("");
  let instructions = $state("");
  let providerId = $state("");
  let modelId = $state("");
  let modelOptions = $state<ModelOptionSelection[]>([]);
  let effort = $state("medium");
  let approvalPolicy = $state<ChatApprovalPolicy>("ask_for_approval");
  let channelId = $state("");
  let folderIds = $state<string[]>([]);
  let defaultFolderId = $state("");
  let membershipRevision = $state<number | null>(null);
  let loadingMembership = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let savedNotice = $state(false);
  let autoStarted = false;
  const selected = $derived(chat.teammates.find((teammate) => teammate.participant.id === selectedId) ?? null);
  const providers = $derived(chat.settings?.providerInstances ?? []);
  const selectedProvider = $derived(providers.find((provider) => provider.configuration.instanceId === providerId) ?? null);
  const models = $derived(selectedProvider?.modelCatalog?.models.filter((model) => model.availability !== "deprecated") ?? []);
  const selectedModel = $derived(models.find((model) => model.id === modelId) ?? null);
  const channel = $derived(chat.activeChannels.find((entry) => entry.id === channelId) ?? chat.selectedChannel);
  const projectFolders = $derived(chat.workingFolders.filter((entry) => (
    entry.workingFolder.projectId === channel?.projectId && entry.workingFolder.archivedAt === null
  )));
  const providerOptions = $derived(providers.map((provider) => ({
    value: provider.configuration.instanceId,
    label: provider.configuration.label,
    description: provider.lastProbe?.state === "healthy"
      ? t("settings.chat.providers.healthy")
      : t("settings.chat.providers.unavailable"),
  })));
  const modelSelectOptions = $derived(models.map((model) => ({ value: model.id, label: model.displayName })));
  const channelOptions = $derived(chat.activeChannels.map((entry) => ({ value: entry.id, label: `#${entry.name}` })));
  const folderOptions = $derived(projectFolders.map((entry) => ({
    value: entry.workingFolder.id,
    label: entry.workingFolder.displayName,
  })));
  const effortOptions = $derived(modelEffortOptions(selectedModel));
  const reviewLines = $derived([
    t("settings.chat.teammates.reviewIdentity", displayName || t("settings.chat.teammates.unnamed"), handle || "…"),
    t("settings.chat.teammates.reviewChannel", channel?.name ?? t("common.none")),
    t("settings.chat.teammates.reviewFolders", folderIds.map((id) => folderOptions.find((entry) => entry.value === id)?.label ?? id).join(", ") || t("common.none")),
    t("settings.chat.teammates.reviewProvider", selectedProvider?.configuration.label ?? t("common.none"), selectedModel?.displayName ?? t("settings.chat.teammates.providerManaged")),
    t("settings.chat.teammates.reviewApproval", approvalLabel(approvalPolicy)),
  ]);

  $effect(() => {
    if (!creating && !selectedId && chat.teammates[0]) selectedId = chat.teammates[0].participant.id;
  });

  $effect(() => {
    if (autoStarted || creating || chat.teammates.length > 0 || !resolveDefaultProviderModel(providers)) return;
    autoStarted = true;
    beginCreate();
  });

  $effect(() => {
    const teammate = selected;
    if (!teammate || creating) return;
    displayName = teammate.participant.displayName;
    handle = teammate.participant.handle ?? "";
    purpose = teammate.purpose;
    instructions = teammate.instructions;
    providerId = teammate.latestPolicy?.providerInstanceId ?? "";
    modelId = teammate.latestPolicy?.modelId ?? "";
    modelOptions = copyModelOptionSelections(teammate.latestPolicy?.modelOptions ?? []);
    effort = teammate.latestPolicy?.effort ?? selectedEffort(modelOptions) ?? "medium";
    channelId = chat.selectedChannelId ?? chat.activeChannels[0]?.id ?? "";
    void loadMembership(teammate, channelId);
  });

  function beginCreate(): void {
    creating = true;
    selectedId = null;
    displayName = "Ganbaru";
    handle = "ganbaru";
    purpose = t("settings.chat.teammates.defaultPurpose");
    instructions = "";
    approvalPolicy = "ask_for_approval";
    channelId = chat.selectedChannelId ?? chat.activeChannels[0]?.id ?? "";
    const resolved = resolveDefaultProviderModel(providers);
    providerId = resolved?.provider.configuration.instanceId ?? "";
    modelId = resolved?.model?.id ?? "";
    modelOptions = copyModelOptionSelections(resolved?.options ?? []);
    effort = selectedEffort(modelOptions) ?? "medium";
    const managed = projectFolders.find((entry) => entry.workingFolder.kind === "managed") ?? projectFolders[0];
    defaultFolderId = managed?.workingFolder.id ?? "";
    folderIds = defaultFolderId ? [defaultFolderId] : [];
    membershipRevision = null;
    error = null;
    savedNotice = false;
  }

  async function loadMembership(teammate: ChatAiTeammateRead, requestedChannelId: string): Promise<void> {
    if (!requestedChannelId) return;
    loadingMembership = true;
    try {
      const memberships = await chatApi.listChatChannelMemberships(requestedChannelId);
      if (selectedId !== teammate.participant.id || channelId !== requestedChannelId) return;
      const membership = memberships.find((entry) => entry.participant.id === teammate.participant.id) ?? null;
      applyMembership(membership);
    } finally {
      loadingMembership = false;
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

  function selectProvider(value: string): void {
    providerId = value;
    const provider = providers.find((entry) => entry.configuration.instanceId === value) ?? null;
    const resolved = provider ? strongestModel(provider) : null;
    modelId = resolved?.id ?? "";
    modelOptions = resolved ? defaultModelOptions(resolved.options) : [];
    effort = selectedEffort(modelOptions) ?? "medium";
  }

  function selectModel(value: string): void {
    modelId = value;
    const model = models.find((entry) => entry.id === value) ?? null;
    modelOptions = model ? defaultModelOptions(model.options) : [];
    effort = selectedEffort(modelOptions) ?? "medium";
  }

  function selectEffort(value: string): void {
    effort = value;
    const key = effortOptionKey(selectedModel);
    if (!key) return;
    modelOptions = [
      ...modelOptions.filter((option) => option.key !== key),
      { key, value: { kind: "choice", value } },
    ];
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
    if (saving || !displayName.trim() || !handle.trim() || !providerId || !channelId || !defaultFolderId) return;
    saving = true;
    error = null;
    savedNotice = false;
    try {
      const policy = {
        providerInstanceId: providerId,
        providerManagedModel: !modelId,
        modelId: modelId || null,
        modelOptions: copyModelOptionSelections(modelOptions),
        effort: effort || null,
        speed: null,
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
          handle: handle.trim(),
          avatar: { schemaVersion: 1, value: { kind: "initials" } },
          purpose: purpose.trim(),
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
          handle: handle.trim(),
          avatar: copyVersionedJson(selected.participant.avatar),
          purpose: purpose.trim(),
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
      savedNotice = true;
    } catch (cause: unknown) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      saving = false;
    }
  }

  async function archiveSelected(): Promise<void> {
    if (!selected) return;
    await chatApi.archiveChatTeammate(selected.participant.id, selected.participant.revision, true);
    selectedId = null;
    await chat.refreshTeammates();
  }

  function approvalLabel(policy: ChatApprovalPolicy): string {
    const labels: Record<ChatApprovalPolicy, string> = {
      ask_for_approval: t("settings.chat.teammates.askApproval"),
      approve_for_me: t("settings.chat.teammates.approveForMe"),
      full_access: t("settings.chat.teammates.fullAccess"),
      custom: t("settings.chat.teammates.customApproval"),
    };
    return labels[policy];
  }

  function strongestModel(provider: ProviderInstanceRead): ProviderModel | null {
    return recommendedProviderModel(provider);
  }

  function effortOptionKey(model: ProviderModel | null): string | null {
    const definition = model?.options.find((option) => (
      option.kind === "choice" && /effort|reasoning/iu.test(`${option.key} ${option.label}`)
    ));
    return definition?.key ?? null;
  }

  function modelEffortOptions(model: ProviderModel | null): Array<{ value: string; label: string }> {
    const key = effortOptionKey(model);
    const definition = model?.options.find((option) => option.key === key);
    if (definition?.kind !== "choice") return [{ value: "medium", label: "Medium" }];
    return definition.options.map((option) => ({ value: option.value, label: option.label }));
  }

  function selectedEffort(options: readonly ModelOptionSelection[]): string | null {
    const selection = options.find((option) => /effort|reasoning/iu.test(option.key));
    return selection?.value.kind === "choice" ? selection.value.value : null;
  }
</script>

<section class="teammate-settings" data-chat-settings-subsection="teammates">
  <header class="directory-header"><div><h2>{t("settings.chat.teammates.heading")}</h2><p>{t("settings.chat.teammates.description")}</p></div><button type="button" class="chat-settings-button" onclick={beginCreate}><Plus size={13} />{t("settings.chat.teammates.add")}</button></header>
  <div class="directory-layout">
    <nav aria-label={t("settings.chat.teammates.directoryLabel")}>
      {#if chat.teammates.length === 0}<p>{t("settings.chat.teammates.empty")}</p>{/if}
      {#each chat.teammates as teammate (teammate.participant.id)}
        <button type="button" class:active={!creating && selectedId === teammate.participant.id} onclick={() => { creating = false; selectedId = teammate.participant.id; }}>
          <ChatParticipantAvatar participant={teammate.participant} size={30} />
          <span><strong>{teammate.participant.displayName}</strong><small>{teammate.purpose || `@${teammate.participant.handle}`}</small></span>
          <i data-state={teammate.configurationState}>{teammate.configurationState === "healthy" ? t("settings.chat.teammates.available") : t("settings.chat.teammates.needsSetup")}</i>
          <small>{t("settings.chat.teammates.channels", teammate.channelCount)}</small>
        </button>
      {/each}
    </nav>
    {#if creating || selected}
      <form class="teammate-editor" onsubmit={(event) => { event.preventDefault(); void save(); }}>
        <div class="editor-heading"><Bot size={18} /><div><h3>{creating ? t("settings.chat.teammates.createHeading") : displayName}</h3><p>{t("settings.chat.teammates.editorDescription")}</p></div></div>
        <div class="field-grid">
          <label><span>{t("settings.chat.teammates.name")}</span><input bind:value={displayName} maxlength="160" required /></label>
          <label><span>{t("settings.chat.teammates.handle")}</span><div class="handle-input"><span>@</span><input bind:value={handle} maxlength="64" pattern="[a-z0-9][a-z0-9_.-]*" required /></div></label>
          <label class="full"><span>{t("settings.chat.teammates.purpose")}</span><input bind:value={purpose} maxlength="1000" /></label>
          <label class="full"><span>{t("settings.chat.teammates.instructions")}</span><textarea bind:value={instructions} maxlength="65536" rows="4"></textarea></label>
          <label><span>{t("settings.chat.teammates.provider")}</span><CustomSelect inline class="w-full" value={providerId} options={providerOptions} ariaLabel={t("settings.chat.teammates.provider")} onChange={selectProvider} /></label>
          <label><span>{t("settings.chat.teammates.model")}</span><CustomSelect inline class="w-full" value={modelId} options={modelSelectOptions} ariaLabel={t("settings.chat.teammates.model")} onChange={selectModel} /></label>
          <label><span>{t("settings.chat.teammates.effort")}</span><CustomSelect inline class="w-full" value={effort} options={effortOptions} ariaLabel={t("settings.chat.teammates.effort")} onChange={selectEffort} /></label>
          <label><span>{t("settings.chat.teammates.approval")}</span><CustomSelect inline class="w-full" value={approvalPolicy} options={(["ask_for_approval", "approve_for_me", "full_access", "custom"] as const).map((value) => ({ value, label: approvalLabel(value) }))} ariaLabel={t("settings.chat.teammates.approval")} onChange={(value) => { approvalPolicy = value as ChatApprovalPolicy; }} /></label>
          <label><span>{t("settings.chat.teammates.channel")}</span><CustomSelect inline class="w-full" value={channelId} options={channelOptions} ariaLabel={t("settings.chat.teammates.channel")} onChange={(value) => { channelId = value; if (selected) void loadMembership(selected, value); }} /></label>
          <label><span>{t("settings.chat.teammates.defaultFolder")}</span><CustomSelect inline class="w-full" value={defaultFolderId} options={folderOptions} ariaLabel={t("settings.chat.teammates.defaultFolder")} onChange={(value) => { defaultFolderId = value; if (!folderIds.includes(value)) folderIds = [...folderIds, value]; }} /></label>
        </div>
        <fieldset disabled={loadingMembership}><legend>{t("settings.chat.teammates.allowedFolders")}</legend><div class="folder-grants">{#each folderOptions as folder (folder.value)}<label><input type="checkbox" checked={folderIds.includes(folder.value)} onchange={() => toggleFolder(folder.value)} /><span>{folder.label}</span>{#if folder.value === defaultFolderId}<small>{t("settings.chat.teammates.default")}</small>{/if}</label>{/each}</div></fieldset>
        <section class="grant-review" aria-label={t("settings.chat.teammates.reviewHeading")}><h4>{t("settings.chat.teammates.reviewHeading")}</h4><ul>{#each reviewLines as line}<li><Check size={13} />{line}</li>{/each}</ul></section>
        {#if error}<p class="form-error" role="alert">{error}</p>{/if}{#if savedNotice}<p class="saved" role="status">{t("settings.chat.teammates.saved")}</p>{/if}
        <footer>{#if selected && !creating}<button type="button" class="archive-button" onclick={() => void archiveSelected()}><Archive size={13} />{t("settings.chat.teammates.archive")}</button>{/if}<span></span><button type="submit" class="setup-primary-button" disabled={saving || !providerId || !channelId || !defaultFolderId}><Save size={13} />{saving ? t("settings.chat.teammates.saving") : t("settings.chat.teammates.save")}</button></footer>
      </form>
    {/if}
  </div>
</section>

<style>
  .teammate-settings { display:grid; gap:1rem; }.directory-header { display:flex; flex-wrap:wrap; align-items:start; justify-content:space-between; gap:0.75rem; padding-inline:0.25rem; }.directory-header h2 { font-size:0.87rem; font-weight:600; }.directory-header p { margin-top:0.2rem; color:var(--muted-foreground); font-size:0.78rem; }
  .directory-layout { display:grid; grid-template-columns:minmax(12rem,0.7fr) minmax(18rem,1.5fr); overflow:hidden; border:1px solid var(--border); border-radius:0.65rem; background:color-mix(in srgb,var(--card) 38%,transparent); }
  nav { min-height:24rem; border-right:1px solid var(--border); padding:0.4rem; } nav > p { padding:1rem; color:var(--muted-foreground); font-size:0.75rem; } nav > button { display:grid; width:100%; grid-template-columns:auto minmax(0,1fr) auto; gap:0.2rem 0.5rem; align-items:center; border-radius:0.45rem; padding:0.45rem; text-align:left; } nav > button.active { background:var(--accent); } nav > button > span { display:grid; min-width:0; } nav strong,nav small { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; } nav strong { font-size:0.76rem; } nav small { color:var(--muted-foreground); font-size:0.64rem; } nav i { border-radius:999px; background:var(--accent); padding:0.1rem 0.3rem; color:var(--muted-foreground); font-size:0.58rem; font-style:normal; } nav i[data-state="healthy"] { color:var(--action-confirm); } nav > button > small:last-child { grid-column:2; }
  .teammate-editor { display:grid; align-content:start; gap:0.85rem; padding:1rem; }.editor-heading { display:flex; align-items:start; gap:0.55rem; }.editor-heading h3 { font-size:0.86rem; font-weight:600; }.editor-heading p { margin-top:0.15rem; color:var(--muted-foreground); font-size:0.7rem; }
  .field-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:0.65rem; }.field-grid label { display:grid; align-content:start; gap:0.3rem; color:var(--muted-foreground); font-size:0.68rem; font-weight:500; }.field-grid label.full { grid-column:1/-1; }.field-grid input,.field-grid textarea { min-width:0; border:1px solid var(--border); border-radius:0.4rem; background:var(--background); padding:0.4rem 0.5rem; color:var(--foreground); outline:none; font-weight:400; }.field-grid textarea { resize:vertical; }.handle-input { display:flex; align-items:center; border:1px solid var(--border); border-radius:0.4rem; background:var(--background); padding-left:0.45rem; }.handle-input input { border:0; background:transparent; }
  fieldset { display:grid; gap:0.35rem; } legend { color:var(--muted-foreground); font-size:0.68rem; font-weight:500; }.folder-grants { display:flex; flex-wrap:wrap; gap:0.35rem; }.folder-grants label { display:flex; align-items:center; gap:0.3rem; border:1px solid var(--border); border-radius:999px; padding:0.25rem 0.45rem; font-size:0.68rem; }.folder-grants small { color:var(--muted-foreground); }
  .grant-review { border:1px solid color-mix(in srgb,var(--primary) 24%,var(--border)); border-radius:0.5rem; background:color-mix(in srgb,var(--primary) 5%,transparent); padding:0.65rem; }.grant-review h4 { font-size:0.72rem; font-weight:600; }.grant-review ul { display:grid; gap:0.25rem; margin-top:0.4rem; }.grant-review li { display:flex; align-items:start; gap:0.35rem; color:var(--muted-foreground); font-size:0.68rem; }.grant-review li :global(svg) { flex:0 0 auto; color:var(--action-confirm); }
  footer { display:flex; align-items:center; gap:0.5rem; } footer span { flex:1; }.archive-button { display:flex; align-items:center; gap:0.3rem; color:var(--destructive); font-size:0.7rem; }.form-error { color:var(--destructive); font-size:0.7rem; }.saved { color:var(--action-confirm); font-size:0.7rem; }
  @media (max-width:700px) { .directory-layout { grid-template-columns:1fr; }.directory-layout nav { min-height:0; max-height:12rem; overflow-y:auto; border-right:0; border-bottom:1px solid var(--border); }.field-grid { grid-template-columns:1fr; }.field-grid label.full { grid-column:auto; } }
</style>
