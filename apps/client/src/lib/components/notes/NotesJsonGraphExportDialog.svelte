<script lang="ts">
  import { onMount, tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { toRoundTripDiagnosticItem } from "$lib/notes/round-trip-diagnostics";
  import type {
    NotesJsonGraphExportDiagnostic,
    NotesJsonGraphExportSaveResult,
  } from "$lib/notes/types";
  import NotesRoundTripDiagnostics from "./NotesRoundTripDiagnostics.svelte";

  let {
    onExport,
    onCancel,
  }: {
    onExport: (input: {
      includeIndexes: boolean;
      includeHistory: boolean;
      includeTemplates: boolean;
      includeLocalState: boolean;
      pretty: boolean;
    }) => Promise<NotesJsonGraphExportSaveResult>;
    onCancel: () => void;
  } = $props();

  const { t } = getLocalization();
  let includeIndexes = $state(true);
  let includeHistory = $state(true);
  let includeTemplates = $state(true);
  let includeLocalState = $state(true);
  let pretty = $state(true);
  let exporting = $state(false);
  let error = $state<string | null>(null);
  let result = $state<NotesJsonGraphExportSaveResult | null>(null);
  let dialogEl = $state<HTMLDivElement | null>(null);
  const canExport = $derived(!exporting);
  const roundTripDiagnostics = $derived(
    result?.export?.diagnostics.map((diagnostic) =>
      toRoundTripDiagnosticItem({
        code: diagnostic.code,
        severity: diagnostic.severity,
        message: diagnostic.message,
        sourceLabel: jsonGraphSourceLabel(diagnostic),
      }),
    ) ?? [],
  );
  const roundTripCounts = $derived(
    result?.export
      ? [
          {
            id: "pages",
            label: t("notes.roundTripCountPages"),
            value: result.export.exported_page_count,
          },
          {
            id: "blocks",
            label: t("notes.roundTripCountBlocks"),
            value: result.export.exported_block_count,
          },
          {
            id: "comments",
            label: t("notes.roundTripCountComments"),
            value: result.export.exported_comment_count,
          },
          {
            id: "data-sources",
            label: t("notes.roundTripCountDataSources"),
            value: result.export.exported_data_source_count,
          },
          {
            id: "files",
            label: t("notes.roundTripCountFiles"),
            value: result.export.exported_file_count,
          },
          {
            id: "indexes",
            label: t("notes.roundTripCountIndexes"),
            value: result.export.exported_index_record_count,
          },
          {
            id: "tables",
            label: t("notes.roundTripCountTables"),
            value: result.export.exported_table_count,
          },
          {
            id: "records",
            label: t("notes.roundTripCountRecords"),
            value: result.export.exported_record_count,
          },
          {
            id: "warnings",
            label: t("notes.roundTripCountWarnings"),
            value: result.export.warning_count,
          },
        ]
      : [],
  );

  onMount(() => {
    void tick().then(() => {
      dialogEl?.focus();
    });
  });

  async function submit(): Promise<void> {
    if (!canExport) return;
    exporting = true;
    error = null;
    result = null;
    try {
      result = await onExport({
        includeIndexes,
        includeHistory,
        includeTemplates,
        includeLocalState,
        pretty,
      });
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      exporting = false;
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

  function jsonGraphSourceLabel(diagnostic: NotesJsonGraphExportDiagnostic): string | null {
    if (diagnostic.table_name && diagnostic.row_id) {
      return t("notes.roundTripSourceTableRow", diagnostic.table_name, diagnostic.row_id);
    }
    if (diagnostic.table_name) return diagnostic.table_name;
    if (diagnostic.row_id) return t("notes.roundTripSourceObject", diagnostic.row_id);
    return null;
  }
</script>

<div class="fixed inset-0 z-90 flex items-center justify-center p-3">
  <button
    class="absolute inset-0 border-0 bg-black/50 p-0"
    type="button"
    aria-label={t("common.close")}
    onclick={onCancel}
  ></button>
  <div
    bind:this={dialogEl}
    class="relative z-10 flex max-h-[min(92vh,38rem)] w-[min(34rem,100%)] flex-col rounded-md border border-border bg-card text-card-foreground shadow-lg outline-none"
    role="dialog"
    aria-modal="true"
    aria-label={t("notes.jsonGraphExportDialogTitle")}
    tabindex="-1"
    onkeydown={handleKeydown}
  >
    <div class="shrink-0 border-b border-border px-4 py-3">
      <h2 class="text-[1rem] font-semibold text-foreground">
        {t("notes.jsonGraphExportDialogTitle")}
      </h2>
    </div>

    <div class="min-h-0 flex-1 overflow-auto px-4 py-3">
      <div class="grid gap-2">
        <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
          <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={includeIndexes} />
          <span>{t("notes.jsonGraphExportIncludeIndexes")}</span>
        </label>
        <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
          <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={includeHistory} />
          <span>{t("notes.jsonGraphExportIncludeHistory")}</span>
        </label>
        <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
          <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={includeTemplates} />
          <span>{t("notes.jsonGraphExportIncludeTemplates")}</span>
        </label>
        <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
          <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={includeLocalState} />
          <span>{t("notes.jsonGraphExportIncludeLocalState")}</span>
        </label>
        <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
          <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={pretty} />
          <span>{t("notes.jsonGraphExportPretty")}</span>
        </label>

        {#if error}
          <div class="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-[0.8rem] text-destructive">
            {t("notes.jsonGraphExportFailed", error)}
          </div>
        {/if}

        {#if result}
          <div class="rounded-md border border-border bg-muted/35 px-3 py-2">
            <div class="text-[0.8rem] font-medium text-foreground">
              {#if result.saved && result.export}
                {t(
                  "notes.jsonGraphExportComplete",
                  result.export.exported_page_count,
                  result.export.exported_block_count,
                  result.export.exported_data_source_count,
                  result.export.exported_file_count,
                  result.export.warning_count,
                )}
              {:else}
                {t("notes.jsonGraphExportCanceled")}
              {/if}
            </div>
            {#if result.export}
              <div class="mt-2">
                <NotesRoundTripDiagnostics counts={roundTripCounts} diagnostics={roundTripDiagnostics} />
              </div>
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
        disabled={!canExport}
        onclick={() => {
          void submit();
        }}
      >
        {exporting ? t("notes.jsonGraphExportExporting") : t("notes.jsonGraphExportSubmit")}
      </button>
    </div>
  </div>
</div>
