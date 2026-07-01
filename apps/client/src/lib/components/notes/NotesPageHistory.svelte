<script lang="ts">
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { blockPlainText } from "$lib/notes/block-factory";
  import { buildNotesChildIdsByParent, flattenNotesBlockTree } from "$lib/notes/block-tree";
  import { notesPageTitle } from "$lib/notes/page-title";
  import type { NotesBlock, NotesBlockTreeItem, NotesBlockType, NotesLoadedPage } from "$lib/notes/types";
  import { getNotes } from "$lib/stores/notes.svelte";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Copy from "@lucide/svelte/icons/copy";
  import History from "@lucide/svelte/icons/history";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";

  const notes = getNotes();
  const localization = getLocalization();
  const { t } = localization;
  let open = $state(false);
  let selectedSnapshotId = $state<string | null>(null);
  let restoreSnapshotId = $state<string | null>(null);
  let lastPageId = $state<string | null>(null);

  const selectedSnapshot = $derived(
    notes.pageHistorySnapshots.find((snapshot) => snapshot.id === selectedSnapshotId) ?? null,
  );
  const restoreSnapshot = $derived(
    notes.pageHistorySnapshots.find((snapshot) => snapshot.id === restoreSnapshotId) ?? null,
  );
  const versionTitle = $derived(
    notes.pageHistoryVersion
      ? notesPageTitle(notes.pageHistoryVersion.page, t("notes.untitled"))
      : "",
  );
  const versionItems = $derived(historyPreviewItems(notes.pageHistoryVersion));
  const retentionValue = $derived(retentionDaysToValue(notes.pageHistorySettings?.retention_days ?? 30));

  $effect(() => {
    const pageId = notes.selectedPageId;
    if (pageId === lastPageId) return;
    lastPageId = pageId;
    selectedSnapshotId = null;
    restoreSnapshotId = null;
  });

  $effect(() => {
    if (!open || notes.pageHistorySettings || notes.pageHistorySettingsLoading) return;
    void notes.loadPageHistorySettings();
  });

  $effect(() => {
    if (!open || selectedSnapshotId || notes.pageHistorySnapshotsLoading) return;
    const firstSnapshot = notes.pageHistorySnapshots[0];
    if (!firstSnapshot) return;
    selectedSnapshotId = firstSnapshot.id;
    void notes.loadPageHistoryVersion(firstSnapshot.id);
  });

  $effect(() => {
    if (!selectedSnapshotId) return;
    if (notes.pageHistorySnapshots.some((snapshot) => snapshot.id === selectedSnapshotId)) return;
    selectedSnapshotId = null;
  });

  function historyPreviewItems(loaded: NotesLoadedPage | null): NotesBlockTreeItem[] {
    if (!loaded) return [];
    const blocksById = Object.fromEntries(
      loaded.blocks.results.map((block) => [block.id, block]),
    ) as Record<string, NotesBlock>;
    const childIdsByParentId = buildNotesChildIdsByParent(loaded.blocks.results);
    return flattenNotesBlockTree({ blocksById, childIdsByParentId }, loaded.page.id).slice(0, 30);
  }

  function retentionDaysToValue(retentionDays: number | null): string {
    return retentionDays === null ? "forever" : String(retentionDays);
  }

  function retentionValueToDays(value: string): number | null {
    if (value === "forever") return null;
    const parsed = Number.parseInt(value, 10);
    return Number.isFinite(parsed) ? parsed : 30;
  }

  function formatTime(value: string): string {
    return new Date(value).toLocaleString(localization.locale);
  }

  function blockTypeLabel(type: NotesBlockType): string {
    switch (type) {
      case "paragraph":
        return t("notes.blockType.paragraph");
      case "heading_1":
        return t("notes.blockType.heading1");
      case "heading_2":
        return t("notes.blockType.heading2");
      case "heading_3":
        return t("notes.blockType.heading3");
      case "heading_4":
        return t("notes.blockType.heading4");
      case "bulleted_list_item":
        return t("notes.blockType.bullet");
      case "numbered_list_item":
        return t("notes.blockType.numbered");
      case "to_do":
        return t("notes.blockType.todo");
      case "toggle":
        return t("notes.blockType.toggle");
      case "callout":
        return t("notes.blockType.callout");
      case "quote":
        return t("notes.blockType.quote");
      case "child_page":
        return t("notes.blockType.childPage");
      case "child_database":
        return t("notes.blockType.childDatabase");
      case "breadcrumb":
        return t("notes.blockType.breadcrumb");
      case "table_of_contents":
        return t("notes.blockType.tableOfContents");
      case "column_list":
        return t("notes.blockType.columns");
      case "column":
        return t("notes.blockType.columns");
      case "table":
        return t("notes.blockType.table");
      case "table_row":
        return t("notes.blockType.table");
      case "tab":
        return t("notes.blockType.tab");
      case "image":
        return t("notes.blockType.image");
      case "video":
        return t("notes.blockType.video");
      case "audio":
        return t("notes.blockType.audio");
      case "file":
        return t("notes.blockType.file");
      case "pdf":
        return t("notes.blockType.pdf");
      case "bookmark":
        return t("notes.blockType.bookmark");
      case "link_preview":
        return t("notes.blockType.linkPreview");
      case "synced_block":
        return t("notes.blockType.syncedBlock");
      case "template":
        return t("notes.blockType.template");
      case "button":
        return t("notes.blockType.button");
      case "embed":
        return t("notes.blockType.embed");
      case "equation":
        return t("notes.blockType.equation");
      case "divider":
        return t("notes.blockType.divider");
      case "code":
        return t("notes.blockType.code");
      case "unsupported":
        return t("notes.blockType.unsupported");
    }
  }

  function reasonLabel(reason: string): string {
    switch (reason) {
      case "create_child_page":
        return t("notes.pageHistoryReason.createChildPage");
      case "create_child_page_from_block":
        return t("notes.pageHistoryReason.createChildPageFromBlock");
      case "duplicate_page":
        return t("notes.pageHistoryReason.duplicatePage");
      case "move_page":
        return t("notes.pageHistoryReason.movePage");
      case "update_page":
        return t("notes.pageHistoryReason.updatePage");
      case "trash_page":
        return t("notes.pageHistoryReason.trashPage");
      case "archive_page":
        return t("notes.pageHistoryReason.archivePage");
      case "append_block_children":
        return t("notes.pageHistoryReason.appendBlockChildren");
      case "update_block":
        return t("notes.pageHistoryReason.updateBlock");
      case "trash_block":
        return t("notes.pageHistoryReason.trashBlock");
      case "trash_blocks":
        return t("notes.pageHistoryReason.trashBlocks");
      case "move_block":
        return t("notes.pageHistoryReason.moveBlock");
      case "move_blocks":
        return t("notes.pageHistoryReason.moveBlocks");
      case "duplicate_block":
        return t("notes.pageHistoryReason.duplicateBlock");
      case "duplicate_blocks":
        return t("notes.pageHistoryReason.duplicateBlocks");
      case "restore":
        return t("notes.pageHistoryReason.restore");
      case "copy_history_blocks":
        return t("notes.pageHistoryReason.copyHistoryBlocks");
      case "apply_page_template":
        return t("notes.pageHistoryReason.applyPageTemplate");
      default:
        return t("notes.pageHistoryReason.unknown");
    }
  }

  function openSnapshot(snapshotId: string): void {
    selectedSnapshotId = snapshotId;
    void notes.loadPageHistoryVersion(snapshotId);
  }

  function toggleOpen(): void {
    open = !open;
    if (open) void notes.reloadPageHistory();
  }
