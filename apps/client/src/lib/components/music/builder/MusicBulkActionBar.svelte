<script lang="ts">
  import CheckSquare from "@lucide/svelte/icons/square-check-big";
  import ListPlus from "@lucide/svelte/icons/list-plus";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import { getLocalization } from "$lib/i18n/translator.svelte";

  let {
    selectedCount,
    allVisibleSelected,
    playlistMode,
    onSelectVisible,
    onClear,
    onAddToPlaylists,
    onRemove = () => undefined,
    onWeight = () => undefined,
    onSnooze,
    onReviewState,
    onSignals,
    onFocusFit = () => undefined,
    onAvailability,
  }: {
    selectedCount: number;
    allVisibleSelected: boolean;
    playlistMode: boolean;
    onSelectVisible: () => void;
    onClear: () => void;
    onAddToPlaylists: () => void;
    onRemove?: () => void;
    onWeight?: () => void;
    onSnooze?: () => void;
    onReviewState?: () => void;
    onSignals?: () => void;
    onFocusFit?: () => void;
    onAvailability?: () => void;
  } = $props();
  const { t } = getLocalization();
</script>

<div class="flex shrink-0 items-center gap-1.5 border-t border-border/70 bg-card px-2 py-2 shadow-[0_-8px_24px_color-mix(in_srgb,var(--background)_45%,transparent)]">
  <span class="shrink-0 text-[0.68rem] font-semibold">{t("music.builder.selectedCount", selectedCount)}</span>
  <div class="flex min-w-0 flex-1 gap-1.5 overflow-x-auto" data-music-scrollable="true">
    <button type="button" onclick={onSelectVisible} class="bulk-button"><CheckSquare size={12} />{allVisibleSelected ? t("music.builder.visibleSelected") : t("music.builder.selectVisible")}</button>
    <button type="button" onclick={onAddToPlaylists} class="bulk-button"><ListPlus size={12} />{t("music.builder.addToPlaylists")}</button>
    {#if playlistMode}<button type="button" onclick={onWeight} class="bulk-button">{t("music.builder.weightLabel")}</button><button type="button" onclick={onRemove} class="bulk-button text-destructive"><Trash2 size={12} />{t("music.builder.removeFromThisPlaylist")}</button>{/if}
    {#if onSnooze}<button type="button" onclick={onSnooze} class="bulk-button">{t("music.builder.snoozeAction")}</button>{/if}
    {#if onReviewState}<button type="button" onclick={onReviewState} class="bulk-button">{t("music.builder.reviewState")}</button>{/if}
    {#if onSignals}<button type="button" onclick={onSignals} class="bulk-button">{t("music.builder.descriptiveSignals")}</button>{/if}
    {#if playlistMode}<button type="button" onclick={onFocusFit} class="bulk-button">{t("music.builder.focusFit")}</button>{/if}
    {#if onAvailability}<button type="button" onclick={onAvailability} class="bulk-button"><CircleAlert size={12} />{t("music.builder.availability")}</button>{/if}
  </div>
  <button type="button" onclick={onClear} class="grid h-7 w-7 shrink-0 place-items-center rounded-md bg-secondary" aria-label={t("music.builder.clearSelection")}><X size={13} /></button>
</div>

<style>
  .bulk-button { display: inline-flex; height: 1.75rem; flex: none; align-items: center; gap: 0.3rem; border-radius: 0.375rem; background: var(--secondary); padding-inline: 0.5rem; color: var(--secondary-foreground); font-size: 0.63rem; font-weight: 500; }
</style>
