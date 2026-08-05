<script lang="ts">
  import ChatTimeline from "./ChatTimeline.svelte";
  import type { ChatTurnId } from "$lib/chat/contracts";

  let {
    bottomInsetPx = 0,
    embedded = false,
    hideUserMessages = false,
    teammateName = null,
    turnId = null,
  }: {
    bottomInsetPx?: number;
    embedded?: boolean;
    hideUserMessages?: boolean;
    teammateName?: string | null;
    turnId?: ChatTurnId | null;
  } = $props();
</script>

<ChatTimeline {bottomInsetPx} {embedded} {hideUserMessages} {teammateName} {turnId} />
