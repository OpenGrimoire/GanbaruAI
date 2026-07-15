import {
  getMusicIssues,
  getMusicItemWindow,
  getMusicPlaylistSummaries,
  getMusicSourceSummaries,
} from "$lib/api/music-library";
import type {
  MusicGroupBy,
  MusicIssue,
  MusicItemAvailability,
  MusicItemListEntry,
  MusicItemSort,
  MusicItemWindow,
  MusicItemWindowRequest,
  MusicLibrarySourceKind,
  MusicPlaylistSummary,
  MusicReviewState,
  MusicSortDirection,
  MusicSourceSummary,
} from "$lib/music/library-contracts";

export type MusicBuilderLocation =
  | { kind: "review" }
  | { kind: "library" }
  | { kind: "playlist"; playlistId: string }
  | { kind: "sources" }
  | { kind: "issues" };

export interface MusicDestinationState {
  search: string;
  sourceKind: MusicLibrarySourceKind | null;
  availability: MusicItemAvailability | null;
  reviewState: MusicReviewState | null;
  sourceCollectionId: string | null;
  snoozed: boolean | null;
  sort: MusicItemSort;
  direction: MusicSortDirection;
  groupBy: MusicGroupBy;
  offset: number;
  limit: number;
  scrollTop: number;
  selectedItemId: string | null;
}

export interface MusicLibraryControllerApi {
  itemWindow(request: MusicItemWindowRequest): Promise<MusicItemWindow>;
  playlistSummaries(nowMs: number, offset: number, limit: number): Promise<MusicPlaylistSummary[]>;
  sourceSummaries(nowMs: number, offset: number, limit: number): Promise<MusicSourceSummary[]>;
  issues(offset: number, limit: number): Promise<MusicIssue[]>;
}

export interface OptimisticMutation<T> {
  key: string;
  label: string;
  apply(): void;
  rollback(): void;
  persist(): Promise<T>;
  undo?(): Promise<void>;
}

interface UndoEntry {
  label: string;
  run(): Promise<void>;
}

const defaultApi: MusicLibraryControllerApi = {
  itemWindow: getMusicItemWindow,
  playlistSummaries: getMusicPlaylistSummaries,
  sourceSummaries: getMusicSourceSummaries,
  issues: getMusicIssues,
};

const emptyWindow: MusicItemWindow = {
  items: [],
  groups: [],
  totalCount: 0,
  offset: 0,
  limit: 50,
};

function locationKey(location: MusicBuilderLocation): string {
  return location.kind === "playlist" ? `playlist:${location.playlistId}` : location.kind;
}

function defaultDestinationState(location: MusicBuilderLocation): MusicDestinationState {
  return {
    search: "",
    sourceKind: null,
    availability: null,
    reviewState: null,
    sourceCollectionId: null,
    snoozed: null,
    sort: location.kind === "playlist" ? "manual-position" : "title",
    direction: "ascending",
    groupBy: "none",
    offset: 0,
    limit: 50,
    scrollTop: 0,
    selectedItemId: null,
  };
}

function hasItemWindow(location: MusicBuilderLocation): boolean {
  return location.kind === "review" || location.kind === "library" || location.kind === "playlist";
}

function itemWindowRequest(
  location: MusicBuilderLocation,
  state: MusicDestinationState,
  nowMs: number,
): MusicItemWindowRequest {
  return {
    destination: location.kind === "playlist" ? "playlist" : location.kind === "review" ? "review" : "library",
    playlistId: location.kind === "playlist" ? location.playlistId : null,
    search: state.search,
    sourceKind: state.sourceKind,
    availability: state.availability,
    reviewState: state.reviewState,
    sourceCollectionId: state.sourceCollectionId,
    snoozed: state.snoozed,
    sort: state.sort,
    direction: state.direction,
    groupBy: state.groupBy,
    nowMs,
    offset: state.offset,
    limit: state.limit,
  };
}

export class MusicLibraryController {
  vaultId = $state<string | null>(null);
  location = $state<MusicBuilderLocation>({ kind: "review" });
  destinationStates = $state<Record<string, MusicDestinationState>>({
    review: defaultDestinationState({ kind: "review" }),
    library: defaultDestinationState({ kind: "library" }),
    sources: defaultDestinationState({ kind: "sources" }),
    issues: defaultDestinationState({ kind: "issues" }),
  });
  windows = $state<Record<string, MusicItemWindow>>({});
  playlistSummaries = $state<MusicPlaylistSummary[]>([]);
  sourceSummaries = $state<MusicSourceSummary[]>([]);
  issues = $state<MusicIssue[]>([]);
  busy = $state(false);
  error = $state<Error | null>(null);
  undoCount = $state(0);
  lastUndoLabel = $state<string | null>(null);

