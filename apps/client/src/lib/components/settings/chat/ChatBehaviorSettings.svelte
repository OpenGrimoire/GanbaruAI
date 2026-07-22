<script lang="ts">
  import { onMount } from "svelte";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import * as chatApi from "$lib/api/chat";
  import type { ChatDiagnosticsRead } from "$lib/chat/contracts";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import CustomSelect from "../CustomSelect.svelte";
  import ToggleSetting from "../ToggleSetting.svelte";

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let saving = $state(false);
  let error = $state<string | null>(null);
  let status = $state<string | null>(null);
  let diagnostics = $state<ChatDiagnosticsRead | null>(null);
  let maintenance = $state<"stop" | "rebuild" | null>(null);
  let confirmation = $state("");
  let advancedOpen = $state(false);
  const behavior = $derived(chat.settings?.configuration.behavior ?? null);
  const sendKeyOptions = $derived([
    { value: "enter", label: t("settings.chat.behavior.enter") },
    { value: "mod_enter", label: t("settings.chat.behavior.modEnter") },
  ]);

  async function update(patch: Partial<NonNullable<typeof behavior>>): Promise<void> {
    if (!behavior || saving) return;
    saving = true;
    error = null;
    try {
      await chat.updateBehavior({ ...behavior, ...patch });
    } catch (cause: unknown) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      saving = false;
    }
  }

  async function loadDiagnostics(): Promise<void> {
    diagnostics = await chatApi.readChatDiagnostics();
  }

  async function updateDiagnosticPreferences(patch: Partial<ChatDiagnosticsRead["preferences"]>): Promise<string> {
    const current = diagnostics;
    if (!current) throw new Error(t("settings.chat.behavior.diagnosticsUnavailable"));
    await chatApi.updateChatDiagnosticPreferences({ ...current.preferences, ...patch });
    return t("settings.chat.behavior.preferencesSaved");
  }

  async function run(action: () => Promise<string | null>): Promise<void> {
    if (saving) return;
    saving = true;
    error = null;
    status = null;
    try {
      status = await action();
      await loadDiagnostics();
    } catch (cause: unknown) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      saving = false;
    }
  }

  function expectedConfirmation(): string {
    return maintenance === "stop" ? "STOP ALL CHAT PROCESSES" : "REBUILD CHAT PROJECTIONS";
  }

  function openMaintenance(kind: "stop" | "rebuild"): void {
    maintenance = kind;
    confirmation = "";
    status = null;
  }

  function cancelMaintenance(): void {
    maintenance = null;
    confirmation = "";
  }

  function diagnosticFieldLabel(field: string): string {
    const labels: Record<string, string> = {
      provider_family: t("settings.chat.behavior.diagnosticProviderFamily"),
      provider_instance: t("settings.chat.behavior.diagnosticProviderInstance"),
      thread_id: t("settings.chat.behavior.diagnosticThreadId"),
      turn_id: t("settings.chat.behavior.diagnosticTurnId"),
      event_type: t("settings.chat.behavior.diagnosticEventType"),
      protocol_label: t("settings.chat.behavior.diagnosticProtocolLabel"),
      timestamp: t("settings.chat.behavior.diagnosticTimestamp"),
      prompts_responses: t("settings.chat.behavior.diagnosticPromptsResponses"),
      tool_content: t("settings.chat.behavior.diagnosticToolContent"),
      paths: t("settings.chat.behavior.diagnosticPaths"),
      environment_credentials: t("settings.chat.behavior.diagnosticEnvironmentCredentials"),
    };
    return labels[field] ?? field;
  }

  function diagnosticStorageLabel(storage: string): string {
    return storage === "active_vault_sqlite"
      ? t("settings.chat.behavior.activeVaultSqlite")
      : storage;
  }

  onMount(() => {
    void loadDiagnostics().catch((cause: unknown) => {
      error = cause instanceof Error ? cause.message : String(cause);
    });
  });
</script>

