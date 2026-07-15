<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { containMusicDialogFocus } from "$lib/music/music-dialog-focus";
  import type { MusicBulkEditController } from "$lib/music/music-bulk-edit-controller.svelte";
  import type { MusicFocusFit, MusicItemSignal } from "$lib/music/library-contracts";

  let {
    controller,
    mode,
    playlistId = null,
    onClose,
    onSaved,
  }: {
    controller: MusicBulkEditController;
    mode: "signals" | "focus-fit";
    playlistId?: string | null;
    onClose: () => void;
    onSaved: () => void;
  } = $props();

  const { t } = getLocalization();
  const signalOptions: MusicItemSignal[] = ["lyrics", "sudden-changes", "high-intensity", "calm", "repetitive", "energizing"];
  const focusFitOptions: MusicFocusFit[] = ["helpful", "neutral", "potentially-distracting", "unknown"];
  let signals = $state<MusicItemSignal[]>([]);
  let focusFit = $state<MusicFocusFit>("unknown");

  function toggleSignal(signal: MusicItemSignal): void {
    signals = signals.includes(signal) ? signals.filter((entry) => entry !== signal) : [...signals, signal];
  }

  async function save(): Promise<void> {
    const saved = mode === "signals"
      ? await controller.setSignals(signals)
      : playlistId ? await controller.setFocusFit(playlistId, focusFit) : false;
    if (saved) onSaved();
  }
</script>

<div class="absolute inset-0 z-60 grid place-items-center bg-background/65 p-3 backdrop-blur-sm">
  <div use:containMusicDialogFocus={{ onEscape: onClose, escapeDisabled: controller.saving }} role="dialog" aria-modal="true" aria-labelledby="music-bulk-classification-title" tabindex="-1" class="flex max-h-full w-full max-w-md flex-col overflow-hidden rounded-xl border border-border/70 bg-card shadow-2xl">
    <header class="border-b border-border/60 p-4"><h2 id="music-bulk-classification-title" class="text-sm font-semibold">{mode === "signals" ? t("music.builder.bulkSignalsTitle") : t("music.builder.bulkFocusFitTitle")}</h2><p class="mt-1 text-[0.68rem] leading-relaxed text-muted-foreground">{mode === "signals" ? t("music.builder.bulkSignalsDescription", controller.itemIds.length) : t("music.builder.bulkFocusFitDescription", controller.itemIds.length)}</p></header>
    <div class="min-h-0 overflow-y-auto p-4">
      {#if mode === "signals"}
        <div class="flex flex-wrap gap-2" role="group" aria-label={t("music.builder.descriptiveSignals")}>{#each signalOptions as signal}<button type="button" aria-pressed={signals.includes(signal)} onclick={() => toggleSignal(signal)} class="rounded-full border border-border/70 px-2.5 py-1.5 text-[0.68rem] aria-pressed:border-primary aria-pressed:bg-primary/10">{t(`music.builder.signal.${signal}`)}</button>{/each}</div>
        <p class="mt-3 rounded-lg bg-warning/8 p-2.5 text-[0.65rem] leading-relaxed text-muted-foreground">{t("music.builder.bulkSignalsReplaceHint")}</p>
      {:else}
        {#if playlistId && controller.focusFits[playlistId] === "mixed"}<p class="mb-3 rounded-lg bg-primary/8 p-2.5 text-[0.65rem] leading-relaxed">{t("music.builder.mixedFocusFitHint")}</p>{/if}
        <div class="grid gap-2">{#each focusFitOptions as value}<button type="button" aria-pressed={focusFit === value} onclick={() => focusFit = value} class="rounded-lg border border-border/70 p-3 text-left aria-pressed:border-primary aria-pressed:bg-primary/8"><strong class="block text-xs">{t(`music.builder.focusFitValue.${value}`)}</strong><span class="mt-0.5 block text-[0.64rem] text-muted-foreground">{t(`music.builder.focusFitDescription.${value}`)}</span></button>{/each}</div>
      {/if}
      {#if controller.error}<p class="mt-3 text-[0.68rem] text-destructive" role="alert">{controller.error}</p>{/if}
    </div>
    <footer class="flex flex-wrap justify-end gap-2 border-t border-border/60 p-3"><button type="button" onclick={onClose} disabled={controller.saving} class="h-8 rounded-md bg-secondary px-3 text-xs font-medium disabled:opacity-50">{t("music.builder.cancel")}</button><button type="button" onclick={() => { void save(); }} disabled={controller.saving} class="h-8 rounded-md bg-primary px-3 text-xs font-semibold text-primary-foreground disabled:opacity-40">{controller.saving ? t("music.builder.saving") : t("music.builder.applyChanges")}</button></footer>
  </div>
</div>
