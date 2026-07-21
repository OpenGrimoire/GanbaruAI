<script lang="ts">
  import FolderSearch from "@lucide/svelte/icons/folder-search";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Settings from "@lucide/svelte/icons/settings";
  import type { InteractionMode, ModelId, ProviderInstanceId, SafetyMode } from "$lib/chat/contracts";
  import { resolveChatFirstUseState } from "$lib/chat/shell-model";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";

  const { t } = getLocalization();
  const chat = getChat();
  const settings = getSettingsLauncher();
  let providerId = $state<ProviderInstanceId | null>(null);
  let modelId = $state<ModelId | "provider_managed" | null>(null);
  let safetyMode = $state<SafetyMode | null>(null);
  let interactionMode = $state<InteractionMode | null>(null);
  let restoredSelectionKey = $state<string | null>(null);
  let operationError = $state<string | null>(null);
  const firstUse = $derived(resolveChatFirstUseState({ providers: chat.settings?.providerInstances ?? [], workspaces: chat.workspaces, selectedWorkspaceId: chat.selectedWorkspaceId, selectedThreadId: chat.selectedThreadId, threads: [...chat.activeThreads, ...chat.archivedThreads] }));
  const healthyProviders = $derived((chat.settings?.providerInstances ?? []).filter((entry) => entry.configuration.enabled && entry.lastProbe?.state === "healthy"));
  const selectedProvider = $derived(healthyProviders.find((entry) => entry.configuration.instanceId === providerId) ?? null);
  const models = $derived(selectedProvider?.modelCatalog?.models.filter((model) => selectedProvider.configuration.visibleModelIds.length === 0 || selectedProvider.configuration.visibleModelIds.includes(model.id) || model.id === modelId) ?? []);

  $effect(() => {
    const workspaceId = chat.selectedWorkspaceId;
    if (!workspaceId || providerId) return;
    const preference = chat.settings?.configuration.workspaceProviderPreferences[workspaceId];
    if (preference && healthyProviders.some((entry) => entry.configuration.instanceId === preference)) providerId = preference;
  });

  $effect(() => {
    const workspaceId = chat.selectedWorkspaceId;
    const instanceId = providerId;
    if (!workspaceId || !instanceId) {
      restoredSelectionKey = null;
      return;
    }
    const key = `${workspaceId}:${instanceId}`;
    if (restoredSelectionKey === key) return;
    const remembered = chat.settings?.configuration.rememberedSelections.find((entry) => entry.workspaceId === workspaceId && entry.providerInstanceId === instanceId);
    if (!remembered) return;
    modelId = remembered.providerManagedModel ? "provider_managed" : remembered.modelId;
    safetyMode = remembered.safetyMode;
    interactionMode = remembered.interactionMode;
    restoredSelectionKey = key;
  });

  function run(action: () => Promise<unknown>): void {
    operationError = null;
    void action().catch((error: unknown) => {
      operationError = error instanceof Error ? error.message : String(error);
    });
  }

  function bindingLabel(status: "available" | "unbound" | "missing" | "repository_mismatch"): string {
    switch (status) {
      case "available": return t("settings.chat.workspaces.available");
      case "missing": return t("settings.chat.workspaces.missing");
      case "repository_mismatch": return t("settings.chat.workspaces.mismatch");
      default: return t("settings.chat.workspaces.unbound");
    }
  }
</script>

