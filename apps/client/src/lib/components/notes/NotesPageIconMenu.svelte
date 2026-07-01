<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    NOTES_PAGE_EMOJI_ICON_CHOICES,
    createNotesEmojiPageIcon,
  } from "$lib/notes/page-icon";
  import type { NotesPageIcon } from "$lib/notes/types";
  import X from "@lucide/svelte/icons/x";

  let {
    icon,
    onSelect,
  }: {
    icon: NotesPageIcon | null;
    onSelect: (icon: NotesPageIcon | null) => void;
  } = $props();

  const { t } = getLocalization();
</script>

<div
  class="absolute left-0 top-10 z-30 w-56 rounded-md border border-border bg-popover p-2 text-popover-foreground shadow-lg"
  role="menu"
  aria-label={t("notes.pageIcon")}
  data-app-floating-surface
>
  <div class="px-1 pb-1 text-[0.733333rem] font-medium text-muted-foreground">
    {t("notes.pageIconEmoji")}
  </div>
  <div class="grid grid-cols-8 gap-1">
    {#each NOTES_PAGE_EMOJI_ICON_CHOICES as choice}
      {@const selected = icon?.type === "emoji" && icon.emoji === choice}
      <button
        class={`flex size-6 items-center justify-center rounded text-[1rem] leading-none hover:bg-accent ${
          selected ? "bg-accent text-accent-foreground" : ""
        }`}
        type="button"
        aria-label={t("notes.usePageIcon", choice)}
        data-app-tooltip={t("notes.usePageIcon", choice)}
        onclick={() => {
          onSelect(createNotesEmojiPageIcon(choice));
        }}
      >
        <span aria-hidden="true">{choice}</span>
      </button>
    {/each}
  </div>
  {#if icon}
    <button
      class="mt-2 flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
      type="button"
      onclick={() => {
        onSelect(null);
      }}
    >
      <X class="size-4" />
      <span>{t("notes.removePageIcon")}</span>
    </button>
  {/if}
</div>
