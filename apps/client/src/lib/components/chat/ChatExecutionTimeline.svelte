<script lang="ts">
  import ChatTimeline from "./ChatTimeline.svelte";
  import type { ChatThreadShellRead, ChatTimelinePageRead, ChatTurnId } from "$lib/chat/contracts";

  let {
    bottomInsetPx = 0,
    embedded = false,
    hideUserMessages = false,
    teammateName = null,
    turnId = null,
    timelinePage = null,
    executionThread = null,
  }: {
    bottomInsetPx?: number;
    embedded?: boolean;
    hideUserMessages?: boolean;
    teammateName?: string | null;
    turnId?: ChatTurnId | null;
    timelinePage?: ChatTimelinePageRead | null;
    executionThread?: ChatThreadShellRead | null;
  } = $props();
</script>

<ChatTimeline {bottomInsetPx} {embedded} {hideUserMessages} {teammateName} {turnId} {timelinePage} {executionThread} />
