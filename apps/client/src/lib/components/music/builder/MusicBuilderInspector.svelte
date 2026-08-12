<script lang="ts">
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Disc3 from "@lucide/svelte/icons/disc-3";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import Play from "@lucide/svelte/icons/play";
  import Pencil from "@lucide/svelte/icons/pencil";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import X from "@lucide/svelte/icons/x";
  import Clock3 from "@lucide/svelte/icons/clock-3";
  import { fade } from "svelte/transition";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { pickArtworkFile } from "$lib/api/music";
  import type { MusicBuilderInspectorController } from "$lib/music/music-builder-inspector.svelte";
  import type { MusicItemSignal } from "$lib/music/library-contracts";
  import { formatDateTime } from "$lib/i18n/formatters";
  import { formatMusicDuration } from "$lib/music/music-builder-presentation";
  import MusicBuilderAsyncState from "./MusicBuilderAsyncState.svelte";
  import MusicAdvancedMembershipEditor from "./MusicAdvancedMembershipEditor.svelte";

  let {
    controller,
    showClose = false,
    onClose = () => undefined,
    onPlay = () => undefined,
    onRepair = () => undefined,
    onMetadataSaved = () => undefined,
    activePlaylistId = null,
    onPreviewMembership = () => undefined,
    onShowFile = () => undefined,
    onReviewState = () => undefined,
    onSnooze = () => undefined,
    onResetStatistics = () => undefined,
    playlistNames = {},
    onEditMembership = () => undefined,
    sourceNames = {},
    onOpenSource = () => undefined,
  }: {
    controller: MusicBuilderInspectorController;
    showClose?: boolean;
    onClose?: () => void;
    onPlay?: (itemId: string) => void;
    onRepair?: (itemId: string) => void;
    onMetadataSaved?: () => void;
    activePlaylistId?: string | null;
    onPreviewMembership?: (membership: import("$lib/music/library-contracts").MusicPlaylistMembership) => void;
    onShowFile?: (itemId: string) => void;
    onReviewState?: (itemId: string) => void;
    onSnooze?: (itemId: string) => void;
    onResetStatistics?: (itemId: string, mode: "recent" | "all") => void;
    playlistNames?: Record<string, string>;
    onEditMembership?: (itemId: string) => void;
    sourceNames?: Record<string, string>;
    onOpenSource?: (sourceId: string) => void;
  } = $props();

  const { t, locale } = getLocalization();
  const item = $derived(controller.detail?.item ?? null);
  const title = $derived(item?.titleOverride?.trim() || item?.originalTitle || "");
  const artist = $derived(item?.artistOverride?.trim() || item?.originalArtist || t("music.builder.noArtist"));
  const album = $derived(item?.albumOverride?.trim() || item?.originalAlbum || t("music.builder.noAlbum"));
  const activeMembership = $derived(controller.detail?.memberships.find((membership) => membership.playlistId === activePlaylistId) ?? controller.detail?.memberships[0] ?? null);
  let metadataEditing = $state(false);
  let titleDraft = $state("");
  let artistDraft = $state("");
  let albumDraft = $state("");
  let artworkDraft = $state("");
  let statisticsResetOpen = $state(false);
  const signalOptions: MusicItemSignal[] = ["lyrics", "sudden-changes", "high-intensity", "calm", "repetitive", "energizing"];

  function toggleSignal(signal: MusicItemSignal): void {
    if (!controller.detail || controller.saving) return;
    const next = controller.detail.signals.includes(signal)
      ? controller.detail.signals.filter((entry) => entry !== signal)
      : [...controller.detail.signals, signal];
    void controller.saveSignals(next);
  }

  function lastPlayedLabel(): string {
    const value = controller.detail?.statistics?.lastPlayedAt;
    if (!value) return t("music.builder.neverPlayedLabel");
    return formatDateTime(locale, value, { dateStyle: "medium", timeStyle: "short" });
  }

  function sectionOpen(id: string): boolean { return controller.expandedSections.has(id); }
  function availabilityLabel(): string {
    if (!item) return "";
    if (item.availability === "available") return t("music.builder.available");
    if (item.availability === "missing") return t("music.builder.missing");
    if (item.availability === "unavailable") return t("music.builder.unavailable");
    if (item.availability === "ambiguous") return t("music.builder.ambiguous");
    return t("music.builder.unknownAvailability");
  }

  function beginMetadataEdit(): void {
    if (!item) return;
    titleDraft = item.titleOverride ?? "";
    artistDraft = item.artistOverride ?? "";
    albumDraft = item.albumOverride ?? "";
    artworkDraft = item.artworkOverride ?? "";
    metadataEditing = true;
  }

  async function saveMetadata(reset = false): Promise<void> {
    const value = (draft: string): string | null => draft.trim() || null;
    const saved = await controller.saveMetadataOverrides({
      titleOverride: reset ? null : value(titleDraft),
      artistOverride: reset ? null : value(artistDraft),
      albumOverride: reset ? null : value(albumDraft),
      artworkOverride: reset ? null : value(artworkDraft),
    });
    if (saved) { metadataEditing = false; onMetadataSaved(); }
  }

  async function chooseArtwork(): Promise<void> {
    const selected = await pickArtworkFile();
    if (selected) artworkDraft = selected;
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
            <div class="grid h-18 w-18 shrink-0 place-items-center rounded-xl bg-linear-to-br from-primary/15 to-secondary text-muted-foreground shadow-inner"><Disc3 size={28} strokeWidth={1.25} /></div>
            <div class="min-w-0 flex-1 self-center">
              <h2 class="line-clamp-2 text-sm font-semibold leading-snug text-foreground">{title}</h2>
              <p class="mt-1 truncate text-xs text-muted-foreground">{artist}</p>
              <p class="mt-0.5 truncate text-[0.68rem] text-muted-foreground/80">{album}</p>
            </div>
          </div>
          <div class="mt-3 flex gap-2">
            <button type="button" onclick={() => onPlay(item.id)} class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-lg bg-primary text-xs font-semibold text-primary-foreground hover:bg-primary/90"><Play size={13} fill="currentColor" />{t("music.play")}</button>
            {#if item.sourceKind === "local-file"}
              <button type="button" onclick={() => onShowFile(item.id)} class="inline-flex h-8 w-9 items-center justify-center rounded-lg bg-secondary text-secondary-foreground hover:bg-accent" aria-label={t("music.showCurrentFileLocation")}><FolderOpen size={14} /></button>
            {/if}
            <button type="button" onclick={() => onSnooze(item.id)} class="inline-flex h-8 w-9 items-center justify-center rounded-lg bg-secondary text-secondary-foreground hover:bg-accent" aria-label={t("music.builder.snoozeAction")}><Clock3 size={14} /></button>
            {#if item.sourceKind === "local-file" && item.availability !== "available"}
              <button type="button" onclick={() => onRepair(item.id)} class="inline-flex h-8 items-center justify-center rounded-lg bg-secondary px-2.5 text-[0.65rem] font-semibold text-secondary-foreground hover:bg-accent">{t("music.builder.repair")}</button>
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
              <button type="button" onclick={() => onReviewState(item.id)} class="mx-2 mb-2 h-7 rounded-md bg-secondary px-2 text-[0.65rem] font-medium">{t("music.builder.changeReviewState")}</button>
            {/if}
          </section>

          <section class="inspector-section">
            <button type="button" onclick={() => controller.toggleSection("signals")} aria-expanded={sectionOpen("signals")}><span>{t("music.builder.descriptiveSignals")}</span><ChevronDown class={sectionOpen("signals") ? "rotate-180" : ""} size={13} /></button>
            {#if sectionOpen("signals")}
              <div class="px-2 pb-2">
                <p class="text-[0.62rem] leading-relaxed text-muted-foreground">{t("music.builder.descriptiveSignalsHint")}</p>
                <div class="mt-2 flex flex-wrap gap-1.5" role="group" aria-label={t("music.builder.descriptiveSignals")}>
                  {#each signalOptions as signal}
                    <button type="button" aria-pressed={controller.detail?.signals.includes(signal)} disabled={controller.saving} onclick={() => toggleSignal(signal)} class="rounded-full border border-border/70 px-2 py-1 text-[0.63rem] text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary/10 aria-pressed:text-foreground disabled:opacity-50">{t(`music.builder.signal.${signal}`)}</button>
                  {/each}
                </div>
                {#if controller.signalUndo}<button type="button" onclick={() => { void controller.undoSignals(); }} disabled={controller.saving} class="mt-2 h-7 rounded-md bg-secondary px-2 text-[0.62rem] font-medium disabled:opacity-50">{t("music.builder.undoSignalEdit")}</button>{/if}
              </div>
            {/if}
          </section>

          <section class="inspector-section">
            <button type="button" onclick={() => controller.toggleSection("provenance")} aria-expanded={sectionOpen("provenance")}><span>{t("music.builder.provenance")}</span><ChevronDown class={sectionOpen("provenance") ? "rotate-180" : ""} size={13} /></button>
            {#if sectionOpen("provenance")}
              <div class="space-y-1 px-2 pb-2">
                {#each controller.detail?.sourceCollectionIds ?? [] as sourceId}
                  <button type="button" onclick={() => onOpenSource(sourceId)} class="flex h-7 w-full items-center gap-2 rounded-md bg-secondary/55 px-2 text-left text-[0.66rem]"><span class="min-w-0 flex-1 truncate">{sourceNames[sourceId] ?? sourceId}</span><span class="text-muted-foreground">{t("music.builder.openSource")}</span></button>
                {:else}<p class="text-[0.65rem] text-muted-foreground">{t("music.builder.noSourceCollection")}</p>{/each}
                {#if controller.detail?.signals.length}<div class="flex flex-wrap gap-1 pt-1">{#each controller.detail.signals as signal}<span class="rounded-full bg-secondary px-2 py-0.5 text-[0.6rem] text-muted-foreground">{signal}</span>{/each}</div>{/if}
              </div>
            {/if}
          </section>

          {#if activeMembership}
            <section class="inspector-section">
              <button type="button" onclick={() => controller.toggleSection("advanced")} aria-expanded={sectionOpen("advanced")}><span>{t("music.builder.advanced")}</span><ChevronDown class={sectionOpen("advanced") ? "rotate-180" : ""} size={13} /></button>
              {#if sectionOpen("advanced")}<MusicAdvancedMembershipEditor controller={controller} membership={activeMembership} durationMs={item.durationMs} onPreview={onPreviewMembership} />{/if}
            </section>
          {/if}

          <section class="inspector-section">
            <button type="button" onclick={() => controller.toggleSection("metadata")} aria-expanded={sectionOpen("metadata")}><span>{t("music.builder.metadataOverrides")}</span><ChevronDown class={sectionOpen("metadata") ? "rotate-180" : ""} size={13} /></button>
            {#if sectionOpen("metadata")}
              {#if metadataEditing}
                <div class="space-y-2 px-2 pb-2">
                  <label class="block text-[0.64rem] text-muted-foreground">{t("music.builder.title")}<input bind:value={titleDraft} placeholder={item.originalTitle} class="mt-1 h-8 w-full rounded-md border border-border/70 bg-background px-2 text-[0.68rem] text-foreground outline-none" /></label>
                  <label class="block text-[0.64rem] text-muted-foreground">{t("music.builder.artist")}<input bind:value={artistDraft} placeholder={item.originalArtist} class="mt-1 h-8 w-full rounded-md border border-border/70 bg-background px-2 text-[0.68rem] text-foreground outline-none" /></label>
                  <label class="block text-[0.64rem] text-muted-foreground">{t("music.builder.album")}<input bind:value={albumDraft} placeholder={item.originalAlbum} class="mt-1 h-8 w-full rounded-md border border-border/70 bg-background px-2 text-[0.68rem] text-foreground outline-none" /></label>
                  <div class="rounded-lg border border-border/60 bg-background p-2"><span class="block text-[0.64rem] text-muted-foreground">{t("music.builder.artworkOverride")}</span><p class="mt-1 truncate text-[0.65rem] text-foreground">{artworkDraft || t("music.builder.originalArtwork")}</p><div class="mt-2 flex gap-1.5"><button type="button" onclick={() => { void chooseArtwork(); }} class="h-7 rounded-md bg-secondary px-2 text-[0.65rem]">{t("music.builder.chooseArtwork")}</button><button type="button" onclick={() => artworkDraft = ""} class="h-7 rounded-md bg-secondary px-2 text-[0.65rem]">{t("music.builder.useOriginalArtwork")}</button></div></div>
                  <p class="text-[0.61rem] leading-relaxed text-muted-foreground">{t("music.builder.metadataFilesUntouched")}</p>
                  <div class="flex justify-end gap-1.5"><button type="button" onclick={() => metadataEditing = false} class="h-7 rounded-md bg-secondary px-2 text-[0.65rem]">{t("music.builder.cancel")}</button><button type="button" onclick={() => { void saveMetadata(true); }} class="h-7 rounded-md bg-secondary px-2 text-[0.65rem]">{t("music.builder.resetOverrides")}</button><button type="button" onclick={() => { void saveMetadata(); }} disabled={controller.saving} class="h-7 rounded-md bg-primary px-2 text-[0.65rem] font-medium text-primary-foreground disabled:opacity-40">{t("music.builder.savePlaylist")}</button></div>
                </div>
              {:else}
                <div class="px-2 pb-2"><p class="text-[0.64rem] leading-relaxed text-muted-foreground">{t("music.builder.originalMetadataPreserved")}</p><button type="button" onclick={beginMetadataEdit} class="mt-2 inline-flex h-7 items-center gap-1.5 rounded-md bg-secondary px-2 text-[0.65rem]"><Pencil size={11} />{t("music.builder.editMetadata")}</button></div>
              {/if}
            {/if}
          </section>

          <section class="inspector-section">
            <button type="button" onclick={() => controller.toggleSection("memberships")} aria-expanded={sectionOpen("memberships")}><span>{t("music.builder.memberships")}</span><ChevronDown class={sectionOpen("memberships") ? "rotate-180" : ""} size={13} /></button>
            {#if sectionOpen("memberships")}
              <button type="button" onclick={() => onEditMembership(item.id)} class="mx-2 mb-2 h-7 rounded-md bg-primary/10 px-2 text-[0.65rem] font-medium text-primary">{t("music.builder.editMemberships")}</button>
              {#if controller.detail?.memberships.length}
                <div class="space-y-1 px-2 pb-2">
                  {#each controller.detail.memberships as membership (membership.id)}
                    <div class="flex items-center gap-2 rounded-lg bg-secondary/55 px-2 py-1.5 text-[0.68rem]"><ListMusic size={12} /><span class="min-w-0 flex-1 truncate">{playlistNames[membership.playlistId] ?? membership.playlistId}</span><span class="text-muted-foreground">{t(`music.builder.weight.${membership.weight}`)}</span></div>
                  {/each}
                </div>
              {:else}<p class="px-2 pb-2 text-[0.68rem] text-muted-foreground">{t("music.builder.playlistsMembership", 0)}</p>{/if}
            {/if}
          </section>

          <section class="inspector-section">
            <button type="button" onclick={() => controller.toggleSection("listening")} aria-expanded={sectionOpen("listening")}><span>{t("music.builder.listening")}</span><ChevronDown class={sectionOpen("listening") ? "rotate-180" : ""} size={13} /></button>
            {#if sectionOpen("listening")}
              <dl>
                <div><dt>{t("music.builder.lastPlayedLabel")}</dt><dd title={lastPlayedLabel()}>{lastPlayedLabel()}</dd></div>
                <div><dt>{t("music.builder.playCount")}</dt><dd>{controller.detail?.statistics?.playCount ?? 0}</dd></div>
                <div><dt>{t("music.builder.completionCount")}</dt><dd>{controller.detail?.statistics?.completionCount ?? 0}</dd></div>
                <div><dt>{t("music.builder.skipCount")}</dt><dd>{controller.detail?.statistics?.skipCount ?? 0}</dd></div>
              </dl>
              {#if statisticsResetOpen}
                <div class="mx-2 mb-2 rounded-lg border border-border/60 bg-background p-2">
                  <p class="text-[0.62rem] leading-relaxed text-muted-foreground">{t("music.builder.resetStatisticsExplanation")}</p>
                  <div class="mt-2 flex flex-wrap justify-end gap-1.5"><button type="button" onclick={() => statisticsResetOpen = false} class="h-7 rounded-md bg-secondary px-2 text-[0.62rem]">{t("music.builder.cancel")}</button><button type="button" onclick={() => { onResetStatistics(item.id, "recent"); statisticsResetOpen = false; }} class="h-7 rounded-md bg-secondary px-2 text-[0.62rem]">{t("music.builder.clearRecentSelections")}</button><button type="button" onclick={() => { onResetStatistics(item.id, "all"); statisticsResetOpen = false; }} class="h-7 rounded-md bg-destructive px-2 text-[0.62rem] font-medium text-destructive-foreground">{t("music.builder.resetListeningStatistics")}</button></div>
                </div>
              {:else}
                <button type="button" onclick={() => statisticsResetOpen = true} class="mx-2 mb-2 inline-flex h-7 items-center gap-1.5 rounded-lg px-2 text-[0.65rem] text-muted-foreground hover:bg-accent hover:text-accent-foreground"><RotateCcw size={11} />{t("music.builder.manageListeningHistory")}</button>
              {/if}
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
  .inspector-section > button { display: flex; width: 100%; height: 2rem; align-items: center; justify-content: space-between; padding-inline: 0.6rem; color: var(--foreground); font-size: calc(0.69rem * var(--type-scale)); font-weight: 600; }
  .inspector-section > button :global(svg) { transition: transform 140ms ease; }
  .inspector-section dl { padding: 0 0.55rem 0.45rem; }
  .inspector-section dl > div { display: flex; min-height: 1.55rem; align-items: center; justify-content: space-between; gap: 0.75rem; border-top: 1px solid color-mix(in srgb, var(--border) 35%, transparent); font-size: calc(0.65rem * var(--type-scale)); }
  .inspector-section dt { color: var(--muted-foreground); }
  .inspector-section dd { max-width: 60%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; text-align: right; color: var(--foreground); }
  @media (prefers-reduced-motion: reduce) { .inspector-section > button :global(svg) { transition: none; } }
</style>