<section class="flex flex-col gap-4">
  <div class="px-1">
    <h2 class="text-[0.866667rem] font-semibold text-foreground">{t("settings.chat.behavior.heading")}</h2>
    <p class="mt-1 text-[0.8rem] text-muted-foreground">{t("settings.chat.behavior.description")}</p>
  </div>
  {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
  {#if status}<p role="status" class="text-sm text-action-confirm">{status}</p>{/if}
  {#if behavior}
    <div class="flex flex-col gap-3">
      <CustomSelect
        label={t("settings.chat.behavior.sendKey")}
        description={t("settings.chat.behavior.sendKeyDescription")}
        value={behavior.sendKey}
        options={sendKeyOptions}
        onChange={(value) => void update({ sendKey: value === "mod_enter" ? "mod_enter" : "enter" })}
        disabled={saving}
      />
      <label class="number-setting">
        <span class="min-w-0 flex-1"><span class="block text-[0.866667rem] text-foreground">{t("settings.chat.behavior.terminalScrollback")}</span><span class="mt-0.5 block text-[0.8rem] text-muted-foreground">{t("settings.chat.behavior.terminalScrollbackDescription")}</span></span>
        <input type="number" min="1000" max="100000" step="1000" value={behavior.terminalScrollbackLines} disabled={saving} onchange={(event) => void update({ terminalScrollbackLines: event.currentTarget.valueAsNumber })} />
      </label>
      <label class="number-setting">
        <span class="min-w-0 flex-1"><span class="block text-[0.866667rem] text-foreground">{t("settings.chat.behavior.idleTimeout")}</span><span class="mt-0.5 block text-[0.8rem] text-muted-foreground">{t("settings.chat.behavior.idleTimeoutDescription")}</span></span>
        <input type="number" min="60" max="7200" step="60" value={behavior.idleSessionTimeoutSeconds} disabled={saving} onchange={(event) => void update({ idleSessionTimeoutSeconds: event.currentTarget.valueAsNumber })} />
      </label>
    </div>
    <div class="h-px shrink-0 scale-y-50 bg-border" aria-hidden="true"></div>
    <div class="flex flex-col gap-3">
      <ToggleSetting label={t("settings.chat.behavior.restoreThread")} checked={behavior.restoreLastSelectedThread} disabled={saving} onChange={(value) => void update({ restoreLastSelectedThread: value })} />
      <ToggleSetting label={t("settings.chat.behavior.reasoning")} checked={behavior.showReasoningSummaries} disabled={saving} onChange={(value) => void update({ showReasoningSummaries: value })} />
      <ToggleSetting label={t("settings.chat.behavior.foldWork")} checked={behavior.automaticallyFoldSettledWork} disabled={saving} onChange={(value) => void update({ automaticallyFoldSettledWork: value })} />
      <ToggleSetting label={t("settings.chat.behavior.confirmPaste")} checked={behavior.confirmMultilineTerminalPaste} disabled={saving} onChange={(value) => void update({ confirmMultilineTerminalPaste: value })} />
    </div>
  {/if}
  {#if diagnostics}
    <div class="rounded-lg border border-border bg-card/30">
      <button
        type="button"
        class="flex w-full items-center gap-2 px-3 py-2.5 text-left text-[0.8rem] font-medium text-foreground hover:bg-accent/40"
        aria-expanded={advancedOpen}
        onclick={() => { advancedOpen = !advancedOpen; }}
      >
        <ChevronRight size={13} class="transition-transform {advancedOpen ? 'rotate-90' : ''}" />
        <span>{t("settings.chat.behavior.advanced")}</span>
      </button>
      {#if advancedOpen}
      <div class="flex flex-col gap-3 border-t border-border p-3">
    <div class="grid gap-3 rounded-lg border border-border bg-background/50 p-3 sm:grid-cols-2">
      <div><div class="text-xs text-muted-foreground">{t("settings.chat.behavior.credentialStore")}</div><div class="mt-1 text-sm font-medium">{diagnostics.credentialStoreAvailable ? t("settings.chat.behavior.credentialAvailable") : t("settings.chat.behavior.credentialUnavailable")}</div></div>
      <div><div class="text-xs text-muted-foreground">{t("settings.chat.behavior.projectionHealth")}</div><div class="mt-1 text-sm font-medium">{diagnostics.projectionHealthy ? t("settings.chat.behavior.projectionHealthy") : t("settings.chat.behavior.projectionUnhealthy", formatNumber(localization.locale, diagnostics.inconsistentProjectionCount))}</div></div>
      <div><div class="text-xs text-muted-foreground">{t("settings.chat.behavior.processes")}</div><div class="mt-1 text-sm">{t("settings.chat.behavior.processCounts", formatNumber(localization.locale, diagnostics.liveProviderProcesses), formatNumber(localization.locale, diagnostics.activeTurns), formatNumber(localization.locale, diagnostics.liveTerminals))}</div></div>
      <div><div class="text-xs text-muted-foreground">{t("settings.chat.behavior.probes")}</div><div class="mt-1 text-sm">{t("settings.chat.behavior.probeCounts", formatNumber(localization.locale, diagnostics.providerProbeHealthy), formatNumber(localization.locale, diagnostics.providerProbeUnhealthy), formatNumber(localization.locale, diagnostics.providerProbeUnknown))}</div></div>
      <div><div class="text-xs text-muted-foreground">{t("settings.chat.behavior.attachments")}</div><div class="mt-1 text-sm">{t("settings.chat.behavior.storageCounts", formatNumber(localization.locale, diagnostics.counts.attachmentCount), formatNumber(localization.locale, diagnostics.counts.attachmentBytes))}</div><div class="text-xs text-muted-foreground">{t("settings.chat.behavior.cleanupCounts", formatNumber(localization.locale, diagnostics.counts.pendingAttachmentCleanup), formatNumber(localization.locale, diagnostics.counts.failedAttachmentCleanup))}</div></div>
      <div><div class="text-xs text-muted-foreground">{t("settings.chat.behavior.commandArtifacts")}</div><div class="mt-1 text-sm">{t("settings.chat.behavior.storageCounts", formatNumber(localization.locale, diagnostics.counts.commandOutputEvents), formatNumber(localization.locale, diagnostics.counts.commandOutputBytes))}</div><div class="text-xs text-muted-foreground">{t("settings.chat.behavior.outputBound")}</div></div>
      <div class="sm:col-span-2"><div class="text-xs text-muted-foreground">{t("settings.chat.behavior.checkpoints")}</div><div class="mt-1 text-sm">{t("settings.chat.behavior.checkpointCounts", formatNumber(localization.locale, diagnostics.counts.checkpointFailures), formatNumber(localization.locale, diagnostics.counts.pendingCheckpointCleanup), formatNumber(localization.locale, diagnostics.counts.failedCheckpointCleanup))}</div></div>
    </div>

    <div class="space-y-3 rounded-lg border border-border p-4">
      <ToggleSetting label={t("settings.chat.behavior.captureDiagnostics")} checked={diagnostics.preferences.captureEnabled} disabled={saving} onChange={(captureEnabled) => void run(() => updateDiagnosticPreferences({ captureEnabled }))} />
      <label class="setup-field"><span>{t("settings.chat.behavior.retentionDays")}</span><input type="number" min="1" max="30" value={diagnostics.preferences.retentionDays} onchange={(event) => void run(() => updateDiagnosticPreferences({ retentionDays: event.currentTarget.valueAsNumber }))} /></label>
      <p class="text-xs text-muted-foreground">{t("settings.chat.behavior.captureFields", diagnostics.capturedFields.map(diagnosticFieldLabel).join(", "))}</p>
      <p class="text-xs text-muted-foreground">{t("settings.chat.behavior.excludeFields", diagnostics.excludedFields.map(diagnosticFieldLabel).join(", "))}</p>
      <p class="text-xs text-muted-foreground">{t("settings.chat.behavior.diagnosticStorage", diagnosticStorageLabel(diagnostics.storageLocation))}</p>
      <p class="text-xs text-muted-foreground">{t("settings.chat.behavior.diagnosticUsage", formatNumber(localization.locale, diagnostics.counts.retainedEvents), formatNumber(localization.locale, diagnostics.counts.retainedBytes))}</p>
      <div class="flex flex-wrap gap-2"><button type="button" class="chat-settings-action" disabled={saving} onclick={() => void run(async () => { await chatApi.exportRedactedChatDiagnostics(t("settings.chat.behavior.exportDiagnostics")); return t("settings.chat.behavior.exportComplete"); })}>{t("settings.chat.behavior.exportDiagnostics")}</button><button type="button" class="chat-settings-action" disabled={saving} onclick={() => void run(async () => { const count = await chatApi.deleteChatDiagnostics(); return t("settings.chat.behavior.deletedDiagnostics", formatNumber(localization.locale, count)); })}>{t("settings.chat.behavior.deleteDiagnostics")}</button></div>
    </div>

    <div class="space-y-3 rounded-lg border border-border p-4">
      <h3 class="text-sm font-medium">{t("settings.chat.behavior.maintenance")}</h3>
      <p class="text-xs text-muted-foreground">{t("settings.chat.behavior.maintenanceDescription")}</p>
      <div class="flex flex-wrap gap-2"><button type="button" class="chat-settings-action" disabled={saving} onclick={() => openMaintenance("stop")}>{t("settings.chat.behavior.stopAll")}</button><button type="button" class="chat-settings-action" disabled={saving} onclick={() => openMaintenance("rebuild")}>{t("settings.chat.behavior.rebuild")}</button><button type="button" class="chat-settings-action" disabled={saving} onclick={() => void run(async () => { const count = await chatApi.retryChatCheckpointCleanup(); return t("settings.chat.behavior.cleanupRetried", formatNumber(localization.locale, count)); })}>{t("settings.chat.behavior.retryCleanup")}</button></div>
      {#if maintenance}
        <div class="space-y-2 rounded-md border border-status-tentative/60 p-3">
          <p class="text-xs">{maintenance === "stop" ? t("settings.chat.behavior.stopImpact") : t("settings.chat.behavior.rebuildImpact")}</p>
          <label class="setup-field"><span>{t("settings.chat.behavior.typeConfirmation", expectedConfirmation())}</span><input bind:value={confirmation} autocomplete="off" spellcheck="false" /></label>
          <div class="flex gap-2"><button type="button" class="chat-settings-action" disabled={saving || confirmation !== expectedConfirmation()} onclick={() => void run(async () => { if (maintenance === "stop") { const result = await chatApi.stopAllChatProcesses(confirmation); cancelMaintenance(); return t("settings.chat.behavior.stopped", formatNumber(localization.locale, result.providerProcessesStopped), formatNumber(localization.locale, result.terminalsStopped)); } const result = await chatApi.rebuildChatProjections(confirmation); cancelMaintenance(); return t("settings.chat.behavior.rebuilt", formatNumber(localization.locale, result.rebuiltThreads)); })}>{t("settings.chat.behavior.confirm")}</button><button type="button" class="chat-settings-action" disabled={saving} onclick={cancelMaintenance}>{t("common.cancel")}</button></div>
        </div>
      {/if}
    </div>
      </div>
      {/if}
    </div>
  {:else}
    <p class="text-sm text-muted-foreground">{t("common.loading")}</p>
  {/if}
</section>

<style>
  .chat-settings-action { border: 1px solid var(--border); border-radius: 0.375rem; padding: 0.35rem 0.65rem; font-size: 0.733333rem; }
  .chat-settings-action:disabled { cursor: not-allowed; opacity: 0.5; }
  .number-setting { display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 0.25rem; }
  .number-setting input { width: 7rem; min-height: 1.75rem; border-radius: 0.375rem; border: 1px solid var(--border); background: var(--card); padding-inline: 0.5rem; color: var(--foreground); font-size: 0.8rem; outline: none; }
  .number-setting input:focus { border-color: var(--ring); box-shadow: 0 0 0 1px var(--ring); }
  .number-setting input:disabled { cursor: not-allowed; opacity: 0.5; }
  @media (max-width: 480px) { .number-setting { align-items: flex-start; flex-direction: column; } }
</style>
