<script lang="ts">
  import { onMount } from "svelte";
  import {
    createNotesDataSourceRowPage,
    duplicateNotesPage,
    listNotesDataSourceRowPages,
    trashNotesPage,
  } from "$lib/api/notes";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { NotesPage, NotesRichText } from "$lib/notes/types";
  import Copy from "@lucide/svelte/icons/copy";
  import FileText from "@lucide/svelte/icons/file-text";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";

  let {
    dataSourceId,
    onSelectPage,
  }: {
    dataSourceId: string;
    onSelectPage: (pageId: string) => void;
  } = $props();

  const { t } = getLocalization();

  let rows = $state<NotesPage[]>([]);
  let loading = $state(false);
  let mutating = $state(false);
  let error = $state<string | null>(null);
  let draftTitle = $state("");

  onMount(() => {
    void reloadRows();
  });

  async function reloadRows(): Promise<void> {
    loading = true;
    error = null;
    try {
      rows = await listNotesDataSourceRowPages(dataSourceId);
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      loading = false;
    }
  }

  async function createRow(): Promise<void> {
    const title = draftTitle.trim() || t("notes.untitled");
    mutating = true;
    error = null;
    try {
      const loaded = await createNotesDataSourceRowPage(dataSourceId, {
        id: crypto.randomUUID(),
        first_block_id: crypto.randomUUID(),
        title,
      });
      draftTitle = "";
      rows = [loaded.page, ...rows.filter((row) => row.id !== loaded.page.id)];
      onSelectPage(loaded.page.id);
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function duplicateRow(pageId: string): Promise<void> {
    mutating = true;
    error = null;
    try {
      const loaded = await duplicateNotesPage(pageId, {});
      rows = [loaded.page, ...rows.filter((row) => row.id !== loaded.page.id)];
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function trashRow(pageId: string): Promise<void> {
    mutating = true;
    error = null;
    try {
      await trashNotesPage(pageId, true);
      rows = rows.filter((row) => row.id !== pageId);
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  function pageTitle(page: NotesPage): string {
    const titleProperty = Object.values(page.properties).find(isTitleProperty);
    const title = titleProperty ? richTextPlainText(titleProperty.title) : "";
    return title.trim() || t("notes.untitled");
  }

  function isTitleProperty(value: unknown): value is { type: "title"; title: NotesRichText[] } {
    if (!isRecord(value)) return false;
    return value.type === "title" && Array.isArray(value.title);
  }

  function richTextPlainText(items: NotesRichText[]): string {
    return items.map((item) => item.plain_text).join("");
  }

  function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === "object" && value !== null && !Array.isArray(value);
  }
</script>

<section class="space-y-2 border-t border-border pt-3" aria-label={t("notes.databaseRowsTitle")}>
  <div class="flex min-w-0 flex-wrap items-center gap-2 text-[0.8rem] text-muted-foreground">
    <span class="min-w-0 flex-1 truncate">
      {#if loading}
        {t("notes.databaseRowsLoading")}
      {:else if error}
        {t("notes.databaseRowsFailed", error)}
      {:else}
        {t("notes.databaseRowsCount", rows.length)}
      {/if}
    </span>
    <button
      type="button"
      class="inline-flex size-8 items-center justify-center rounded-md hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
      disabled={loading || mutating}
      aria-label={t("notes.databaseRowsReload")}
      title={t("notes.databaseRowsReload")}
      onclick={() => {
        void reloadRows();
      }}
    >
      <RefreshCw class="size-3.5" aria-hidden="true" />
    </button>
  </div>

  <div class="flex min-w-0 flex-wrap items-center gap-2">
    <input
      class="h-8 min-w-0 flex-1 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
      value={draftTitle}
      placeholder={t("notes.databaseRowsNewPlaceholder")}
      disabled={mutating}
      oninput={(event) => {
        draftTitle = event.currentTarget.value;
      }}
      onkeydown={(event) => {
        event.stopPropagation();
        if (event.key === "Enter") {
          event.preventDefault();
          void createRow();
        }
      }}
    />
    <button
      type="button"
      class="inline-flex h-8 items-center gap-1 rounded-md bg-primary px-2 text-[0.8rem] text-primary-foreground disabled:pointer-events-none disabled:opacity-50"
      disabled={mutating}
      onclick={() => {
        void createRow();
      }}
    >
      <Plus class="size-3.5" aria-hidden="true" />
      <span>{t("notes.databaseRowsAdd")}</span>
    </button>
  </div>

  {#if rows.length > 0}
    <div class="min-w-0 divide-y divide-border rounded-md border border-border">
      {#each rows as row (row.id)}
        {@const title = pageTitle(row)}
        <div class="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 px-2 py-1.5">
          <button
            type="button"
            class="flex min-h-8 min-w-0 items-center gap-2 rounded-sm text-left outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
            aria-label={t("notes.databaseRowsOpen", title)}
            onclick={() => onSelectPage(row.id)}
          >
            <FileText class="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
            <span class="min-w-0 truncate text-[0.866667rem] text-foreground">{title}</span>
          </button>
          <div class="flex shrink-0 items-center gap-1">
            <button
              type="button"
              class="inline-flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:pointer-events-none disabled:opacity-50"
              disabled={mutating}
              aria-label={t("notes.databaseRowsDuplicate", title)}
              title={t("notes.databaseRowsDuplicate", title)}
              onclick={() => {
                void duplicateRow(row.id);
              }}
            >
              <Copy class="size-3.5" aria-hidden="true" />
            </button>
            <button
              type="button"
              class="inline-flex size-8 items-center justify-center rounded-md text-destructive hover:bg-destructive/10 disabled:pointer-events-none disabled:opacity-50"
              disabled={mutating}
              aria-label={t("notes.databaseRowsTrash", title)}
              title={t("notes.databaseRowsTrash", title)}
              onclick={() => {
                void trashRow(row.id);
              }}
            >
              <Trash2 class="size-3.5" aria-hidden="true" />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {:else if !loading && !error}
    <p class="text-[0.8rem] text-muted-foreground">{t("notes.databaseRowsEmpty")}</p>
  {/if}
</section>
