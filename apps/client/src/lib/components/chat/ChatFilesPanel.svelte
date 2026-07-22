<script lang="ts">
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import File from "@lucide/svelte/icons/file";
  import Folder from "@lucide/svelte/icons/folder";
  import Paperclip from "@lucide/svelte/icons/paperclip";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Copy from "@lucide/svelte/icons/copy";
  import * as chatApi from "$lib/api/chat";
  import type { ChatWorkspaceFileEntry, ChatWorkspaceFilePreview, ChatWorkspacePathRead } from "$lib/chat/contracts";
  import { boundTerminalContext } from "$lib/chat/terminal-model";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";

  let {
    directoryPath,
    selectedPath,
    onStateChange,
  }: {
    directoryPath: string;
    selectedPath: string | null;
    onStateChange: (change: { directoryPath?: string; selectedPath?: string | null }) => void;
  } = $props();

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let entries = $state<ChatWorkspaceFileEntry[]>([]);
  let preview = $state<ChatWorkspaceFilePreview | null>(null);
  let previewElement: HTMLElement | undefined = $state();
  let query = $state("");
  let searchResults = $state<ChatWorkspacePathRead[]>([]);
  let includeIgnored = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let loadedScope = "";
  const workspaceId = $derived(chat.selectedWorkspaceId);
  const changedPaths = $derived(new Set(
    chat.timelinePages.flatMap((page) => page.turns.flatMap((turn) => turn.changedFiles.map((file) => file.relativePath))),
  ));

  $effect(() => {
    const workspace = workspaceId;
    const scope = `${workspace ?? ""}:${chat.selectedThreadId ?? ""}`;
    if (!workspace || scope === loadedScope) return;
    loadedScope = scope;
    preview = null;
    void loadDirectory(directoryPath);
    if (selectedPath) void selectFile(selectedPath);
  });

  $effect(() => {
    const workspace = workspaceId;
    const value = query.trim();
    if (!workspace || !value) {
      searchResults = [];
      return;
    }
    let cancelled = false;
    const timer = window.setTimeout(() => {
      void chatApi.searchChatWorkspacePaths(workspace, value, includeIgnored, null, 100)
        .then((page) => { if (!cancelled) searchResults = page.entries; })
        .catch((reason: unknown) => { if (!cancelled) error = message(reason); });
    }, 120);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  });

  async function loadDirectory(path: string): Promise<void> {
    if (!workspaceId) return;
    loading = true;
    error = null;
    try {
      const result = await chatApi.listChatWorkspaceDirectory(workspaceId, path, includeIgnored);
      onStateChange({ directoryPath: result.relativePath });
      entries = result.entries;
    } catch (reason: unknown) {
      error = message(reason);
    } finally {
      loading = false;
    }
  }

  async function selectFile(path: string): Promise<void> {
    if (!workspaceId) return;
    onStateChange({ selectedPath: path });
    preview = null;
    error = null;
    try {
      preview = await chatApi.previewChatWorkspaceFile(workspaceId, path);
    } catch (reason: unknown) {
      error = message(reason);
    }
  }

  function openEntry(entry: ChatWorkspaceFileEntry | ChatWorkspacePathRead): void {
    if (entry.kind === "directory") {
      query = "";
      void loadDirectory(entry.relativePath);
    } else {
      void selectFile(entry.relativePath);
    }
  }

  function parentPath(): string {
    return directoryPath.split("/").slice(0, -1).join("/");
  }

  function attachFileReference(path: string): void {
    const mentions = chat.composer.mentions.filter((entry) => entry.relativePath !== path);
    chat.setComposerMentions([...mentions, { relativePath: path, kind: "file", ignored: false }]);
  }

  async function attachSelection(): Promise<void> {
    if (!workspaceId || !selectedPath) return;
    const browserSelection = window.getSelection();
    if (!browserSelection || !previewElement?.contains(browserSelection.anchorNode)) return;
    const selection = browserSelection.toString();
    const bounded = boundTerminalContext(selection, 128 * 1024);
    if (!bounded.text) return;
    const attachment = await chatApi.importChatTextSnippet(
      workspaceId,
      crypto.randomUUID(),
      `${selectedPath} selection.txt`,
      bounded.text,
    );
    chat.setComposerAttachments([...chat.composer.attachmentIds, attachment.id]);
  }

  function message(reason: unknown): string {
    return reason instanceof Error ? reason.message : String(reason);
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="space-y-2 border-b border-border p-2">
    <input class="chat-field h-8 w-full" type="search" bind:value={query} placeholder={t("chat.inspector.searchFiles")} aria-label={t("chat.inspector.searchFiles")} />
    <label class="flex items-center gap-2 text-[0.666667rem] text-muted-foreground">
      <input type="checkbox" bind:checked={includeIgnored} onchange={() => { void loadDirectory(directoryPath); }} />
      {t("chat.inspector.showIgnored")}
    </label>
  </div>

  <div class="grid min-h-0 flex-1 grid-rows-[minmax(8rem,0.8fr)_minmax(10rem,1.2fr)]">
    <div class="min-h-0 overflow-auto border-b border-border p-2">
      {#if directoryPath}
        <button type="button" class="chat-file-row" onclick={() => { void loadDirectory(parentPath()); }}><ArrowLeft size={13} /><span class="truncate">..</span></button>
      {/if}
      {#if error}<p role="alert" class="p-2 text-xs text-destructive">{error}</p>{/if}
      {#if loading}
        <p class="p-2 text-xs text-muted-foreground">{t("common.loading")}</p>
      {:else}
        {@const shown = query.trim() ? searchResults : entries}
        {#each shown as entry (entry.relativePath)}
          <button type="button" class="chat-file-row" class:selected={selectedPath === entry.relativePath} onclick={() => openEntry(entry)} title={entry.relativePath}>
            {#if entry.kind === "directory"}<Folder size={13} />{:else}<File size={13} />{/if}
            <span class="min-w-0 flex-1 truncate text-left">{entry.displayName}</span>
            {#if changedPaths.has(entry.relativePath)}<span class="rounded bg-primary/10 px-1 text-[0.583333rem] text-primary">{t("chat.inspector.changed")}</span>{/if}
            {#if entry.ignored}<span class="text-[0.583333rem] text-muted-foreground">{t("chat.composer.ignored")}</span>{/if}
          </button>
        {:else}
          <p class="p-2 text-xs text-muted-foreground">{t("chat.inspector.emptyDirectory")}</p>
        {/each}
      {/if}
    </div>

    <section bind:this={previewElement} class="flex min-h-0 flex-col" aria-label={t("chat.inspector.filePreview")}>
      {#if preview}
        <header class="flex items-center gap-1 border-b border-border p-2">
          <strong class="min-w-0 flex-1 truncate text-xs" title={preview.relativePath}>{preview.relativePath}</strong>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.copyPath")} onclick={() => navigator.clipboard.writeText(selectedPath ?? "")}><Copy size={13} /></button>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.attachFile")} onclick={() => selectedPath && attachFileReference(selectedPath)}><Paperclip size={13} /></button>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.openExternally")} onclick={() => workspaceId && selectedPath && chatApi.openChatWorkspaceFile(workspaceId, selectedPath)}><ExternalLink size={13} /></button>
        </header>
        {#if preview.text !== null}
          <pre class="min-h-0 flex-1 select-text overflow-auto p-3 font-mono text-[0.666667rem] leading-5">{#each preview.text.split("\n") as line, index}<span class="block"><span class="mr-3 inline-block w-8 select-none text-right text-muted-foreground">{index + 1}</span>{line}</span>{/each}</pre>
          <button type="button" class="chat-secondary-button m-2 self-start" onclick={() => { void attachSelection().catch((reason) => { error = message(reason); }); }}>{t("chat.inspector.attachSelection")}</button>
        {:else}
          <div class="m-auto p-4 text-center text-xs text-muted-foreground">
            <p>{preview.binary ? t("chat.inspector.binary") : t("chat.inspector.previewUnavailable")}</p>
            <p>{t("chat.inspector.fileSizeBytes", formatNumber(localization.locale, preview.byteSize))}</p>
          </div>
        {/if}
      {:else}
        <p class="m-auto p-4 text-xs text-muted-foreground">{t("chat.inspector.filePreview")}</p>
      {/if}
    </section>
  </div>
</div>

<style>
  .chat-file-row { display: flex; width: 100%; align-items: center; gap: 0.375rem; border-radius: 0.25rem; padding: 0.3rem 0.4rem; font-size: 0.75rem; }
  .chat-file-row:hover, .chat-file-row.selected { background: var(--accent); }
</style>
