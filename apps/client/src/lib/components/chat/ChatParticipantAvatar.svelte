<script lang="ts">
  import Bot from "@lucide/svelte/icons/bot";
  import type { ChatParticipantRead } from "$lib/chat/contracts";

  let { participant, size = 32 }: { participant: ChatParticipantRead; size?: number } = $props();
</script>

<span
  class="participant-avatar"
  class:agent={participant.kind === "ai_teammate"}
  style={`width:${size}px;height:${size}px`}
  aria-hidden="true"
>
  {#if participant.kind === "ai_teammate"}
    <Bot size={Math.max(12, Math.round(size * 0.48))} />
  {:else}
    {participant.displayName.slice(0, 1).toLocaleUpperCase()}
  {/if}
</span>

<style>
  .participant-avatar { display:inline-grid; flex:0 0 auto; place-items:center; border:1px solid color-mix(in srgb,var(--border) 75%,transparent); border-radius:0.45rem; background:var(--accent); color:var(--foreground); font-size:0.72rem; font-weight:700; }
  .participant-avatar.agent { background:color-mix(in srgb,var(--primary) 13%,var(--accent)); color:color-mix(in srgb,var(--primary) 72%,var(--foreground)); }
</style>
