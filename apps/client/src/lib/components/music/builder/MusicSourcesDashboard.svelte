<script lang="ts">
  import AlertTriangle from "@lucide/svelte/icons/triangle-alert";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import MoreHorizontal from "@lucide/svelte/icons/ellipsis";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Youtube from "@lucide/svelte/icons/youtube";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicSourceSummary } from "$lib/music/library-contracts";
  import type { MusicSourcesController } from "$lib/music/music-sources-controller.svelte";
  import MusicBuilderAsyncState from "./MusicBuilderAsyncState.svelte";
  import MusicDetectedFolderCard from "./MusicDetectedFolderCard.svelte";

  let {
    controller,
    summaries,
    onAdd,
    onRefreshAll,
    onRefreshSource,
    onRelink,
    onRemove,
    onOpenIssues,
    onDetectedFolderAdded = () => undefined,
  }: {
    controller: MusicSourcesController;
    summaries: MusicSourceSummary[];
    onAdd: () => void;
    onRefreshAll: () => void;
    onRefreshSource: (collectionId: string) => void;
    onRelink: (collectionId: string) => void;
    onRemove: (collectionId: string) => void;
    onOpenIssues: () => void;
    onDetectedFolderAdded?: () => void;
  } = $props();

  const { t, locale } = getLocalization();
  const summaryById = $derived(new Map(summaries.map((summary) => [summary.id, summary])));
  const totalIssues = $derived(summaries.reduce((total, summary) => total + summary.openIssueCount, 0));
  const totalNew = $derived(summaries.reduce((total, summary) => total + summary.newCount, 0));
  const totalMissing = $derived(summaries.reduce((total, summary) => total + summary.missingCount, 0));
  const totalAmbiguous = $derived(summaries.reduce((total, summary) => total + summary.ambiguousCount, 0));
  const totalUnavailable = $derived(summaries.reduce((total, summary) => total + summary.unavailableCount, 0));

  function healthLabel(summary: MusicSourceSummary | undefined): string {
    if (!summary) return t("music.builder.unknownAvailability");
    if (summary.health === "healthy") return t("music.builder.sourceHealthy");
    if (summary.health === "stale") return t("music.builder.sourceStale");
    if (summary.health === "disabled") return t("music.builder.sourceDisabled");
    return t("music.builder.sourceIssues");
  }

  function formatRefresh(value: number | null): string {
    if (!value) return t("music.builder.neverRefreshed");
    return new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeStyle: "short" }).format(value);
  }
</script>

