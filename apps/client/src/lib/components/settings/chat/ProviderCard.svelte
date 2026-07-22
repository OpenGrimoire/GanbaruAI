<script lang="ts">
  import CheckCircle2 from "@lucide/svelte/icons/check-circle-2";
  import CircleOff from "@lucide/svelte/icons/circle-off";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Power from "@lucide/svelte/icons/power";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
  import type { ProviderInstanceRead } from "$lib/chat/contracts";
  import ChatProviderIcon from "$lib/components/chat/ChatProviderIcon.svelte";
  import { formatDateTime } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";

  let {
    provider,
    busy = false,
    onRefresh,
    onEdit,
    onToggle,
    onRemove,
  }: {
    provider: ProviderInstanceRead;
    busy?: boolean;
    onRefresh: () => void;
    onEdit: () => void;
    onToggle: () => void;
    onRemove: () => void;
  } = $props();

  const localization = getLocalization();
  const { t } = localization;
  let revealAccount = $state(false);
  const stateLabel = $derived.by(() => {
    if (!provider.configuration.enabled) return t("settings.chat.providers.disabled");
    switch (provider.lastProbe?.state) {
      case "healthy": return t("settings.chat.providers.healthy");
      case "authentication_required": return t("settings.chat.providers.authenticationRequired");
      case "unsupported_version": return t("settings.chat.providers.unsupported");
      case undefined: return t("settings.chat.providers.neverChecked");
      default: return t("settings.chat.providers.unavailable");
    }
  });
  const healthy = $derived(provider.configuration.enabled && provider.lastProbe?.state === "healthy");
</script>

<article class="flex min-w-0 flex-wrap items-start gap-3 px-3 py-3.5">
  <div class="mt-0.5 grid size-8 shrink-0 place-items-center rounded-md border border-border bg-background">
    <ChatProviderIcon
      familyId={provider.configuration.familyId}
      label={provider.configuration.label}
      accentColor={provider.configuration.accentColor}
      size={16}
    />
  </div>
  <div class="min-w-40 flex-1">
    <div class="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
      <h3 class="truncate text-[0.866667rem] font-semibold text-foreground">{provider.configuration.label}</h3>
      <span class="inline-flex items-center gap-1 text-[0.7rem] {healthy ? 'text-action-confirm' : 'text-muted-foreground'}">
        {#if healthy}<CheckCircle2 size={11} />{:else if provider.configuration.enabled}<TriangleAlert size={11} />{:else}<CircleOff size={11} />{/if}
        {stateLabel}
      </span>
    </div>
    <div class="mt-1 flex min-w-0 flex-wrap gap-x-3 gap-y-0.5 text-[0.733333rem] text-muted-foreground">
      <span class="truncate">{provider.configuration.executable || t("common.none")}</span>
      {#if provider.lastProbe?.version}<span>{provider.lastProbe.version}</span>{/if}
      <span class="truncate">
        {#if revealAccount}{provider.lastProbe?.accountLabel ?? t("common.none")}{:else}{t("settings.chat.providers.accountHidden")}{/if}
      </span>
      <span>{provider.lastSuccessfulProbeAt ? t("settings.chat.providers.lastSuccessful", formatDateTime(localization.locale, new Date(provider.lastSuccessfulProbeAt), { dateStyle: "medium", timeStyle: "short" })) : provider.lastProbe ? t("settings.chat.providers.lastChecked", formatDateTime(localization.locale, new Date(provider.lastProbe.checkedAt), { dateStyle: "medium", timeStyle: "short" })) : ""}</span>
    </div>
    {#if provider.lastProbe?.state === "authentication_required"}
      <p class="mt-1.5 text-[0.733333rem] text-status-tentative">{t("settings.chat.providers.loginHelp")}</p>
    {/if}
  </div>
  <div class="ml-auto flex shrink-0 items-center gap-0.5">
    <button type="button" class="provider-action" disabled={busy} aria-label={t("settings.chat.providers.refresh")} title={t("settings.chat.providers.refresh")} onclick={onRefresh}><RefreshCw size={13} /></button>
    <button type="button" class="provider-action" aria-label={revealAccount ? t("settings.chat.providers.hideAccount") : t("settings.chat.providers.revealAccount")} title={revealAccount ? t("settings.chat.providers.hideAccount") : t("settings.chat.providers.revealAccount")} onclick={() => { revealAccount = !revealAccount; }}>
      {#if revealAccount}<EyeOff size={13} />{:else}<Eye size={13} />{/if}
    </button>
    <button type="button" class="provider-action" aria-label={t("settings.chat.providers.edit")} title={t("settings.chat.providers.edit")} onclick={onEdit}><Pencil size={13} /></button>
    <button type="button" class="provider-action" disabled={busy} aria-label={provider.configuration.enabled ? t("settings.chat.providers.disable") : t("settings.chat.providers.enable")} title={provider.configuration.enabled ? t("settings.chat.providers.disable") : t("settings.chat.providers.enable")} onclick={onToggle}><Power size={13} /></button>
    <button type="button" class="provider-action text-destructive" disabled={busy} aria-label={t("settings.chat.providers.remove")} title={t("settings.chat.providers.remove")} onclick={onRemove}><Trash2 size={13} /></button>
  </div>
</article>

<style>
  .provider-action { display: inline-grid; width: 1.75rem; height: 1.75rem; place-items: center; border-radius: 0.375rem; color: var(--muted-foreground); }
  .provider-action:hover:not(:disabled) { background: var(--accent); color: var(--foreground); }
  .provider-action:disabled { cursor: not-allowed; opacity: 0.45; }
</style>
