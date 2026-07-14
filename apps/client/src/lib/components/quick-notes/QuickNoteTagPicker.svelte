<script lang="ts">
  import { tick } from "svelte";
  import Check from "@lucide/svelte/icons/check";
  import Tag from "@lucide/svelte/icons/tag";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { QuickNoteTag } from "$lib/quick-notes/types";
  import { portal } from "$lib/utils/portal";

  let {
    tagId,
    tags,
    onselect,
    buttonClass = "",
  }: {
    tagId: string | null;
    tags: readonly QuickNoteTag[];
    onselect: (tagId: string | null) => void;
    buttonClass?: string;
  } = $props();

  const { t } = getLocalization();
  let open = $state(false);
  let button = $state<HTMLButtonElement | null>(null);
  let firstOption = $state<HTMLButtonElement | null>(null);
  let left = $state(8);
  let top = $state(8);

  async function openPicker(): Promise<void> {
    if (!button) return;
    const rect = button.getBoundingClientRect();
    const width = 220;
    const height = Math.min(300, 44 + (tags.length + 1) * 34);
    left = Math.min(Math.max(8, rect.left), Math.max(8, window.innerWidth - width - 8));
    top = rect.bottom + height + 8 <= window.innerHeight
      ? rect.bottom + 4
      : Math.max(8, rect.top - height - 4);
    open = true;
    await tick();
    firstOption?.focus();
  }

  function choose(next: string | null): void {
    onselect(next);
    open = false;
    void tick().then(() => button?.focus());
  }
</script>

<button
  bind:this={button}
  type="button"
  class={buttonClass}
  aria-label={t("quickNotes.tag.assign")}
  title={t("quickNotes.tag.assign")}
  aria-haspopup="dialog"
  aria-expanded={open}
  onclick={() => { if (open) open = false; else void openPicker(); }}
>
  <Tag class="size-4" strokeWidth={1.5} aria-hidden="true" />
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
    class="fixed z-100 max-h-75 w-55 overflow-y-auto rounded-lg border border-border bg-popover p-1.5 text-popover-foreground shadow-xl"
    style="left: {left}px; top: {top}px;"
    role="dialog"
    tabindex="-1"
    aria-label={t("quickNotes.tag.assign")}
    onkeydown={(event) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      open = false;
      void tick().then(() => button?.focus());
    }}
  >
    <button
      bind:this={firstOption}
      type="button"
      class="flex h-8 w-full items-center gap-2 rounded-md px-2 text-left text-xs hover:bg-accent"
      onclick={() => choose(null)}
    >
      <span class="flex size-4 items-center justify-center">{#if tagId === null}<Check class="size-3.5" strokeWidth={1.5} />{/if}</span>
      <span>{t("quickNotes.tag.none")}</span>
    </button>
    {#each tags as tag}
      <button
        type="button"
        class="flex h-8 w-full items-center gap-2 rounded-md px-2 text-left text-xs hover:bg-accent"
        onclick={() => choose(tag.id)}
      >
        <span class="flex size-4 items-center justify-center">{#if tagId === tag.id}<Check class="size-3.5" strokeWidth={1.5} />{/if}</span>
        <span class="truncate">{tag.name}</span>
      </button>
    {/each}
  </div>
{/if}
