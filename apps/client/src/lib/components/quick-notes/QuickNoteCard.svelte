<script lang="ts">
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import Pin from "@lucide/svelte/icons/pin";
  import PinOff from "@lucide/svelte/icons/pin-off";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getQuickNoteColor } from "$lib/quick-notes/colors";
  import type { QuickNote, QuickNotesCollection, QuickNoteTag } from "$lib/quick-notes/types";
  import type { Theme } from "$lib/stores/themes";
  import QuickNoteColorPicker from "./QuickNoteColorPicker.svelte";
  import QuickNoteRichText from "./QuickNoteRichText.svelte";
  import QuickNoteTagPicker from "./QuickNoteTagPicker.svelte";

  let {
    note,
    collection,
    theme,
    tags,
    onopen,
    onpin,
    oncolor,
    ontag,
    onarchive,
    onunarchive,
    ontrash,
    onrestore,
    ondelete,
  }: {
    note: QuickNote;
    collection: QuickNotesCollection;
    theme: Theme;
    tags: readonly QuickNoteTag[];
    onopen: () => void;
    onpin: (pinned: boolean) => void;
    oncolor: (color: QuickNote["color"]) => void;
    ontag: (tagId: string | null) => void;
    onarchive: () => void;
    onunarchive: () => void;
    ontrash: () => void;
    onrestore: () => void;
    ondelete: () => void;
  } = $props();

  const { t } = getLocalization();
  const colors = $derived(getQuickNoteColor(note.color, theme));
  const tag = $derived(tags.find((candidate) => candidate.id === note.tagId) ?? null);
  const surfaceStyle = $derived(`--quick-card-bg: ${colors.bg}; --quick-card-fg: ${colors.text};`);
  const actionClass = "flex size-7 items-center justify-center rounded-md transition-colors hover:bg-black/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-current dark:hover:bg-white/10";
</script>

<article
  class="quick-note-card group overflow-hidden rounded-xl"
  style={surfaceStyle}
>
  <button
    type="button"
    class="block w-full px-3.5 pb-2 pt-3 text-left outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-current"
    onclick={onopen}
  >
    {#if note.title}
      <h3 class="mb-1.5 wrap-break-word text-[0.92rem] font-semibold leading-snug">{note.title}</h3>
    {/if}
    {#if tag}<span class="mb-1.5 inline-flex max-w-full truncate rounded-md bg-black/8 px-1.5 py-0.5 text-[0.65rem] font-medium dark:bg-white/10">{tag.name}</span>{/if}
    {#if note.runs.length > 0}
      <div class="quick-note-preview relative max-h-65 overflow-hidden text-[0.82rem] leading-relaxed">
        <QuickNoteRichText runs={note.runs} />
        {#if note.previewTruncated || note.bodyPlainText.length > 1200}
          <span class="quick-note-fade pointer-events-none absolute inset-x-0 bottom-0 h-12"></span>
        {/if}
      </div>
    {/if}
  </button>
  <div class="quick-note-actions flex min-h-9 items-center gap-0.5 px-2 pb-1.5" role="toolbar">
    {#if collection === "active"}
      <button class={actionClass} type="button" aria-label={note.pinned ? t("quickNotes.action.unpin") : t("quickNotes.action.pin")} title={note.pinned ? t("quickNotes.action.unpin") : t("quickNotes.action.pin")} onclick={() => onpin(!note.pinned)}>
        {#if note.pinned}<PinOff class="size-3.5" strokeWidth={1.5} />{:else}<Pin class="size-3.5" strokeWidth={1.5} />{/if}
      </button>
    {/if}
    {#if collection !== "trash"}
      <QuickNoteColorPicker color={note.color} {theme} onselect={oncolor} buttonClass={actionClass} />
      <QuickNoteTagPicker tagId={note.tagId} {tags} onselect={ontag} buttonClass={actionClass} />
      <button class={actionClass} type="button" aria-label={collection === "archive" ? t("quickNotes.action.unarchive") : t("quickNotes.action.archive")} title={collection === "archive" ? t("quickNotes.action.unarchive") : t("quickNotes.action.archive")} onclick={collection === "archive" ? onunarchive : onarchive}>
        {#if collection === "archive"}<ArchiveRestore class="size-3.5" strokeWidth={1.5} />{:else}<Archive class="size-3.5" strokeWidth={1.5} />{/if}
      </button>
      <button class={actionClass} type="button" aria-label={t("quickNotes.action.trash")} title={t("quickNotes.action.trash")} onclick={ontrash}>
        <Trash2 class="size-3.5" strokeWidth={1.5} />
      </button>
    {:else}
      <button class={actionClass} type="button" aria-label={t("quickNotes.action.restore")} title={t("quickNotes.action.restore")} onclick={onrestore}>
        <RotateCcw class="size-3.5" strokeWidth={1.5} />
      </button>
      <button class={actionClass} type="button" aria-label={t("quickNotes.action.deletePermanently")} title={t("quickNotes.action.deletePermanently")} onclick={ondelete}>
        <Trash2 class="size-3.5" strokeWidth={1.5} />
      </button>
    {/if}
  </div>
</article>

<style>
  .quick-note-card {
    background: var(--quick-card-bg);
    color: var(--quick-card-fg);
  }

  .quick-note-fade {
    background: linear-gradient(to bottom, transparent, var(--quick-card-bg));
  }

  @media (hover: hover) and (pointer: fine) {
    .quick-note-actions {
      opacity: 0;
      transition: opacity 120ms ease;
    }

    .quick-note-card:hover .quick-note-actions,
    .quick-note-card:focus-within .quick-note-actions {
      opacity: 1;
    }
  }
</style>
