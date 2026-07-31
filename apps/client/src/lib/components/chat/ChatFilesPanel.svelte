<script lang="ts">
  import { onMount } from "svelte";
  import Copy from "@lucide/svelte/icons/copy";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Eye from "@lucide/svelte/icons/eye";
  import PanelLeftClose from "@lucide/svelte/icons/panel-left-close";
  import PanelLeftOpen from "@lucide/svelte/icons/panel-left-open";
  import Paperclip from "@lucide/svelte/icons/paperclip";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Save from "@lucide/svelte/icons/save";
  import Search from "@lucide/svelte/icons/search";
  import MessageSquarePlus from "@lucide/svelte/icons/message-square-plus";
  import TextSelect from "@lucide/svelte/icons/text-select";
  import * as chatApi from "$lib/api/chat";
  import type {
    ChatWorkspaceChangeBatch,
    ChatWorkspaceObserverStatusRead,
    ProjectWorkingFolderFileEntry,
    ProjectWorkingFolderFilePreview,
    ProjectWorkingFolderPathRead,
  } from "$lib/chat/contracts";
  import { flattenChatFileTree, type ChatFileTreeRow } from "$lib/chat/file-tree-model";
  import {
    mergeWorkspaceChangeBatches,
    workspaceDirectoryRefreshTargets,
    workspacePathAfterRenames,
    workspacePreviewImpact,
  } from "$lib/chat/workspace-change-model";
  import {
    subscribeChatWorkspaceChanges,
    subscribeChatWorkspaceObserverStatus,
  } from "$lib/chat/workspace-observer-client";
  import { splitPaneResizeBounds } from "$lib/chat/inspector-model";
  import { chatErrorMessage } from "$lib/chat/error-presentation";
  import { alignPanelSizeToDevicePixel, panelWidthFromKey } from "$lib/chat/responsive-layout";
  import { boundTerminalContext } from "$lib/chat/terminal-model";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ChatCodePreview from "./ChatCodePreview.svelte";
  import type { ChatCodeSelection } from "./ChatCodePreview.svelte";
  import ChatFileIcon from "./ChatFileIcon.svelte";
  import ChatPaneResizeHandle from "./ChatPaneResizeHandle.svelte";
  import ChatWorkspaceFileTree from "./ChatWorkspaceFileTree.svelte";

  let {
    active = true,
    directoryPath,
    selectedPath,
    treeVisible,
    treeWidthPx,
    onStateChange,
    onReviewCreated = () => {},
  }: {
    active?: boolean;
    directoryPath: string;
    selectedPath: string | null;
    treeVisible: boolean;
    treeWidthPx: number;
    onStateChange: (change: {
      directoryPath?: string;
      selectedPath?: string | null;
      treeVisible?: boolean;
      treeWidthPx?: number;
    }) => void;
    onReviewCreated?: () => void;
  } = $props();

  const DEFAULT_TREE_WIDTH = 220;
  const MIN_TREE_WIDTH = 144;
  const MIN_PREVIEW_WIDTH = 120;
  const MAX_TREE_WIDTH = 360;

  interface FileMutationContext {
    requestId: number;
    scopeRevision: number;
    workingFolderId: string;
    executionEnvironmentId: string | null;
    relativePath: string;
  }

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let rootEntries = $state<ProjectWorkingFolderFileEntry[]>([]);
  let childrenByDirectory = $state<Record<string, ProjectWorkingFolderFileEntry[]>>({});
  let expandedPaths = $state<string[]>([]);
  let loadingPaths = $state<string[]>([]);
  let preview = $state<ProjectWorkingFolderFilePreview | null>(null);
  let draftText = $state("");
  let editorSelection = $state<ChatCodeSelection | null>(null);
  let reviewComposerOpen = $state(false);
  let reviewDraft = $state("");
  let creatingReview = $state(false);
  let savingFile = $state(false);
  let saveConflict = $state(false);
  let fileDeleted = $state(false);
  let conflictDiskText = $state<string | null>(null);
  let saveCopyPath = $state("");
  let previewElement: HTMLElement | undefined = $state();
  let panelElement: HTMLDivElement | undefined = $state();
  let panelWidth = $state(0);
  let renderedTreeWidth = $state(DEFAULT_TREE_WIDTH);
  let resizingTree = $state(false);
  let query = $state("");
  let searchResults = $state<ProjectWorkingFolderPathRead[]>([]);
  let includeIgnored = $state(false);
  let loadingRoot = $state(false);
  let refreshingTree = $state(false);
  let loadingPreview = $state(false);
  let observerStatus = $state<ChatWorkspaceObserverStatusRead | null>(null);
  let error = $state<string | null>(null);
  let loadedScope = "";
  let selectedPathSyncKey = "";
  let treeRequestId = 0;
  let previewRequestId = 0;
  let visibleTreeRefreshRequestId = 0;
  let searchRefreshRevision = $state(0);
  let pendingWorkspaceChange: ChatWorkspaceChangeBatch | null = null;
  let reconcilingWorkspaceChanges = false;
  let filesPanelMounted = false;
  let fileScopeRevision = 0;
  let fileMutationRequestId = 0;
  let treeResizeFrame: number | null = null;
  let treeResizeEndFrame: number | null = null;
  let activeScopeKey = "";
  const workingFolderId = $derived(chat.selectedWorkingFolderId);
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
  const fileDirty = $derived(
    preview?.text !== null && preview?.text !== undefined && draftText !== preview.text,
  );
  const observerMatchesScope = $derived(
    observerStatus?.workingFolderId === workingFolderId
      && observerStatus?.executionEnvironmentId === chat.selectedExecutionEnvironmentId,
  );

  onMount(() => {
    filesPanelMounted = true;
    const observer = new ResizeObserver(([entry]) => {
      if (entry) panelWidth = entry.contentRect.width;
    });
    if (panelElement) observer.observe(panelElement);
    return () => {
      filesPanelMounted = false;
      pendingWorkspaceChange = null;
      observer.disconnect();
      if (treeResizeFrame !== null) window.cancelAnimationFrame(treeResizeFrame);
      if (treeResizeEndFrame !== null) window.cancelAnimationFrame(treeResizeEndFrame);
    };
  });

  $effect(() => {
    if (!active) {
      pendingWorkspaceChange = null;
      observerStatus = null;
      return;
    }
    const unsubscribeChanges = subscribeChatWorkspaceChanges((batch) => {
      pendingWorkspaceChange = mergeWorkspaceChangeBatches(pendingWorkspaceChange, batch);
      if (!reconcilingWorkspaceChanges) void drainWorkspaceChanges();
    });
    const unsubscribeStatus = subscribeChatWorkspaceObserverStatus((status) => {
      observerStatus = status;
    });
    return () => {
      unsubscribeChanges();
      unsubscribeStatus();
    };
  });

  $effect(() => {
    const configuredWidth = treeWidthPx;
    const availableWidth = panelWidth;
    if (!resizingTree) {
      renderedTreeWidth = availableWidth > 0
        ? alignTreeWidth(configuredWidth)
        : configuredWidth;
    }
  });

  $effect(() => {
    const workspace = workingFolderId;
    const executionEnvironmentId = chat.selectedExecutionEnvironmentId;
    const scope = `${workspace ?? ""}:${chat.selectedThreadId ?? ""}:${executionEnvironmentId ?? ""}`;
    if (scope === loadedScope) return;
    loadedScope = scope;
    selectedPathSyncKey = "";
    fileScopeRevision += 1;
    fileMutationRequestId += 1;
    treeRequestId += 1;
    previewRequestId += 1;
    pendingWorkspaceChange = null;
    rootEntries = [];
    childrenByDirectory = {};
    expandedPaths = [];
    loadingPaths = [];
    loadingRoot = false;
    loadingPreview = false;
    refreshingTree = false;
    preview = null;
    draftText = "";
    editorSelection = null;
    reviewComposerOpen = false;
    reviewDraft = "";
    creatingReview = false;
    saveConflict = false;
    fileDeleted = false;
    conflictDiskText = null;
    saveCopyPath = "";
    savingFile = false;
    activeScopeKey = "";
  });

  $effect(() => {
    const isActive = active;
    const workspace = workingFolderId;
    const executionEnvironmentId = chat.selectedExecutionEnvironmentId;
    const scopeKey = loadedScope;
    if (!isActive || !workspace) {
      activeScopeKey = "";
      return;
    }
    if (scopeKey === activeScopeKey) return;
    activeScopeKey = scopeKey;
    void loadRoot(workspace, directoryPath, executionEnvironmentId);
    if (preview && selectedPath) void refreshPreviewFromDisk(selectedPath, false);
  });

  $effect(() => {
    const workspace = workingFolderId;
    const environmentId = chat.selectedExecutionEnvironmentId;
    const path = selectedPath;
    const key = `${loadedScope}\u0000${path ?? ""}`;
    if (!active || !workspace || !path || preview?.relativePath === path || selectedPathSyncKey === key) return;
    selectedPathSyncKey = key;
    const previousPath = preview?.relativePath ?? null;
    void selectFile(path, environmentId).then((opened) => {
      if (!opened && previousPath && selectedPath === path && workingFolderId === workspace
        && chat.selectedExecutionEnvironmentId === environmentId) {
        onStateChange({ selectedPath: previousPath });
      }
    });
  });

  $effect(() => {
    const workspace = workingFolderId;
    const value = query.trim();
    const showIgnored = includeIgnored;
    const executionEnvironmentId = chat.selectedExecutionEnvironmentId;
    const requestedRevision = searchRefreshRevision;
    if (!active || !workspace || !value) {
      searchResults = [];
      return;
    }
    let cancelled = false;
    const timer = window.setTimeout(() => {
      void chatApi.searchChatWorkingFolderPaths(workspace, value, showIgnored, null, 100, executionEnvironmentId)
        .then((page) => {
          if (!cancelled && requestedRevision === searchRefreshRevision) searchResults = page.entries;
        })
        .catch((reason: unknown) => { if (!cancelled) error = message(reason); });
    }, 160);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  });

  async function loadRoot(
    workspace: string,
    pathToReveal = "",
    executionEnvironmentId = chat.selectedExecutionEnvironmentId,
  ): Promise<void> {
    const requestId = ++treeRequestId;
    loadingRoot = true;
    error = null;
    try {
      const result = await chatApi.listProjectWorkingFolderDirectory(workspace, "", includeIgnored, executionEnvironmentId);
      if (requestId !== treeRequestId || workspace !== workingFolderId) return;
      rootEntries = result.entries;
      if (pathToReveal) await revealDirectory(pathToReveal);
    } catch (reason: unknown) {
      if (requestId === treeRequestId) error = message(reason);
    } finally {
      if (requestId === treeRequestId) loadingRoot = false;
    }
  }

  async function loadDirectory(path: string): Promise<void> {
    const workspace = workingFolderId;
    if (!workspace || childrenByDirectory[path] || loadingPaths.includes(path)) return;
    const requestId = treeRequestId;
    loadingPaths = [...loadingPaths, path];
    try {
      const result = await chatApi.listProjectWorkingFolderDirectory(workspace, path, includeIgnored, chat.selectedExecutionEnvironmentId);
      if (requestId !== treeRequestId || workspace !== workingFolderId) return;
      childrenByDirectory = { ...childrenByDirectory, [path]: result.entries };
    } catch (reason: unknown) {
      if (requestId === treeRequestId) error = message(reason);
    } finally {
      loadingPaths = loadingPaths.filter((entry) => entry !== path);
    }
  }

  async function refreshLoadedDirectories(
    paths = ["", ...Object.keys(childrenByDirectory)],
    showProgress = true,
    showIgnored = includeIgnored,
  ): Promise<void> {
    const workspace = workingFolderId;
    if (!workspace || paths.length === 0) return;
    const requestId = ++treeRequestId;
    const environmentId = chat.selectedExecutionEnvironmentId;
    const uniquePaths = [...new Set(paths)];
    if (showProgress) {
      visibleTreeRefreshRequestId = requestId;
      refreshingTree = true;
    }
    if (rootEntries.length === 0 && uniquePaths.includes("")) loadingRoot = true;
    error = null;
    try {
      for (let index = 0; index < uniquePaths.length; index += 3) {
        const group = uniquePaths.slice(index, index + 3);
        const reads = await Promise.all(group.map(async (path) => {
          try {
            const value = await chatApi.listProjectWorkingFolderDirectory(
              workspace,
              path,
              showIgnored,
              environmentId,
            );
            return { path, value, reason: null };
          } catch (reason: unknown) {
            return { path, value: null, reason };
          }
        }));
        if (requestId !== treeRequestId || workspace !== workingFolderId) return;
        const nextChildren = { ...childrenByDirectory };
        for (const read of reads) {
          if (read.value) {
            if (read.path === "") rootEntries = read.value.entries;
            else nextChildren[read.path] = read.value.entries;
            continue;
          }
          if (read.path !== "" && errorCode(read.reason) === "not_found") {
            for (const path of Object.keys(nextChildren)) {
              if (path === read.path || path.startsWith(`${read.path}/`)) delete nextChildren[path];
            }
            expandedPaths = expandedPaths.filter((path) => (
              path !== read.path && !path.startsWith(`${read.path}/`)
            ));
            continue;
          }
          if (read.reason !== null) error = message(read.reason);
        }
        childrenByDirectory = nextChildren;
      }
    } finally {
      if (requestId === treeRequestId) loadingRoot = false;
      if (showProgress && visibleTreeRefreshRequestId === requestId) refreshingTree = false;
    }
  }

  async function drainWorkspaceChanges(): Promise<void> {
    if (reconcilingWorkspaceChanges) return;
    reconcilingWorkspaceChanges = true;
    try {
      while (filesPanelMounted && pendingWorkspaceChange) {
        const batch = pendingWorkspaceChange;
        pendingWorkspaceChange = null;
        await reconcileWorkspaceChange(batch);
      }
    } catch (reason: unknown) {
      if (filesPanelMounted) error = message(reason);
    } finally {
      reconcilingWorkspaceChanges = false;
      if (filesPanelMounted && pendingWorkspaceChange) void drainWorkspaceChanges();
    }
  }

  async function reconcileWorkspaceChange(batch: ChatWorkspaceChangeBatch): Promise<void> {
    if (batch.workingFolderId !== workingFolderId
      || batch.executionEnvironmentId !== chat.selectedExecutionEnvironmentId) return;
    if (batch.renames.length > 0) {
      rootEntries = rootEntries.map((entry) => workspaceEntryAfterRenames(entry, batch));
      const nextChildren: Record<string, ProjectWorkingFolderFileEntry[]> = {};
      for (const [directory, entries] of Object.entries(childrenByDirectory)) {
        const mappedDirectory = workspacePathAfterRenames(directory, batch.renames);
        nextChildren[mappedDirectory] = entries.map((entry) => (
          workspaceEntryAfterRenames(entry, batch)
        ));
      }
      childrenByDirectory = nextChildren;
      expandedPaths = [...new Set(expandedPaths.map((path) => (
        workspacePathAfterRenames(path, batch.renames)
      )))];
    }
    const loadedDirectories = Object.keys(childrenByDirectory);
    const refreshTargets = workspaceDirectoryRefreshTargets(batch, loadedDirectories)
      .map((path) => workspacePathAfterRenames(path, batch.renames));
    if (refreshTargets.length > 0) {
      await refreshLoadedDirectories(refreshTargets, false);
    }
    searchRefreshRevision += 1;
    await reconcileSelectedFile(batch);
  }

  async function reconcileSelectedFile(batch: ChatWorkspaceChangeBatch): Promise<void> {
    const currentPath = selectedPath;
    if (!currentPath) return;
    const impact = workspacePreviewImpact(batch, currentPath, fileDirty);
    if (impact.kind === "none") return;
    if (impact.kind === "renamed" || impact.kind === "deleted_after_rename") {
      onStateChange({ selectedPath: impact.relativePath });
      await refreshPreviewFromDisk(impact.relativePath, true, currentPath);
      return;
    }
    await refreshPreviewFromDisk(currentPath, false);
  }

  function workspaceEntryAfterRenames(
    entry: ProjectWorkingFolderFileEntry,
    batch: ChatWorkspaceChangeBatch,
  ): ProjectWorkingFolderFileEntry {
    const relativePath = workspacePathAfterRenames(entry.relativePath, batch.renames);
    return {
      ...entry,
      relativePath,
      displayName: relativePath === entry.relativePath
        ? entry.displayName
        : relativePath.slice(relativePath.lastIndexOf("/") + 1),
    };
  }

  async function refreshPreviewFromDisk(
    path: string,
    moved: boolean,
    previousPath = path,
  ): Promise<void> {
    const workspace = workingFolderId;
    const current = preview;
    if (!workspace) return;
    const environmentId = chat.selectedExecutionEnvironmentId;
    const scopeRevision = fileScopeRevision;
    const requestId = ++previewRequestId;
    try {
      const disk = await chatApi.previewProjectWorkingFolderFile(
        workspace,
        path,
        environmentId,
      );
      if (requestId !== previewRequestId
        || !fileScopeIsCurrent(workspace, environmentId, scopeRevision)
        || (selectedPath !== path && selectedPath !== previousPath)) return;
      const diskChanged = moved || disk.contentRevision !== current?.contentRevision;
      if (fileDirty && diskChanged) {
        if (moved) preview = disk;
        fileDeleted = false;
        saveConflict = true;
        conflictDiskText = null;
        if (!saveCopyPath) saveCopyPath = copyPath(path);
        return;
      }
      if (!diskChanged) return;
      preview = disk;
      draftText = disk.text ?? "";
      fileDeleted = false;
      saveConflict = false;
      conflictDiskText = null;
      saveCopyPath = "";
    } catch (reason: unknown) {
      if (requestId !== previewRequestId
        || !fileScopeIsCurrent(workspace, environmentId, scopeRevision)
        || (selectedPath !== path && selectedPath !== previousPath)) return;
      if (errorCode(reason) === "not_found") {
        fileDeleted = true;
        saveConflict = fileDirty;
        conflictDiskText = null;
        if (fileDirty && !saveCopyPath) saveCopyPath = copyPath(path);
        return;
      }
      error = message(reason);
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

  function toggleDirectory(entry: ProjectWorkingFolderFileEntry): void {
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

  async function selectFile(
    path: string,
    executionEnvironmentId = chat.selectedExecutionEnvironmentId,
    discardDirty = false,
  ): Promise<boolean> {
    const workspace = workingFolderId;
    if (!workspace) return false;
    if (!discardDirty && path !== preview?.relativePath && fileDirty
      && !window.confirm(t("chat.inspector.discardUnsaved"))) return false;
    selectedPathSyncKey = `${loadedScope}\u0000${path}`;
    fileMutationRequestId += 1;
    savingFile = false;
    const scopeRevision = fileScopeRevision;
    const requestId = ++previewRequestId;
    onStateChange({ selectedPath: path });
    preview = null;
    loadingPreview = true;
    error = null;
    try {
      const result = await chatApi.previewProjectWorkingFolderFile(workspace, path, executionEnvironmentId);
      if (requestId === previewRequestId
        && fileScopeIsCurrent(workspace, executionEnvironmentId, scopeRevision)) {
        preview = result;
        draftText = result.text ?? "";
        editorSelection = null;
        reviewComposerOpen = false;
        reviewDraft = "";
        saveConflict = false;
        fileDeleted = false;
        conflictDiskText = null;
        saveCopyPath = "";
      }
    } catch (reason: unknown) {
      if (requestId === previewRequestId
        && fileScopeIsCurrent(workspace, executionEnvironmentId, scopeRevision)) {
        error = message(reason);
      }
    } finally {
      if (requestId === previewRequestId) loadingPreview = false;
    }
    return true;
  }

  async function saveFile(): Promise<void> {
    if (fileDeleted) {
      await recreateDeletedFile();
      return;
    }
    if (saveConflict) return;
    const workspace = workingFolderId;
    const current = preview;
    if (!workspace || !current?.contentRevision || !fileDirty || savingFile) return;
    const mutation = beginFileMutation(workspace, current.relativePath);
    savingFile = true;
    error = null;
    try {
      const saved = await chatApi.saveProjectWorkingFolderFile({
        workingFolderId: workspace,
        relativePath: current.relativePath,
        contents: draftText,
        expectedRevision: current.contentRevision,
        executionEnvironmentId: mutation.executionEnvironmentId,
      });
      if (!fileMutationIsCurrent(mutation)) return;
      preview = saved;
      draftText = saved.text ?? "";
      saveConflict = false;
      fileDeleted = false;
      conflictDiskText = null;
      saveCopyPath = "";
      await refreshParentDirectory(saved.relativePath);
    } catch (reason: unknown) {
      if (!fileMutationIsCurrent(mutation)) return;
      error = message(reason);
      saveConflict = errorCode(reason) === "conflict";
      if (saveConflict && !saveCopyPath) saveCopyPath = copyPath(current.relativePath);
    } finally {
      if (fileMutationScopeIsCurrent(mutation)) savingFile = false;
    }
  }

  function errorCode(reason: unknown): string | null {
    if (typeof reason !== "object" || reason === null || !("code" in reason)) return null;
    return typeof reason.code === "string" ? reason.code : null;
  }

  function beginFileMutation(workspace: string, relativePath: string): FileMutationContext {
    return {
      requestId: ++fileMutationRequestId,
      scopeRevision: fileScopeRevision,
      workingFolderId: workspace,
      executionEnvironmentId: chat.selectedExecutionEnvironmentId,
      relativePath,
    };
  }

  function fileMutationScopeIsCurrent(context: FileMutationContext): boolean {
    return context.requestId === fileMutationRequestId
      && fileScopeIsCurrent(
        context.workingFolderId,
        context.executionEnvironmentId,
        context.scopeRevision,
      );
  }

  function fileMutationIsCurrent(context: FileMutationContext): boolean {
    return fileMutationScopeIsCurrent(context)
      && preview?.relativePath === context.relativePath;
  }

  function fileScopeIsCurrent(
    workspace: string,
    executionEnvironmentId: string | null,
    scopeRevision: number,
  ): boolean {
    return scopeRevision === fileScopeRevision
      && workspace === workingFolderId
      && executionEnvironmentId === chat.selectedExecutionEnvironmentId;
  }

  function copyPath(path: string): string {
    const slash = path.lastIndexOf("/");
    const directory = slash >= 0 ? path.slice(0, slash + 1) : "";
    const name = slash >= 0 ? path.slice(slash + 1) : path;
    const dot = name.lastIndexOf(".");
    return dot > 0
      ? `${directory}${name.slice(0, dot)}.ganbaru-copy${name.slice(dot)}`
      : `${directory}${name}.ganbaru-copy`;
  }

  async function compareConflict(): Promise<void> {
    const workspace = workingFolderId;
    const current = preview;
    if (!workspace || !current) return;
    const environmentId = chat.selectedExecutionEnvironmentId;
    const scopeRevision = fileScopeRevision;
    try {
      const disk = await chatApi.previewProjectWorkingFolderFile(
        workspace,
        current.relativePath,
        environmentId,
      );
      if (!fileScopeIsCurrent(workspace, environmentId, scopeRevision)
        || preview?.relativePath !== current.relativePath) return;
      conflictDiskText = disk.text ?? "";
    } catch (reason: unknown) {
      if (fileScopeIsCurrent(workspace, environmentId, scopeRevision)
        && preview?.relativePath === current.relativePath) error = message(reason);
    }
  }

  async function saveConflictCopy(): Promise<void> {
    const workspace = workingFolderId;
    const current = preview;
    const target = saveCopyPath.trim();
    if (!workspace || !current || !target || savingFile) return;
    const mutation = beginFileMutation(workspace, current.relativePath);
    savingFile = true;
    error = null;
    try {
      const saved = fileDeleted
        ? await chatApi.recreateProjectWorkingFolderFile({
          workingFolderId: workspace,
          relativePath: target,
          contents: draftText,
          confirmed: true,
          executionEnvironmentId: mutation.executionEnvironmentId,
        })
        : await chatApi.saveProjectWorkingFolderFileCopy({
          workingFolderId: workspace,
          sourceRelativePath: current.relativePath,
          targetRelativePath: target,
          contents: draftText,
          executionEnvironmentId: mutation.executionEnvironmentId,
        });
      if (!fileMutationIsCurrent(mutation)) return;
      preview = saved;
      draftText = saved.text ?? "";
      onStateChange({ selectedPath: saved.relativePath });
      saveConflict = false;
      fileDeleted = false;
      conflictDiskText = null;
      saveCopyPath = "";
      await refreshParentDirectory(saved.relativePath);
    } catch (reason: unknown) {
      if (!fileMutationIsCurrent(mutation)) return;
      error = message(reason);
    } finally {
      if (fileMutationScopeIsCurrent(mutation)) savingFile = false;
    }
  }

  function reloadSelectedFile(): void {
    if (!selectedPath) return;
    if (fileDirty && !window.confirm(t("chat.inspector.discardUnsaved"))) return;
    void selectFile(selectedPath, chat.selectedExecutionEnvironmentId, true);
  }

  async function overwriteExternalFile(): Promise<void> {
    const workspace = workingFolderId;
    const current = preview;
    if (!workspace || !current || fileDeleted || savingFile
      || !window.confirm(t("chat.inspector.confirmOverwriteFile"))) return;
    const mutation = beginFileMutation(workspace, current.relativePath);
    savingFile = true;
    error = null;
    try {
      const disk = await chatApi.previewProjectWorkingFolderFile(
        workspace,
        current.relativePath,
        mutation.executionEnvironmentId,
      );
      if (!fileMutationIsCurrent(mutation)) return;
      if (!disk.contentRevision) throw new Error(t("chat.inspector.previewUnavailable"));
      const saved = await chatApi.saveProjectWorkingFolderFile({
        workingFolderId: workspace,
        relativePath: current.relativePath,
        contents: draftText,
        expectedRevision: disk.contentRevision,
        executionEnvironmentId: mutation.executionEnvironmentId,
      });
      if (!fileMutationIsCurrent(mutation)) return;
      preview = saved;
      draftText = saved.text ?? "";
      saveConflict = false;
      conflictDiskText = null;
      saveCopyPath = "";
      await refreshParentDirectory(saved.relativePath);
    } catch (reason: unknown) {
      if (!fileMutationIsCurrent(mutation)) return;
      error = message(reason);
      saveConflict = errorCode(reason) === "conflict";
    } finally {
      if (fileMutationScopeIsCurrent(mutation)) savingFile = false;
    }
  }

  async function recreateDeletedFile(): Promise<void> {
    const workspace = workingFolderId;
    const current = preview;
    if (!workspace || !current || !fileDeleted || savingFile
      || !window.confirm(t("chat.inspector.confirmRecreateFile"))) return;
    const mutation = beginFileMutation(workspace, current.relativePath);
    savingFile = true;
    error = null;
    try {
      const saved = await chatApi.recreateProjectWorkingFolderFile({
        workingFolderId: workspace,
        relativePath: current.relativePath,
        contents: draftText,
        confirmed: true,
        executionEnvironmentId: mutation.executionEnvironmentId,
      });
      if (!fileMutationIsCurrent(mutation)) return;
      preview = saved;
      draftText = saved.text ?? "";
      fileDeleted = false;
      saveConflict = false;
      conflictDiskText = null;
      saveCopyPath = "";
      await refreshParentDirectory(saved.relativePath);
    } catch (reason: unknown) {
      if (!fileMutationIsCurrent(mutation)) return;
      error = message(reason);
      if (errorCode(reason) === "conflict") saveConflict = true;
    } finally {
      if (fileMutationScopeIsCurrent(mutation)) savingFile = false;
    }
  }

  function openEntry(entry: ProjectWorkingFolderFileEntry): void {
    if (entry.kind === "directory") toggleDirectory(entry);
    else void selectFile(entry.relativePath);
  }

  function reloadTree(): void {
    void (async () => {
      await refreshLoadedDirectories();
      searchRefreshRevision += 1;
      if (selectedPath) await refreshPreviewFromDisk(selectedPath, false);
    })();
  }

  function toggleIgnored(): void {
    const next = !includeIgnored;
    includeIgnored = next;
    void refreshLoadedDirectories(undefined, true, next);
  }

  async function refreshParentDirectory(path: string): Promise<void> {
    const parent = path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : "";
    if (parent === "" || Object.hasOwn(childrenByDirectory, parent)) {
      await refreshLoadedDirectories([parent], false);
    }
    searchRefreshRevision += 1;
  }

  function attachFileReference(path: string): void {
    const mentions = chat.composer.mentions.filter((entry) => entry.relativePath !== path);
    chat.setComposerMentions([...mentions, { relativePath: path, kind: "file", ignored: false }]);
  }

  function treeResizeBounds(): { minimum: number; maximum: number } {
    return splitPaneResizeBounds(
      panelWidth,
      MIN_TREE_WIDTH,
      MIN_PREVIEW_WIDTH,
      MAX_TREE_WIDTH,
    );
  }

  function alignTreeWidth(value: number): number {
    const bounds = treeResizeBounds();
    const anchor = panelElement?.getBoundingClientRect().left ?? 0;
    return alignPanelSizeToDevicePixel({
      value,
      minimum: bounds.minimum,
      maximum: bounds.maximum,
      anchor,
      direction: "from-start",
      devicePixelRatio: window.devicePixelRatio,
    });
  }

  function beginTreeResize(event: PointerEvent): void {
    event.preventDefault();
    if (panelElement) panelWidth = panelElement.getBoundingClientRect().width;
    if (treeResizeEndFrame !== null) window.cancelAnimationFrame(treeResizeEndFrame);
    treeResizeEndFrame = null;
    resizingTree = true;
    const startX = event.clientX;
    const startWidth = renderedTreeWidth;
    const target = event.currentTarget as HTMLElement;
    let pendingWidth = startWidth;
    target.focus();
    target.setPointerCapture(event.pointerId);
    const move = (moveEvent: PointerEvent) => {
      pendingWidth = alignTreeWidth(startWidth + moveEvent.clientX - startX);
      if (treeResizeFrame !== null) return;
      treeResizeFrame = window.requestAnimationFrame(() => {
        renderedTreeWidth = pendingWidth;
        treeResizeFrame = null;
      });
    };
    const end = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      target.removeEventListener("pointercancel", end);
      if (treeResizeFrame !== null) window.cancelAnimationFrame(treeResizeFrame);
      treeResizeFrame = null;
      renderedTreeWidth = pendingWidth;
      onStateChange({ treeWidthPx: pendingWidth });
      treeResizeEndFrame = window.requestAnimationFrame(() => {
        treeResizeEndFrame = window.requestAnimationFrame(() => {
          resizingTree = false;
          treeResizeEndFrame = null;
        });
      });
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
    target.addEventListener("pointercancel", end);
  }

  function resizeTreeFromKey(event: KeyboardEvent): void {
    const bounds = treeResizeBounds();
    const next = panelWidthFromKey({
      current: renderedTreeWidth,
      minimum: bounds.minimum,
      maximum: bounds.maximum,
      defaultValue: DEFAULT_TREE_WIDTH,
      step: 16,
      direction: "standard",
      key: event.key,
    });
    if (next === null) return;
    event.preventDefault();
    renderedTreeWidth = alignTreeWidth(next);
    onStateChange({ treeWidthPx: renderedTreeWidth });
  }

  async function attachSelection(): Promise<void> {
    const workspace = workingFolderId;
    const path = selectedPath;
    if (!workspace || !path) return;
    const environmentId = chat.selectedExecutionEnvironmentId;
    const scopeRevision = fileScopeRevision;
    const bounded = boundTerminalContext(editorSelection?.text ?? "", 128 * 1024);
    if (!bounded.text) return;
    const attachment = await chatApi.importChatTextSnippet(
      workspace,
      crypto.randomUUID(),
      `${path} selection.txt`,
      bounded.text,
    );
    if (!fileScopeIsCurrent(workspace, environmentId, scopeRevision) || selectedPath !== path) return;
    chat.setComposerAttachments([...chat.composer.attachmentIds, attachment.id]);
  }

  async function createReviewComment(): Promise<void> {
    const threadId = chat.selectedThreadId;
    const current = preview;
    const selection = editorSelection;
    if (!threadId || !current?.contentRevision || !selection?.text || !reviewDraft.trim() || creatingReview) return;
    const workspace = workingFolderId;
    if (!workspace) return;
    const environmentId = chat.selectedExecutionEnvironmentId;
    const scopeRevision = fileScopeRevision;
    creatingReview = true;
    error = null;
    try {
      await chatApi.createChatReviewComment({
        id: crypto.randomUUID(),
        threadId,
        relativePath: current.relativePath,
        contentRevision: current.contentRevision,
        startLine: selection.startLine,
        startColumn: selection.startColumn,
        endLine: selection.endLine,
        endColumn: selection.endColumn,
        selectedText: selection.text,
        commentText: reviewDraft.trim(),
      });
      if (!fileScopeIsCurrent(workspace, environmentId, scopeRevision)
        || chat.selectedThreadId !== threadId
        || preview?.relativePath !== current.relativePath) return;
      reviewDraft = "";
      reviewComposerOpen = false;
      onReviewCreated();
    } catch (reason: unknown) {
      if (fileScopeIsCurrent(workspace, environmentId, scopeRevision)
        && chat.selectedThreadId === threadId
        && preview?.relativePath === current.relativePath) error = message(reason);
    } finally {
      if (fileScopeIsCurrent(workspace, environmentId, scopeRevision)
        && chat.selectedThreadId === threadId) creatingReview = false;
    }
  }

  function message(reason: unknown): string {
    return chatErrorMessage(reason);
  }
</script>

<div class="files-panel">
  <div bind:this={panelElement} class="files-layout" class:resizing={resizingTree} style={`--file-tree-width:${renderedTreeWidth}px`}>
    {#if treeVisible}
      <aside class="file-tree-pane" aria-label={t("chat.inspector.files")}>
        <div class="tree-toolbar">
          <label class="file-search"><Search size={13} /><input type="search" bind:value={query} placeholder={t("chat.inspector.searchFiles")} aria-label={t("chat.inspector.searchFiles")} /></label>
          <button type="button" class="tree-action" class:active={includeIgnored} aria-pressed={includeIgnored} title={t("chat.inspector.showIgnored")} aria-label={t("chat.inspector.showIgnored")} onclick={toggleIgnored}><Eye size={13} /></button>
          <button type="button" class="tree-action" title={t("chat.inspector.refreshFiles")} aria-label={t("chat.inspector.refreshFiles")} onclick={reloadTree}><RefreshCw size={13} class={refreshingTree ? "animate-spin" : ""} /></button>
          <button type="button" class="tree-action" title={t("chat.inspector.hideFileTree")} aria-label={t("chat.inspector.hideFileTree")} onclick={() => onStateChange({ treeVisible: false })}><PanelLeftClose size={13} /></button>
        </div>
        {#if observerMatchesScope && observerStatus?.mode === "polling"}
          <p role="status" class="live-update-status">{t("chat.inspector.liveUpdatesPolling")}</p>
        {:else if observerMatchesScope && observerStatus?.mode === "unavailable"}
          <p role="status" class="live-update-status warning">{t("chat.inspector.liveUpdatesUnavailable")}</p>
        {:else if observerMatchesScope && observerStatus?.degradedReason}
          <p role="status" class="live-update-status warning">{t("chat.inspector.liveUpdatesDegraded")}</p>
        {/if}
        {#if error}<p role="alert" class="border-b border-destructive/30 p-2 text-xs text-destructive">{error}</p>{/if}
        {#if loadingRoot}
          <p class="p-2 text-xs text-muted-foreground">{t("common.loading")}</p>
        {:else if shownRows.length > 0}
          <ChatWorkspaceFileTree
            rows={shownRows}
            {selectedPath}
            {changedPaths}
            {loadingPaths}
            refreshing={refreshingTree}
            onToggle={toggleDirectory}
            onSelect={openEntry}
          />
        {:else}
          <p class="p-2 text-xs text-muted-foreground">{query.trim() ? t("chat.inspector.noFileResults") : t("chat.inspector.emptyDirectory")}</p>
        {/if}
      </aside>
      <ChatPaneResizeHandle orientation="vertical" value={renderedTreeWidth} minimum={treeResizeBounds().minimum} maximum={treeResizeBounds().maximum} label={t("chat.resizeFileTree")} active={resizingTree} onPointerDown={beginTreeResize} onKeyDown={resizeTreeFromKey} />
    {/if}

    <section bind:this={previewElement} class="file-editor" aria-label={t("chat.inspector.filePreview")}>
      <header class="editor-heading">
        {#if !treeVisible}<button type="button" class="chat-icon-button" title={t("chat.inspector.showFileTree")} aria-label={t("chat.inspector.showFileTree")} onclick={() => onStateChange({ treeVisible: true })}><PanelLeftOpen size={13} /></button>{/if}
        {#if preview}
          <ChatFileIcon path={preview.relativePath} />
          <strong class="min-w-0 flex-1 truncate text-[0.733333rem] font-medium" title={preview.relativePath}>{preview.relativePath}</strong>
          {#if fileDirty}<span class="dirty-indicator" title={t("chat.inspector.unsavedChanges")} aria-label={t("chat.inspector.unsavedChanges")}></span>{/if}
          <button type="button" class="chat-icon-button" disabled={savingFile || (!fileDeleted && (saveConflict || !fileDirty || !preview.contentRevision))} title={savingFile ? t("chat.inspector.savingFile") : fileDeleted ? t("chat.inspector.recreateFile") : t("chat.inspector.saveFile")} aria-label={savingFile ? t("chat.inspector.savingFile") : fileDeleted ? t("chat.inspector.recreateFile") : t("chat.inspector.saveFile")} onclick={() => void saveFile()}><Save size={13} /></button>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.copyPath")} aria-label={t("chat.inspector.copyPath")} onclick={() => navigator.clipboard.writeText(selectedPath ?? "")}><Copy size={13} /></button>
          <button type="button" class="chat-icon-button" disabled={fileDeleted} title={t("chat.inspector.attachFile")} aria-label={t("chat.inspector.attachFile")} onclick={() => selectedPath && attachFileReference(selectedPath)}><Paperclip size={13} /></button>
          <button type="button" class="chat-icon-button" title={t("chat.inspector.attachSelection")} aria-label={t("chat.inspector.attachSelection")} onclick={() => { void attachSelection().catch((reason) => { error = message(reason); }); }}><TextSelect size={13} /></button>
          <button type="button" class="chat-icon-button" disabled={fileDeleted || !editorSelection?.text || !preview.contentRevision || !chat.selectedThreadId} title={t("chat.review.addComment")} aria-label={t("chat.review.addComment")} onclick={() => { reviewComposerOpen = !reviewComposerOpen; }}><MessageSquarePlus size={13} /></button>
          <button type="button" class="chat-icon-button" disabled={fileDeleted} title={t("chat.inspector.openExternally")} aria-label={t("chat.inspector.openExternally")} onclick={() => workingFolderId && selectedPath && chatApi.openProjectWorkingFolderFile(workingFolderId, selectedPath, chat.selectedExecutionEnvironmentId)}><ExternalLink size={13} /></button>
        {:else}
          <span class="min-w-0 flex-1 truncate text-[0.733333rem] text-muted-foreground">{t("chat.inspector.filePreview")}</span>
        {/if}
      </header>
      {#if reviewComposerOpen && editorSelection?.text}
        <form class="review-composer" onsubmit={(event) => { event.preventDefault(); void createReviewComment(); }}>
          <label for="chat-review-comment">{t("chat.review.commentOnSelection", editorSelection.startLine, editorSelection.endLine)}</label>
          <textarea id="chat-review-comment" bind:value={reviewDraft} maxlength="65536" placeholder={t("chat.review.commentPlaceholder")}></textarea>
          <div>
            <button type="button" onclick={() => { reviewComposerOpen = false; reviewDraft = ""; }}>{t("common.cancel")}</button>
            <button type="submit" class="primary" disabled={!reviewDraft.trim() || creatingReview}>{creatingReview ? t("common.loading") : t("chat.review.add")}</button>
          </div>
        </form>
      {/if}
      {#if error && !treeVisible}<p role="alert" class="border-b border-destructive/30 p-2 text-xs text-destructive">{error}</p>{/if}
      {#if fileDeleted}
        <div role="alert" class="save-conflict">
          <span>{fileDirty ? t("chat.inspector.fileDeletedWithUnsavedChanges") : t("chat.inspector.fileDeletedOnDisk")}</span>
          <div><button type="button" onclick={() => void recreateDeletedFile()}>{t("chat.inspector.recreateFile")}</button></div>
          {#if fileDirty}
            <form onsubmit={(event) => { event.preventDefault(); void saveConflictCopy(); }}><input bind:value={saveCopyPath} aria-label={t("chat.inspector.saveCopyPath")} /><button type="submit" disabled={!saveCopyPath.trim() || savingFile}>{t("chat.inspector.saveCopy")}</button></form>
          {/if}
        </div>
      {:else if saveConflict}
        <div role="alert" class="save-conflict">
          <span>{t("chat.inspector.saveConflict")}</span>
          <div><button type="button" onclick={() => void compareConflict()}>{t("chat.inspector.compareFile")}</button><button type="button" onclick={reloadSelectedFile}>{t("chat.inspector.reloadFile")}</button><button type="button" onclick={() => void overwriteExternalFile()}>{t("chat.inspector.overwriteFile")}</button></div>
          <form onsubmit={(event) => { event.preventDefault(); void saveConflictCopy(); }}><input bind:value={saveCopyPath} aria-label={t("chat.inspector.saveCopyPath")} /><button type="submit" disabled={!saveCopyPath.trim() || savingFile}>{t("chat.inspector.saveCopy")}</button></form>
        </div>
      {/if}
      {#if conflictDiskText !== null && preview}
        <section class="conflict-compare" aria-label={t("chat.inspector.compareFile")}>
          <div><strong>{t("chat.inspector.diskVersion")}</strong><pre>{conflictDiskText}</pre></div>
          <div><strong>{t("chat.inspector.yourVersion")}</strong><pre>{draftText}</pre></div>
        </section>
      {/if}
      {#if loadingPreview}
        <p class="m-auto text-xs text-muted-foreground">{t("common.loading")}</p>
      {:else if preview && preview.text !== null}
        {@const previewPath = preview.relativePath}
        <ChatCodePreview
          text={draftText}
          relativePath={preview.relativePath}
          onChange={(text) => {
            draftText = text;
            if (fileDeleted && !saveCopyPath) saveCopyPath = copyPath(previewPath);
          }}
          onSelectionChange={(selection) => { editorSelection = selection; }}
          onSave={() => void saveFile()}
        />
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
  .files-panel { display: flex; height: 100%; min-height: 0; flex-direction: column; }
  .files-layout { display: flex; min-height: 0; flex: 1; }
  .files-layout.resizing, .files-layout.resizing * { user-select: none; }
  .file-tree-pane { display: flex; width: var(--file-tree-width); min-width: 0; min-height: 0; flex: 0 0 var(--file-tree-width); flex-direction: column; overflow: hidden; background: color-mix(in srgb, var(--cal-bg) 96%, var(--muted)); }
  .tree-toolbar { display: flex; min-height: 2.45rem; flex: 0 0 auto; align-items: center; gap: 0.15rem; border-bottom: 1px solid var(--border); padding: 0.3rem; }
  .file-search { display: flex; min-width: 0; min-height: 1.75rem; flex: 1; align-items: center; gap: 0.35rem; border-radius: 0.4rem; padding-inline: 0.4rem; color: var(--muted-foreground); }
  .file-search:focus-within { background: var(--background); box-shadow: inset 0 0 0 1px var(--ring); color: var(--foreground); }
  .file-search input { min-width: 0; flex: 1; background: transparent; color: var(--foreground); font-size: 0.7rem; outline: none; }
  .tree-action { display: inline-grid; width: 1.7rem; height: 1.7rem; flex: 0 0 auto; place-items: center; border-radius: 0.35rem; color: var(--muted-foreground); }
  .tree-action:hover, .tree-action.active { background: var(--accent); color: var(--foreground); }
  .live-update-status { flex: 0 0 auto; border-bottom: 1px solid var(--border); padding: 0.3rem 0.45rem; color: var(--muted-foreground); font-size: 0.66rem; line-height: 1.2; }
  .live-update-status.warning { color: var(--status-tentative); }
  .file-editor { display: flex; min-width: 0; min-height: 0; flex: 1; flex-direction: column; overflow: hidden; }
  .editor-heading { display: flex; min-height: 2.45rem; flex: 0 0 auto; align-items: center; gap: 0.25rem; border-bottom: 1px solid var(--border); padding: 0.3rem 0.4rem; }
  .dirty-indicator { width: 0.45rem; height: 0.45rem; flex: 0 0 auto; border-radius: 999px; background: var(--status-tentative); }
  .save-conflict { display: grid; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 0.35rem; border-bottom: 1px solid color-mix(in srgb, var(--destructive) 35%, var(--border)); background: color-mix(in srgb, var(--destructive) 8%, var(--background)); padding: 0.4rem 0.55rem; color: var(--destructive); font-size: 0.7rem; }
  .save-conflict > div { display: flex; gap: 0.25rem; }
  .save-conflict form { grid-column: 1 / -1; display: flex; min-width: 0; gap: 0.35rem; }
  .save-conflict input { min-width: 0; flex: 1; border: 1px solid var(--border); border-radius: 0.35rem; background: var(--background); padding: 0.25rem 0.4rem; color: var(--foreground); }
  .save-conflict button { flex: 0 0 auto; border-radius: 0.35rem; padding: 0.2rem 0.45rem; color: var(--foreground); }
  .save-conflict button:hover { background: var(--accent); }
  .conflict-compare { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); max-height: 35%; overflow: hidden; border-bottom: 1px solid var(--border); }
  .conflict-compare > div { min-width: 0; overflow: auto; padding: 0.4rem; }
  .conflict-compare > div + div { border-left: 1px solid var(--border); }
  .conflict-compare strong { font-size: 0.66rem; }
  .conflict-compare pre { margin-top: 0.3rem; white-space: pre-wrap; overflow-wrap: anywhere; font-size: 0.66rem; }
  .review-composer { display: grid; flex: 0 0 auto; gap: 0.4rem; border-bottom: 1px solid var(--border); padding: 0.55rem; background: var(--background); }
  .review-composer label { font-size: 0.7rem; color: var(--muted-foreground); }
  .review-composer textarea { min-height: 4rem; max-height: 9rem; resize: vertical; border: 1px solid var(--border); border-radius: 0.4rem; background: var(--cal-bg); padding: 0.45rem; font-size: 0.733333rem; color: var(--foreground); outline: none; }
  .review-composer textarea:focus { border-color: var(--ring); }
  .review-composer div { display: flex; justify-content: flex-end; gap: 0.35rem; }
  .review-composer button { border-radius: 0.35rem; padding: 0.3rem 0.55rem; font-size: 0.7rem; }
  .review-composer button:hover { background: var(--accent); }
  .review-composer button.primary { background: var(--primary); color: var(--primary-foreground); }
  .review-composer button:disabled { opacity: 0.5; }
</style>
