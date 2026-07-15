<script lang="ts">
  import Gauge from "@lucide/svelte/icons/gauge";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicBulkEditController } from "$lib/music/music-bulk-edit-controller.svelte";
  import type { MusicWeight } from "$lib/music/library-contracts";

  let {
    controller,
    playlistId,
    onClose,
    onSaved,
  }: {
    controller: MusicBulkEditController;
    playlistId: string;
    onClose: () => void;
    onSaved: () => void;
  } = $props();
  const { t } = getLocalization();
  const weights: MusicWeight[] = ["rarely", "less-often", "normal", "more-often", "much-more-often"];

  async function choose(weight: MusicWeight): Promise<void> {
    if (await controller.setWeight(playlistId, weight)) onSaved();
  }
</script>

<div class="absolute inset-0 z-60 grid place-items-center bg-background/65 p-3 backdrop-blur-sm">
  <div role="dialog" aria-modal="true" aria-labelledby="music-bulk-weight-title" class="w-full max-w-sm rounded-xl border border-border/70 bg-card p-4 shadow-2xl">
    <div class="flex items-center gap-3"><div class="grid h-9 w-9 place-items-center rounded-lg bg-primary/10 text-primary"><Gauge size={17} /></div><div><h2 id="music-bulk-weight-title" class="text-sm font-semibold">{t("music.builder.bulkWeightTitle")}</h2><p class="mt-0.5 text-[0.68rem] text-muted-foreground">{t("music.builder.bulkWeightDescription", controller.itemIds.length)}</p></div></div>
    <div class="mt-4 grid gap-1.5">
      {#each weights as weight}
        <button type="button" onclick={() => { void choose(weight); }} disabled={controller.saving} class="flex min-h-10 items-center justify-between rounded-lg border border-border/60 bg-background px-3 text-left text-xs transition hover:border-primary/50 hover:bg-primary/5 disabled:opacity-40"><span class="font-medium">{t(`music.builder.weight.${weight}`)}</span>{#if weight === "normal"}<span class="text-[0.62rem] text-muted-foreground">{t("music.builder.defaultLabel")}</span>{/if}</button>
      {/each}
    </div>
    {#if controller.error}<p class="mt-3 text-[0.68rem] text-destructive" role="alert">{controller.error}</p>{/if}
    <div class="mt-4 flex justify-end"><button type="button" onclick={onClose} class="h-8 rounded-md bg-secondary px-3 text-xs font-medium">{t("music.builder.cancel")}</button></div>
  </div>
</div>
