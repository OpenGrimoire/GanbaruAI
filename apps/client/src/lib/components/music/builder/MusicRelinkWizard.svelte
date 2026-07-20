<script lang="ts">
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import Check from "@lucide/svelte/icons/check";
  import FolderSearch from "@lucide/svelte/icons/folder-search";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { containMusicDialogFocus } from "$lib/music/music-dialog-focus";
  import type { MusicSourceCollection } from "$lib/music/library-contracts";
  import type { MusicSourcesController } from "$lib/music/music-sources-controller.svelte";

  let { controller, collection, onClose, onApplied }: { controller: MusicSourcesController; collection: MusicSourceCollection; onClose: () => void; onApplied: () => void } = $props();
  const { t } = getLocalization();
  let step = $state<"choose" | "planning" | "review" | "applying">("choose");
  let replacementPath = $state("");
  let error = $state<string | null>(null);
  let decisions = $state<Record<string, string | null | undefined>>({});
  const ambiguousEntries = $derived(controller.relinkEntries.filter((entry) => entry.matchKind === "ambiguous"));
  const everyAmbiguityResolved = $derived(ambiguousEntries.every((entry) => decisions[entry.id] !== undefined));

  async function chooseFolder(): Promise<void> {
    error = null;
    const selected = await controller.chooseLocalFolder();
    if (!selected) return;
    replacementPath = selected.selection.folderPath;
    step = "planning";
    try { await controller.planRelink(collection.localRootId!, replacementPath); step = "review"; }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); step = "choose"; }
  }

  async function apply(): Promise<void> {
    if (!everyAmbiguityResolved) return;
    step = "applying";
    error = null;
    try {
      await controller.applyRelink(Object.entries(decisions).flatMap(([entryId, itemId]) => itemId ? [{ entryId, itemId }] : []));
      onApplied();
    } catch (cause) { error = cause instanceof Error ? cause.message : String(cause); step = "review"; }
  }

  async function close(): Promise<void> {
    if (controller.relinkPlan?.state === "ready") await controller.cancelRelink();
    onClose();
  }
</script>

