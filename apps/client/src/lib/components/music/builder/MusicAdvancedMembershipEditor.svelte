<script lang="ts">
  import Plus from "@lucide/svelte/icons/plus";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { onMount } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicBuilderInspectorController } from "$lib/music/music-builder-inspector.svelte";
  import { formatMusicTimecode, parseMusicTimecode, validateMusicSkipRanges } from "$lib/music/music-membership-settings";
  import type { MusicFocusFit, MusicMembershipSkipRange, MusicPlaylistMembership } from "$lib/music/library-contracts";

  let {
    controller,
    membership,
    durationMs,
    onPreview = () => undefined,
  }: {
    controller: MusicBuilderInspectorController;
    membership: MusicPlaylistMembership;
    durationMs: number | null;
    onPreview?: (membership: MusicPlaylistMembership) => void;
  } = $props();

  const { t } = getLocalization();
  let startDraft = $state("");
  let endDraft = $state("");
  let volumeDraft = $state("");
  let rateDraft = $state("");
  let enabled = $state(true);
  let focusFit = $state<MusicFocusFit>("unknown");
  let ranges = $state<Array<{ id: string; start: string; end: string }>>([]);
  let error = $state<string | null>(null);

  onMount(() => resetDraft());

  function resetDraft(): void {
    startDraft = formatMusicTimecode(membership.startMs);
    endDraft = formatMusicTimecode(membership.endMs);
    volumeDraft = membership.volume === null ? "" : String(membership.volume);
    rateDraft = membership.rate === null ? "" : String(membership.rate);
    enabled = membership.enabled;
    focusFit = membership.focusFit;
    ranges = (controller.detail?.membershipSkipRanges ?? [])
      .filter((range) => range.membershipId === membership.id)
      .map((range) => ({ id: range.id, start: formatMusicTimecode(range.startMs), end: formatMusicTimecode(range.endMs) }));
    error = null;
  }

  function addRange(): void {
    ranges = [...ranges, { id: crypto.randomUUID(), start: "", end: "" }];
  }

  function removeRange(id: string): void {
    ranges = ranges.filter((range) => range.id !== id);
  }

  async function save(): Promise<void> {
    const startMs = startDraft.trim() ? parseMusicTimecode(startDraft) : null;
    const endMs = endDraft.trim() ? parseMusicTimecode(endDraft) : null;
    const volume = volumeDraft.trim() ? Number(volumeDraft) : null;
    const rate = rateDraft.trim() ? Number(rateDraft) : null;
    if ((startDraft.trim() && startMs === null) || (endDraft.trim() && endMs === null)
      || (startMs !== null && endMs !== null && endMs <= startMs)
      || (durationMs !== null && endMs !== null && endMs > durationMs)
      || (volume !== null && (!Number.isFinite(volume) || volume < 0 || volume > 1))
      || (rate !== null && (!Number.isFinite(rate) || rate < 0.25 || rate > 2))) {
      error = t("music.builder.advancedSettingsInvalid");
      return;
    }
    const parsedRanges: MusicMembershipSkipRange[] = [];
    for (const [sortOrder, range] of ranges.entries()) {
      const rangeStart = parseMusicTimecode(range.start);
      const rangeEnd = parseMusicTimecode(range.end);
      if (rangeStart === null || rangeEnd === null) { error = t("music.builder.advancedSettingsInvalid"); return; }
      parsedRanges.push({ id: range.id, membershipId: membership.id, startMs: rangeStart, endMs: rangeEnd, sortOrder });
    }
    if (validateMusicSkipRanges(parsedRanges)) { error = t("music.builder.skipRangesOverlap"); return; }
    const previous = { ...membership };
    Object.assign(membership, { startMs, endMs, volume, rate, enabled, focusFit });
    if (await controller.saveAdvancedMembership(membership, parsedRanges)) {
      error = null;
    } else {
      Object.assign(membership, previous);
      error = controller.error?.message ?? t("music.builder.advancedSettingsInvalid");
    }
  }
</script>

