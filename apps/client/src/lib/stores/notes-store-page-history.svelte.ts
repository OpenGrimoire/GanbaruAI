import {
  copyNotesPageHistoryBlocks,
  getNotesPageHistorySettings,
  listNotesPageHistorySnapshots,
  loadNotesPageHistorySnapshot,
  restoreNotesPageHistorySnapshot,
  updateNotesPageHistorySettings,
} from "$lib/api/notes";
import type {
  NotesLoadedPage,
  NotesPageHistorySettings,
  NotesPageHistorySnapshot,
} from "$lib/notes/types";

export interface NotesPageHistoryControllerContext {
  readSelectedPageId: () => string | null;
  loadPageTree: (pageId: string) => Promise<void>;
  reloadPages: (selectedPageIdOverride?: string | null) => Promise<void>;
  hydrateUndo: (pageId: string | null) => Promise<void>;
  requestPageLoadFocus: (requestedBlockId?: string | null) => void;
  flushPendingBlockSaves: () => Promise<void>;
  setLoadError: (message: string) => void;
}

export interface NotesPageHistoryController {
  readonly snapshots: NotesPageHistorySnapshot[];
  readonly snapshotsLoading: boolean;
  readonly snapshotsError: string | null;
  readonly version: NotesLoadedPage | null;
  readonly versionLoading: boolean;
  readonly versionError: string | null;
  readonly settings: NotesPageHistorySettings | null;
  readonly settingsLoading: boolean;
  readonly settingsError: string | null;
  readonly actionLoading: boolean;
  readonly actionError: string | null;
  resetPageState: () => void;
  loadSettings: () => Promise<void>;
  updateRetention: (retentionDays: number | null) => Promise<void>;
  reloadSnapshots: (pageId?: string | null) => Promise<void>;
  loadVersion: (snapshotId: string) => Promise<void>;
  restoreVersion: (snapshotId: string) => Promise<void>;
  copyBlocks: (snapshotId: string) => Promise<void>;
}

/**
 * Create the page history state boundary for the Notes store.
 */
