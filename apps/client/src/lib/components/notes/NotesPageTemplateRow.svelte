<script lang="ts">
  import { tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { dismissOnOutside } from "$lib/utils/dismiss-on-outside";
  import type { NotesPageTemplate } from "$lib/notes/types";
  import Copy from "@lucide/svelte/icons/copy";
  import FileText from "@lucide/svelte/icons/file-text";
  import MoreHorizontal from "@lucide/svelte/icons/more-horizontal";
  import Pencil from "@lucide/svelte/icons/pencil";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";

  let {
    template,
    canUpdateFromCurrentPage,
    onApply,
    onRename,
    onUpdateFromCurrentPage,
    onDuplicate,
    onDelete,
  }: {
    template: NotesPageTemplate;
    canUpdateFromCurrentPage: boolean;
    onApply: () => void;
    onRename: (name: string) => void;
    onUpdateFromCurrentPage: () => void;
    onDuplicate: () => void;
    onDelete: () => void;
  } = $props();

  const { t } = getLocalization();
  let editing = $state(false);
  let menuOpen = $state(false);
  let nameDraft = $state("");
  let renameInput = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (!editing) nameDraft = template.name;
  });

  $effect(() => {
    if (!editing) return;
    void tick().then(() => {
      renameInput?.focus();
      renameInput?.select();
    });
  });

  function saveRename(): void {
    const name = nameDraft.trim();
    editing = false;
    menuOpen = false;
    if (!name || name === template.name) {
      nameDraft = template.name;
      return;
    }
    onRename(name);
  }

  function closeMenu(): void {
    menuOpen = false;
  }

  function handleRenameKeydown(event: KeyboardEvent): void {
    if (event.key === "Enter") {
      event.preventDefault();
      saveRename();
    }
    if (event.key === "Escape") {
      event.preventDefault();
      editing = false;
      nameDraft = template.name;
    }
  }
</script>

<div class="notes-page-template-row relative" use:dismissOnOutside={{ enabled: menuOpen, onDismiss: closeMenu }}>
  {#if editing}
    <input
      bind:this={renameInput}
      class="w-full rounded-md border border-border bg-background px-2 py-1.5 text-[0.8rem] text-foreground outline-none"
      aria-label={t("notes.renamePageTemplate")}
      bind:value={nameDraft}
      onkeydown={handleRenameKeydown}
      onblur={saveRename}
    />
  {:else}
    <div class="flex min-w-0 items-center rounded-md text-foreground hover:bg-accent/70">
      <button
        class="flex min-w-0 flex-1 items-center gap-1.5 py-1.5 pl-2 pr-1 text-left text-[0.8rem]"
        type="button"
        aria-label={t("notes.applyPageTemplate", template.name)}
        onclick={onApply}
      >
        <FileText class="size-3.5 shrink-0 text-muted-foreground" />
        <span class="min-w-0 flex-1">
          <span class="block truncate font-medium">{template.name}</span>
          <span class="block truncate text-[0.7rem] text-muted-foreground">
            {t("notes.pageTemplateBlockCount", template.block_count)}
          </span>
        </span>
      </button>
      <button
        class="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-background/80 hover:text-foreground"
        type="button"
        aria-label={t("notes.pageTemplateActions")}
        onclick={(event) => {
          event.stopPropagation();
          menuOpen = !menuOpen;
        }}
      >
        <MoreHorizontal class="size-4" />
      </button>
    </div>
  {/if}

  {#if menuOpen}
    <div
      class="absolute right-1 top-8 z-20 min-w-44 rounded-md border border-border bg-popover py-1 text-popover-foreground shadow-lg"
      data-app-floating-surface
    >
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
        type="button"
        onclick={() => {
          menuOpen = false;
          onApply();
        }}
      >
        <FileText class="size-4" />
        <span>{t("notes.applyTemplate")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
        type="button"
        onclick={() => {
          editing = true;
          menuOpen = false;
        }}
      >
        <Pencil class="size-4" />
        <span>{t("notes.renamePageTemplate")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent disabled:opacity-50"
        type="button"
        disabled={!canUpdateFromCurrentPage}
        onclick={() => {
          menuOpen = false;
          onUpdateFromCurrentPage();
        }}
      >
        <RefreshCw class="size-4" />
        <span>{t("notes.updatePageTemplate")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
        type="button"
        onclick={() => {
          menuOpen = false;
          onDuplicate();
        }}
      >
        <Copy class="size-4" />
        <span>{t("notes.duplicatePageTemplate")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] text-destructive hover:bg-accent"
        type="button"
        onclick={() => {
          menuOpen = false;
          onDelete();
        }}
      >
        <Trash2 class="size-4" />
        <span>{t("notes.deletePageTemplate")}</span>
      </button>
    </div>
  {/if}
</div>
