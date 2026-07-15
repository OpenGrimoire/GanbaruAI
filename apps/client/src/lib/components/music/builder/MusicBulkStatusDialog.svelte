<script lang="ts">
  import Clock3 from "@lucide/svelte/icons/clock-3";
  import CheckCircle2 from "@lucide/svelte/icons/circle-check-big";
  import { onMount } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicBulkEditController } from "$lib/music/music-bulk-edit-controller.svelte";
  import type { MusicReviewState, MusicSnoozeScope } from "$lib/music/library-contracts";

  let {
    controller,
    mode,
    playlistId = null,
    onClose,
    onSaved,
  }: {
    controller: MusicBulkEditController;
    mode: "review" | "snooze";
    playlistId?: string | null;
    onClose: () => void;
    onSaved: () => void;
  } = $props();
  const { t } = getLocalization();
  let scope = $state<MusicSnoozeScope>("all-playlists");

  onMount(() => {
    scope = playlistId ? "playlist" : "all-playlists";
  });

  async function review(state: MusicReviewState): Promise<void> {
    if (await controller.setReviewState(state)) onSaved();
  }

  async function snooze(duration: "today" | "day" | "week" | "until-resumed"): Promise<void> {
    const now = Date.now();
    let endsAt: number | null = null;
    if (duration === "today") {
      const tomorrow = new Date(now);
      tomorrow.setHours(24, 0, 0, 0);
      endsAt = tomorrow.getTime();
    }
    if (duration === "day") endsAt = now + 86_400_000;
    if (duration === "week") endsAt = now + 604_800_000;
    const effectiveScope = scope === "playlist" && playlistId ? "playlist" : "all-playlists";
    if (await controller.snooze(effectiveScope, effectiveScope === "playlist" ? playlistId : null, endsAt)) onSaved();
  }
</script>

<div class="absolute inset-0 z-60 grid place-items-center bg-background/65 p-3 backdrop-blur-sm">
  <div role="dialog" aria-modal="true" aria-labelledby="music-bulk-status-title" class="w-full max-w-sm rounded-xl border border-border/70 bg-card p-4 shadow-2xl">
    <div class="flex items-center gap-3"><div class="grid h-9 w-9 place-items-center rounded-lg bg-primary/10 text-primary">{#if mode === "review"}<CheckCircle2 size={17} />{:else}<Clock3 size={17} />{/if}</div><div><h2 id="music-bulk-status-title" class="text-sm font-semibold">{mode === "review" ? t("music.builder.bulkReviewTitle") : t("music.builder.bulkSnoozeTitle")}</h2><p class="mt-0.5 text-[0.68rem] text-muted-foreground">{t("music.builder.bulkStatusDescription", controller.itemIds.length)}</p></div></div>
    {#if mode === "review"}
      <div class="mt-4 grid gap-1.5">
        <button type="button" onclick={() => { void review("reviewed"); }} class="status-choice">{t("music.builder.markReviewed")}</button>
        <button type="button" onclick={() => { void review("unreviewed"); }} class="status-choice">{t("music.builder.markUnreviewed")}</button>
        <button type="button" onclick={() => { void review("ignored"); }} class="status-choice">{t("music.builder.ignore")}</button>
      </div>
    {:else}
      {#if playlistId}
        <div class="mt-4 grid grid-cols-2 rounded-lg bg-secondary p-1 text-[0.68rem]">
          <button type="button" aria-pressed={scope === "playlist"} onclick={() => scope = "playlist"} class="rounded-md px-2 py-1.5 aria-pressed:bg-card aria-pressed:shadow-sm">{t("music.builder.currentPlaylistOnly")}</button>
          <button type="button" aria-pressed={scope === "all-playlists"} onclick={() => scope = "all-playlists"} class="rounded-md px-2 py-1.5 aria-pressed:bg-card aria-pressed:shadow-sm">{t("music.builder.allPlaylists")}</button>
        </div>
      {/if}
      <div class="mt-3 grid gap-1.5">
        <button type="button" onclick={() => { void snooze("today"); }} class="status-choice">{t("music.builder.restOfToday")}</button>
        <button type="button" onclick={() => { void snooze("day"); }} class="status-choice">{t("music.builder.oneDay")}</button>
        <button type="button" onclick={() => { void snooze("week"); }} class="status-choice">{t("music.builder.oneWeek")}</button>
        <button type="button" onclick={() => { void snooze("until-resumed"); }} class="status-choice">{t("music.builder.untilResumed")}</button>
      </div>
    {/if}
    {#if controller.selectionStale}<p class="mt-3 text-[0.68rem] text-warning" role="alert">{t("music.builder.selectionChanged")}</p>{/if}
    {#if controller.error}<p class="mt-3 text-[0.68rem] text-destructive" role="alert">{controller.error}</p>{/if}
    <div class="mt-4 flex justify-end"><button type="button" onclick={onClose} class="h-8 rounded-md bg-secondary px-3 text-xs font-medium">{t("music.builder.cancel")}</button></div>
  </div>
</div>

<style>
  .status-choice { min-height: 2.5rem; border: 1px solid color-mix(in srgb, var(--border) 65%, transparent); border-radius: 0.6rem; background: var(--background); padding-inline: 0.75rem; text-align: left; font-size: 0.72rem; font-weight: 500; transition: border-color 120ms ease, background-color 120ms ease; }
  .status-choice:hover { border-color: color-mix(in srgb, var(--primary) 50%, var(--border)); background: color-mix(in srgb, var(--primary) 5%, var(--background)); }
</style>