export function createNotesPageHistoryController(
  context: NotesPageHistoryControllerContext,
): NotesPageHistoryController {
  let snapshots = $state<NotesPageHistorySnapshot[]>([]);
  let snapshotsLoading = $state(false);
  let snapshotsError = $state<string | null>(null);
  let version = $state<NotesLoadedPage | null>(null);
  let versionSnapshotId: string | null = null;
  let versionLoading = $state(false);
  let versionError = $state<string | null>(null);
  let settings = $state<NotesPageHistorySettings | null>(null);
  let settingsLoading = $state(false);
  let settingsError = $state<string | null>(null);
  let actionLoading = $state(false);
  let actionError = $state<string | null>(null);
  let snapshotsRequestId = 0;
  let versionRequestId = 0;
  let settingsRequestId = 0;

  function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }

  function resetPageState(): void {
    snapshotsRequestId += 1;
    versionRequestId += 1;
    snapshots = [];
    snapshotsLoading = false;
    snapshotsError = null;
    version = null;
    versionSnapshotId = null;
    versionLoading = false;
    versionError = null;
    actionLoading = false;
    actionError = null;
  }

  async function loadSettings(): Promise<void> {
    const requestId = ++settingsRequestId;
    settingsLoading = true;
    settingsError = null;
    try {
      const nextSettings = await getNotesPageHistorySettings();
      if (requestId !== settingsRequestId) return;
      settings = nextSettings;
    } catch (error) {
      if (requestId !== settingsRequestId) return;
      settingsError = errorMessage(error);
      context.setLoadError(settingsError);
    } finally {
      if (requestId === settingsRequestId) settingsLoading = false;
    }
  }

  async function updateRetention(retentionDays: number | null): Promise<void> {
    const requestId = ++settingsRequestId;
    settingsLoading = true;
    settingsError = null;
    try {
      const nextSettings = await updateNotesPageHistorySettings({ retention_days: retentionDays });
      if (requestId !== settingsRequestId) return;
      settings = nextSettings;
      await reloadSnapshots();
    } catch (error) {
      if (requestId !== settingsRequestId) return;
      settingsError = errorMessage(error);
      context.setLoadError(settingsError);
    } finally {
      if (requestId === settingsRequestId) settingsLoading = false;
    }
  }

  async function reloadSnapshots(pageId: string | null = context.readSelectedPageId()): Promise<void> {
    const requestId = ++snapshotsRequestId;
    if (!pageId) {
      resetPageState();
      return;
    }
    snapshotsLoading = true;
    snapshotsError = null;
    try {
      const nextSnapshots = await listNotesPageHistorySnapshots(pageId);
      if (requestId !== snapshotsRequestId) return;
      snapshots = [...nextSnapshots];
      if (version && version.page.id !== pageId) {
        version = null;
        versionSnapshotId = null;
      }
      if (version && !nextSnapshots.some((snapshot) => snapshot.id === versionSnapshotId)) {
        version = null;
        versionSnapshotId = null;
      }
    } catch (error) {
      if (requestId !== snapshotsRequestId) return;
      snapshots = [];
      snapshotsError = errorMessage(error);
    } finally {
      if (requestId === snapshotsRequestId) snapshotsLoading = false;
    }
  }

  async function loadVersion(snapshotId: string): Promise<void> {
    const pageId = context.readSelectedPageId();
    const requestId = ++versionRequestId;
    if (!pageId) {
      version = null;
      return;
    }
    versionLoading = true;
    versionError = null;
    try {
      const loaded = await loadNotesPageHistorySnapshot(pageId, snapshotId);
      if (requestId !== versionRequestId) return;
      version = loaded;
      versionSnapshotId = snapshotId;
    } catch (error) {
      if (requestId !== versionRequestId) return;
      version = null;
      versionSnapshotId = null;
      versionError = errorMessage(error);
    } finally {
      if (requestId === versionRequestId) versionLoading = false;
    }
  }

  async function restoreVersion(snapshotId: string): Promise<void> {
    const pageId = context.readSelectedPageId();
    if (!pageId) return;
    actionLoading = true;
    actionError = null;
    try {
      await context.flushPendingBlockSaves();
      await restoreNotesPageHistorySnapshot(pageId, snapshotId);
      await context.loadPageTree(pageId);
      await context.reloadPages(pageId);
      await context.hydrateUndo(pageId);
      context.requestPageLoadFocus();
    } catch (error) {
      actionError = errorMessage(error);
      context.setLoadError(actionError);
    } finally {
      actionLoading = false;
    }
  }

  async function copyBlocks(snapshotId: string): Promise<void> {
    const pageId = context.readSelectedPageId();
    if (!pageId) return;
    actionLoading = true;
    actionError = null;
    try {
      await context.flushPendingBlockSaves();
      const copied = await copyNotesPageHistoryBlocks(pageId, snapshotId, {
        after_block_id: null,
      });
      await context.loadPageTree(pageId);
      await context.reloadPages(pageId);
      context.requestPageLoadFocus(copied.results[0]?.id ?? null);
    } catch (error) {
      actionError = errorMessage(error);
      context.setLoadError(actionError);
    } finally {
      actionLoading = false;
    }
  }

  return {
    get snapshots() {
      return snapshots;
    },
    get snapshotsLoading() {
      return snapshotsLoading;
    },
    get snapshotsError() {
      return snapshotsError;
    },
    get version() {
      return version;
    },
    get versionLoading() {
      return versionLoading;
    },
    get versionError() {
      return versionError;
    },
    get settings() {
      return settings;
    },
    get settingsLoading() {
      return settingsLoading;
    },
    get settingsError() {
      return settingsError;
    },
    get actionLoading() {
      return actionLoading;
    },
    get actionError() {
      return actionError;
    },
    resetPageState,
    loadSettings,
    updateRetention,
    reloadSnapshots,
    loadVersion,
    restoreVersion,
    copyBlocks,
  };
}
