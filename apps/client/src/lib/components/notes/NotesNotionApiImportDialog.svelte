<script lang="ts">
  import { onMount, tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { toRoundTripDiagnosticItem } from "$lib/notes/round-trip-diagnostics";
  import type { NotesNotionApiImportResult } from "$lib/notes/types";
  import NotesRoundTripDiagnostics from "./NotesRoundTripDiagnostics.svelte";

  let {
    onImport,
    onCancel,
  }: {
    onImport: (input: {
      integrationToken: string;
      sourceWorkspaceId: string | null;
      pageIds: string[];
      dataSourceIds: string[];
      includeComments: boolean;
      includeUsers: boolean;
      keepExternalFileReferences: boolean;
      pageSize: number;
    }) => Promise<NotesNotionApiImportResult>;
    onCancel: () => void;
  } = $props();

  const { t } = getLocalization();
  let integrationToken = $state("");
  let sourceWorkspaceId = $state("");
  let pageIdsText = $state("");
  let dataSourceIdsText = $state("");
  let includeComments = $state(true);
  let includeUsers = $state(true);
  let keepExternalFileReferences = $state(false);
  let pageSize = $state(50);
  let importing = $state(false);
  let error = $state<string | null>(null);
  let result = $state<NotesNotionApiImportResult | null>(null);
  let tokenInputEl = $state<HTMLInputElement | null>(null);

  const pageIds = $derived(splitSourceIds(pageIdsText));
  const dataSourceIds = $derived(splitSourceIds(dataSourceIdsText));
  const canImport = $derived(
    integrationToken.trim().length > 0
      && (pageIds.length > 0 || dataSourceIds.length > 0)
      && pageSize >= 1
      && pageSize <= 100
      && !importing,
  );
  const roundTripDiagnostics = $derived(
    result?.diagnostics.map((diagnostic) =>
      toRoundTripDiagnosticItem({
        code: diagnostic.code,
        severity: diagnostic.severity,
        message: diagnostic.message,
        sourceLabel: diagnostic.source_object_id
          ? t("notes.roundTripSourceObject", diagnostic.source_object_id)
          : null,
      }),
    ) ?? [],
  );
  const roundTripCounts = $derived(
    result
      ? [
          {
            id: "pages",
            label: t("notes.roundTripCountPages"),
            value: result.imported_page_count,
          },
          {
            id: "blocks",
            label: t("notes.roundTripCountBlocks"),
            value: result.imported_block_count,
          },
          {
            id: "data-sources",
            label: t("notes.roundTripCountDataSources"),
            value: result.imported_data_source_count,
          },
          {
            id: "comments",
            label: t("notes.roundTripCountComments"),
            value: result.imported_comment_count,
          },
          {
            id: "files",
            label: t("notes.roundTripCountFiles"),
            value: result.imported_file_count,
          },
          {
            id: "users",
            label: t("notes.roundTripCountUsers"),
            value: result.imported_user_count,
          },
          {
            id: "unsupported-blocks",
            label: t("notes.roundTripCountUnsupportedBlocks"),
            value: result.unsupported_block_count,
          },
          {
            id: "requests",
            label: t("notes.roundTripCountRequests"),
            value: result.request_count,
          },
          {
            id: "retries",
            label: t("notes.roundTripCountRetries"),
            value: result.retry_count,
          },
          {
            id: "rate-limits",
            label: t("notes.roundTripCountRateLimits"),
            value: result.rate_limit_count,
          },
        ]
      : [],
  );

  onMount(() => {
    void tick().then(() => {
      tokenInputEl?.focus();
    });
  });

  async function submit(): Promise<void> {
    if (!canImport) return;
    importing = true;
    error = null;
    result = null;
    try {
      result = await onImport({
        integrationToken,
        sourceWorkspaceId: sourceWorkspaceId.trim() || null,
        pageIds,
        dataSourceIds,
        includeComments,
        includeUsers,
        keepExternalFileReferences,
        pageSize,
      });
      integrationToken = "";
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      importing = false;
    }
  }

  function splitSourceIds(value: string): string[] {
    const seen = new Set<string>();
    return value
      .split(/[\n,]/u)
      .map((item) => item.trim())
      .filter((item) => item.length > 0)
      .filter((item) => {
        if (seen.has(item)) return false;
        seen.add(item);
        return true;
      });
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
    class="relative z-10 flex max-h-[min(92vh,46rem)] w-[min(44rem,100%)] flex-col rounded-md border border-border bg-card text-card-foreground shadow-lg outline-none"
    role="dialog"
    aria-modal="true"
    aria-label={t("notes.notionApiImportDialogTitle")}
    tabindex="-1"
    onclick={(event) => event.stopPropagation()}
    onkeydown={handleKeydown}
  >
    <div class="shrink-0 border-b border-border px-4 py-3">
      <h2 class="text-[1rem] font-semibold text-foreground">
        {t("notes.notionApiImportDialogTitle")}
      </h2>
    </div>

    <div class="min-h-0 flex-1 overflow-auto px-4 py-3">
      <div class="grid gap-3">
        <label class="grid gap-1.5">
          <span class="text-[0.8rem] font-medium text-foreground">
            {t("notes.notionApiImportTokenLabel")}
          </span>
          <input
            bind:this={tokenInputEl}
            class="min-h-9 rounded-md border border-border bg-background px-2.5 py-1.5 text-[0.866667rem] text-foreground outline-none focus:border-ring"
            type="password"
            bind:value={integrationToken}
            autocomplete="off"
            spellcheck="false"
          />
        </label>

        <label class="grid gap-1.5">
          <span class="text-[0.8rem] font-medium text-foreground">
            {t("notes.notionApiImportWorkspaceLabel")}
          </span>
          <input
            class="min-h-9 rounded-md border border-border bg-background px-2.5 py-1.5 text-[0.866667rem] text-foreground outline-none focus:border-ring"
            bind:value={sourceWorkspaceId}
            spellcheck="false"
          />
        </label>

        <div class="grid gap-3 md:grid-cols-2">
          <label class="grid gap-1.5">
            <span class="text-[0.8rem] font-medium text-foreground">
              {t("notes.notionApiImportPagesLabel")}
            </span>
            <textarea
              class="min-h-28 resize-y rounded-md border border-border bg-background px-2.5 py-2 font-mono text-[0.8rem] leading-relaxed text-foreground outline-none focus:border-ring"
              bind:value={pageIdsText}
              spellcheck="false"
            ></textarea>
          </label>

          <label class="grid gap-1.5">
            <span class="text-[0.8rem] font-medium text-foreground">
              {t("notes.notionApiImportDataSourcesLabel")}
            </span>
            <textarea
              class="min-h-28 resize-y rounded-md border border-border bg-background px-2.5 py-2 font-mono text-[0.8rem] leading-relaxed text-foreground outline-none focus:border-ring"
              bind:value={dataSourceIdsText}
              spellcheck="false"
            ></textarea>
          </label>
        </div>

        <label class="grid max-w-36 gap-1.5">
          <span class="text-[0.8rem] font-medium text-foreground">
            {t("notes.notionApiImportPageSizeLabel")}
          </span>
          <input
            class="min-h-9 rounded-md border border-border bg-background px-2.5 py-1.5 text-[0.866667rem] text-foreground outline-none focus:border-ring"
            type="number"
            min="1"
            max="100"
            bind:value={pageSize}
          />
        </label>

        <div class="grid gap-2">
          <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
            <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={includeComments} />
            <span>{t("notes.notionApiImportIncludeComments")}</span>
          </label>
          <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
            <input class="mt-0.5 size-3.5 shrink-0 accent-primary" type="checkbox" bind:checked={includeUsers} />
            <span>{t("notes.notionApiImportIncludeUsers")}</span>
          </label>
          <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
            <input
              class="mt-0.5 size-3.5 shrink-0 accent-primary"
              type="checkbox"
              bind:checked={keepExternalFileReferences}
            />
            <span>{t("notes.notionApiImportKeepExternalFiles")}</span>
          </label>
        </div>

        {#if error}
          <div class="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-[0.8rem] text-destructive">
            {t("notes.notionApiImportFailed", error)}
          </div>
        {/if}

        {#if result}
          <div class="rounded-md border border-border bg-muted/35 px-3 py-2">
            <div class="text-[0.8rem] font-medium text-foreground">
              {t(
                "notes.notionApiImportComplete",
                result.imported_page_count,
                result.imported_block_count,
                result.imported_comment_count,
              )}
            </div>
            <div class="mt-1 text-[0.733333rem] text-muted-foreground">
              {t(
                "notes.notionApiImportRequests",
                result.request_count,
                result.retry_count,
                result.rate_limit_count,
              )}
            </div>
            <div class="mt-2">
              <NotesRoundTripDiagnostics counts={roundTripCounts} diagnostics={roundTripDiagnostics} />
            </div>
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
        {importing ? t("notes.notionApiImportImporting") : t("notes.notionApiImportSubmit")}
      </button>
    </div>
  </div>
</div>
