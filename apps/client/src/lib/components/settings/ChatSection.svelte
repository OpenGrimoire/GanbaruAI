<script lang="ts">
  import { onMount, tick } from "svelte";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import type { ChatProviderSetupTarget, ChatSettingsSubsection } from "./types";
  import ProviderCard from "./chat/ProviderCard.svelte";
  import ChatModelsSettings from "./chat/ChatModelsSettings.svelte";
  import ChatWorkspacesSettings from "./chat/ChatWorkspacesSettings.svelte";
  import ChatBehaviorSettings from "./chat/ChatBehaviorSettings.svelte";

  let {
    initialSubsection,
    onOpenProviderSetup = () => {},
  }: {
    initialSubsection?: ChatSettingsSubsection;
    onOpenProviderSetup?: (target: ChatProviderSetupTarget) => void;
  } = $props();
  const { t } = getLocalization();
  const chat = getChat();
  let busyIds = $state<string[]>([]);
  let error = $state<string | null>(null);
  let removeId = $state<string | null>(null);
  let rootElement: HTMLDivElement | undefined = $state();
  const removeProvider = $derived(chat.settings?.providerInstances.find((entry) => entry.configuration.instanceId === removeId) ?? null);

  onMount(() => {
    void chat.ensureLoaded().then(async () => {
      if (!initialSubsection) return;
      await tick();
      const subsection = rootElement?.querySelector<HTMLElement>(
        `[data-chat-settings-subsection="${initialSubsection}"]`,
      );
      subsection?.scrollIntoView({ block: "start" });
      subsection?.querySelector<HTMLElement>("button, input, select, textarea")?.focus();
    }).catch((cause) => { error = errorMessage(cause); });
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
    const providers = chat.settings?.providerInstances ?? [];
    for (let index = 0; index < providers.length; index += 2) {
      await Promise.allSettled(providers.slice(index, index + 2).map((provider) => perform(
        provider.configuration.instanceId,
        async () => { await chat.probeProvider(provider.configuration.instanceId); },
      )));
    }
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

<div bind:this={rootElement} class="flex flex-col gap-7 pb-4">
  <header>
    <h1 class="text-lg font-semibold text-foreground">{t("settings.chat.heading")}</h1>
    <p class="mt-1 max-w-2xl text-sm text-muted-foreground">{t("settings.chat.description")}</p>
  </header>

  {#if chat.loading}
    <div class="rounded-lg border border-dashed border-border p-6 text-center text-sm text-muted-foreground">{t("common.loading")}</div>
  {:else}
    <section class="flex flex-col gap-4" data-chat-settings-subsection="providers">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div><h2 class="text-[0.866667rem] font-semibold text-foreground">{t("settings.chat.providers.heading")}</h2><p class="mt-1 text-xs text-muted-foreground">{t("settings.chat.providers.description")}</p></div>
        <div class="flex gap-2"><button type="button" class="chat-settings-button" disabled={(chat.settings?.providerInstances.length ?? 0) === 0} onclick={() => void refreshAll()}><RefreshCw size={13} />{t("settings.chat.providers.refreshAll")}</button><button type="button" class="chat-settings-button" onclick={() => onOpenProviderSetup({ mode: "create" })}><Plus size={13} />{t("settings.chat.providers.add")}</button></div>
      </div>
      {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
      {#if (chat.settings?.providerInstances.length ?? 0) === 0}
        <p class="rounded-lg border border-dashed border-border p-4 text-sm text-muted-foreground">{t("settings.chat.providers.empty")}</p>
      {:else}
        <div class="grid gap-3 xl:grid-cols-2">
          {#each chat.settings?.providerInstances ?? [] as provider (provider.configuration.instanceId)}
            <ProviderCard
              {provider}
              busy={busyIds.includes(provider.configuration.instanceId)}
              onRefresh={() => void perform(provider.configuration.instanceId, async () => { await chat.probeProvider(provider.configuration.instanceId); })}
              onEdit={() => onOpenProviderSetup({ mode: "edit", instanceId: provider.configuration.instanceId })}
              onToggle={() => void perform(provider.configuration.instanceId, () => chat.setProviderEnabled(provider.configuration.instanceId, !provider.configuration.enabled))}
              onRemove={() => { removeId = provider.configuration.instanceId; }}
            />
          {/each}
        </div>
      {/if}
    </section>

    <div class="h-px bg-border/70"></div>
    <div data-chat-settings-subsection="models"><ChatModelsSettings /></div>
    <div class="h-px bg-border/70"></div>
    <div data-chat-settings-subsection="workspaces"><ChatWorkspacesSettings /></div>
    <div class="h-px bg-border/70"></div>
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
  :global(.setup-field input), :global(.setup-field select), :global(.chat-inline-field input), :global(.chat-inline-field select) { min-height: 2.25rem; border-radius: 0.375rem; border: 1px solid var(--border); background: var(--background); padding: 0.375rem 0.625rem; color: var(--foreground); outline: none; }
  :global(.setup-field input:focus), :global(.setup-field select:focus), :global(.chat-inline-field input:focus), :global(.chat-inline-field select:focus) { border-color: var(--ring); box-shadow: 0 0 0 1px var(--ring); }
  :global(.setup-secondary-button), :global(.setup-primary-button) { display: inline-flex; min-height: 2.25rem; align-items: center; justify-content: center; border-radius: 0.375rem; padding: 0.375rem 0.75rem; font-size: 0.8rem; font-weight: 600; }
  :global(.setup-secondary-button) { border: 1px solid var(--border); background: var(--background); }
  :global(.setup-primary-button) { background: var(--primary); color: var(--primary-foreground); }
</style>
