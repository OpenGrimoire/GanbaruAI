<script lang="ts">
  import { notesPageCoverAssetUrl } from "$lib/api/notes-page-covers";
  import {
    isNotesPageCoverAssetPath,
    notesPageCoverAssetPath,
    notesPageCoverUrl,
  } from "$lib/notes/page-cover";
  import type { NotesPageCover } from "$lib/notes/types";
  import { BUILD_PLATFORM_PROFILE, platformHasCapability } from "$lib/platform";
  import ImageIcon from "@lucide/svelte/icons/image";

  let {
    cover,
    unavailableLabel,
    objectFit = "cover",
  }: {
    cover: NotesPageCover | null;
    unavailableLabel: string;
    objectFit?: "cover" | "contain";
  } = $props();

  let assetUrl = $state<string | null>(null);
  let assetRequestId = 0;

  const assetPath = $derived(notesPageCoverAssetPath(cover));
  const remoteImageUrlsAvailable = platformHasCapability(
    BUILD_PLATFORM_PROFILE,
    "notes.external-image-references",
  );
  const externalUrl = $derived(remoteImageUrlsAvailable ? notesPageCoverUrl(cover) : null);

  $effect(() => {
    const currentPath = assetPath;
    const requestId = ++assetRequestId;
    assetUrl = null;
    if (!currentPath || !isNotesPageCoverAssetPath(currentPath)) return;
    void notesPageCoverAssetUrl(currentPath)
      .then((url) => {
        if (requestId === assetRequestId) assetUrl = url;
      })
      .catch(() => {
        if (requestId === assetRequestId) assetUrl = null;
      });
  });
</script>

{#if assetUrl}
  <img class={`size-full ${objectFit === "contain" ? "object-contain" : "object-cover"}`} src={assetUrl} alt="" />
{:else if externalUrl}
  <img class={`size-full ${objectFit === "contain" ? "object-contain" : "object-cover"}`} src={externalUrl} alt="" />
{:else}
  <div class="flex size-full items-center justify-center gap-2 bg-muted text-[0.8rem] text-muted-foreground">
    <ImageIcon class="size-4" />
    <span class="max-w-full truncate px-2">{unavailableLabel}</span>
  </div>
{/if}