  currentKey = $derived(locationKey(this.location));
  currentState = $derived(this.destinationStates[this.currentKey] ?? defaultDestinationState(this.location));
  currentWindow = $derived(this.windows[this.currentKey] ?? emptyWindow);
  selectedItem = $derived.by<MusicItemListEntry | null>(() => {
    const selectedId = this.currentState.selectedItemId;
    if (!selectedId) return null;
    return this.currentWindow.items.find((item) => item.id === selectedId) ?? null;
  });

  private readonly api: MusicLibraryControllerApi;
  private readonly now: () => number;
  private refreshGeneration = 0;
  private mutationRevisions: Record<string, number> = {};
  private undoEntries: UndoEntry[] = [];

  constructor(api: MusicLibraryControllerApi = defaultApi, now: () => number = Date.now) {
    this.api = api;
    this.now = now;
  }

  setVault(vaultId: string | null): void {
    const normalized = vaultId?.trim() || null;
    if (normalized === this.vaultId) return;
    this.vaultId = normalized;
    this.refreshGeneration += 1;
    this.windows = {};
    this.playlistSummaries = [];
    this.sourceSummaries = [];
    this.issues = [];
    this.error = null;
    this.busy = false;
    this.mutationRevisions = {};
    this.undoEntries = [];
    this.syncUndoProjection();
    for (const state of Object.values(this.destinationStates)) state.selectedItemId = null;
  }

  navigate(location: MusicBuilderLocation): void {
    const key = locationKey(location);
    if (!this.destinationStates[key]) {
      this.destinationStates[key] = defaultDestinationState(location);
    }
    this.location = location;
  }

  patchCurrentState(patch: Partial<MusicDestinationState>): void {
    const key = this.currentKey;
    const current = this.destinationStates[key] ?? defaultDestinationState(this.location);
    this.destinationStates[key] = { ...current, ...patch };
  }

  selectItem(itemId: string | null): void {
    this.patchCurrentState({ selectedItemId: itemId });
  }

  setScrollTop(scrollTop: number): void {
    this.patchCurrentState({ scrollTop: Math.max(0, scrollTop) });
  }

  async refresh(): Promise<boolean> {
    if (!this.vaultId) return false;
    const generation = ++this.refreshGeneration;
    const vaultId = this.vaultId;
    const location = this.location;
    const key = locationKey(location);
    const state = { ...(this.destinationStates[key] ?? defaultDestinationState(location)) };
    const nowMs = this.now();
    this.busy = true;
    this.error = null;
    try {
      const [window, playlists, sources, issues] = await Promise.all([
        hasItemWindow(location)
          ? this.api.itemWindow(itemWindowRequest(location, state, nowMs))
          : Promise.resolve(null),
        this.api.playlistSummaries(nowMs, 0, 500),
        this.api.sourceSummaries(nowMs, 0, 500),
        this.api.issues(0, 500),
      ]);
      if (!this.isCurrent(generation, vaultId)) return false;
      if (window) this.windows[key] = window;
      this.playlistSummaries = playlists;
      this.sourceSummaries = sources;
      this.issues = issues;
      return true;
    } catch (error) {
      if (!this.isCurrent(generation, vaultId)) return false;
      this.error = error instanceof Error ? error : new Error(String(error));
      return false;
    } finally {
      if (this.isCurrent(generation, vaultId)) this.busy = false;
    }
  }

  async runOptimistic<T>(mutation: OptimisticMutation<T>): Promise<T> {
    const revision = (this.mutationRevisions[mutation.key] ?? 0) + 1;
    this.mutationRevisions[mutation.key] = revision;
    mutation.apply();
    try {
      const result = await mutation.persist();
      if (this.mutationRevisions[mutation.key] === revision && mutation.undo) {
        this.pushUndo(mutation.label, mutation.undo);
      }
      return result;
    } catch (error) {
      if (this.mutationRevisions[mutation.key] === revision) mutation.rollback();
      throw error;
    }
  }

  async undoLast(): Promise<boolean> {
    const entry = this.undoEntries.pop();
    this.syncUndoProjection();
    if (!entry) return false;
    await entry.run();
    return true;
  }

  clearUndo(): void {
    this.undoEntries = [];
    this.syncUndoProjection();
  }

  private isCurrent(generation: number, vaultId: string): boolean {
    return generation === this.refreshGeneration && vaultId === this.vaultId;
  }

  private pushUndo(label: string, run: () => Promise<void>): void {
    this.undoEntries.push({ label, run });
    if (this.undoEntries.length > 50) this.undoEntries.splice(0, this.undoEntries.length - 50);
    this.syncUndoProjection();
  }

  private syncUndoProjection(): void {
    this.undoCount = this.undoEntries.length;
    this.lastUndoLabel = this.undoEntries.at(-1)?.label ?? null;
  }
}

export function createMusicLibraryController(
  api: MusicLibraryControllerApi = defaultApi,
  now: () => number = Date.now,
): MusicLibraryController {
  return new MusicLibraryController(api, now);
}