<div class="flex min-h-0 flex-1 items-center justify-center overflow-y-auto p-4">
  <section class="flex w-full max-w-2xl flex-col items-center text-center">
    {#if operationError}<p role="alert" class="mb-3 text-sm text-destructive">{operationError}</p>{/if}
    <div class="mb-4 flex size-12 items-center justify-center rounded-2xl border border-border bg-card"><MessageSquare size={22} /></div>
    {#if firstUse.kind === "no_provider"}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.noProviderTitle")}</h2><p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.firstUse.noProviderDescription")}</p><div class="mt-5 flex flex-wrap justify-center gap-2"><button type="button" class="chat-primary-button" onclick={() => settings.open("chat", { chatSubsection: "providers" })}><Settings size={15} />{t("chat.firstUse.setUpProvider")}</button></div><div class="mt-5 flex flex-wrap justify-center gap-2 text-xs text-muted-foreground"><span>Codex</span><span>Claude</span><span>Cursor</span><span>OpenCode</span></div>
    {:else if firstUse.kind === "no_workspace"}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.noWorkspaceTitle")}</h2><p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.firstUse.noWorkspaceDescription")}</p><div class="mt-5 flex flex-wrap justify-center gap-2"><button type="button" class="chat-primary-button" onclick={() => settings.open("chat", { chatSubsection: "workspaces" })}>{t("chat.firstUse.bindProject")}</button><button type="button" class="chat-secondary-button" onclick={() => settings.open("chat", { chatSubsection: "workspaces" })}>{t("chat.firstUse.useStandalone")}</button></div>
    {:else if firstUse.kind === "select_workspace"}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.selectWorkspaceTitle")}</h2><p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.firstUse.selectWorkspaceDescription")}</p><div class="mt-5 grid w-full max-w-md gap-2">{#each chat.workspaces.filter((entry) => entry.workspace.archivedAt === null) as workspace}<button type="button" class="rounded-lg border border-border bg-card p-3 text-left hover:bg-accent" onclick={() => chat.selectWorkspace(workspace.workspace.id)}><span class="block text-sm font-medium">{workspace.workspace.displayName}</span><span class="block text-xs text-muted-foreground">{bindingLabel(workspace.bindingStatus)}</span></button>{/each}</div>
    {:else if firstUse.kind === "missing_binding"}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.missingBindingTitle")}</h2><p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.firstUse.missingBindingDescription", firstUse.workspace.workspace.displayName)}</p>{#if firstUse.workspace.workspace.repositoryIdentity}<code class="mt-3 max-w-full truncate text-xs text-muted-foreground">{firstUse.workspace.workspace.repositoryIdentity}</code>{/if}<button type="button" class="chat-primary-button mt-5" onclick={() => run(() => chat.rebindWorkspace(firstUse.workspace.workspace.id, t("chat.firstUse.chooseWorkspaceFolder")))}><FolderSearch size={15} />{t("chat.firstUse.locateFolder")}</button>
    {:else if firstUse.kind === "provider_unavailable"}
      {@const unavailableProvider = firstUse.provider}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.unavailableTitle")}</h2><p class="mt-2 text-sm text-muted-foreground">{unavailableProvider?.lastProbe?.detail ?? t("chat.status.providerUnavailable")}</p><div class="mt-5 flex gap-2">{#if unavailableProvider}<button type="button" class="chat-primary-button" onclick={() => run(() => chat.probeProvider(unavailableProvider.configuration.instanceId))}><RefreshCw size={15} />{t("chat.firstUse.retryProvider")}</button>{/if}<button type="button" class="chat-secondary-button" onclick={() => settings.open("chat", { chatSubsection: "providers" })}><Settings size={15} />{t("chat.firstUse.openSettings")}</button></div>
    {:else if firstUse.kind === "archived_thread"}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.archivedTitle")}</h2><p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.firstUse.archivedDescription")}</p><button type="button" class="chat-primary-button mt-5" onclick={() => run(() => chat.restoreThread(firstUse.thread))}>{t("chat.restore")}</button>
    {:else if firstUse.kind === "no_thread"}
      <h2 class="text-xl font-semibold">{t("chat.firstUse.noThreadTitle")}</h2><p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.firstUse.noThreadDescription")}</p>
      <div class="mt-6 grid w-full gap-3 text-left sm:grid-cols-2">
        <label class="chat-choice"><span>{t("chat.hero.workspace")}</span><select value={chat.selectedWorkspaceId} onchange={(event) => chat.selectWorkspace(event.currentTarget.value)}>{#each chat.workspaces.filter((entry) => entry.workspace.archivedAt === null) as workspace}<option value={workspace.workspace.id}>{workspace.workspace.displayName}</option>{/each}</select></label>
        <label class="chat-choice"><span>{t("chat.hero.provider")}</span><select bind:value={providerId}><option value={null}>{t("chat.hero.chooseProvider")}</option>{#each healthyProviders as provider}<option value={provider.configuration.instanceId}>{provider.configuration.label}{#if chat.settings?.configuration.workspaceProviderPreferences[chat.selectedWorkspaceId ?? ""] === provider.configuration.instanceId} ({t("chat.hero.workspacePreference")}){/if}</option>{/each}</select></label>
        <label class="chat-choice"><span>{t("chat.hero.model")}</span><select bind:value={modelId} disabled={!providerId}><option value={null}>{t("chat.hero.chooseModel")}</option><option value="provider_managed">{t("chat.hero.providerManaged")}</option>{#each models as model}<option value={model.id} disabled={model.availability === "unavailable" || model.availability === "deprecated"}>{model.displayName}{#if selectedProvider && selectedProvider.configuration.visibleModelIds.length > 0 && !selectedProvider.configuration.visibleModelIds.includes(model.id)} ({t("settings.chat.models.hidden")}){/if}</option>{/each}</select></label>
        <label class="chat-choice"><span>{t("chat.hero.safety")}</span><select bind:value={safetyMode}><option value={null}>{t("chat.hero.chooseSafety")}</option><option value="supervised">{t("chat.hero.supervised")}</option><option value="auto_accept_edits">{t("chat.hero.autoAccept")}</option><option value="full_access">{t("chat.hero.fullAccess")}</option></select></label>
        <label class="chat-choice sm:col-span-2"><span>{t("chat.hero.interaction")}</span><select bind:value={interactionMode}><option value={null}>{t("chat.hero.chooseInteraction")}</option><option value="build">{t("chat.hero.build")}</option><option value="plan">{t("chat.hero.plan")}</option></select></label>
      </div>
      {#if restoredSelectionKey}<p class="mt-3 text-xs text-muted-foreground">{t("chat.hero.rememberedSelection")}</p>{/if}
      <p class="mt-5 text-xs text-muted-foreground">{t("chat.hero.composerLater")}</p>
    {:else}
      <h2 class="text-lg font-semibold">{firstUse.thread.title}</h2>
      <p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.hero.conversationPending")}</p>
    {/if}
  </section>
</div>

<style>
  :global(.chat-primary-button), :global(.chat-secondary-button) { display: inline-flex; min-height: 2.25rem; align-items: center; justify-content: center; gap: 0.4rem; border-radius: 0.375rem; padding: 0.4rem 0.8rem; font-size: 0.8rem; font-weight: 600; }
  :global(.chat-primary-button) { background: var(--primary); color: var(--primary-foreground); }
  :global(.chat-secondary-button) { border: 1px solid var(--border); background: var(--background); color: var(--foreground); }
  .chat-choice { display: flex; flex-direction: column; gap: 0.375rem; color: var(--muted-foreground); font-size: 0.733333rem; }
  .chat-choice select { min-height: 2.5rem; border-radius: 0.5rem; border: 1px solid var(--border); background: var(--card); padding: 0.4rem 0.6rem; color: var(--foreground); }
</style>
