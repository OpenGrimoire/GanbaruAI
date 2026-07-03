<script lang="ts">
  import { onMount, tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { NotesNotionExportImportResult } from "$lib/notes/types";

  let {
    onImport,
    onCancel,
  }: {
    onImport: (input: {
      exportRootPath: string;
      sourceWorkspaceId: string | null;
      keepExternalFileReferences: boolean;
      copyLocalFileReferences: boolean;
      importMarkdown: boolean;
      importHtml: boolean;
      importCsv: boolean;
    }) => Promise<NotesNotionExportImportResult>;
    onCancel: () => void;
  } = $props();

  const { t } = getLocalization();
  let exportRootPath = $state("");
  let sourceWorkspaceId = $state("");
  let keepExternalFileReferences = $state(false);
  let copyLocalFileReferences = $state(true);
  let importMarkdown = $state(true);
  let importHtml = $state(true);
  let importCsv = $state(true);
  let importing = $state(false);
  let error = $state<string | null>(null);
  let result = $state<NotesNotionExportImportResult | null>(null);
  let pathInputEl = $state<HTMLInputElement | null>(null);

  const canImport = $derived(
    exportRootPath.trim().length > 0
      && (importMarkdown || importHtml || importCsv)
      && !importing,
  );

  const warningCount = $derived(
    result?.diagnostics.filter((diagnostic) => diagnostic.severity !== "info").length ?? 0,
  );

  onMount(() => {
    void tick().then(() => {
      pathInputEl?.focus();
    });
  });

  async function submit(): Promise<void> {
    if (!canImport) return;
    importing = true;
    error = null;
    result = null;
    try {
      result = await onImport({
        exportRootPath,
        sourceWorkspaceId: sourceWorkspaceId.trim() || null,
        keepExternalFileReferences,
        copyLocalFileReferences,
        importMarkdown,
        importHtml,
        importCsv,
      });
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      importing = false;
    }
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      onCancel();
      return;
    }
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
      event.preventDefault();
      event.stopPropagation();
      void submit();
      return;
    }
    event.stopPropagation();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-90 flex items-center justify-center p-3"
  onclick={(event) => {
    event.stopPropagation();
    onCancel();
  }}
>
  <div class="absolute inset-0 bg-black/50"></div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="relative z-10 flex max-h-[min(92vh,44rem)] w-[min(42rem,100%)] flex-col rounded-md border border-border bg-card text-card-foreground shadow-lg outline-none"
    role="dialog"
    aria-modal="true"
    aria-label={t("notes.notionExportImportDialogTitle")}
    tabindex="-1"
    onclick={(event) => event.stopPropagation()}
    onkeydown={handleKeydown}
  >
    <div class="shrink-0 border-b border-border px-4 py-3">
      <h2 class="text-[1rem] font-semibold text-foreground">
        {t("notes.notionExportImportDialogTitle")}
      </h2>
    </div>

    <div class="min-h-0 flex-1 overflow-auto px-4 py-3">
      <div class="grid gap-3">
        <label class="grid gap-1.5">
          <span class="text-[0.8rem] font-medium text-foreground">
            {t("notes.notionExportImportFolderLabel")}
          </span>
          <input
            bind:this={pathInputEl}
            class="min-h-9 rounded-md border border-border bg-background px-2.5 py-1.5 text-[0.866667rem] text-foreground outline-none focus:border-ring"
            bind:value={exportRootPath}
            spellcheck="false"
          />
        </label>

        <label class="grid gap-1.5">
          <span class="text-[0.8rem] font-medium text-foreground">
            {t("notes.notionExportImportWorkspaceLabel")}
          </span>
          <input
            class="min-h-9 rounded-md border border-border bg-background px-2.5 py-1.5 text-[0.866667rem] text-foreground outline-none focus:border-ring"
            bind:value={sourceWorkspaceId}
            spellcheck="false"
          />
        </label>

        <div class="grid gap-2">
          <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
            <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={importMarkdown} />
            <span>{t("notes.notionExportImportMarkdown")}</span>
          </label>
          <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
            <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={importHtml} />
            <span>{t("notes.notionExportImportHtml")}</span>
          </label>
          <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
            <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={importCsv} />
            <span>{t("notes.notionExportImportCsv")}</span>
          </label>
          <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
            <input
              class="mt-0.5 size-3.5 shrink-0 accent-primary"
              type="checkbox"
              bind:checked={copyLocalFileReferences}
            />
            <span>{t("notes.notionExportImportCopyFiles")}</span>
          </label>
          <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
            <input
              class="mt-0.5 size-3.5 shrink-0 accent-primary"
              type="checkbox"
              bind:checked={keepExternalFileReferences}
            />
            <span>{t("notes.notionExportImportKeepExternalFiles")}</span>
          </label>
        </div>

        {#if error}
          <div class="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-[0.8rem] text-destructive">
            {t("notes.notionExportImportFailed", error)}
          </div>
        {/if}

        {#if result}
          <div class="rounded-md border border-border bg-muted/35 px-3 py-2">
            <div class="text-[0.8rem] font-medium text-foreground">
              {t(
                "notes.notionExportImportComplete",
                result.imported_page_count,
                result.imported_data_source_count,
                result.imported_file_count,
                warningCount,
              )}
            </div>
            {#if result.diagnostics.length > 0}
              <ul class="mt-2 grid max-h-36 gap-1 overflow-auto text-[0.733333rem] text-muted-foreground">
                {#each result.diagnostics as diagnostic, index (`${diagnostic.code}-${index}`)}
                  <li>
                    {#if diagnostic.source_path}
                      {t("notes.notionExportImportDiagnosticWithPath", diagnostic.source_path, diagnostic.message)}
                    {:else}
                      {diagnostic.message}
                    {/if}
                  </li>
                {/each}
              </ul>
            {/if}
          </div>
        {/if}
      </div>
    </div>

    <div class="flex shrink-0 flex-wrap justify-end gap-2 border-t border-border px-4 py-3">
      <button
        class="rounded-md border border-border bg-card px-3 py-1.5 text-[0.866667rem] font-medium text-foreground hover:bg-accent"
        type="button"
        onclick={onCancel}
      >
        {t("common.close")}
      </button>
      <button
        class="rounded-md bg-primary px-3 py-1.5 text-[0.866667rem] font-medium text-primary-foreground hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-45"
        type="button"
        disabled={!canImport}
        onclick={() => {
          void submit();
        }}
      >
        {importing ? t("notes.notionExportImportImporting") : t("notes.notionExportImportSubmit")}
      </button>
    </div>
  </div>
</div>
