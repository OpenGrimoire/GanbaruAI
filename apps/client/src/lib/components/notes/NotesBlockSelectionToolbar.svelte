<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import ClipboardPaste from "@lucide/svelte/icons/clipboard-paste";
  import Copy from "@lucide/svelte/icons/copy";
  import CopyPlus from "@lucide/svelte/icons/copy-plus";
  import Scissors from "@lucide/svelte/icons/scissors";
  import Trash2 from "@lucide/svelte/icons/trash-2";

  let {
    selectedCount,
    hasSelectedRoots,
    clipboardAvailable,
    canMoveUp,
    canMoveDown,
    busy,
    error,
    onCopy,
    onCut,
    onPaste,
    onDuplicate,
    onMoveUp,
    onMoveDown,
    onDelete,
  }: {
    selectedCount: number;
    hasSelectedRoots: boolean;
    clipboardAvailable: boolean;
    canMoveUp: boolean;
    canMoveDown: boolean;
    busy: boolean;
    error: string | null;
    onCopy: () => void;
    onCut: () => void;
    onPaste: () => void;
    onDuplicate: () => void;
    onMoveUp: () => void;
    onMoveDown: () => void;
    onDelete: () => void;
  } = $props();

  const { t } = getLocalization();
  const textButtonClass = "inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs hover:bg-accent disabled:pointer-events-none disabled:opacity-50";
  const iconButtonClass = "inline-flex h-8 items-center justify-center rounded-md px-2 hover:bg-accent disabled:pointer-events-none disabled:opacity-50";
</script>

<div
  class="sticky top-2 z-20 mb-2 flex min-w-0 flex-wrap items-center gap-1 rounded-md border border-border bg-popover/95 px-2 py-1.5 text-xs text-popover-foreground shadow-sm backdrop-blur"
  role="toolbar"
  aria-label={t("notes.selectionActions")}
>
  <span class="mr-1 shrink-0 font-medium text-muted-foreground">
    {t("notes.selectedBlocks", selectedCount)}
  </span>
  <button type="button" class={textButtonClass} disabled={busy || !hasSelectedRoots} aria-label={t("notes.copySelection")} title={t("notes.copySelection")} onclick={onCopy}>
    <Copy size={14} aria-hidden="true" />
    <span>{t("notes.copySelection")}</span>
  </button>
  <button type="button" class={textButtonClass} disabled={busy || !hasSelectedRoots} aria-label={t("notes.cutSelection")} title={t("notes.cutSelection")} onclick={onCut}>
    <Scissors size={14} aria-hidden="true" />
    <span>{t("notes.cutSelection")}</span>
  </button>
  <button type="button" class={textButtonClass} disabled={busy || !clipboardAvailable} aria-label={t("notes.pasteSelection")} title={t("notes.pasteSelection")} onclick={onPaste}>
    <ClipboardPaste size={14} aria-hidden="true" />
    <span>{t("notes.pasteSelection")}</span>
  </button>
  <button type="button" class={textButtonClass} disabled={busy || !hasSelectedRoots} aria-label={t("notes.duplicateSelection")} title={t("notes.duplicateSelection")} onclick={onDuplicate}>
    <CopyPlus size={14} aria-hidden="true" />
    <span>{t("notes.duplicateSelection")}</span>
  </button>
  <button type="button" class={iconButtonClass} disabled={busy || !canMoveUp} aria-label={t("notes.moveSelectionUp")} title={t("notes.moveSelectionUp")} onclick={onMoveUp}>
    <ArrowUp size={15} aria-hidden="true" />
  </button>
  <button type="button" class={iconButtonClass} disabled={busy || !canMoveDown} aria-label={t("notes.moveSelectionDown")} title={t("notes.moveSelectionDown")} onclick={onMoveDown}>
    <ArrowDown size={15} aria-hidden="true" />
  </button>
  <button
    type="button"
    class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs text-destructive hover:bg-destructive/10 disabled:pointer-events-none disabled:opacity-50"
    disabled={busy || !hasSelectedRoots}
    aria-label={t("notes.deleteSelection")}
    title={t("notes.deleteSelection")}
    onclick={onDelete}
  >
    <Trash2 size={14} aria-hidden="true" />
    <span>{t("notes.deleteSelection")}</span>
  </button>
  {#if error}
    <span class="min-w-0 flex-1 truncate text-destructive" role="status">
      {t("notes.selectionActionFailed")} {error}
    </span>
  {/if}
</div>