<div class="space-y-2 px-2 pb-2">
  <div class="grid grid-cols-2 gap-2">
    <label class="text-[0.63rem] text-muted-foreground">{t("music.builder.segmentStart")}<input bind:value={startDraft} placeholder="0:00" class="membership-input" /></label>
    <label class="text-[0.63rem] text-muted-foreground">{t("music.builder.segmentEnd")}<input bind:value={endDraft} placeholder={formatMusicTimecode(durationMs)} class="membership-input" /></label>
    <label class="text-[0.63rem] text-muted-foreground">{t("music.builder.volumeOverride")}<input bind:value={volumeDraft} inputmode="decimal" placeholder={t("music.builder.itemDefault")} class="membership-input" /></label>
    <label class="text-[0.63rem] text-muted-foreground">{t("music.builder.rateOverride")}<input bind:value={rateDraft} inputmode="decimal" placeholder={t("music.builder.itemDefault")} class="membership-input" /></label>
  </div>
  <label class="flex items-center gap-2 rounded-md bg-secondary/55 p-2 text-[0.65rem]"><input type="checkbox" bind:checked={enabled} class="accent-primary" />{t("music.builder.membershipEnabled")}</label>
  <label class="block text-[0.63rem] text-muted-foreground">{t("music.builder.focusFit")}<select bind:value={focusFit} class="membership-input"><option value="unknown">{t("music.builder.focusFitValue.unknown")}</option><option value="helpful">{t("music.builder.focusFitValue.helpful")}</option><option value="neutral">{t("music.builder.focusFitValue.neutral")}</option><option value="potentially-distracting">{t("music.builder.focusFitValue.potentially-distracting")}</option></select></label>
  <p class="text-[0.6rem] leading-relaxed text-muted-foreground">{t("music.builder.focusFitHint")}</p>
  <div class="rounded-lg border border-border/55 p-2">
    <div class="flex items-center justify-between"><strong class="text-[0.65rem]">{t("music.builder.skipRanges")}</strong><button type="button" onclick={addRange} class="inline-flex h-7 items-center gap-1 rounded-md bg-secondary px-2 text-[0.62rem]"><Plus size={11} />{t("music.builder.addRange")}</button></div>
    {#each ranges as range (range.id)}
      <div class="mt-1.5 grid grid-cols-[1fr_1fr_auto] gap-1.5"><input bind:value={range.start} placeholder="0:00" class="membership-input mt-0" aria-label={t("music.builder.segmentStart")} /><input bind:value={range.end} placeholder="0:10" class="membership-input mt-0" aria-label={t("music.builder.segmentEnd")} /><button type="button" onclick={() => removeRange(range.id)} class="grid h-8 w-8 place-items-center rounded-md bg-secondary" aria-label={t("music.builder.removeRange")}><Trash2 size={11} /></button></div>
    {/each}
  </div>
  {#if error}<p class="text-[0.62rem] text-destructive" role="alert">{error}</p>{/if}
  <p class="text-[0.6rem] leading-relaxed text-muted-foreground">{t("music.builder.capabilitySettingsHint")}</p>
  <div class="flex flex-wrap justify-end gap-1.5"><button type="button" onclick={resetDraft} class="h-7 rounded-md bg-secondary px-2 text-[0.63rem]">{t("music.builder.resetOverrides")}</button><button type="button" onclick={() => onPreview(membership)} class="h-7 rounded-md bg-secondary px-2 text-[0.63rem]">{t("music.builder.previewSegment")}</button><button type="button" onclick={() => { void save(); }} disabled={controller.saving} class="h-7 rounded-md bg-primary px-2 text-[0.63rem] font-medium text-primary-foreground disabled:opacity-40">{t("music.builder.savePlaylist")}</button></div>
</div>

<style>
  .membership-input { margin-top: 0.25rem; height: 2rem; width: 100%; border: 1px solid color-mix(in srgb, var(--border) 70%, transparent); border-radius: 0.375rem; background: var(--background); padding-inline: 0.5rem; color: var(--foreground); font-size: 0.66rem; outline: none; }
</style>
