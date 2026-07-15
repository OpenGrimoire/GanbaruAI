<script lang="ts">
  import Music2 from "@lucide/svelte/icons/music-2";
  import Youtube from "@lucide/svelte/icons/youtube";
  import type { MusicLibrarySourceKind } from "$lib/music/library-contracts";
  import { musicArtworkDataUrl } from "$lib/music/music-artwork-cache";

  let {
    path = null,
    sourceKind,
    version,
  }: {
    path?: string | null;
    sourceKind: MusicLibrarySourceKind;
    version: number;
  } = $props();

  let currentUrl = $state<string | null>(null);
  let requestVersion = 0;

  $effect(() => {
    const requestedPath = path;
    version;
    const request = ++requestVersion;
    if (!requestedPath) {
      currentUrl = null;
      return;
    }
    void musicArtworkDataUrl(requestedPath).then((url) => {
      if (request !== requestVersion || !url) return;
      const image = new Image();
      image.onload = () => {
        if (request === requestVersion) currentUrl = url;
      };
      image.src = url;
    });
  });
</script>

{#if currentUrl}
  <img src={currentUrl} alt="" aria-hidden="true" class="h-full w-full object-cover" />
{:else if sourceKind === "youtube-video"}
  <Youtube size={17} strokeWidth={1.5} />
{:else}
  <Music2 size={16} strokeWidth={1.5} />
{/if}
