<script lang="ts">
  import Bot from "@lucide/svelte/icons/bot";
  import Boxes from "@lucide/svelte/icons/boxes";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import ShieldCheck from "@lucide/svelte/icons/shield-check";
  import SlidersHorizontal from "@lucide/svelte/icons/sliders-horizontal";
  import type { Component } from "svelte";
  import type { ProviderRefreshResult } from "$lib/chat/contracts";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { cn } from "$lib/utils";
  import type { ChatProviderSetupTarget, ChatSettingsSubsection } from "./types";
  import ProviderCard from "./chat/ProviderCard.svelte";
  import ChatModelsSettings from "./chat/ChatModelsSettings.svelte";
  import ChatPermissionsSettings from "./chat/ChatPermissionsSettings.svelte";
  import ChatBehaviorSettings from "./chat/ChatBehaviorSettings.svelte";
  import ChatTeammatesSettings from "./chat/ChatTeammatesSettings.svelte";

  let {
    initialSubsection,
    onOpenProviderSetup = () => {},
  }: {
    initialSubsection?: ChatSettingsSubsection;
    onOpenProviderSetup?: (target: ChatProviderSetupTarget) => void;
  } = $props();
  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let busyIds = $state<string[]>([]);
  let refreshingAll = $state(false);
  let refreshResult = $state<ProviderRefreshResult | null>(null);
  let providersRefreshedOnOpen = $state(false);
  let error = $state<string | null>(null);
  let removeId = $state<string | null>(null);
  let activeTab = $state<ChatSettingsSubsection>("teammates");
  const removeProvider = $derived(chat.settings?.providerInstances.find((entry) => entry.configuration.instanceId === removeId) ?? null);
  const tabs: ReadonlyArray<{
    id: ChatSettingsSubsection;
    label: () => string;
    icon: Component;
  }> = [
    { id: "teammates", label: () => t("settings.chat.teammates.heading"), icon: Bot },
    { id: "providers", label: () => t("settings.chat.providers.heading"), icon: Bot },
    { id: "models", label: () => t("settings.chat.models.heading"), icon: Boxes },
    { id: "permissions", label: () => t("settings.chat.permissions.heading"), icon: ShieldCheck },
    { id: "behavior", label: () => t("settings.chat.behavior.heading"), icon: SlidersHorizontal },
  ];

  $effect.pre(() => {
    if (initialSubsection) activeTab = initialSubsection;
  });

  $effect(() => {
    void chat.ensureLoaded().catch((cause) => { error = errorMessage(cause); });
  });

  $effect(() => {
    if (activeTab !== "providers" || !chat.settings || providersRefreshedOnOpen) return;
    providersRefreshedOnOpen = true;
    void refreshAll();
  });

  function setBusy(instanceId: string, busy: boolean): void {
    busyIds = busy ? [...new Set([...busyIds, instanceId])] : busyIds.filter((id) => id !== instanceId);
  }

  async function perform(instanceId: string, action: () => Promise<void>): Promise<void> {
    setBusy(instanceId, true);
    error = null;
    try { await action(); } catch (cause: unknown) { error = errorMessage(cause); } finally { setBusy(instanceId, false); }
  }

  async function refreshAll(): Promise<void> {
    if (refreshingAll) return;
    refreshingAll = true;
    refreshResult = null;
    error = null;
    try {
      refreshResult = await chat.refreshAllProviders();
    } catch (cause: unknown) {
      error = errorMessage(cause);
    } finally {
      refreshingAll = false;
    }
  }

  function refreshSummary(result: ProviderRefreshResult): string {
    return t(
      "settings.chat.providers.refreshSummary",
      formatNumber(localization.locale, result.familiesScanned),
      formatNumber(localization.locale, result.providersChecked),
      formatNumber(localization.locale, result.providersDiscovered),
      formatNumber(localization.locale, result.issues),
    );
  }

  async function confirmRemove(): Promise<void> {
    if (!removeProvider) return;
    const id = removeProvider.configuration.instanceId;
    removeId = null;
    await perform(id, async () => {
      const result = await chat.removeProvider(id);
      if (result.credentialCleanupFailed) error = t("settings.chat.providers.credentialCleanupFailed");
    });
  }

  function errorMessage(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }
</script>

