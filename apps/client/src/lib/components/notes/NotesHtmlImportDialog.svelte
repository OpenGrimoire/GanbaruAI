<script lang="ts">
  import { onMount, tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { NotesHtmlImportResult } from "$lib/notes/types";

  let {
    onImport,
    onCancel,
  }: {
    onImport: (input: {
      html: string;
      title: string | null;
      sourceName: string | null;
      keepExternalFileReferences: boolean;
    }) => Promise<NotesHtmlImportResult>;
    onCancel: () => void;
  } = $props();

  const { t } = getLocalization();
  let html = $state("");
  let title = $state("");
  let sourceName = $state("");
  let keepExternalFileReferences = $state(false);
  let importing = $state(false);
  let error = $state<string | null>(null);
  let result = $state<NotesHtmlImportResult | null>(null);
  let dialogEl = $state<HTMLDivElement | null>(null);
  let textareaEl = $state<HTMLTextAreaElement | null>(null);

  const canImport = $derived(html.trim().length > 0 && !importing);
  const warningCount = $derived(
    result?.diagnostics.filter((diagnostic) => diagnostic.severity !== "info").length ?? 0,
  );

  onMount(() => {
    void tick().then(() => {
      textareaEl?.focus();
    });
  });

  async function submit(): Promise<void> {
    if (!canImport) return;
    importing = true;
    error = null;
    result = null;
    try {
      result = await onImport({
        html,
        title: title.trim() || null,
        sourceName: sourceName.trim() || null,
        keepExternalFileReferences,
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
    bind:this={dialogEl}
    class="relative z-10 flex max-h-[min(92vh,44rem)] w-[min(42rem,100%)] flex-col rounded-md border border-border bg-card text-card-foreground shadow-lg outline-none"
    role="dialog"
    aria-modal="true"
    aria-label={t("notes.htmlImportDialogTitle")}
    tabindex="-1"
    onclick={(event) => event.stopPropagation()}
    onkeydown={handleKeydown}
  >
    <div class="shrink-0 border-b border-border px-4 py-3">
      <h2 class="text-[1rem] font-semibold text-foreground">
        {t("notes.htmlImportDialogTitle")}
      </h2>
    </div>

    <div class="min-h-0 flex-1 overflow-auto px-4 py-3">
      <div class="grid gap-3">
        <label class="grid gap-1.5">
          <span class="text-[0.8rem] font-medium text-foreground">
            {t("notes.htmlImportTitleLabel")}
          </span>
          <input
            class="min-h-9 rounded-md border border-border bg-background px-2.5 py-1.5 text-[0.866667rem] text-foreground outline-none focus:border-ring"
            bind:value={title}
            placeholder={t("notes.htmlImportTitlePlaceholder")}
          />
        </label>

        <label class="grid gap-1.5">
          <span class="text-[0.8rem] font-medium text-foreground">
            {t("notes.htmlImportSourceLabel")}
          </span>
          <input
            class="min-h-9 rounded-md border border-border bg-background px-2.5 py-1.5 text-[0.866667rem] text-foreground outline-none focus:border-ring"
            bind:value={sourceName}
            placeholder={t("notes.htmlImportSourcePlaceholder")}
          />
        </label>

        <label class="grid gap-1.5">
          <span class="text-[0.8rem] font-medium text-foreground">
            {t("notes.htmlImportHtmlLabel")}
          </span>
          <textarea
            bind:this={textareaEl}
            class="min-h-52 resize-y rounded-md border border-border bg-background px-2.5 py-2 font-mono text-[0.8rem] leading-relaxed text-foreground outline-none focus:border-ring"
            bind:value={html}
            spellcheck="false"
          ></textarea>
        </label>

        <label class="flex items-start gap-2 rounded-md border border-border bg-muted/35 px-3 py-2 text-[0.8rem] text-foreground">
          <input
            class="mt-0.5 size-3.5 shrink-0 accent-primary"
            type="checkbox"
            bind:checked={keepExternalFileReferences}
          />
          <span>{t("notes.htmlImportKeepExternalMedia")}</span>
        </label>

        {#if error}
          <div class="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-[0.8rem] text-destructive">
            {t("notes.htmlImportFailed", error)}
          </div>
        {/if}

        {#if result}
          <div class="rounded-md border border-border bg-muted/35 px-3 py-2">
            <div class="text-[0.8rem] font-medium text-foreground">
              {t("notes.htmlImportComplete", result.imported_block_count, warningCount)}
            </div>
            {#if result.diagnostics.length > 0}
              <ul class="mt-2 grid max-h-32 gap-1 overflow-auto text-[0.733333rem] text-muted-foreground">
                {#each result.diagnostics as diagnostic, index (`${diagnostic.code}-${index}`)}
                  <li>
                    {#if diagnostic.line}
                      {t("notes.htmlImportDiagnosticWithLine", diagnostic.line, diagnostic.message)}
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
        {importing ? t("notes.htmlImportImporting") : t("notes.htmlImportSubmit")}
      </button>
    </div>
  </div>
</div>
