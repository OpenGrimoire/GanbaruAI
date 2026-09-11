<script lang="ts">
  import FolderX from "@lucide/svelte/icons/folder-x";
  import Library from "@lucide/svelte/icons/library";
  import RadioTower from "@lucide/svelte/icons/radio-tower";
  import ShieldCheck from "@lucide/svelte/icons/shield-check";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { containMusicDialogFocus } from "$lib/music/music-dialog-focus";
  import type { MusicSourceCollection } from "$lib/music/library-contracts";
  import type { MusicSourcesController } from "$lib/music/music-sources-controller.svelte";

  let { controller, collection, onClose, onRemoved }: { controller: MusicSourcesController; collection: MusicSourceCollection; onClose: () => void; onRemoved: () => void } = $props();
  const { t } = getLocalization();
  let choice = $state<"binding" | "source" | "orphans">("binding");
  let saving = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (collection.kind !== "local-root" && choice === "binding") choice = "source";
  });

  async function confirm(): Promise<void> {
    saving = true;
    error = null;
    try {
      if (choice === "binding" && collection.localRootId) await controller.forgetBinding(collection.localRootId);
      else await controller.confirmRemoval(collection, choice === "orphans");
      onRemoved();
    } catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
    finally { saving = false; }
  }
</script>

<div class="absolute inset-0 z-50 grid place-items-center bg-background/60 p-2 backdrop-blur-sm">
  <button type="button" class="absolute inset-0" onclick={onClose} aria-label={t("music.builder.close")}></button>
  <div use:containMusicDialogFocus={{ onEscape: onClose, escapeDisabled: saving, onEnter: () => { void confirm(); }, enterDisabled: saving }} class="relative flex max-h-full w-[min(34rem,100%)] flex-col overflow-hidden rounded-2xl border border-border/70 bg-popover shadow-2xl" role="alertdialog" aria-modal="true" aria-label={t("music.builder.sourceRemovalTitle")} tabindex="-1">
    <header class="border-b border-border/55 px-4 py-3"><h2 class="text-sm font-semibold">{t("music.builder.sourceRemovalTitle")}</h2><p class="mt-1 truncate text-[0.68rem] text-muted-foreground">{collection.name}</p></header>
    <div class="min-h-0 flex-1 overflow-y-auto p-3">
      <div class="flex items-start gap-2 rounded-xl bg-primary/7 p-2.5 text-[0.68rem] leading-relaxed text-muted-foreground"><ShieldCheck class="mt-0.5 shrink-0 text-primary" size={15} /><span>{t("music.builder.sourceRemovalSafe")}</span></div>
      <div class="mt-3 space-y-1.5">
        {#if collection.kind === "local-root"}<button type="button" onclick={() => choice = "binding"} class:selected={choice === "binding"} class="removal-choice"><span><FolderX size={17} /></span><span class="min-w-0 flex-1"><strong>{t("music.builder.removeBinding")}</strong><small>{t("music.builder.dataPreserved")}</small></span><i></i></button>{/if}
        <button type="button" onclick={() => choice = "source"} class:selected={choice === "source"} class="removal-choice"><span><RadioTower size={17} /></span><span class="min-w-0 flex-1"><strong>{t("music.builder.stopDiscovery")}</strong><small>{t("music.builder.dataPreserved")}</small></span><i></i></button>
        <button type="button" onclick={() => choice = "orphans"} class:selected={choice === "orphans"} class="removal-choice"><span><Library size={17} /></span><span class="min-w-0 flex-1"><strong>{t("music.builder.removeOrphans")}</strong><small>{t("music.builder.affectedItems", controller.removalImpact?.orphanedItemCount ?? 0)}</small></span><i></i></button>
      </div>
      {#if controller.removalImpact}<div class="mt-3 grid grid-cols-2 gap-2"><div class="impact"><strong>{controller.removalImpact.itemCount}</strong><span>{t("music.builder.affectedItems", controller.removalImpact.itemCount)}</span></div><div class="impact"><strong>{controller.removalImpact.membershipCount}</strong><span>{t("music.builder.affectedMemberships", controller.removalImpact.membershipCount)}</span></div></div>{/if}
      {#if error}<p class="mt-3 rounded-lg bg-destructive/10 px-2.5 py-2 text-[0.68rem] text-destructive" role="alert">{error}</p>{/if}
    </div>
    <footer class="flex shrink-0 justify-end gap-2 border-t border-border/55 px-3 py-2"><button type="button" onclick={onClose} class="h-8 rounded-lg bg-secondary px-3 text-[0.68rem] font-semibold">{t("music.builder.cancel")} ({t("common.escapeKey")})</button><button type="button" onclick={() => { void confirm(); }} disabled={saving} class="h-8 rounded-lg bg-destructive px-3 text-[0.68rem] font-semibold text-destructive-foreground disabled:opacity-45">{t("music.builder.confirmRemoval")} ({t("common.enterKey")})</button></footer>
  </div>
</div>

<style>
  .removal-choice { display: flex; width: 100%; align-items: center; gap: 0.65rem; border: 1px solid color-mix(in srgb, var(--border) 65%, transparent); border-radius: 0.75rem; padding: 0.65rem; text-align: left; }
  .removal-choice > span:first-child { display: grid; height: 2rem; width: 2rem; flex: none; place-items: center; border-radius: 0.6rem; background: var(--secondary); color: var(--muted-foreground); }
  .removal-choice strong, .removal-choice small { display: block; }
  .removal-choice strong { font-size: calc(0.7rem * var(--type-scale)); }
  .removal-choice small { margin-top: 0.15rem; color: var(--muted-foreground); font-size: calc(0.61rem * var(--type-scale)); }
  .removal-choice i { height: 0.8rem; width: 0.8rem; flex: none; border: 1px solid var(--border); border-radius: 999px; }
  .removal-choice.selected { border-color: color-mix(in srgb, var(--primary) 42%, var(--border)); background: color-mix(in srgb, var(--primary) 7%, var(--card)); }
  .removal-choice.selected i { border: 3px solid var(--primary); }
  .impact { display: flex; min-width: 0; flex-direction: column; border-radius: 0.65rem; background: var(--secondary); padding: 0.55rem; }
  .impact strong { font-size: calc(0.8rem * var(--type-scale)); }
  .impact span { color: var(--muted-foreground); font-size: calc(0.6rem * var(--type-scale)); }
</style>
