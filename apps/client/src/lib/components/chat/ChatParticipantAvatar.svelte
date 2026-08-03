<script lang="ts">
  import Bot from "@lucide/svelte/icons/bot";
  import type { ChatParticipantRead } from "$lib/chat/contracts";
  import { chatParticipantDisplayName } from "$lib/chat/participant-display";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import ProfileAvatar from "$lib/components/profile/ProfileAvatar.svelte";

  let { participant, size = 32 }: { participant: ChatParticipantRead; size?: number } = $props();
  const preferences = getPreferences();
  const { t } = getLocalization();
  const displayName = $derived(chatParticipantDisplayName(
    participant,
    preferences.profileDisplayName,
    t("chat.timeline.you"),
  ));
</script>

<span
  class="participant-avatar"
  class:agent={participant.kind === "ai_teammate"}
  class:local={participant.kind === "local_user"}
  style={`width:${size}px;height:${size}px`}
  aria-hidden="true"
>
  {#if participant.kind === "local_user"}
    <ProfileAvatar {displayName} imagePath={preferences.profileImagePath} {size} />
  {:else if participant.kind === "ai_teammate"}
    <Bot size={Math.max(12, Math.round(size * 0.48))} />
  {:else}
    {displayName.slice(0, 1).toLocaleUpperCase()}
  {/if}
</span>

<style>
  .participant-avatar { display:inline-grid; flex:0 0 auto; place-items:center; border:1px solid color-mix(in srgb,var(--border) 75%,transparent); border-radius:0.45rem; background:var(--accent); color:var(--foreground); font-size:0.72rem; font-weight:700; }
  .participant-avatar.local { overflow:hidden; border:0; background:transparent; }
  .participant-avatar.local :global(.profile-avatar) { display:grid; }
  .participant-avatar.agent { background:color-mix(in srgb,var(--primary) 13%,var(--accent)); color:color-mix(in srgb,var(--primary) 72%,var(--foreground)); }
</style>
