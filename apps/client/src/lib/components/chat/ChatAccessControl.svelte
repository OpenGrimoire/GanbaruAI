<script lang="ts">
  import { tick } from "svelte";
  import type { SafetyMode } from "$lib/chat/contracts";
  import * as chatApi from "$lib/api/chat";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ChatControlMenu, { type ChatControlOption } from "./ChatControlMenu.svelte";

  const { t } = getLocalization();
  const chat = getChat();
  let fullAccessDialog: HTMLElement | undefined = $state();
  let confirmationReturnFocus: HTMLElement | null = null;
  let fullAccessDialogOpen = $state(false);
  let fullAccessTrusted = $state(false);
  let trustKey = $state("");
  let error = $state<string | null>(null);
  const provider = $derived(chat.settings?.providerInstances.find((entry) => entry.configuration.instanceId === chat.composer.providerInstanceId) ?? null);
  const safetyOptions = $derived<ChatControlOption[]>([
    { value: "", label: t("chat.hero.chooseSafety"), icon: "shield" },
    { value: "supervised", label: t("chat.hero.supervised"), description: t("chat.composer.supervisedDescription"), icon: "shield-check" },
    { value: "auto_accept_edits", label: t("chat.hero.autoAccept"), description: t("chat.composer.autoAcceptDescription"), icon: "file-pen" },
    { value: "full_access", label: t("chat.hero.fullAccess"), description: t("chat.composer.fullAccessShortDescription"), icon: "shield-off" },
  ]);

  $effect(() => {
    const workspaceId = chat.composer.workspaceId;
    const providerId = chat.composer.providerInstanceId;
    const key = workspaceId && providerId ? `${workspaceId}:${providerId}` : "";
    if (key === trustKey) return;
    trustKey = key;
    fullAccessTrusted = false;
    if (workspaceId && providerId) {
      void chatApi.hasChatFullAccessTrust(providerId, workspaceId).then((trusted) => {
        if (trustKey === key) fullAccessTrusted = trusted;
      }).catch(() => undefined);
    }
  });

  function chooseSafety(value: SafetyMode | ""): void {
    if (value === "full_access" && !fullAccessTrusted) {
      confirmationReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      fullAccessDialogOpen = true;
      void tick().then(() => firstFocusable(fullAccessDialog)?.focus());
      return;
    }
    chat.setComposerModes(value || null, chat.composer.interactionMode);
  }

  async function confirmFullAccess(): Promise<void> {
    if (!chat.composer.workspaceId || !chat.composer.providerInstanceId) return;
    error = null;
    try {
      await chatApi.setChatFullAccessTrust(chat.composer.providerInstanceId, chat.composer.workspaceId, true);
      fullAccessTrusted = true;
      chat.setComposerModes("full_access", chat.composer.interactionMode);
      closeDialog();
    } catch (cause: unknown) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  function closeDialog(): void {
    fullAccessDialogOpen = false;
    const target = confirmationReturnFocus;
    confirmationReturnFocus = null;
    queueMicrotask(() => target?.isConnected && target.focus());
  }

  function focusableElements(container: HTMLElement): HTMLElement[] {
    return [...container.querySelectorAll<HTMLElement>(
      "button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex='-1'])",
    )].filter((element) => !element.hidden && element.getClientRects().length > 0);
  }

  function firstFocusable(container: HTMLElement | undefined): HTMLElement | undefined {
    return container ? focusableElements(container)[0] : undefined;
  }

  function handleDialogKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      closeDialog();
      return;
    }
    if (event.key !== "Tab" || !(event.currentTarget instanceof HTMLElement)) return;
    const focusable = focusableElements(event.currentTarget);
    const first = focusable[0];
    const last = focusable.at(-1);
    if (!first || !last) return;
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }
</script>

<div class:full-access={chat.composer.safetyMode === "full_access"} class="access-control">
  <ChatControlMenu value={chat.composer.safetyMode ?? ""} options={safetyOptions} ariaLabel={t("chat.hero.safety")} dataField="safety" onChange={(value) => chooseSafety(value as SafetyMode | "")} compact minimal />
</div>

{#if fullAccessDialogOpen}
  <div class="fixed inset-0 z-60 grid place-items-center bg-black/40 p-4">
    <button type="button" class="absolute inset-0" aria-label={t("chat.cancel")} onclick={closeDialog}></button>
    <div bind:this={fullAccessDialog} class="relative w-full max-w-md rounded-lg border border-border bg-background p-4 shadow-2xl" role="dialog" aria-modal="true" aria-labelledby="full-access-title" tabindex="-1" onkeydown={handleDialogKeydown}>
      <h2 id="full-access-title" class="font-semibold">{t("chat.composer.fullAccessTitle")}</h2>
      <p class="mt-2 text-sm text-muted-foreground">{t("chat.composer.fullAccessDescription", provider?.configuration.label ?? "", chat.selectedWorkspace?.workspace.displayName ?? "")}</p>
      {#if error}<p role="alert" class="mt-2 text-sm text-destructive">{error}</p>{/if}
      <div class="mt-4 flex justify-end gap-2">
        <button type="button" class="chat-secondary-button" onclick={closeDialog}>{t("chat.cancel")}</button>
        <button type="button" class="chat-primary-button" onclick={() => void confirmFullAccess()}>{t("chat.composer.confirmFullAccess")}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .access-control { min-width: 0; color: var(--muted-foreground); }
  .access-control.full-access { color: var(--status-tentative); }
  .access-control :global(.control-trigger) { color: currentColor; }
</style>
