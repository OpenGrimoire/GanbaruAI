<script lang="ts">
  import { onMount, tick } from "svelte";
  import AlertTriangle from "@lucide/svelte/icons/triangle-alert";
  import Copy from "@lucide/svelte/icons/copy";
  import Columns2 from "@lucide/svelte/icons/columns-2";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import GitCommitHorizontal from "@lucide/svelte/icons/git-commit-horizontal";
  import History from "@lucide/svelte/icons/history";
  import Paperclip from "@lucide/svelte/icons/paperclip";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import Rows3 from "@lucide/svelte/icons/rows-3";
  import Space from "@lucide/svelte/icons/space";
  import * as chatApi from "$lib/api/chat";
  import type {
    ChatChangeScope,
    ChatCheckpointDiffRead,
    ChatCheckpointFileDiffRead,
    ChatChangedFileRead,
  } from "$lib/chat/contracts";
  import { splitDiffFits, splitPaneResizeBounds } from "$lib/chat/inspector-model";
  import { alignPanelSizeToDevicePixel } from "$lib/chat/responsive-layout";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ChatChangedFileTree from "./ChatChangedFileTree.svelte";
  import ChatFileIcon from "./ChatFileIcon.svelte";
  import ChatPaneResizeHandle from "./ChatPaneResizeHandle.svelte";

  let {
    scope,
    selectedFile,
    fileListHeightPx,
    whitespaceIgnored,
    diffView,
    onStateChange,
  }: {
    scope: ChatChangeScope;
    selectedFile: string | null;
    fileListHeightPx: number;
    whitespaceIgnored: boolean;
    diffView: "auto" | "unified" | "split";
    onStateChange: (update: {
      scope?: ChatChangeScope;
      selectedFile?: string | null;
      fileListHeightPx?: number;
      whitespaceIgnored?: boolean;
      diffView?: "auto" | "unified" | "split";
    }) => void;
  } = $props();

  const DEFAULT_FILE_LIST_HEIGHT = 160;
  const MIN_FILE_LIST_HEIGHT = 112;
  const MIN_DIFF_HEIGHT = 64;
  const MAX_FILE_LIST_HEIGHT = 420;

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let diff = $state<ChatCheckpointDiffRead | null>(null);
  let fileDiff = $state<ChatCheckpointFileDiffRead | null>(null);
  let renderedLines = $state<{ kind: "context" | "addition" | "deletion" | "header"; text: string; oldLine: number | null; newLine: number | null }[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let restoring = $state(false);
  let diffHost: HTMLElement | undefined = $state();
  let splitElement: HTMLDivElement | undefined = $state();
  let diffScroller: HTMLElement | undefined = $state();
  let diffWidth = $state(0);
  let splitHeight = $state(0);
  let renderedFileListHeight = $state(DEFAULT_FILE_LIST_HEIGHT);
  let resizingFileList = $state(false);
  let fileListResizeFrame: number | null = null;
  let fileListResizeEndFrame: number | null = null;
  const threadId = $derived(chat.selectedThreadId);
  const workspaceId = $derived(chat.selectedWorkspaceId);
  const splitView = $derived(diffView === "split" || (diffView === "auto" && splitDiffFits(diffWidth)));

  onMount(() => {
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        if (entry.target === diffHost) diffWidth = entry.contentRect.width;
        if (entry.target === splitElement) splitHeight = entry.contentRect.height;
      }
    });
    if (diffHost) observer.observe(diffHost);
    if (splitElement) observer.observe(splitElement);
    return () => {
      observer.disconnect();
      if (fileListResizeFrame !== null) window.cancelAnimationFrame(fileListResizeFrame);
      if (fileListResizeEndFrame !== null) window.cancelAnimationFrame(fileListResizeEndFrame);
    };
  });

  $effect(() => {
    const configuredHeight = fileListHeightPx;
    const availableHeight = splitHeight;
    if (!resizingFileList) {
      renderedFileListHeight = availableHeight > 0
        ? alignFileListHeight(configuredHeight)
        : configuredHeight;
    }
  });

  $effect(() => {
    const thread = threadId;
    const currentScope = scope;
    const revision = chat.selectedThread?.revision;
    if (!thread || revision === undefined) return;
    void loadDiff(thread, currentScope);
  });

  $effect(() => {
    const thread = threadId;
    const path = selectedFile;
    const ignored = whitespaceIgnored;
    const pre = diff?.preCheckpointId;
    const post = diff?.postCheckpointId;
    if (!thread || !path || !pre || !post) {
      fileDiff = null;
      renderedLines = [];
      return;
    }
    void loadFileDiff(thread, pre, post, path, ignored);
  });

  async function loadDiff(thread: string, currentScope: ChatChangeScope): Promise<void> {
    loading = true;
    error = null;
    try {
      const result = await chatApi.readChatCheckpointDiff(thread, currentScope);
      if (thread !== threadId || currentScope !== scope) return;
      diff = result;
      const retained = selectedFile && result.files.some((file) => file.relativePath === selectedFile)
        ? selectedFile
        : result.files[0]?.relativePath ?? null;
      if (retained !== selectedFile) onStateChange({ selectedFile: retained });
    } catch (reason: unknown) {
      error = message(reason);
    } finally {
      loading = false;
    }
  }

  async function loadFileDiff(
    thread: string,
    pre: string,
    post: string,
    path: string,
    ignored: boolean,
  ): Promise<void> {
    const previousScrollTop = diffScroller?.scrollTop ?? 0;
    fileDiff = null;
    renderedLines = [];
    try {
      const result = await chatApi.readChatCheckpointFileDiff(thread, pre, post, path, ignored);
      if (path !== selectedFile || ignored !== whitespaceIgnored) return;
      fileDiff = result;
      renderedLines = result.patch ? await parsePatchLines(result.patch) : [];
      await tick();
      if (path === selectedFile && diffScroller) diffScroller.scrollTop = previousScrollTop;
    } catch (reason: unknown) {
      error = message(reason);
    }
  }

  async function parsePatchLines(patch: string): Promise<typeof renderedLines> {
    const { parsePatch } = await import("diff");
    const parsed = parsePatch(patch);
    const lines: typeof renderedLines = [];
    for (const file of parsed) {
      lines.push({ kind: "header", text: file.newFileName || file.oldFileName || "", oldLine: null, newLine: null });
      for (const hunk of file.hunks) {
        lines.push({ kind: "header", text: `@@ -${hunk.oldStart},${hunk.oldLines} +${hunk.newStart},${hunk.newLines} @@`, oldLine: null, newLine: null });
        let oldLine = hunk.oldStart;
        let newLine = hunk.newStart;
        for (const line of hunk.lines) {
          const marker = line[0];
          lines.push({
            kind: marker === "+" ? "addition" : marker === "-" ? "deletion" : "context",
            text: line,
            oldLine: marker === "+" ? null : oldLine,
            newLine: marker === "-" ? null : newLine,
          });
          if (marker !== "+") oldLine += 1;
          if (marker !== "-") newLine += 1;
        }
      }
    }
    return lines.slice(0, 12_000);
  }

  function select(file: ChatChangedFileRead): void {
    onStateChange({ selectedFile: file.relativePath });
  }

  function attach(path: string): void {
    const mentions = chat.composer.mentions.filter((entry) => entry.relativePath !== path);
    chat.setComposerMentions([...mentions, { relativePath: path, kind: "file", ignored: false }]);
  }

  function fileListResizeBounds(): { minimum: number; maximum: number } {
    return splitPaneResizeBounds(
      splitHeight,
      MIN_FILE_LIST_HEIGHT,
      MIN_DIFF_HEIGHT,
      MAX_FILE_LIST_HEIGHT,
    );
  }

  function alignFileListHeight(value: number): number {
    const bounds = fileListResizeBounds();
    const anchor = splitElement?.getBoundingClientRect().top ?? 0;
    return alignPanelSizeToDevicePixel({
      value,
      minimum: bounds.minimum,
      maximum: bounds.maximum,
      anchor,
      direction: "from-start",
      devicePixelRatio: window.devicePixelRatio,
    });
  }

  function beginFileListResize(event: PointerEvent): void {
    event.preventDefault();
    if (splitElement) splitHeight = splitElement.getBoundingClientRect().height;
    if (fileListResizeEndFrame !== null) window.cancelAnimationFrame(fileListResizeEndFrame);
    fileListResizeEndFrame = null;
    resizingFileList = true;
    const startY = event.clientY;
    const startHeight = renderedFileListHeight;
    const target = event.currentTarget as HTMLElement;
    let pendingHeight = startHeight;
    target.focus();
    target.setPointerCapture(event.pointerId);
    const move = (moveEvent: PointerEvent) => {
      pendingHeight = alignFileListHeight(startHeight + moveEvent.clientY - startY);
      if (fileListResizeFrame !== null) return;
      fileListResizeFrame = window.requestAnimationFrame(() => {
        renderedFileListHeight = pendingHeight;
        fileListResizeFrame = null;
      });
    };
    const end = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      target.removeEventListener("pointercancel", end);
      if (fileListResizeFrame !== null) window.cancelAnimationFrame(fileListResizeFrame);
      fileListResizeFrame = null;
      renderedFileListHeight = pendingHeight;
      onStateChange({ fileListHeightPx: pendingHeight });
      fileListResizeEndFrame = window.requestAnimationFrame(() => {
        fileListResizeEndFrame = window.requestAnimationFrame(() => {
          resizingFileList = false;
          fileListResizeEndFrame = null;
        });
      });
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
    target.addEventListener("pointercancel", end);
  }

  function resizeFileListFromKey(event: KeyboardEvent): void {
    const bounds = fileListResizeBounds();
    let next: number;
    switch (event.key) {
      case "ArrowUp": next = renderedFileListHeight - 16; break;
      case "ArrowDown": next = renderedFileListHeight + 16; break;
      case "Home": next = bounds.minimum; break;
      case "End": next = bounds.maximum; break;
      case "Enter": next = DEFAULT_FILE_LIST_HEIGHT; break;
      default: return;
    }
    event.preventDefault();
    renderedFileListHeight = alignFileListHeight(next);
    onStateChange({ fileListHeightPx: renderedFileListHeight });
  }

  async function restoreCheckpoint(): Promise<void> {
    if (!threadId || !diff?.preCheckpointId || !chat.selectedThread) return;
    restoring = true;
    error = null;
    try {
      const preview = await chatApi.previewChatCheckpointRestore(threadId, diff.preCheckpointId);
      const affected = preview.files.map((file) => file.relativePath).join("\n");
      const confirmed = window.confirm(
        [t("chat.timeline.revert"), affected, ...preview.warnings].filter(Boolean).join("\n\n"),
      );
      if (!confirmed) return;
      await chatApi.executeChatCheckpointRestore({
        command: {
          clientCommandId: crypto.randomUUID(),
          expectedThreadRevision: chat.selectedThread.revision,
        },
        threadId,
        previewId: preview.previewId,
        confirmed: true,
      });
      await chat.handleNativeChange(threadId);
      await loadDiff(threadId, scope);
    } catch (reason: unknown) {
      error = message(reason);
    } finally {
      restoring = false;
    }
  }

  function message(reason: unknown): string {
    return reason instanceof Error ? reason.message : String(reason);
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="diff-toolbar">
    <div class="scope-control" role="group">
      <button type="button" class:active={scope === "current_turn"} class="chat-scope-button" title={t("chat.inspector.currentTurn")} onclick={() => onStateChange({ scope: "current_turn" })}><GitCommitHorizontal size={13} /><span>{t("chat.inspector.currentTurn")}</span></button>
      <button type="button" class:active={scope === "entire_thread"} class="chat-scope-button" title={t("chat.inspector.entireThread")} onclick={() => onStateChange({ scope: "entire_thread" })}><History size={13} /><span>{t("chat.inspector.entireThread")}</span></button>
    </div>
    <button type="button" class="whitespace-toggle" class:active={whitespaceIgnored} aria-pressed={whitespaceIgnored} title={t("chat.inspector.ignoreWhitespace")} onclick={() => onStateChange({ whitespaceIgnored: !whitespaceIgnored })}><Space size={14} /></button>
    {#if diff?.available}
      <span class="diff-stat"><span class="text-action-confirm">{t("chat.inspector.additions", formatNumber(localization.locale, diff.additions))}</span><span class="text-destructive">{t("chat.inspector.deletions", formatNumber(localization.locale, diff.deletions))}</span></span>
      <button type="button" class="chat-icon-button" disabled={restoring} title={t("chat.timeline.revert")} onclick={() => { void restoreCheckpoint(); }}><RotateCcw size={13} /></button>
    {/if}
  </div>

  {#if error}<p role="alert" class="border-b border-destructive/30 p-2 text-xs text-destructive">{error}</p>{/if}
  {#if diff?.providerMismatch}
    <p class="flex items-center gap-2 border-b border-status-tentative/30 bg-status-tentative/10 p-2 text-xs text-status-tentative"><AlertTriangle size={13} />{t("chat.inspector.providerMismatch")}</p>
  {/if}
  {#if chat.selectedWorkspace?.workspace.repositoryKind === "none"}
    <p class="border-b border-border p-2 text-xs text-muted-foreground">{t("chat.inspector.nonGitNotice")}</p>
  {/if}

  <div bind:this={splitElement} class="changes-layout" class:resizing={resizingFileList} style={`--changed-file-list-height:${renderedFileListHeight}px`}>
    <div class="changed-file-list min-h-0 overflow-auto p-2">
      {#if loading}
        <p class="p-2 text-xs text-muted-foreground">{t("common.loading")}</p>
      {:else if diff?.files.length}
        <ChatChangedFileTree files={diff.files} {selectedFile} onSelect={select} />
      {:else}
        <p class="p-2 text-xs text-muted-foreground">{diff?.unavailableReason ?? t("chat.inspector.noChanges")}</p>
      {/if}
    </div>

    <ChatPaneResizeHandle orientation="horizontal" value={renderedFileListHeight} minimum={fileListResizeBounds().minimum} maximum={fileListResizeBounds().maximum} label={t("chat.resizeChangedFileList")} active={resizingFileList} onPointerDown={beginFileListResize} onKeyDown={resizeFileListFromKey} />

    <section bind:this={diffHost} class="diff-pane">
      {#if selectedFile}
        <header class="diff-heading">
          <ChatFileIcon path={selectedFile} />
          <strong class="min-w-0 flex-1 truncate text-xs" title={selectedFile}>{selectedFile}</strong>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.copyPath")} onclick={() => navigator.clipboard.writeText(selectedFile ?? "")}><Copy size={13} /></button>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.attachFile")} onclick={() => selectedFile && attach(selectedFile)}><Paperclip size={13} /></button>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.openExternally")} onclick={() => workspaceId && selectedFile && chatApi.openChatWorkspaceFile(workspaceId, selectedFile)}><ExternalLink size={13} /></button>
          {#if splitDiffFits(diffWidth)}<button type="button" class="diff-view-toggle" title={splitView ? t("chat.inspector.unifiedDiff") : t("chat.inspector.splitDiff")} aria-label={splitView ? t("chat.inspector.unifiedDiff") : t("chat.inspector.splitDiff")} onclick={() => onStateChange({ diffView: splitView ? "unified" : "split" })}>{#if splitView}<Rows3 size={13} />{:else}<Columns2 size={13} />{/if}</button>{/if}
        </header>
      {/if}
      {#if fileDiff?.binary}
        <p class="m-auto p-4 text-xs text-muted-foreground">{t("chat.inspector.binary")}</p>
      {:else if fileDiff?.patch}
        <div bind:this={diffScroller} class="diff-code">
          {#if splitView}
            {#each renderedLines as line}
              {#if line.kind === "header"}
                <div class="split-header">{line.text}</div>
              {:else}
                <div class="split-line">
                  <span class="line-number">{line.oldLine ?? ""}</span><span class="split-code {line.kind === "addition" ? "empty" : line.kind}">{line.kind === "addition" ? "" : line.text.slice(1)}</span>
                  <span class="line-number">{line.newLine ?? ""}</span><span class="split-code {line.kind === "deletion" ? "empty" : line.kind}">{line.kind === "deletion" ? "" : line.text.slice(1)}</span>
                </div>
              {/if}
            {/each}
          {:else}
            {#each renderedLines as line}
              <div class="diff-line {line.kind}">
                <span class="line-number">{line.oldLine ?? ""}</span><span class="line-number">{line.newLine ?? ""}</span><span class="whitespace-pre">{line.text}</span>
              </div>
            {/each}
          {/if}
          {#if fileDiff.truncated}<p class="p-3 text-muted-foreground">{t("chat.inspector.largeDiff")}</p>{/if}
        </div>
      {:else}
        <p class="m-auto p-4 text-xs text-muted-foreground">{selectedFile ? t("common.loading") : t("chat.inspector.noChanges")}</p>
      {/if}
    </section>
  </div>
</div>

<style>
  .changes-layout { display: flex; min-height: 0; flex: 1; flex-direction: column; }
  .changes-layout.resizing, .changes-layout.resizing * { user-select: none; }
  .changed-file-list { height: var(--changed-file-list-height); flex: 0 0 var(--changed-file-list-height); }
  .diff-pane { display: flex; min-height: 0; flex: 1; flex-direction: column; }
  .diff-toolbar { display: flex; min-height: 2.7rem; flex: 0 0 auto; align-items: center; gap: 0.3rem; overflow-x: auto; border-bottom: 1px solid var(--border); padding: 0.35rem 0.4rem; }
  .scope-control { display: flex; flex: 0 0 auto; border: 1px solid var(--border); border-radius: 0.45rem; padding: 0.15rem; }
  .chat-scope-button { display: inline-flex; min-height: 1.65rem; align-items: center; gap: 0.3rem; border-radius: 0.3rem; padding: 0.2rem 0.4rem; color: var(--muted-foreground); font-size: 0.633333rem; }
  .chat-scope-button.active { background: var(--accent); color: var(--accent-foreground); }
  .whitespace-toggle, .diff-view-toggle { display: inline-grid; width: 1.8rem; height: 1.8rem; flex: 0 0 auto; place-items: center; border-radius: 0.4rem; color: var(--muted-foreground); }
  .whitespace-toggle:hover, .whitespace-toggle.active, .diff-view-toggle:hover { background: var(--accent); color: var(--foreground); }
  .diff-stat { display: flex; margin-left: auto; gap: 0.35rem; font-family: "SF Mono", "SFMono-Regular", Consolas, monospace; font-size: 0.633333rem; }
  .diff-heading { display: flex; min-height: 2.5rem; align-items: center; gap: 0.25rem; border-bottom: 1px solid var(--border); padding: 0.3rem 0.4rem 0.3rem 0.6rem; }
  .diff-code { min-height: 0; flex: 1; overflow: auto; font-family: "SF Mono", "SFMono-Regular", "JetBrains Mono", "Cascadia Code", Consolas, "Liberation Mono", Menlo, monospace; font-size: 0.7rem; line-height: 1.35rem; }
  .diff-line { display: grid; min-width: max-content; grid-template-columns: 2.75rem 2.75rem minmax(0, 1fr); padding-right: 0.75rem; }
  .diff-line.addition { background: color-mix(in srgb, var(--action-confirm) 13%, transparent); }
  .diff-line.deletion { background: color-mix(in srgb, var(--destructive) 12%, transparent); }
  .diff-line.header { background: var(--muted); color: var(--muted-foreground); }
  .line-number { user-select: none; border-right: 1px solid var(--border); padding-right: 0.35rem; text-align: right; color: var(--muted-foreground); }
  .whitespace-pre { white-space: pre; padding-left: 0.5rem; }
  .split-header { min-width: max-content; background: var(--muted); padding: 0 0.5rem; color: var(--muted-foreground); }
  .split-line { display: grid; min-width: 45rem; grid-template-columns: 2.75rem minmax(18rem, 1fr) 2.75rem minmax(18rem, 1fr); }
  .split-code { min-height: 1.25rem; white-space: pre; border-right: 1px solid var(--border); padding-inline: 0.5rem; }
  .split-code.addition { background: color-mix(in srgb, var(--action-confirm) 13%, transparent); }
  .split-code.deletion { background: color-mix(in srgb, var(--destructive) 12%, transparent); }
  .split-code.empty { background: color-mix(in srgb, var(--muted) 50%, transparent); }
  @container chat-shell (max-width: 430px) { .chat-scope-button span { display: none; } }
</style>
