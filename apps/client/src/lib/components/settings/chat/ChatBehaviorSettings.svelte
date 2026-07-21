<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ToggleSetting from "../ToggleSetting.svelte";

  const { t } = getLocalization();
  const chat = getChat();
  let saving = $state(false);
  let error = $state<string | null>(null);
  const behavior = $derived(chat.settings?.configuration.behavior ?? null);

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
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-[0.866667rem] font-semibold text-foreground">{t("settings.chat.behavior.heading")}</h2>
  {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
  {#if behavior}
    <div class="grid gap-3 sm:grid-cols-2">
      <label class="setup-field"><span>{t("settings.chat.behavior.sendKey")}</span><select value={behavior.sendKey} onchange={(event) => void update({ sendKey: event.currentTarget.value === "mod_enter" ? "mod_enter" : "enter" })}><option value="enter">{t("settings.chat.behavior.enter")}</option><option value="mod_enter">{t("settings.chat.behavior.modEnter")}</option></select></label>
      <label class="setup-field"><span>{t("settings.chat.behavior.terminalScrollback")}</span><input type="number" min="1000" max="100000" step="1000" value={behavior.terminalScrollbackLines} onchange={(event) => void update({ terminalScrollbackLines: event.currentTarget.valueAsNumber })} /></label>
      <label class="setup-field"><span>{t("settings.chat.behavior.idleTimeout")}</span><input type="number" min="60" max="7200" step="60" value={behavior.idleSessionTimeoutSeconds} onchange={(event) => void update({ idleSessionTimeoutSeconds: event.currentTarget.valueAsNumber })} /></label>
    </div>
    <div class="grid gap-3 sm:grid-cols-2">
      <ToggleSetting label={t("settings.chat.behavior.restoreThread")} checked={behavior.restoreLastSelectedThread} disabled={saving} onChange={(value) => void update({ restoreLastSelectedThread: value })} />
      <ToggleSetting label={t("settings.chat.behavior.reasoning")} checked={behavior.showReasoningSummaries} disabled={saving} onChange={(value) => void update({ showReasoningSummaries: value })} />
      <ToggleSetting label={t("settings.chat.behavior.foldWork")} checked={behavior.automaticallyFoldSettledWork} disabled={saving} onChange={(value) => void update({ automaticallyFoldSettledWork: value })} />
      <ToggleSetting label={t("settings.chat.behavior.confirmPaste")} checked={behavior.confirmMultilineTerminalPaste} disabled={saving} onChange={(value) => void update({ confirmMultilineTerminalPaste: value })} />
    </div>
  {/if}
  <div class="grid gap-3 rounded-lg border border-border bg-card/40 p-4 sm:grid-cols-2">
    <div><div class="text-xs text-muted-foreground">{t("settings.chat.behavior.credentialStore")}</div><div class="mt-1 text-sm font-medium">{chat.settings?.credentialStoreAvailability === "available" ? t("settings.chat.behavior.credentialAvailable") : t("settings.chat.behavior.credentialUnavailable")}</div></div>
    <div><div class="text-xs text-muted-foreground">{t("settings.chat.behavior.projectionHealth")}</div><div class="mt-1 text-sm font-medium">{t("settings.chat.behavior.projectionHealthy")}</div></div>
    <p class="text-xs text-muted-foreground sm:col-span-2">{t("settings.chat.behavior.diagnosticsDescription")}</p>
  </div>
</section>
