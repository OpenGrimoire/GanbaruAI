<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import type { MusicBuilderInspectorController } from "$lib/music/music-builder-inspector.svelte";
  import type { MusicPlaylistMembership } from "$lib/music/library-contracts";
  import MusicBuilderInspector from "./MusicBuilderInspector.svelte";

  let {
    mode,
    open,
    controller,
    activePlaylistId,
    playlistNames,
    sourceNames,
    closeLabel,
    onClose,
    onPlay,
    onShowFile,
    onReviewState,
    onSnooze,
    onEditMembership,
    onResetStatistics,
    onRepair,
    onMetadataSaved,
    onPreviewMembership,
    onOpenSource,
  }: {
    mode: "wide" | "medium" | "narrow";
    open: boolean;
    controller: MusicBuilderInspectorController;
    activePlaylistId: string | null;
    playlistNames: Record<string, string>;
    sourceNames: Record<string, string>;
    closeLabel: string;
    onClose: () => void;
    onPlay: (itemId: string) => void;
    onShowFile: (itemId: string) => void;
    onReviewState: (itemId: string) => void;
    onSnooze: (itemId: string) => void;
    onEditMembership: (itemId: string) => void;
    onResetStatistics: (itemId: string, mode: "recent" | "all") => void;
    onRepair: (itemId: string) => void;
    onMetadataSaved: () => void;
    onPreviewMembership: (membership: MusicPlaylistMembership) => void;
    onOpenSource: (sourceId: string) => void;
  } = $props();
  let reducedMotion = $state(false);

  onMount(() => {
    const query = window.matchMedia("(prefers-reduced-motion: reduce)");
    const update = (): void => { reducedMotion = query.matches; };
    update();
    query.addEventListener("change", update);
    return () => query.removeEventListener("change", update);
  });
</script>

{#if mode === "wide" && open}
  <MusicBuilderInspector {controller} {activePlaylistId} {playlistNames} {sourceNames} {onPlay} {onShowFile} {onReviewState} {onSnooze} {onEditMembership} {onResetStatistics} {onRepair} {onMetadataSaved} {onPreviewMembership} {onOpenSource} />
{:else if open && mode === "medium"}
  <div class="absolute inset-0 z-30 bg-background/45 backdrop-blur-[1px]">
    <button type="button" class="absolute inset-0" onclick={onClose} aria-label={closeLabel}></button>
    <div class="relative ml-auto h-full w-[min(23rem,72%)] border-l border-border/70 shadow-2xl" transition:fly={{ x: 36, duration: reducedMotion ? 0 : 160 }}><MusicBuilderInspector {controller} {activePlaylistId} {playlistNames} {sourceNames} showClose {onClose} {onPlay} {onShowFile} {onReviewState} {onSnooze} {onEditMembership} {onResetStatistics} {onRepair} {onMetadataSaved} {onPreviewMembership} {onOpenSource} /></div>
  </div>
{:else if open && mode === "narrow"}
  <div class="absolute inset-0 z-30 bg-background" transition:fly={{ x: 28, duration: reducedMotion ? 0 : 150 }}><MusicBuilderInspector {controller} {activePlaylistId} {playlistNames} {sourceNames} showClose {onClose} {onPlay} {onShowFile} {onReviewState} {onSnooze} {onEditMembership} {onResetStatistics} {onRepair} {onMetadataSaved} {onPreviewMembership} {onOpenSource} /></div>
{/if}
