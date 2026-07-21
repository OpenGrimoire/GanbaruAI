<script lang="ts">
  import CheckCircle2 from "@lucide/svelte/icons/check-circle-2";
  import CircleOff from "@lucide/svelte/icons/circle-off";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
  import type { ProviderInstanceRead } from "$lib/chat/contracts";
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

<article class="rounded-lg border border-border bg-card/60 p-4" style={`--provider-accent:${provider.configuration.accentColor ?? "var(--primary)"}`}>
  <div class="flex min-w-0 items-start gap-3">
    <div class="mt-0.5 size-9 shrink-0 rounded-lg border border-border bg-background p-1.5">
      <div class="size-full rounded-md" style="background-color:var(--provider-accent)"></div>
    </div>
    <div class="min-w-0 flex-1">
      <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
        <h3 class="truncate text-[0.866667rem] font-semibold text-foreground">{provider.configuration.label}</h3>
        <span class="inline-flex items-center gap-1 text-[0.733333rem] {healthy ? 'text-success' : 'text-muted-foreground'}">
          {#if healthy}<CheckCircle2 size={12} />{:else if provider.configuration.enabled}<TriangleAlert size={12} />{:else}<CircleOff size={12} />{/if}
          {stateLabel}
        </span>
      </div>
      <div class="mt-1 grid gap-1 text-[0.733333rem] text-muted-foreground sm:grid-cols-2">
        <span class="truncate">{t("settings.chat.providers.executable")}: {provider.configuration.executable || t("common.none")}</span>
        <span>{t("settings.chat.providers.version")}: {provider.lastProbe?.version ?? t("common.none")}</span>
        <span class="truncate">
          {t("settings.chat.providers.account")}:
          {#if revealAccount}{provider.lastProbe?.accountLabel ?? t("common.none")}{:else}••••••{/if}
        </span>
        <span>{provider.lastSuccessfulProbeAt ? t("settings.chat.providers.lastSuccessful", formatDateTime(localization.locale, new Date(provider.lastSuccessfulProbeAt), { dateStyle: "medium", timeStyle: "short" })) : provider.lastProbe ? t("settings.chat.providers.lastChecked", formatDateTime(localization.locale, new Date(provider.lastProbe.checkedAt), { dateStyle: "medium", timeStyle: "short" })) : ""}</span>
      </div>
      {#if provider.lastProbe?.state === "authentication_required"}
        <p class="mt-2 text-[0.733333rem] text-warning">{t("settings.chat.providers.loginHelp")}</p>
      {/if}
    </div>
  </div>
  <div class="mt-3 flex flex-wrap gap-1.5 border-t border-border/60 pt-3">
    <button type="button" class="chat-settings-button" disabled={busy} onclick={onRefresh}><RefreshCw size={13} />{t("settings.chat.providers.refresh")}</button>
    <button type="button" class="chat-settings-button" onclick={() => { revealAccount = !revealAccount; }}>
      {#if revealAccount}<EyeOff size={13} />{t("settings.chat.providers.hideAccount")}{:else}<Eye size={13} />{t("settings.chat.providers.revealAccount")}{/if}
    </button>
    <button type="button" class="chat-settings-button" onclick={onEdit}>{t("settings.chat.providers.edit")}</button>
    <button type="button" class="chat-settings-button" disabled={busy} onclick={onToggle}>
      {provider.configuration.enabled ? t("settings.chat.providers.disable") : t("settings.chat.providers.enable")}
    </button>
    <button type="button" class="chat-settings-button text-destructive" disabled={busy} onclick={onRemove}>{t("settings.chat.providers.remove")}</button>
  </div>
</article>

<style>
  :global(.chat-settings-button) {
    display: inline-flex;
    min-height: 2rem;
    align-items: center;
    gap: 0.375rem;
    border-radius: 0.375rem;
    border: 1px solid var(--border);
    background: var(--background);
    padding: 0.25rem 0.625rem;
    font-size: 0.733333rem;
    font-weight: 500;
  }
  :global(.chat-settings-button:hover:not(:disabled)) { background: var(--accent); }
  :global(.chat-settings-button:disabled) { cursor: not-allowed; opacity: 0.5; }
</style>
