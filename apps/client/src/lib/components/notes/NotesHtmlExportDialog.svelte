<script lang="ts">
  import { onMount, tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    buildRoundTripNotesSourceHref,
    roundTripWarningCount,
    toRoundTripDiagnosticItem,
  } from "$lib/notes/round-trip-diagnostics";
  import type {
    NotesHtmlArchiveSaveResult,
    NotesHtmlExportDiagnostic,
  } from "$lib/notes/types";
  import NotesRoundTripDiagnostics from "./NotesRoundTripDiagnostics.svelte";

  let {
    pageTitle,
    onExport,
    onCancel,
  }: {
    pageTitle: string;
    onExport: (input: {
      includePageTree: boolean;
      includeComments: boolean;
      includeResolvedComments: boolean;
      includeAssets: boolean;
      includeDatabaseViews: boolean;
    }) => Promise<NotesHtmlArchiveSaveResult>;
    onCancel: () => void;
  } = $props();

  const { t } = getLocalization();
  let includePageTree = $state(true);
  let includeComments = $state(false);
  let includeResolvedComments = $state(false);
  let includeAssets = $state(true);
  let includeDatabaseViews = $state(true);
  let exporting = $state(false);
  let error = $state<string | null>(null);
  let result = $state<NotesHtmlArchiveSaveResult | null>(null);
  let dialogEl = $state<HTMLDivElement | null>(null);
  const canExport = $derived(!exporting);
  const roundTripDiagnostics = $derived(
    result?.export?.diagnostics.map((diagnostic) =>
      toRoundTripDiagnosticItem({
        code: diagnostic.code,
        severity: diagnostic.severity,
        message: diagnostic.message,
        sourceLabel: htmlExportSourceLabel(diagnostic),
        sourceHref: htmlExportSourceHref(diagnostic),
      }),
    ) ?? [],
  );
  const warningCount = $derived(roundTripWarningCount(roundTripDiagnostics));
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
            id: "assets",
            label: t("notes.roundTripCountAssets"),
            value: result.export.exported_asset_count,
          },
          {
            id: "comments",
            label: t("notes.roundTripCountComments"),
            value: result.export.exported_comment_count,
          },
          {
            id: "views",
            label: t("notes.roundTripCountDatabaseViews"),
            value: result.export.exported_database_view_count,
          },
          {
            id: "warnings",
            label: t("notes.roundTripCountWarnings"),
            value: warningCount,
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
        includePageTree,
        includeComments,
        includeResolvedComments: includeComments && includeResolvedComments,
        includeAssets,
        includeDatabaseViews,
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

  function htmlExportSourceLabel(diagnostic: NotesHtmlExportDiagnostic): string | null {
    if (diagnostic.block_id) return t("notes.roundTripSourceBlock", diagnostic.block_id);
    if (diagnostic.comment_id) return t("notes.roundTripSourceComment", diagnostic.comment_id);
    if (diagnostic.asset_id) return t("notes.roundTripSourceAsset", diagnostic.asset_id);
    if (diagnostic.page_id) return t("notes.roundTripSourcePage", diagnostic.page_id);
    return null;
  }

  function htmlExportSourceHref(diagnostic: NotesHtmlExportDiagnostic): string | null {
    if (typeof window === "undefined") return null;
    return buildRoundTripNotesSourceHref(window.location.href, {
      pageId: diagnostic.page_id,
      blockId: diagnostic.block_id,
    });
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
    aria-label={t("notes.htmlExportDialogTitle")}
    tabindex="-1"
    onkeydown={handleKeydown}
  >
    <div class="shrink-0 border-b border-border px-4 py-3">
      <h2 class="text-[1rem] font-semibold text-foreground">
        {t("notes.htmlExportDialogTitle")}
      </h2>
      <p class="mt-1 text-[0.8rem] text-muted-foreground">
        {pageTitle}
      </p>
    </div>

    <div class="min-h-0 flex-1 overflow-auto px-4 py-3">
      <div class="grid gap-2">
        <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
          <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={includePageTree} />
          <span>{t("notes.htmlExportIncludePageTree")}</span>
        </label>
        <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
          <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={includeAssets} />
          <span>{t("notes.htmlExportIncludeAssets")}</span>
        </label>
        <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
          <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={includeDatabaseViews} />
          <span>{t("notes.htmlExportIncludeDatabaseViews")}</span>
        </label>
        <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
          <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={includeComments} />
          <span>{t("notes.htmlExportIncludeComments")}</span>
        </label>
        <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
          <input
            class="mt-0.5 size-3.5 shrink-0 accent-primary"
            type="checkbox"
            bind:checked={includeResolvedComments}
            disabled={!includeComments}
          />
          <span>{t("notes.htmlExportIncludeResolvedComments")}</span>
        </label>

        {#if error}
          <div class="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-[0.8rem] text-destructive">
            {t("notes.htmlExportFailed", error)}
          </div>
        {/if}

        {#if result}
          <div class="rounded-md border border-border bg-muted/35 px-3 py-2">
            <div class="text-[0.8rem] font-medium text-foreground">
              {#if result.saved && result.export}
                {t(
                  "notes.htmlExportComplete",
                  result.export.exported_page_count,
                  result.export.exported_block_count,
                  result.export.exported_asset_count,
                  warningCount,
                )}
              {:else}
                {t("notes.htmlExportCanceled")}
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
        {exporting ? t("notes.htmlExportExporting") : t("notes.htmlExportSubmit")}
      </button>
    </div>
  </div>
</div>
