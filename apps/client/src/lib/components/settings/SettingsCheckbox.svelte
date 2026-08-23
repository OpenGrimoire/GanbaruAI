<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Minus from "@lucide/svelte/icons/minus";
  import { cn } from "$lib/utils";

  let {
    checked,
    mixed = false,
    label,
    disabled = false,
    onChange,
  }: {
    checked: boolean;
    mixed?: boolean;
    label: string;
    disabled?: boolean;
    onChange: (checked: boolean) => void;
  } = $props();
</script>

<button
  type="button"
  role="checkbox"
  aria-checked={mixed ? "mixed" : checked}
  aria-label={label}
  {disabled}
  class={cn(
    "flex size-4 shrink-0 items-center justify-center rounded-sm border outline-none transition-colors focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
    checked || mixed
      ? "border-primary bg-primary text-primary-foreground"
      : "border-border bg-background text-transparent hover:border-foreground/45",
  )}
  onclick={() => onChange(!(checked || mixed))}
>
  {#if mixed}
    <Minus size={10} strokeWidth={2.6} />
  {:else if checked}
    <Check size={10} strokeWidth={2.6} />
  {/if}
</button>
