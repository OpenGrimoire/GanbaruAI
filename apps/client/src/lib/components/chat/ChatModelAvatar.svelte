<script lang="ts">
  import ChatProviderIcon from "./ChatProviderIcon.svelte";

  let {
    familyId,
    label,
    size = 36,
  }: {
    familyId: string;
    label: string;
    size?: number;
  } = $props();
</script>

<span
  class="model-avatar"
  class:claude={familyId === "claude"}
  class:google={familyId === "google"}
  class:meta={familyId === "meta"}
  class:mistral={familyId === "mistral"}
  class:deepseek={familyId === "deepseek"}
  class:alibaba={familyId === "alibaba"}
  class:minimax={familyId === "minimax"}
  style:width={`${size}px`}
  style:height={`${size}px`}
  role="img"
  aria-label={label}
>
  <ChatProviderIcon familyId={familyId} {label} size={Math.round(size * 0.58)} />
</span>

<style>
  .model-avatar {
    display: inline-grid;
    flex: 0 0 auto;
    place-items: center;
    overflow: hidden;
    border: 1px solid color-mix(in srgb, var(--border) 72%, transparent);
    border-radius: 22%;
    background: color-mix(in srgb, var(--foreground) 8%, var(--card));
  }

  .model-avatar.claude { background: color-mix(in srgb, #d97757 15%, var(--card)); }
  .model-avatar.google { background: color-mix(in srgb, #4285f4 13%, var(--card)); }
  .model-avatar.meta { background: color-mix(in srgb, #0866ff 13%, var(--card)); }
  .model-avatar.mistral { background: color-mix(in srgb, #f59e0b 16%, var(--card)); }
  .model-avatar.deepseek { background: color-mix(in srgb, #4d6bfe 13%, var(--card)); }
  .model-avatar.alibaba { background: color-mix(in srgb, #ff6a00 14%, var(--card)); }
  .model-avatar.minimax { background: color-mix(in srgb, #f05252 13%, var(--card)); }
</style>