</script>

<div class="mt-2">
  <button
    class="inline-flex items-center gap-1.5 rounded-md px-1.5 py-1 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground"
    type="button"
    aria-expanded={open}
    onclick={toggleOpen}
  >
    <History class="size-3.5" />
    <span>
      {#if notes.pageHistorySnapshotsLoading}
        {t("notes.loadingPageHistory")}
      {:else}
        {t("notes.pageHistoryCount", notes.pageHistorySnapshots.length)}
      {/if}
    </span>
    <ChevronDown class={`size-3.5 transition-transform ${open ? "rotate-180" : ""}`} />
  </button>

  {#if open}
    <section class="mt-2 max-w-4xl rounded-md border border-border bg-muted/25 p-2">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div class="text-[0.8rem] font-medium text-foreground">{t("notes.pageHistory")}</div>
        <label class="flex items-center gap-1.5 text-[0.733333rem] text-muted-foreground">
          <span>{t("notes.pageHistoryRetention")}</span>
          <select
            class="h-7 rounded-md border border-input bg-background px-2 text-[0.733333rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
            value={retentionValue}
            disabled={notes.pageHistorySettingsLoading}
            aria-label={t("notes.pageHistoryRetention")}
            onchange={(event) => {
              void notes.updatePageHistoryRetention(retentionValueToDays(event.currentTarget.value));
            }}
          >
            <option value="7">{t("notes.pageHistoryRetention7")}</option>
            <option value="30">{t("notes.pageHistoryRetention30")}</option>
            <option value="90">{t("notes.pageHistoryRetention90")}</option>
            <option value="365">{t("notes.pageHistoryRetention365")}</option>
            <option value="forever">{t("notes.pageHistoryRetentionForever")}</option>
          </select>
        </label>
      </div>

      {#if notes.pageHistorySnapshotsError}
        <div class="mt-2 rounded-md border border-destructive/35 bg-destructive/10 px-2 py-1.5 text-[0.8rem] text-destructive">
          {t("notes.loadPageHistoryFailed", notes.pageHistorySnapshotsError)}
        </div>
      {/if}
      {#if notes.pageHistorySettingsError}
        <div class="mt-2 rounded-md border border-destructive/35 bg-destructive/10 px-2 py-1.5 text-[0.8rem] text-destructive">
          {t("notes.updatePageHistoryRetentionFailed", notes.pageHistorySettingsError)}
        </div>
      {/if}
      {#if notes.pageHistoryActionError}
        <div class="mt-2 rounded-md border border-destructive/35 bg-destructive/10 px-2 py-1.5 text-[0.8rem] text-destructive">
          {t("notes.pageHistoryActionFailed", notes.pageHistoryActionError)}
        </div>
      {/if}

      <div class="mt-2 grid min-w-0 gap-2 lg:grid-cols-[minmax(0,14rem)_minmax(0,1fr)]">
        <div class="max-h-60 min-w-0 overflow-auto rounded-md border border-border bg-background/70">
          {#if notes.pageHistorySnapshotsLoading}
            <div class="px-2 py-1.5 text-[0.8rem] text-muted-foreground">
              {t("notes.loadingPageHistory")}
            </div>
          {:else if notes.pageHistorySnapshots.length === 0}
            <div class="px-2 py-1.5 text-[0.8rem] text-muted-foreground">
              {t("notes.noPageHistory")}
            </div>
          {:else}
            {#each notes.pageHistorySnapshots as snapshot (snapshot.id)}
              <button
                class={`block w-full border-b border-border/70 px-2 py-1.5 text-left last:border-b-0 hover:bg-accent ${
                  selectedSnapshotId === snapshot.id ? "bg-accent text-accent-foreground" : "text-foreground"
                }`}
                type="button"
                aria-label={t("notes.pageHistoryVersionLabel", formatTime(snapshot.created_time))}
                onclick={() => openSnapshot(snapshot.id)}
              >
                <span class="block truncate text-[0.8rem] font-medium">
                  {formatTime(snapshot.created_time)}
                </span>
                <span class="block truncate text-[0.7rem] text-muted-foreground">
                  {reasonLabel(snapshot.reason)}, {t("notes.pageHistoryBlockCount", snapshot.block_count)}
                </span>
              </button>
            {/each}
          {/if}
        </div>

        <div class="min-w-0">
          {#if notes.pageHistoryVersionLoading}
            <div class="text-[0.8rem] text-muted-foreground">{t("notes.loadingPageHistoryVersion")}</div>
          {:else if notes.pageHistoryVersionError}
            <div class="rounded-md border border-destructive/35 bg-destructive/10 px-2 py-1.5 text-[0.8rem] text-destructive">
              {t("notes.loadPageHistoryVersionFailed", notes.pageHistoryVersionError)}
            </div>
          {:else if selectedSnapshot && notes.pageHistoryVersion}
            <div class="flex flex-wrap items-start justify-between gap-2">
              <div class="min-w-0">
                <div class="truncate text-[0.866667rem] font-medium text-foreground">
                  {versionTitle}
                </div>
                <div class="text-[0.733333rem] text-muted-foreground">
                  {t("notes.pageHistoryVersionLabel", formatTime(selectedSnapshot.created_time))}
                </div>
              </div>
              <div class="flex flex-wrap items-center gap-1.5">
                <button
                  class="inline-flex items-center gap-1 rounded-md px-2 py-1 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-50"
                  type="button"
                  disabled={notes.pageHistoryActionLoading || selectedSnapshot.block_count === 0}
                  onclick={() => {
                    void notes.copyPageHistoryBlocks(selectedSnapshot.id);
                  }}
                >
                  <Copy class="size-3.5" />
                  <span>{t("notes.copyPageHistoryBlocks")}</span>
                </button>
                <button
                  class="inline-flex items-center gap-1 rounded-md px-2 py-1 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-50"
                  type="button"
                  disabled={notes.pageHistoryActionLoading}
                  onclick={() => {
                    restoreSnapshotId = selectedSnapshot.id;
                  }}
                >
                  <RotateCcw class="size-3.5" />
                  <span>{t("notes.restorePageHistoryVersion")}</span>
                </button>
              </div>
            </div>

            <div class="mt-2 max-h-64 min-w-0 overflow-auto rounded-md border border-border bg-background/70 px-2 py-1">
              {#if versionItems.length === 0}
                <div class="py-1 text-[0.8rem] text-muted-foreground">
                  {t("notes.pageHistoryPreviewEmpty")}
                </div>
              {:else}
                {#each versionItems as item (item.block.id)}
                  <div
                    class="min-w-0 border-b border-border/60 py-1 text-[0.8rem] last:border-b-0"
                    style={`padding-left: ${Math.min(item.depth, 4) * 0.75}rem`}
                  >
                    <div class="truncate text-[0.7rem] text-muted-foreground">
                      {blockTypeLabel(item.block.type)}
                    </div>
                    <div class="whitespace-pre-wrap wrap-break-word text-foreground">
                      {blockPlainText(item.block).trim() || t("notes.pageHistoryBlockFallback")}
                    </div>
                  </div>
                {/each}
                {#if notes.pageHistoryVersion.blocks.results.length > versionItems.length}
                  <div class="py-1 text-[0.733333rem] text-muted-foreground">
                    {t(
                      "notes.pageHistoryPreviewMore",
                      notes.pageHistoryVersion.blocks.results.length - versionItems.length,
                    )}
                  </div>
                {/if}
              {/if}
            </div>
          {:else}
            <div class="text-[0.8rem] text-muted-foreground">{t("notes.pageHistoryNoVersion")}</div>
          {/if}
        </div>
      </div>
    </section>
  {/if}
</div>

{#if restoreSnapshotId && restoreSnapshot}
  <ConfirmDialog
    title={t("notes.restorePageHistoryConfirmTitle")}
    message={t("notes.restorePageHistoryConfirmMessage", formatTime(restoreSnapshot.created_time))}
    confirmLabel={t("notes.restorePageHistoryConfirm")}
    cancelLabel={t("common.cancel")}
    onConfirm={() => {
      const snapshotId = restoreSnapshotId;
      restoreSnapshotId = null;
      if (snapshotId) void notes.restorePageHistoryVersion(snapshotId);
    }}
    onCancel={() => {
      restoreSnapshotId = null;
    }}
  />
{/if}