<div class="source-dashboard h-full min-h-0 overflow-y-auto p-3">
  <div class="mb-3 flex flex-wrap items-start gap-3">
    <div class="min-w-48 flex-1">
      <h2 class="text-sm font-semibold text-foreground">{t("music.builder.sourceDashboardTitle")}</h2>
      <p class="mt-1 max-w-2xl text-[0.7rem] leading-relaxed text-muted-foreground">{t("music.builder.sourceDashboardDescription")}</p>
    </div>
    <div class="flex shrink-0 gap-1.5">
      <button type="button" onclick={onRefreshAll} disabled={controller.collections.length === 0} class="source-secondary"><RefreshCw size={13} />{t("music.builder.refreshAll")}</button>
      <button type="button" onclick={onAdd} class="source-primary"><Plus size={13} />{t("music.builder.addSource")}</button>
    </div>
  </div>

  {#if controller.detectedDefaultFolder}
    <div class="mb-3"><MusicDetectedFolderCard {controller} onAdded={onDetectedFolderAdded} /></div>
  {/if}

  {#if controller.busy && controller.collections.length === 0}
    <MusicBuilderAsyncState kind="loading" />
  {:else if controller.error && controller.collections.length === 0}
    <MusicBuilderAsyncState kind="error" title={controller.error} onRetry={() => { void controller.load(); }} />
  {:else if controller.collections.length === 0}
    {#if !controller.detectedDefaultFolder}
      <MusicBuilderAsyncState kind="empty" title={t("music.builder.emptySourcesTitle")} description={t("music.builder.emptySourcesDescription")} actionLabel={t("music.builder.addSource")} onAction={onAdd} />
    {/if}
  {:else}
    {#if totalIssues > 0 || totalNew > 0}
      <button type="button" onclick={onOpenIssues} class="mb-3 flex w-full items-center gap-3 rounded-xl border border-border/65 bg-card/65 p-2.5 text-left hover:border-primary/30 hover:bg-accent/35">
        <span class="grid h-9 w-9 shrink-0 place-items-center rounded-xl bg-destructive/10 text-destructive"><AlertTriangle size={17} strokeWidth={1.5} /></span>
        <span class="min-w-0 flex-1"><strong class="block text-xs font-semibold">{t("music.builder.issueSummary")}</strong><span class="mt-1 flex flex-wrap gap-x-2 gap-y-1 text-[0.65rem] text-muted-foreground"><span>{t("music.builder.unreviewedCount", totalNew)}</span><span>{t("music.builder.missingFiles")}: {totalMissing}</span><span>{t("music.builder.ambiguousMatches")}: {totalAmbiguous}</span><span>{t("music.builder.unavailableVideos")}: {totalUnavailable}</span></span></span>
        <span class="rounded-full bg-destructive/10 px-2 py-0.5 text-[0.65rem] font-semibold tabular-nums text-destructive">{totalIssues}</span>
      </button>
    {/if}

    <div class="grid grid-cols-[repeat(auto-fill,minmax(min(17rem,100%),1fr))] gap-2.5">
      {#each controller.collections as collection (collection.id)}
        {@const summary = summaryById.get(collection.id)}
        {@const status = controller.refreshStatuses[collection.id]}
        <article class="source-card">
          <div class="flex items-start gap-2.5">
            <span class="source-icon">{#if collection.kind === "local-root"}<FolderOpen size={18} strokeWidth={1.4} />{:else}<Youtube size={19} strokeWidth={1.4} />{/if}</span>
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2"><h3 class="min-w-0 flex-1 truncate text-xs font-semibold" title={collection.name}>{collection.name}</h3><span class:health-warning={summary?.health === "issues"} class:health-stale={summary?.health === "stale"} class="health-dot"></span></div>
              <p class="mt-0.5 truncate text-[0.63rem] text-muted-foreground">{collection.kind === "local-root" ? t("music.builder.localFolder") : t("music.builder.youtubePlaylist")}</p>
            </div>
            <button type="button" onclick={() => onRemove(collection.id)} class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-muted-foreground hover:bg-accent hover:text-accent-foreground" aria-label={t("music.builder.moreActions")}><MoreHorizontal size={14} /></button>
          </div>

          <div class="mt-3 grid grid-cols-3 gap-1.5">
            <div class="source-stat"><strong>{summary?.itemCount ?? 0}</strong><span>{t("music.tracks", summary?.itemCount ?? 0)}</span></div>
            <div class="source-stat"><strong>{summary?.newCount ?? 0}</strong><span>{t("music.builder.unreviewed")}</span></div>
            <div class="source-stat"><strong>{summary?.openIssueCount ?? 0}</strong><span>{t("music.builder.issues")}</span></div>
          </div>

          <div class="mt-3 min-h-8">
            {#if status && ["queued", "running"].includes(status.state)}
              <div class="rounded-lg bg-primary/8 px-2 py-1.5">
                <div class="flex items-center gap-2 text-[0.63rem]"><RefreshCw class="animate-spin motion-reduce:animate-none" size={11} /><span class="min-w-0 flex-1 truncate">{status.progress ? t("music.builder.refreshProgress", status.progress.processedCount, status.progress.discoveredCount) : t("music.builder.refreshing")}</span><button type="button" class="font-medium hover:underline" onclick={() => { void controller.cancelRefresh(collection.id); }}>{t("music.builder.cancel")}</button></div>
                {#if status.progress}<div class="mt-1 h-1 overflow-hidden rounded-full bg-secondary"><div class="h-full rounded-full bg-primary transition-[width] motion-reduce:transition-none" style={`width: ${status.progress.discoveredCount > 0 ? Math.min(100, status.progress.processedCount / status.progress.discoveredCount * 100) : 5}%`}></div></div>{/if}
              </div>
            {:else}
              <p class="text-[0.63rem] text-muted-foreground"><span class="font-medium text-foreground/80">{healthLabel(summary)}</span><br />{t("music.builder.lastRefresh")}: {formatRefresh(summary?.lastSuccessfulRefreshAt ?? collection.lastSuccessfulRefreshAt)}</p>
            {/if}
          </div>

          <div class="mt-3 flex gap-1.5 border-t border-border/45 pt-2.5">
            <button type="button" class="source-secondary flex-1" onclick={() => onRefreshSource(collection.id)}><RefreshCw size={12} />{t("music.builder.refreshSource")}</button>
            {#if collection.kind === "local-root" && (summary?.missingCount || summary?.ambiguousCount)}
              <button type="button" class="source-secondary" onclick={() => onRelink(collection.id)}>{t("music.builder.repair")}</button>
            {/if}
          </div>
        </article>
      {/each}
    </div>
  {/if}
</div>

<style>
  .source-dashboard { scrollbar-width: thin; scrollbar-color: color-mix(in srgb, var(--foreground) 18%, transparent) transparent; }
  .source-primary, .source-secondary { display: inline-flex; height: 2rem; align-items: center; justify-content: center; gap: 0.35rem; border-radius: 0.6rem; padding-inline: 0.7rem; font-size: 0.68rem; font-weight: 600; white-space: nowrap; }
  .source-primary { background: var(--primary); color: var(--primary-foreground); }
  .source-secondary { background: var(--secondary); color: var(--secondary-foreground); }
  .source-secondary:hover { background: var(--accent); color: var(--accent-foreground); }
  .source-secondary:disabled { opacity: 0.45; }
  .source-card { min-width: 0; overflow: hidden; border: 1px solid color-mix(in srgb, var(--border) 65%, transparent); border-radius: 0.9rem; background: color-mix(in srgb, var(--card) 76%, transparent); padding: 0.75rem; box-shadow: 0 1px 0 color-mix(in srgb, white 3%, transparent); }
  .source-icon { display: grid; height: 2.3rem; width: 2.3rem; flex: none; place-items: center; border-radius: 0.7rem; background: color-mix(in srgb, var(--primary) 9%, var(--secondary)); color: var(--muted-foreground); }
  .health-dot { height: 0.5rem; width: 0.5rem; flex: none; border-radius: 999px; background: color-mix(in srgb, var(--primary) 75%, var(--muted)); }
  .health-warning { background: var(--destructive); }
  .health-stale { background: color-mix(in srgb, var(--destructive) 55%, var(--foreground)); }
  .source-stat { display: flex; min-width: 0; flex-direction: column; border-radius: 0.55rem; background: color-mix(in srgb, var(--secondary) 65%, transparent); padding: 0.35rem 0.45rem; }
  .source-stat strong { font-size: 0.75rem; font-variant-numeric: tabular-nums; }
  .source-stat span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--muted-foreground); font-size: 0.58rem; }
</style>
