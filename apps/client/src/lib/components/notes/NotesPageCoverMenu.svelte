<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    createNotesExternalPageCover,
    notesPageCoverUrl,
  } from "$lib/notes/page-cover";
  import type { NotesPageCover } from "$lib/notes/types";
  import Save from "@lucide/svelte/icons/save";
  import Trash2 from "@lucide/svelte/icons/trash-2";

  let {
    cover,
    onSelect,
  }: {
    cover: NotesPageCover | null;
    onSelect: (cover: NotesPageCover | null) => void;
  } = $props();

  const { t } = getLocalization();
  let urlDraft = $state("");
  let error = $state("");
  const coverUrl = $derived(notesPageCoverUrl(cover));

  $effect(() => {
    urlDraft = coverUrl ?? "";
    error = "";
  });

  function saveCover(): void {
    try {
      onSelect(createNotesExternalPageCover(urlDraft));
      error = "";
    } catch {
      error = t("notes.pageCoverUrlInvalid");
    }
  }
</script>

<form
  class="absolute right-0 top-9 z-30 w-72 rounded-md border border-border bg-popover p-2 text-popover-foreground shadow-lg"
  aria-label={t("notes.pageCover")}
  data-app-floating-surface
  onsubmit={(event) => {
    event.preventDefault();
    saveCover();
  }}
>
  <label class="block text-[0.733333rem] font-medium text-muted-foreground" for="notes-cover-url">
    {t("notes.pageCoverUrl")}
  </label>
  <input
    id="notes-cover-url"
    class="mt-1 w-full rounded-md border border-border bg-background px-2 py-1.5 text-[0.8rem] text-foreground outline-none placeholder:text-muted-foreground"
    type="url"
    bind:value={urlDraft}
    placeholder={t("notes.pageCoverUrlPlaceholder")}
  />
  {#if error}
    <div class="mt-1 text-[0.733333rem] text-destructive">{error}</div>
  {/if}
  <div class="mt-2 flex items-center justify-end gap-1.5">
    {#if cover}
      <button
        class="flex items-center gap-1.5 rounded px-2 py-1.5 text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
        type="button"
        onclick={() => {
          onSelect(null);
        }}
      >
        <Trash2 class="size-3.5" />
        <span>{t("notes.removePageCover")}</span>
      </button>
    {/if}
    <button
      class="flex items-center gap-1.5 rounded bg-primary px-2 py-1.5 text-[0.8rem] text-primary-foreground hover:bg-primary/90"
      type="submit"
    >
      <Save class="size-3.5" />
      <span>{t("notes.savePageCover")}</span>
    </button>
  </div>
</form>
