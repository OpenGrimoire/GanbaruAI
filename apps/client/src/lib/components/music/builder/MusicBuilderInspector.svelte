<script lang="ts">
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Disc3 from "@lucide/svelte/icons/disc-3";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import Play from "@lucide/svelte/icons/play";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import X from "@lucide/svelte/icons/x";
  import { fade } from "svelte/transition";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicBuilderInspectorController } from "$lib/music/music-builder-inspector.svelte";
  import { formatMusicDuration } from "$lib/music/music-builder-presentation";
  import MusicBuilderAsyncState from "./MusicBuilderAsyncState.svelte";

  let {
    controller,
    showClose = false,
    onClose = () => undefined,
    onPlay = () => undefined,
  }: {
    controller: MusicBuilderInspectorController;
    showClose?: boolean;
    onClose?: () => void;
    onPlay?: (itemId: string) => void;
  } = $props();

  const { t } = getLocalization();
  const item = $derived(controller.detail?.item ?? null);
  const title = $derived(item?.titleOverride?.trim() || item?.originalTitle || "");
  const artist = $derived(item?.artistOverride?.trim() || item?.originalArtist || t("music.builder.noArtist"));
  const album = $derived(item?.albumOverride?.trim() || item?.originalAlbum || t("music.builder.noAlbum"));

  function sectionOpen(id: string): boolean { return controller.expandedSections.has(id); }
  function availabilityLabel(): string {
    if (!item) return "";
    if (item.availability === "available") return t("music.builder.available");
    if (item.availability === "missing") return t("music.builder.missing");
    if (item.availability === "unavailable") return t("music.builder.unavailable");
    if (item.availability === "ambiguous") return t("music.builder.ambiguous");
    return t("music.builder.unknownAvailability");
  }
</script>

