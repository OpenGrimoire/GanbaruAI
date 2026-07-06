<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import { cn } from "$lib/utils";

  let {
    checked = $bindable(false),
    label,
    disabled = false,
  }: {
    checked?: boolean;
    label: string;
    disabled?: boolean;
  } = $props();
</script>

<button
  type="button"
  role="checkbox"
  aria-checked={checked}
  aria-label={label}
  data-app-tooltip-disabled="true"
  {disabled}
  class={cn(
    "flex size-5 shrink-0 items-center justify-center rounded border outline-none transition-colors focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
    checked
      ? "border-primary bg-primary text-primary-foreground"
      : "border-border bg-background text-transparent hover:bg-accent",
  )}
  onclick={() => {
    if (!disabled) checked = !checked;
  }}
>
  {#if checked}
    <Check size={12} strokeWidth={2.5} />
  {/if}
</button>
