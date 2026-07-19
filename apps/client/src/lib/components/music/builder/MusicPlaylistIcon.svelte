<script lang="ts">
  import Coffee from "@lucide/svelte/icons/coffee";
  import Footprints from "@lucide/svelte/icons/footprints";
  import Laptop from "@lucide/svelte/icons/laptop";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import Sunrise from "@lucide/svelte/icons/sunrise";
  import TreePine from "@lucide/svelte/icons/tree-pine";
  import ProjectIcon from "$lib/components/projects/ProjectIcon.svelte";

  let {
    playlistId,
    size = 16,
    strokeWidth = 1.75,
  }: {
    playlistId: string;
    size?: number;
    strokeWidth?: number;
  } = $props();
</script>

{#if playlistId === "playlist-default-start-of-day"}
  <Sunrise {size} {strokeWidth} />
{:else if playlistId === "playlist-default-work-focus"}
  <Laptop {size} {strokeWidth} />
{:else if playlistId === "playlist-default-work-ganbare"}
  <Coffee {size} {strokeWidth} />
{:else if playlistId === "playlist-default-break-calm"}
  <TreePine {size} {strokeWidth} />
{:else if playlistId === "playlist-default-break-active"}
  <Footprints {size} {strokeWidth} />
{:else if playlistId === "playlist-default-meditate"}
  <ProjectIcon name="lucide:smile" {size} {strokeWidth} />
{:else if playlistId === "playlist-default-exercise"}
  <ProjectIcon name="lucide:sport-shoe" {size} {strokeWidth} />
{:else if playlistId === "playlist-default-hygiene"}
  <ProjectIcon name="lucide:bath" {size} {strokeWidth} />
{:else if playlistId === "playlist-default-chores"}
  <ProjectIcon name="lucide:shopping-cart" {size} {strokeWidth} />
{:else if playlistId === "playlist-default-cooking"}
  <ProjectIcon name="lucide:apple" {size} {strokeWidth} />
{:else if playlistId === "playlist-default-commute"}
  <ProjectIcon name="lucide:bike" {size} {strokeWidth} />
{:else}
  <ListMusic {size} {strokeWidth} />
{/if}