<aside class="flex h-full min-h-0 flex-col bg-card/55" aria-label={t("music.builder.details")}>
  {#if !controller.itemId}
    <MusicBuilderAsyncState kind="empty" title={t("music.builder.noSelectionTitle")} description={t("music.builder.noSelectionDescription")} />
  {:else if controller.busy}
    <MusicBuilderAsyncState kind="loading" />
  {:else if controller.error || !item}
    <MusicBuilderAsyncState kind="error" title={t("music.builder.inspectorUnavailable")} onRetry={() => { void controller.select(controller.itemId); }} />
  {:else}
    {#key item.id}
      <div class="flex min-h-0 flex-1 flex-col" in:fade={{ duration: 120 }}>
        <div class="relative shrink-0 border-b border-border/50 p-3">
          {#if showClose}
            <button type="button" onclick={onClose} class="absolute right-2 top-2 z-10 inline-flex h-7 w-7 items-center justify-center rounded-lg bg-background/75 text-muted-foreground backdrop-blur-sm hover:bg-accent hover:text-accent-foreground" aria-label={t("music.builder.closeInspector")}><X size={14} /></button>
          {/if}
          <div class="flex gap-3 pr-7">
            <div class="grid h-18 w-18 shrink-0 place-items-center rounded-xl bg-gradient-to-br from-primary/15 to-secondary text-muted-foreground shadow-inner"><Disc3 size={28} strokeWidth={1.25} /></div>
            <div class="min-w-0 flex-1 self-center">
              <h2 class="line-clamp-2 text-sm font-semibold leading-snug text-foreground">{title}</h2>
              <p class="mt-1 truncate text-xs text-muted-foreground">{artist}</p>
              <p class="mt-0.5 truncate text-[0.68rem] text-muted-foreground/80">{album}</p>
            </div>
          </div>
          <div class="mt-3 flex gap-2">
            <button type="button" onclick={() => onPlay(item.id)} class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-lg bg-primary text-xs font-semibold text-primary-foreground hover:bg-primary/90"><Play size={13} fill="currentColor" />{t("music.play")}</button>
            {#if item.sourceKind === "local-file"}
              <button type="button" class="inline-flex h-8 w-9 items-center justify-center rounded-lg bg-secondary text-secondary-foreground hover:bg-accent" aria-label={t("music.showCurrentFileLocation")}><FolderOpen size={14} /></button>
            {/if}
          </div>
        </div>

        <div class="inspector-scroll min-h-0 flex-1 overflow-y-auto p-2">
          <section class="inspector-section">
            <button type="button" onclick={() => controller.toggleSection("details")} aria-expanded={sectionOpen("details")}><span>{t("music.builder.details")}</span><ChevronDown class={sectionOpen("details") ? "rotate-180" : ""} size={13} /></button>
            {#if sectionOpen("details")}
              <dl>
                <div><dt>{t("music.builder.duration")}</dt><dd>{formatMusicDuration(item.durationMs) || "·"}</dd></div>
                <div><dt>{t("music.builder.sourceType")}</dt><dd>{item.sourceKind === "local-file" ? t("music.builder.local") : t("music.builder.youtube")}</dd></div>
                <div><dt>{t("music.builder.availability")}</dt><dd>{availabilityLabel()}</dd></div>
                <div><dt>{t("music.builder.reviewState")}</dt><dd>{item.reviewState}</dd></div>
              </dl>
            {/if}
          </section>

          <section class="inspector-section">
            <button type="button" onclick={() => controller.toggleSection("memberships")} aria-expanded={sectionOpen("memberships")}><span>{t("music.builder.memberships")}</span><ChevronDown class={sectionOpen("memberships") ? "rotate-180" : ""} size={13} /></button>
            {#if sectionOpen("memberships")}
              {#if controller.detail?.memberships.length}
                <div class="space-y-1 px-2 pb-2">
                  {#each controller.detail.memberships as membership (membership.id)}
                    <div class="flex items-center gap-2 rounded-lg bg-secondary/55 px-2 py-1.5 text-[0.68rem]"><ListMusic size={12} /><span class="min-w-0 flex-1 truncate">{membership.playlistId}</span><span class="text-muted-foreground">{membership.weight}</span></div>
                  {/each}
                </div>
              {:else}<p class="px-2 pb-2 text-[0.68rem] text-muted-foreground">{t("music.builder.playlistsMembership", 0)}</p>{/if}
            {/if}
          </section>

          <section class="inspector-section">
            <button type="button" onclick={() => controller.toggleSection("listening")} aria-expanded={sectionOpen("listening")}><span>{t("music.builder.listening")}</span><ChevronDown class={sectionOpen("listening") ? "rotate-180" : ""} size={13} /></button>
            {#if sectionOpen("listening")}
              <dl>
                <div><dt>{t("music.builder.playCount")}</dt><dd>{controller.detail?.statistics?.playCount ?? 0}</dd></div>
                <div><dt>{t("music.builder.completionCount")}</dt><dd>{controller.detail?.statistics?.completionCount ?? 0}</dd></div>
                <div><dt>{t("music.builder.skipCount")}</dt><dd>{controller.detail?.statistics?.skipCount ?? 0}</dd></div>
              </dl>
              <button type="button" class="mx-2 mb-2 inline-flex h-7 items-center gap-1.5 rounded-lg px-2 text-[0.65rem] text-muted-foreground hover:bg-accent hover:text-accent-foreground"><RotateCcw size={11} />{t("music.builder.neverPlayed")}</button>
            {/if}
          </section>
        </div>
      </div>
    {/key}
  {/if}
</aside>

<style>
  .inspector-scroll { scrollbar-width: thin; scrollbar-color: color-mix(in srgb, var(--foreground) 18%, transparent) transparent; }
  .inspector-section { margin-bottom: 0.25rem; overflow: hidden; border: 1px solid color-mix(in srgb, var(--border) 55%, transparent); border-radius: 0.65rem; background: color-mix(in srgb, var(--background) 28%, transparent); }
  .inspector-section > button { display: flex; width: 100%; height: 2rem; align-items: center; justify-content: space-between; padding-inline: 0.6rem; color: var(--foreground); font-size: 0.69rem; font-weight: 600; }
  .inspector-section > button :global(svg) { transition: transform 140ms ease; }
  .inspector-section dl { padding: 0 0.55rem 0.45rem; }
  .inspector-section dl > div { display: flex; min-height: 1.55rem; align-items: center; justify-content: space-between; gap: 0.75rem; border-top: 1px solid color-mix(in srgb, var(--border) 35%, transparent); font-size: 0.65rem; }
  .inspector-section dt { color: var(--muted-foreground); }
  .inspector-section dd { max-width: 60%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; text-align: right; color: var(--foreground); }
  @media (prefers-reduced-motion: reduce) { .inspector-section > button :global(svg) { transition: none; } }
</style>
