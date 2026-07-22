<script lang="ts">
  import Copy from "@lucide/svelte/icons/copy";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Eye from "@lucide/svelte/icons/eye";
  import PanelLeftClose from "@lucide/svelte/icons/panel-left-close";
  import PanelLeftOpen from "@lucide/svelte/icons/panel-left-open";
  import Paperclip from "@lucide/svelte/icons/paperclip";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Search from "@lucide/svelte/icons/search";
  import TextSelect from "@lucide/svelte/icons/text-select";
  import * as chatApi from "$lib/api/chat";
  import type {
    ChatWorkspaceFileEntry,
    ChatWorkspaceFilePreview,
    ChatWorkspacePathRead,
  } from "$lib/chat/contracts";
  import { flattenChatFileTree, type ChatFileTreeRow } from "$lib/chat/file-tree-model";
  import { boundTerminalContext } from "$lib/chat/terminal-model";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ChatCodePreview from "./ChatCodePreview.svelte";
  import ChatFileIcon from "./ChatFileIcon.svelte";
  import ChatWorkspaceFileTree from "./ChatWorkspaceFileTree.svelte";

  let {
    directoryPath,
    selectedPath,
    treeVisible,
    onStateChange,
  }: {
    directoryPath: string;
    selectedPath: string | null;
    treeVisible: boolean;
    onStateChange: (change: {
      directoryPath?: string;
      selectedPath?: string | null;
      treeVisible?: boolean;
    }) => void;
  } = $props();

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let rootEntries = $state<ChatWorkspaceFileEntry[]>([]);
  let childrenByDirectory = $state<Record<string, ChatWorkspaceFileEntry[]>>({});
  let expandedPaths = $state<string[]>([]);
  let loadingPaths = $state<string[]>([]);
  let preview = $state<ChatWorkspaceFilePreview | null>(null);
  let previewElement: HTMLElement | undefined = $state();
  let query = $state("");
  let searchResults = $state<ChatWorkspacePathRead[]>([]);
  let includeIgnored = $state(false);
  let loadingRoot = $state(false);
  let loadingPreview = $state(false);
  let error = $state<string | null>(null);
  let loadedScope = "";
  let treeRequestId = 0;
  let previewRequestId = 0;
  const workspaceId = $derived(chat.selectedWorkspaceId);
  const treeRows = $derived(flattenChatFileTree(rootEntries, childrenByDirectory, expandedPaths));
  const shownRows = $derived.by<ChatFileTreeRow[]>(() => {
    if (!query.trim()) return treeRows;
    return searchResults.map((entry) => ({
      entry: { ...entry, byteSize: null },
      depth: 0,
      expanded: entry.kind === "directory" && expandedPaths.includes(entry.relativePath),
    }));
  });
  const changedPaths = $derived(new Set(
    chat.timelinePages.flatMap((page) => page.turns.flatMap((turn) => turn.changedFiles.map((file) => file.relativePath))),
  ));

  $effect(() => {
    const workspace = workspaceId;
    const scope = `${workspace ?? ""}:${chat.selectedThreadId ?? ""}`;
    if (!workspace || scope === loadedScope) return;
    loadedScope = scope;
    rootEntries = [];
    childrenByDirectory = {};
    expandedPaths = [];
    preview = null;
    void loadRoot(workspace, directoryPath);
    if (selectedPath) void selectFile(selectedPath);
  });

  $effect(() => {
    const workspace = workspaceId;
    const value = query.trim();
    const showIgnored = includeIgnored;
    if (!workspace || !value) {
      searchResults = [];
      return;
    }
    let cancelled = false;
    const timer = window.setTimeout(() => {
      void chatApi.searchChatWorkspacePaths(workspace, value, showIgnored, null, 100)
        .then((page) => { if (!cancelled) searchResults = page.entries; })
        .catch((reason: unknown) => { if (!cancelled) error = message(reason); });
    }, 160);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  });

  async function loadRoot(workspace: string, pathToReveal = ""): Promise<void> {
    const requestId = ++treeRequestId;
    loadingRoot = true;
    error = null;
    try {
      const result = await chatApi.listChatWorkspaceDirectory(workspace, "", includeIgnored);
      if (requestId !== treeRequestId || workspace !== workspaceId) return;
      rootEntries = result.entries;
      if (pathToReveal) await revealDirectory(pathToReveal);
    } catch (reason: unknown) {
      if (requestId === treeRequestId) error = message(reason);
    } finally {
      if (requestId === treeRequestId) loadingRoot = false;
    }
  }

  async function loadDirectory(path: string): Promise<void> {
    const workspace = workspaceId;
    if (!workspace || childrenByDirectory[path] || loadingPaths.includes(path)) return;
    const requestId = treeRequestId;
    loadingPaths = [...loadingPaths, path];
    try {
      const result = await chatApi.listChatWorkspaceDirectory(workspace, path, includeIgnored);
      if (requestId !== treeRequestId || workspace !== workspaceId) return;
      childrenByDirectory = { ...childrenByDirectory, [path]: result.entries };
    } catch (reason: unknown) {
      if (requestId === treeRequestId) error = message(reason);
    } finally {
      if (requestId === treeRequestId) {
        loadingPaths = loadingPaths.filter((entry) => entry !== path);
      }
    }
  }

  async function revealDirectory(path: string): Promise<void> {
    const parts = path.split("/").filter(Boolean);
    let current = "";
    for (const part of parts) {
      current = current ? `${current}/${part}` : part;
      if (!expandedPaths.includes(current)) expandedPaths = [...expandedPaths, current];
      await loadDirectory(current);
    }
  }

  function toggleDirectory(entry: ChatWorkspaceFileEntry): void {
    if (entry.kind !== "directory") return;
    if (query.trim()) {
      query = "";
      void revealDirectory(entry.relativePath);
      return;
    }
    if (expandedPaths.includes(entry.relativePath)) {
      expandedPaths = expandedPaths.filter((path) => path !== entry.relativePath);
      return;
    }
    expandedPaths = [...expandedPaths, entry.relativePath];
    onStateChange({ directoryPath: entry.relativePath });
    void loadDirectory(entry.relativePath);
  }

  async function selectFile(path: string): Promise<void> {
    const workspace = workspaceId;
    if (!workspace) return;
    const requestId = ++previewRequestId;
    onStateChange({ selectedPath: path });
    preview = null;
    loadingPreview = true;
    error = null;
    try {
      const result = await chatApi.previewChatWorkspaceFile(workspace, path);
      if (requestId === previewRequestId && workspace === workspaceId) preview = result;
    } catch (reason: unknown) {
      if (requestId === previewRequestId) error = message(reason);
    } finally {
      if (requestId === previewRequestId) loadingPreview = false;
    }
  }

  function openEntry(entry: ChatWorkspaceFileEntry): void {
    if (entry.kind === "directory") toggleDirectory(entry);
    else void selectFile(entry.relativePath);
  }

  function reloadTree(): void {
    const workspace = workspaceId;
    if (!workspace) return;
    childrenByDirectory = {};
    expandedPaths = [];
    void loadRoot(workspace);
  }

  function toggleIgnored(): void {
    includeIgnored = !includeIgnored;
    reloadTree();
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

<div class="files-panel">
  <div class="files-layout" class:tree-hidden={!treeVisible}>
    {#if treeVisible}
      <aside class="file-tree-pane" aria-label={t("chat.inspector.files")}>
        <div class="tree-toolbar">
          <label class="file-search"><Search size={13} /><input type="search" bind:value={query} placeholder={t("chat.inspector.searchFiles")} aria-label={t("chat.inspector.searchFiles")} /></label>
          <button type="button" class="tree-action" class:active={includeIgnored} aria-pressed={includeIgnored} title={t("chat.inspector.showIgnored")} aria-label={t("chat.inspector.showIgnored")} onclick={toggleIgnored}><Eye size={13} /></button>
          <button type="button" class="tree-action" title={t("chat.inspector.refreshFiles")} aria-label={t("chat.inspector.refreshFiles")} onclick={reloadTree}><RefreshCw size={13} /></button>
          <button type="button" class="tree-action" title={t("chat.inspector.hideFileTree")} aria-label={t("chat.inspector.hideFileTree")} onclick={() => onStateChange({ treeVisible: false })}><PanelLeftClose size={13} /></button>
        </div>
        {#if error}<p role="alert" class="border-b border-destructive/30 p-2 text-xs text-destructive">{error}</p>{/if}
        {#if loadingRoot}
          <p class="p-2 text-xs text-muted-foreground">{t("common.loading")}</p>
        {:else if shownRows.length > 0}
          <ChatWorkspaceFileTree
            rows={shownRows}
            {selectedPath}
            {changedPaths}
            {loadingPaths}
            onToggle={toggleDirectory}
            onSelect={openEntry}
          />
        {:else}
          <p class="p-2 text-xs text-muted-foreground">{query.trim() ? t("chat.inspector.noFileResults") : t("chat.inspector.emptyDirectory")}</p>
        {/if}
      </aside>
    {/if}

    <section bind:this={previewElement} class="file-editor" aria-label={t("chat.inspector.filePreview")}>
      <header class="editor-heading">
        {#if !treeVisible}<button type="button" class="chat-icon-button" title={t("chat.inspector.showFileTree")} aria-label={t("chat.inspector.showFileTree")} onclick={() => onStateChange({ treeVisible: true })}><PanelLeftOpen size={13} /></button>{/if}
        {#if preview}
          <ChatFileIcon path={preview.relativePath} />
          <strong class="min-w-0 flex-1 truncate text-[0.733333rem] font-medium" title={preview.relativePath}>{preview.relativePath}</strong>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.copyPath")} aria-label={t("chat.inspector.copyPath")} onclick={() => navigator.clipboard.writeText(selectedPath ?? "")}><Copy size={13} /></button>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.attachFile")} aria-label={t("chat.inspector.attachFile")} onclick={() => selectedPath && attachFileReference(selectedPath)}><Paperclip size={13} /></button>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.attachSelection")} aria-label={t("chat.inspector.attachSelection")} onclick={() => { void attachSelection().catch((reason) => { error = message(reason); }); }}><TextSelect size={13} /></button>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.openExternally")} aria-label={t("chat.inspector.openExternally")} onclick={() => workspaceId && selectedPath && chatApi.openChatWorkspaceFile(workspaceId, selectedPath)}><ExternalLink size={13} /></button>
        {:else}
          <span class="min-w-0 flex-1 truncate text-[0.733333rem] text-muted-foreground">{t("chat.inspector.filePreview")}</span>
        {/if}
      </header>
      {#if error && !treeVisible}<p role="alert" class="border-b border-destructive/30 p-2 text-xs text-destructive">{error}</p>{/if}
      {#if loadingPreview}
        <p class="m-auto text-xs text-muted-foreground">{t("common.loading")}</p>
      {:else if preview && preview.text !== null}
        <ChatCodePreview text={preview.text} language={preview.language} />
      {:else if preview}
        <div class="m-auto p-4 text-center text-xs text-muted-foreground">
          <p>{preview.binary ? t("chat.inspector.binary") : preview.oversized ? t("chat.inspector.previewOversized") : t("chat.inspector.previewUnavailable")}</p>
          <p>{t("chat.inspector.fileSizeBytes", formatNumber(localization.locale, preview.byteSize))}</p>
        </div>
      {:else}
        <p class="m-auto p-4 text-xs text-muted-foreground">{t("chat.inspector.selectFile")}</p>
      {/if}
    </section>
  </div>
</div>

<style>
  .files-panel { container: files-panel / inline-size; display: flex; height: 100%; min-height: 0; flex-direction: column; }
  .files-layout { display: grid; min-height: 0; flex: 1; grid-template-columns: clamp(9rem, 38%, 16rem) minmax(0, 1fr); }
  .files-layout.tree-hidden { grid-template-columns: minmax(0, 1fr); }
  .file-tree-pane { display: flex; min-width: 0; min-height: 0; flex-direction: column; overflow: hidden; border-right: 1px solid var(--border); background: color-mix(in srgb, var(--cal-bg) 96%, var(--muted)); }
  .tree-toolbar { display: flex; min-height: 2.45rem; flex: 0 0 auto; align-items: center; gap: 0.15rem; border-bottom: 1px solid var(--border); padding: 0.3rem; }
  .file-search { display: flex; min-width: 0; min-height: 1.75rem; flex: 1; align-items: center; gap: 0.35rem; border-radius: 0.4rem; padding-inline: 0.4rem; color: var(--muted-foreground); }
  .file-search:focus-within { background: var(--background); box-shadow: inset 0 0 0 1px var(--ring); color: var(--foreground); }
  .file-search input { min-width: 0; flex: 1; background: transparent; color: var(--foreground); font-size: 0.7rem; outline: none; }
  .tree-action { display: inline-grid; width: 1.7rem; height: 1.7rem; flex: 0 0 auto; place-items: center; border-radius: 0.35rem; color: var(--muted-foreground); }
  .tree-action:hover, .tree-action.active { background: var(--accent); color: var(--foreground); }
  .file-editor { display: flex; min-width: 0; min-height: 0; flex-direction: column; overflow: hidden; }
  .editor-heading { display: flex; min-height: 2.45rem; flex: 0 0 auto; align-items: center; gap: 0.25rem; border-bottom: 1px solid var(--border); padding: 0.3rem 0.4rem; }
  @container files-panel (max-width: 360px) { .files-layout { grid-template-columns: clamp(8rem, 42%, 10rem) minmax(0, 1fr); } }
</style>
