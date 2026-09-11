<script lang="ts">
  import Music2 from "@lucide/svelte/icons/music-2";
  import Youtube from "@lucide/svelte/icons/youtube";
  import type { LocalRootBinding, MusicItemListEntry } from "$lib/music/library-contracts";
  import { musicArtworkDataUrl, musicEmbeddedArtworkDataUrl } from "$lib/music/music-artwork-cache";
  import { musicListArtworkSource } from "$lib/music/music-list-artwork";

  let {
    item,
    bindings,
  }: {
    item: MusicItemListEntry;
    bindings: readonly LocalRootBinding[];
  } = $props();

  let currentUrl = $state<string | null>(null);
  let requestVersion = 0;

  $effect(() => {
    const source = musicListArtworkSource(item, bindings);
    item.updatedAt;
    const request = ++requestVersion;
    currentUrl = null;
    if (!source) return;
    const load = source.kind === "file"
      ? musicArtworkDataUrl(source.path)
      : musicEmbeddedArtworkDataUrl(source.path, source.identity);
    void load.then((url) => {
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
{:else if item.sourceKind === "youtube-video"}
  <Youtube size={17} strokeWidth={1.5} />
{:else}
  <Music2 size={16} strokeWidth={1.5} />
{/if}
