<script lang="ts">
  import CheckCircle2 from "@lucide/svelte/icons/circle-check";
  import FileAudio from "@lucide/svelte/icons/file-audio";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import ShieldAlert from "@lucide/svelte/icons/shield-alert";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { containMusicDialogFocus } from "$lib/music/music-dialog-focus";
  import type { MusicSourcesController } from "$lib/music/music-sources-controller.svelte";
  import { formatMusicDuration } from "$lib/music/music-builder-presentation";

  let { controller, itemId, onClose, onRepaired }: { controller: MusicSourcesController; itemId: string; onClose: () => void; onRepaired: () => void } = $props();
  const { t } = getLocalization();
  let busy = $state(false);
  let acceptedWeak = $state(false);
  let error = $state<string | null>(null);

  function strengthLabel(): string {
    if (controller.itemRepairPreview?.matchStrength === "exact") return t("music.builder.identityExact");
    if (controller.itemRepairPreview?.matchStrength === "likely") return t("music.builder.identityLikely");
    return t("music.builder.identityWeak");
  }

  async function choose(): Promise<void> {
    busy = true;
    error = null;
    acceptedWeak = false;
    try { await controller.chooseItemRepair(itemId); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
    finally { busy = false; }
  }

  async function apply(): Promise<void> {
    busy = true;
    error = null;
    try { await controller.applyItemRepair(acceptedWeak); onRepaired(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
    finally { busy = false; }
  }

  async function undo(): Promise<void> {
    busy = true;
    try { await controller.undoItemRepair(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
    finally { busy = false; }
  }
</script>

<div class="absolute inset-0 z-50 grid place-items-center bg-background/60 p-2 backdrop-blur-sm">
  <button type="button" class="absolute inset-0" onclick={onClose} aria-label={t("music.builder.close")}></button>
  <div use:containMusicDialogFocus={{ onEscape: onClose, escapeDisabled: busy }} class="relative flex max-h-full w-[min(32rem,100%)] flex-col overflow-hidden rounded-2xl border border-border/70 bg-popover shadow-2xl" role="dialog" aria-modal="true" aria-label={t("music.builder.repairItemLocation")} tabindex="-1">
    <header class="border-b border-border/55 px-4 py-3"><h2 class="text-sm font-semibold">{t("music.builder.repairItemLocation")}</h2><p class="mt-1 text-[0.68rem] leading-relaxed text-muted-foreground">{t("music.builder.replacementFileDescription")}</p></header>
    <div class="min-h-0 flex-1 overflow-y-auto p-3">
      {#if controller.itemRepairApplied}
        <div class="grid min-h-40 place-items-center text-center"><div><span class="mx-auto grid h-11 w-11 place-items-center rounded-2xl bg-primary/10 text-primary"><CheckCircle2 size={21} /></span><h3 class="mt-3 text-xs font-semibold">{t("music.builder.locationRepaired")}</h3><button type="button" disabled={busy} onclick={() => { void undo(); }} class="mt-4 inline-flex h-8 items-center gap-1.5 rounded-lg bg-secondary px-3 text-[0.68rem] font-semibold"><RotateCcw size={12} />{t("music.builder.undoRepair")}</button></div></div>
      {:else if busy && !controller.itemRepairPreview}
        <div class="grid min-h-40 place-items-center"><LoaderCircle class="animate-spin text-muted-foreground motion-reduce:animate-none" size={22} /></div>
      {:else if controller.itemRepairPreview}
        <div class="rounded-xl border border-border/65 bg-card/50 p-3"><div class="flex items-start gap-2.5"><span class="grid h-10 w-10 shrink-0 place-items-center rounded-xl bg-secondary text-muted-foreground"><FileAudio size={19} /></span><div class="min-w-0 flex-1"><strong class="block truncate text-xs">{controller.itemRepairPreview.title}</strong><span class="mt-0.5 block truncate text-[0.65rem] text-muted-foreground">{controller.itemRepairPreview.artist || controller.itemRepairPreview.relativePath}</span><span class="mt-1 block text-[0.61rem] text-muted-foreground">{formatMusicDuration(controller.itemRepairPreview.durationMs)} · {Math.round(controller.itemRepairPreview.fileSizeBytes / 1024 / 1024 * 10) / 10} MB</span></div></div><div class:weak={controller.itemRepairPreview.matchStrength === "weak"} class="match-box"><ShieldAlert size={14} /><div><strong>{strengthLabel()}</strong>{#each controller.itemRepairPreview.reasons as reason}<p>{reason}</p>{/each}</div></div></div>
        {#if controller.itemRepairPreview.matchStrength === "weak"}<label class="mt-3 flex items-start gap-2 rounded-xl bg-destructive/8 p-2.5 text-[0.68rem] leading-relaxed"><input type="checkbox" bind:checked={acceptedWeak} class="mt-0.5" /><span><strong class="block text-destructive">{t("music.builder.weakMatchWarning")}</strong><span class="mt-1 block text-muted-foreground">{t("music.builder.acceptWeakMatch")}</span></span></label>{/if}
        <button type="button" onclick={() => { void choose(); }} class="mt-3 text-[0.68rem] font-semibold text-primary hover:underline">{t("music.builder.chooseAnotherFile")}</button>
      {:else}
        <div class="grid min-h-40 place-items-center rounded-xl border border-dashed border-border/70 bg-card/35 p-4 text-center"><div><FileAudio class="mx-auto text-muted-foreground" size={24} /><button type="button" onclick={() => { void choose(); }} class="mt-3 h-8 rounded-lg bg-primary px-3 text-[0.68rem] font-semibold text-primary-foreground">{t("music.builder.chooseReplacementFile")}</button></div></div>
      {/if}
      {#if error}<p class="mt-3 rounded-lg bg-destructive/10 px-2.5 py-2 text-[0.68rem] text-destructive" role="alert">{error}</p>{/if}
    </div>
    <footer class="flex shrink-0 justify-end gap-2 border-t border-border/55 px-3 py-2"><button type="button" onclick={onClose} class="h-8 rounded-lg bg-secondary px-3 text-[0.68rem] font-semibold">{t("music.builder.close")}</button>{#if controller.itemRepairPreview && !controller.itemRepairApplied}<button type="button" onclick={() => { void apply(); }} disabled={busy || (controller.itemRepairPreview.matchStrength === "weak" && !acceptedWeak)} class="h-8 rounded-lg bg-primary px-3 text-[0.68rem] font-semibold text-primary-foreground disabled:opacity-45">{t("music.builder.bindLocation")}</button>{/if}</footer>
  </div>
</div>

<style>
  .match-box { display: flex; align-items: flex-start; gap: 0.5rem; margin-top: 0.75rem; border-radius: 0.65rem; background: color-mix(in srgb, var(--primary) 8%, var(--secondary)); padding: 0.55rem; color: var(--muted-foreground); }
  .match-box strong { display: block; color: var(--foreground); font-size: 0.68rem; }
  .match-box p { margin-top: 0.2rem; font-size: 0.61rem; line-height: 1.4; }
  .match-box.weak { background: color-mix(in srgb, var(--destructive) 8%, var(--secondary)); color: var(--destructive); }
</style>
