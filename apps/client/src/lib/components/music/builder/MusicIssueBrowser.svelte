<script lang="ts">
  import AlertTriangle from "@lucide/svelte/icons/triangle-alert";
  import CheckCircle2 from "@lucide/svelte/icons/circle-check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import FolderSearch from "@lucide/svelte/icons/folder-search";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Youtube from "@lucide/svelte/icons/youtube";
  import { fly } from "svelte/transition";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { groupMusicIssues, type MusicIssueGroup } from "$lib/music/music-issue-presentation";
  import type { MusicIssue } from "$lib/music/library-contracts";
  import MusicBuilderAsyncState from "./MusicBuilderAsyncState.svelte";

  let {
    issues,
    filter = "all",
    expandedGroups,
    compact = false,
    onFilterChange = () => undefined,
    onExpandedGroupsChange,
    onRepair,
    onRefresh,
  }: {
    issues: MusicIssue[];
    filter?: MusicIssueGroup | "all";
    expandedGroups: MusicIssueGroup[];
    compact?: boolean;
    onFilterChange?: (filter: MusicIssueGroup | "all") => void;
    onExpandedGroupsChange: (groups: MusicIssueGroup[]) => void;
    onRepair: (issue: MusicIssue) => void;
    onRefresh: () => void;
  } = $props();
  const { t } = getLocalization();
  const expanded = $derived(new Set(expandedGroups));
  const grouped = $derived(groupMusicIssues(issues));
  const groups = $derived([...grouped.entries()].filter(([group]) => filter === "all" || filter === group));

  function label(group: MusicIssueGroup): string {
    if (group === "missing-local-file") return t("music.builder.missingFiles");
    if (group === "root-unavailable") return t("music.builder.relinkRoot");
    if (group === "ambiguous-match") return t("music.builder.ambiguousMatches");
    if (group === "youtube-unavailable") return t("music.builder.unavailableVideos");
    if (group === "embedding-blocked") return t("music.builder.unavailable");
    return t("music.builder.incompleteRefreshes");
  }

  function toggle(group: MusicIssueGroup): void {
    const next = new Set(expanded);
    if (next.has(group)) next.delete(group); else next.add(group);
    onExpandedGroupsChange([...next]);
  }
</script>

<div class="issue-browser flex h-full min-h-0 flex-col">
  {#if !compact}<div class="flex shrink-0 items-center gap-2 border-b border-border/45 px-3 py-2">
    <div class="min-w-0 flex-1"><h2 class="text-sm font-semibold">{t("music.builder.issues")}</h2><p class="mt-0.5 text-[0.65rem] text-muted-foreground">{t("music.builder.dataPreserved")}</p></div>
    <button type="button" onclick={onRefresh} class="inline-flex h-8 items-center gap-1.5 rounded-lg bg-secondary px-2.5 text-[0.68rem] font-semibold text-secondary-foreground hover:bg-accent"><RefreshCw size={12} />{t("music.builder.refresh")}</button>
  </div>{/if}
  {#if !compact}<div class="flex shrink-0 gap-1.5 overflow-x-auto px-3 py-2">
    <button type="button" onclick={() => onFilterChange("all")} class:active-filter={filter === "all"} class="issue-filter">{t("music.builder.issues")} <span>{issues.length}</span></button>
    {#each [...grouped.entries()] as [group, entries] (group)}<button type="button" onclick={() => onFilterChange(group)} class:active-filter={filter === group} class="issue-filter">{label(group)} <span>{entries.length}</span></button>{/each}
  </div>{/if}
  <div class="issue-scroll min-h-0 flex-1 overflow-y-auto px-3 pb-3">
    {#if issues.length === 0 || groups.length === 0}
      <MusicBuilderAsyncState kind="empty" title={t("music.builder.emptyIssuesTitle")} description={t("music.builder.emptyIssuesDescription")} />
    {:else}
      <div class="space-y-2">
        {#each groups as [group, entries] (group)}
          <section class="overflow-hidden rounded-xl border border-border/60 bg-card/50">
            <button type="button" onclick={() => toggle(group)} class="flex h-9 w-full items-center gap-2 px-2.5 text-left" aria-expanded={expanded.has(group)}>
              <span class="text-muted-foreground">{#if group.includes("youtube") || group === "embedding-blocked"}<Youtube size={14} />{:else if group === "refresh-incomplete"}<RefreshCw size={14} />{:else}<FolderSearch size={14} />{/if}</span><strong class="min-w-0 flex-1 truncate text-[0.7rem]">{label(group)}</strong><span class="rounded-full bg-secondary px-1.5 py-0.5 text-[0.58rem] tabular-nums text-muted-foreground">{entries.length}</span><ChevronDown class={expanded.has(group) ? "rotate-180" : ""} size={12} />
            </button>
            {#if expanded.has(group)}
              <div class="border-t border-border/45 p-1.5">
                {#each entries as issue (issue.id)}
                  <article class="issue-row" transition:fly={{ y: -4, duration: 120 }}>
                    <span class:issue-required={issue.actionRequired} class="mt-0.5 shrink-0 text-muted-foreground">{#if issue.actionRequired}<AlertTriangle size={14} />{:else}<CheckCircle2 size={14} />{/if}</span>
                    <div class="min-w-0 flex-1"><p class="text-[0.7rem] leading-relaxed text-foreground">{issue.message}</p><p class="mt-1 text-[0.61rem] leading-relaxed text-muted-foreground">{t("music.builder.playbackWillSkip")} {t("music.builder.dataPreserved")}</p></div>
                    {#if issue.actionRequired}<button type="button" onclick={() => onRepair(issue)} class="inline-flex h-7 shrink-0 items-center rounded-lg bg-secondary px-2 text-[0.63rem] font-semibold hover:bg-accent">{t("music.builder.repair")}</button>{/if}
                  </article>
                {/each}
              </div>
            {/if}
          </section>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .issue-scroll { scrollbar-width: thin; scrollbar-color: color-mix(in srgb, var(--foreground) 18%, transparent) transparent; }
  .issue-filter { display: inline-flex; height: 1.75rem; flex: none; align-items: center; gap: 0.35rem; border: 1px solid color-mix(in srgb, var(--border) 70%, transparent); border-radius: 999px; padding-inline: 0.6rem; color: var(--muted-foreground); font-size: 0.63rem; white-space: nowrap; }
  .issue-filter span { font-variant-numeric: tabular-nums; }
  .active-filter { border-color: color-mix(in srgb, var(--primary) 35%, var(--border)); background: color-mix(in srgb, var(--primary) 9%, var(--card)); color: var(--foreground); }
  .issue-row { display: flex; align-items: flex-start; gap: 0.6rem; border-radius: 0.65rem; padding: 0.55rem; }
  .issue-row:hover { background: color-mix(in srgb, var(--accent) 45%, transparent); }
  .issue-required { color: var(--destructive); }
</style>
