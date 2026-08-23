<script lang="ts">
  import { redactedSensitiveText } from "./redacted-sensitive-text";

  let {
    value,
    revealLabel,
    hideLabel,
  }: {
    value: string;
    revealLabel: string;
    hideLabel: string;
  } = $props();

  let revealed = $state(false);
  const normalizedValue = $derived(value.trim());
  const redactedValue = $derived(redactedSensitiveText(normalizedValue));
  const actionLabel = $derived(revealed ? hideLabel : revealLabel);
</script>

{#if normalizedValue}
  <button
    type="button"
    class="redacted-sensitive-text"
    class:revealed
    aria-label={revealed ? `${hideLabel}: ${normalizedValue}` : revealLabel}
    data-app-tooltip={actionLabel}
    onclick={() => { revealed = !revealed; }}
  >
    <span>{revealed ? normalizedValue : redactedValue}</span>
  </button>
{/if}

<style>
  .redacted-sensitive-text { display:inline-block; min-width:0; max-width:100%; flex:0 1 auto; border-radius:0.2rem; color:var(--muted-foreground); font-family:var(--font-mono,monospace); font-size:calc(0.66rem * var(--type-scale)); line-height:1rem; text-align:left; transition:color 120ms ease; }
  .redacted-sensitive-text:hover { color:var(--foreground); }
  .redacted-sensitive-text:focus-visible { outline:1px solid var(--ring); outline-offset:2px; }
  .redacted-sensitive-text span { display:block; min-width:0; overflow-wrap:anywhere; padding-inline:0.12rem; filter:blur(2px); user-select:none; transition:filter 120ms ease; }
  .redacted-sensitive-text.revealed span { filter:none; user-select:text; }
</style>