<div class="flex flex-col gap-6 pb-4">
  <div
    class="grid grid-cols-2 gap-1 rounded-md border border-border bg-card p-1 min-[480px]:grid-cols-3 min-[700px]:grid-cols-5 dark:bg-transparent"
    role="tablist"
    aria-label={t("settings.chat.tabLabel")}
  >
    {#each tabs as tab (tab.id)}
      {@const Icon = tab.icon}
      {@const active = activeTab === tab.id}
      <button
        type="button"
        role="tab"
        aria-selected={active}
        class={cn(
          "flex min-h-8 items-center justify-center gap-1.5 rounded-sm px-2 text-center text-[0.8rem] font-medium text-muted-foreground",
          active && "bg-background text-foreground dark:bg-foreground/5",
        )}
        onclick={() => { activeTab = tab.id; }}
      >
        <Icon size={13} strokeWidth={1.75} />
        <span>{tab.label()}</span>
      </button>
    {/each}
  </div>

  {#if chat.loading}
    <div class="rounded-lg border border-dashed border-border p-6 text-center text-sm text-muted-foreground">{t("common.loading")}</div>
  {:else if activeTab === "teammates"}
    <ChatTeammatesSettings />
  {:else if activeTab === "providers"}
    <section class="flex flex-col gap-4" data-chat-settings-subsection="providers">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div class="min-w-0 px-1"><h2 class="text-[0.866667rem] font-semibold text-foreground">{t("settings.chat.providers.heading")}</h2><p class="mt-1 text-[0.8rem] text-muted-foreground">{t("settings.chat.providers.description")}</p></div>
        <div class="flex gap-2"><button type="button" class="chat-settings-button" disabled={refreshingAll} onclick={() => void refreshAll()}><RefreshCw size={13} class={refreshingAll ? "animate-spin" : undefined} />{refreshingAll ? t("settings.chat.providers.scanning") : t("settings.chat.providers.refreshAll")}</button><button type="button" class="chat-settings-button" onclick={() => onOpenProviderSetup({ mode: "create" })}><Plus size={13} />{t("settings.chat.providers.add")}</button></div>
      </div>
      {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
      {#if refreshResult}<p role="status" class={cn("px-1 text-xs text-muted-foreground", refreshResult.issues > 0 && "text-status-tentative")}>{refreshSummary(refreshResult)}</p>{/if}
      {#if (chat.settings?.providerInstances.length ?? 0) === 0}
        <p class="rounded-lg border border-dashed border-border p-4 text-sm text-muted-foreground">{t("settings.chat.providers.empty")}</p>
      {:else}
        <div class="flex flex-col divide-y divide-border rounded-lg border border-border bg-card/40">
          {#each chat.settings?.providerInstances ?? [] as provider (provider.configuration.instanceId)}
            <ProviderCard
              {provider}
              busy={refreshingAll || busyIds.includes(provider.configuration.instanceId)}
              onRefresh={() => void perform(provider.configuration.instanceId, async () => { await chat.probeProvider(provider.configuration.instanceId); })}
              onEdit={() => onOpenProviderSetup({ mode: "edit", instanceId: provider.configuration.instanceId })}
              onToggle={() => void perform(provider.configuration.instanceId, () => chat.setProviderEnabled(provider.configuration.instanceId, !provider.configuration.enabled))}
              onRemove={() => { removeId = provider.configuration.instanceId; }}
            />
          {/each}
        </div>
      {/if}
    </section>
  {:else if activeTab === "models"}
    <div data-chat-settings-subsection="models"><ChatModelsSettings /></div>
  {:else if activeTab === "permissions"}
    <div data-chat-settings-subsection="permissions"><ChatPermissionsSettings /></div>
  {:else}
    <div data-chat-settings-subsection="behavior"><ChatBehaviorSettings /></div>
  {/if}
</div>

{#if removeProvider}
  <ConfirmDialog
    title={t("settings.chat.providers.removeTitle")}
    message={t("settings.chat.providers.removeMessage", removeProvider.configuration.label)}
    confirmLabel={t("settings.chat.providers.remove")}
    cancelLabel={t("chat.cancel")}
    onConfirm={() => void confirmRemove()}
    onCancel={() => { removeId = null; }}
  />
{/if}

<style>
  :global(.chat-settings-button) { display: inline-flex; min-height: 2rem; align-items: center; justify-content: center; gap: 0.375rem; border-radius: 0.375rem; border: 1px solid var(--border); background: var(--background); padding: 0.25rem 0.625rem; font-size: 0.733333rem; font-weight: 500; color: var(--foreground); }
  :global(.chat-settings-button:hover:not(:disabled)) { background: var(--accent); }
  :global(.chat-settings-button:disabled) { cursor: not-allowed; opacity: 0.5; }
  :global(.setup-field), :global(.chat-inline-field) { display: flex; flex-direction: column; gap: 0.375rem; font-size: 0.733333rem; font-weight: 500; color: var(--muted-foreground); }
  :global(.setup-field input), :global(.chat-inline-field input) { min-height: 2.25rem; border-radius: 0.375rem; border: 1px solid var(--border); background: var(--background); padding: 0.375rem 0.625rem; color: var(--foreground); outline: none; }
  :global(.setup-field input:focus), :global(.chat-inline-field input:focus) { border-color: var(--ring); box-shadow: 0 0 0 1px var(--ring); }
  :global(.setup-secondary-button), :global(.setup-primary-button) { display: inline-flex; min-height: 2.25rem; align-items: center; justify-content: center; border-radius: 0.375rem; padding: 0.375rem 0.75rem; font-size: 0.8rem; font-weight: 600; }
  :global(.setup-secondary-button) { border: 1px solid var(--border); background: var(--background); }
  :global(.setup-primary-button) { background: var(--primary); color: var(--primary-foreground); }
  :global(.setup-primary-button:disabled), :global(.setup-secondary-button:disabled) { cursor: not-allowed; opacity: 0.5; }
</style>
