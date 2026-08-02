<script lang="ts">
  import ChatTimeline from "./ChatTimeline.svelte";

  let {
    bottomInsetPx = 0,
    embedded = false,
    hideUserMessages = false,
    teammateName = null,
    effort = null,
  }: {
    bottomInsetPx?: number;
    embedded?: boolean;
    hideUserMessages?: boolean;
    teammateName?: string | null;
    effort?: string | null;
  } = $props();
</script>

<ChatTimeline {bottomInsetPx} {embedded} {hideUserMessages} {teammateName} {effort} />
