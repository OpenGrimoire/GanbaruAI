<script lang="ts">
  import { tick } from "svelte";
  import Palette from "@lucide/svelte/icons/palette";
  import Slash from "@lucide/svelte/icons/slash";
  import Check from "@lucide/svelte/icons/check";
  import { EVENT_COLOR_OPTIONS, getEventColor } from "$lib/components/calendar/utils";
  import type { EventColor } from "$lib/components/calendar/types";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { Theme } from "$lib/stores/themes";
  import { portal } from "$lib/utils/portal";

  let {
    color,
    theme,
    onselect,
    buttonClass = "",
  }: {
    color: EventColor | null;
    theme: Theme;
    onselect: (color: EventColor | null) => void;
    buttonClass?: string;
  } = $props();

  const { t } = getLocalization();
  let open = $state(false);
  let button = $state<HTMLButtonElement | null>(null);
  let left = $state(8);
  let top = $state(8);

  async function openPicker(): Promise<void> {
    if (!button) return;
    const rect = button.getBoundingClientRect();
    const width = 238;
    const height = 184;
    left = Math.min(Math.max(8, rect.left), Math.max(8, window.innerWidth - width - 8));
    top = rect.bottom + height + 8 <= window.innerHeight
      ? rect.bottom + 4
      : Math.max(8, rect.top - height - 4);
    open = true;
    await tick();
  }

  function choose(next: EventColor | null): void {
    onselect(next);
    open = false;
    void tick().then(() => button?.focus());
  }
</script>

<button
  bind:this={button}
  type="button"
  class={buttonClass}
  aria-label={t("quickNotes.action.color")}
  title={t("quickNotes.action.color")}
  aria-haspopup="dialog"
  aria-expanded={open}
  onclick={() => { if (open) open = false; else void openPicker(); }}
>
  <Palette class="size-4" strokeWidth={1.5} aria-hidden="true" />
</button>

{#if open}
  <button
    use:portal
    type="button"
    class="fixed inset-0 z-90 cursor-default"
    aria-label={t("common.close")}
    onclick={() => { open = false; }}
  ></button>
  <div
    use:portal
    class="fixed z-100 grid grid-cols-6 gap-2 rounded-lg border border-border bg-popover p-2.5 text-popover-foreground shadow-xl"
    style="left: {left}px; top: {top}px; width: 238px;"
    role="dialog"
    aria-label={t("quickNotes.action.color")}
  >
    <button
      type="button"
      class="relative size-7 rounded-md border border-border bg-card"
      aria-label={t("quickNotes.action.clearColor")}
      title={t("quickNotes.action.clearColor")}
      onclick={() => choose(null)}
    >
      <Slash class="absolute inset-0 m-auto size-4 text-muted-foreground" strokeWidth={1.5} />
      {#if color === null}<Check class="absolute -right-1 -top-1 size-3.5 rounded-full bg-foreground p-0.5 text-background" strokeWidth={1.5} />{/if}
    </button>
    {#each EVENT_COLOR_OPTIONS as entry, index}
      {@const resolved = getEventColor(entry, theme)}
      <button
        type="button"
        class="relative size-7 rounded-md border border-black/10 dark:border-white/10"
        style="background-color: {resolved.bg};"
        aria-label={`${t("quickNotes.action.color")} ${index + 1}`}
        title={`${t("quickNotes.action.color")} ${index + 1}`}
        onclick={() => choose(entry)}
      >
        {#if color === entry}<Check class="absolute inset-0 m-auto size-4" strokeWidth={1.5} style="color: {resolved.text};" />{/if}
      </button>
    {/each}
  </div>
{/if}