<div class="absolute inset-0 z-50 grid place-items-center bg-background/60 p-2 backdrop-blur-sm">
  <button type="button" class="absolute inset-0" onclick={() => { void close(); }} aria-label={t("music.builder.close")}></button>
  <div use:containMusicDialogFocus={{ onEscape: () => { void close(); }, escapeDisabled: step === "applying" }} class="relative flex max-h-full w-[min(42rem,100%)] flex-col overflow-hidden rounded-2xl border border-border/70 bg-popover shadow-2xl" role="dialog" aria-modal="true" aria-label={t("music.builder.relinkRoot")} tabindex="-1">
    <header class="flex min-h-11 items-center gap-2 border-b border-border/55 px-3"><button type="button" onclick={() => { void close(); }} class="inline-flex h-7 w-7 items-center justify-center rounded-lg text-muted-foreground hover:bg-accent" aria-label={t("music.builder.back")}><ArrowLeft size={14} /></button><h2 class="min-w-0 flex-1 truncate text-sm font-semibold">{t("music.builder.relinkRoot")}: {collection.name}</h2></header>
    <div class="relink-scroll min-h-0 flex-1 overflow-y-auto p-3">
      <ol class="mb-4 grid grid-cols-3 gap-1.5 text-[0.62rem]">
        <li class:step-active={step === "choose"} class="step"><span>1</span>{t("music.builder.chooseFolder")}</li>
        <li class:step-active={step === "planning"} class="step"><span>2</span>{t("music.builder.refreshing")}</li>
        <li class:step-active={step === "review" || step === "applying"} class="step"><span>3</span>{t("music.builder.relinkReview")}</li>
      </ol>
      {#if step === "choose"}
        <div class="grid min-h-44 place-items-center rounded-xl border border-dashed border-border/75 bg-card/35 p-5 text-center"><div><span class="mx-auto grid h-12 w-12 place-items-center rounded-2xl bg-secondary text-muted-foreground"><FolderSearch size={22} /></span><h3 class="mt-3 text-xs font-semibold">{t("music.builder.relinkChoose")}</h3><p class="mt-1 max-w-sm text-[0.68rem] leading-relaxed text-muted-foreground">{t("music.builder.dataPreserved")}</p><button type="button" onclick={() => { void chooseFolder(); }} class="mt-4 h-8 rounded-lg bg-primary px-3 text-[0.68rem] font-semibold text-primary-foreground">{t("music.builder.chooseFolder")}</button></div></div>
      {:else if step === "planning"}
        <div class="grid min-h-44 place-items-center"><div class="text-center"><LoaderCircle class="mx-auto animate-spin text-muted-foreground motion-reduce:animate-none" size={23} /><p class="mt-2 text-[0.68rem] text-muted-foreground">{t("music.builder.refreshing")}</p></div></div>
      {:else if controller.relinkPlan}
        <div class="grid grid-cols-5 gap-1.5 max-[480px]:grid-cols-3">
          <div class="relink-stat"><strong>{controller.relinkPlan.exactCount}</strong><span>{t("music.builder.exactMatches")}</span></div><div class="relink-stat"><strong>{controller.relinkPlan.likelyCount}</strong><span>{t("music.builder.likelyMatches")}</span></div><div class="relink-stat"><strong>{controller.relinkPlan.ambiguousCount}</strong><span>{t("music.builder.ambiguousMatches")}</span></div><div class="relink-stat"><strong>{controller.relinkPlan.missingCount}</strong><span>{t("music.builder.missing")}</span></div><div class="relink-stat"><strong>{controller.relinkPlan.newCount}</strong><span>{t("music.builder.newMatches")}</span></div>
        </div>
        {#if ambiguousEntries.length > 0}
          <div class="mt-3 space-y-1.5">
            {#each ambiguousEntries as entry (entry.id)}
              <div class="rounded-xl border border-border/60 bg-card/45 p-2.5"><p class="truncate text-[0.7rem] font-medium">{entry.candidateRelativePath ?? entry.id}</p><p class="mt-0.5 text-[0.61rem] text-muted-foreground">{t("music.builder.ambiguous")}</p><div class="mt-2 flex flex-wrap gap-1.5">{#each entry.candidateItemIds as itemId}<button type="button" onclick={() => decisions[entry.id] = itemId} class:selected-decision={decisions[entry.id] === itemId} class="decision">{#if decisions[entry.id] === itemId}<Check size={11} />{/if}{itemId}</button>{/each}<button type="button" onclick={() => decisions[entry.id] = null} class:selected-decision={decisions[entry.id] === null} class="decision">{t("music.builder.missing")}</button></div></div>
            {/each}
          </div>
          {#if !everyAmbiguityResolved}<p class="mt-2 text-[0.65rem] text-destructive">{t("music.builder.unresolvedAmbiguities")}</p>{/if}
        {:else}<p class="mt-3 rounded-xl bg-primary/8 p-2.5 text-[0.68rem] text-muted-foreground">{t("music.builder.dataPreserved")}</p>{/if}
      {/if}
      {#if error}<p class="mt-3 rounded-lg bg-destructive/10 px-2.5 py-2 text-[0.68rem] text-destructive" role="alert">{error}</p>{/if}
    </div>
    <footer class="flex shrink-0 justify-end gap-2 border-t border-border/55 px-3 py-2"><button type="button" onclick={() => { void close(); }} class="h-8 rounded-lg bg-secondary px-3 text-[0.68rem] font-semibold">{t("music.builder.cancel")}</button>{#if step === "review" || step === "applying"}<button type="button" onclick={() => { void apply(); }} disabled={!everyAmbiguityResolved || step === "applying"} class="h-8 rounded-lg bg-primary px-3 text-[0.68rem] font-semibold text-primary-foreground disabled:opacity-45">{t("music.builder.applyRelink")}</button>{/if}</footer>
  </div>
</div>

<style>
  .relink-scroll { scrollbar-width: thin; scrollbar-color: color-mix(in srgb, var(--foreground) 18%, transparent) transparent; }
  .step { display: flex; min-width: 0; align-items: center; gap: 0.35rem; border-radius: 0.55rem; background: color-mix(in srgb, var(--secondary) 55%, transparent); padding: 0.35rem 0.5rem; color: var(--muted-foreground); }
  .step span { display: grid; height: 1.1rem; width: 1.1rem; flex: none; place-items: center; border-radius: 999px; background: var(--background); font-size: 0.55rem; }
  .step-active { background: color-mix(in srgb, var(--primary) 10%, var(--secondary)); color: var(--foreground); font-weight: 600; }
  .relink-stat { display: flex; min-width: 0; flex-direction: column; border-radius: 0.65rem; background: var(--secondary); padding: 0.5rem; }
  .relink-stat strong { font-size: 0.8rem; font-variant-numeric: tabular-nums; }
  .relink-stat span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--muted-foreground); font-size: 0.58rem; }
  .decision { display: inline-flex; min-height: 1.7rem; max-width: 100%; align-items: center; gap: 0.25rem; overflow: hidden; border: 1px solid var(--border); border-radius: 0.55rem; padding-inline: 0.55rem; color: var(--muted-foreground); font-size: 0.61rem; text-overflow: ellipsis; white-space: nowrap; }
  .selected-decision { border-color: color-mix(in srgb, var(--primary) 45%, var(--border)); background: color-mix(in srgb, var(--primary) 10%, var(--card)); color: var(--foreground); }
</style>
