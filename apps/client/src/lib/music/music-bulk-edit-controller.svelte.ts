import { bulkEditMusicMemberships, bulkSetMusicReviewState, bulkSnoozeMusicItems, getMusicInspectorDetail, getMusicMembershipMatrix, setMusicItemSignals } from "$lib/api/music-library";
import type { MusicLibraryController } from "$lib/music/music-library-controller.svelte";
import type { MusicItemSignal, MusicPlaylistSummary, MusicReviewState, MusicSnoozeScope, MusicWeight } from "$lib/music/library-contracts";

export type MusicBulkPlaylistState = "checked" | "mixed" | "unchecked";

export class MusicBulkEditController {
  itemIds = $state<string[]>([]);
  states = $state<Record<string, MusicBulkPlaylistState>>({});
  initialCounts = $state<Record<string, number>>({});
  search = $state("");
  loading = $state(false);
  saving = $state(false);
  error = $state<string | null>(null);
  selectionStale = $state(false);

  constructor(
    private readonly library: MusicLibraryController,
    private readonly now: () => number = Date.now,
    private readonly id: () => string = () => crypto.randomUUID(),
  ) {}

  get checkedIds(): Set<string> {
    return new Set(Object.entries(this.states).filter(([, state]) => state === "checked").map(([id]) => id));
  }

  get mixedIds(): Set<string> {
    return new Set(Object.entries(this.states).filter(([, state]) => state === "mixed").map(([id]) => id));
  }

  useSelection(itemIds: readonly string[]): void {
    this.itemIds = [...new Set(itemIds)];
    this.error = null;
    this.selectionStale = false;
  }

  async open(itemIds: readonly string[], playlists: readonly MusicPlaylistSummary[]): Promise<boolean> {
    this.itemIds = [...new Set(itemIds)];
    this.search = "";
    this.error = null;
    this.selectionStale = false;
    this.loading = true;
    try {
      const matrix = await getMusicMembershipMatrix(this.itemIds);
      const counts: Record<string, number> = {};
      for (const entry of matrix) counts[entry.playlistId] = (counts[entry.playlistId] ?? 0) + 1;
      this.initialCounts = counts;
      this.states = Object.fromEntries(playlists.map((playlist) => {
        const count = counts[playlist.id] ?? 0;
        return [playlist.id, count === 0 ? "unchecked" : count === this.itemIds.length ? "checked" : "mixed"];
      }));
      return true;
    } catch (error) {
      this.error = error instanceof Error ? error.message : String(error);
      return false;
    } finally {
      this.loading = false;
    }
  }

  toggle(playlistId: string): void {
    const state = this.states[playlistId] ?? "unchecked";
    this.states = {
      ...this.states,
      [playlistId]: state === "checked" ? "unchecked" : "checked",
    };
  }

  async saveMemberships(): Promise<boolean> {
    if (this.saving || this.itemIds.length === 0) return false;
    const addPlaylistIds: string[] = [];
    const removePlaylistIds: string[] = [];
    for (const [playlistId, state] of Object.entries(this.states)) {
      const initialCount = this.initialCounts[playlistId] ?? 0;
      if (state === "checked" && initialCount < this.itemIds.length) addPlaylistIds.push(playlistId);
      if (state === "unchecked" && initialCount > 0) removePlaylistIds.push(playlistId);
    }
    if (addPlaylistIds.length === 0 && removePlaylistIds.length === 0) return true;
    return this.persist({ addPlaylistIds, removePlaylistIds, weightPlaylistIds: [], weight: null });
  }

  async setWeight(playlistId: string, weight: MusicWeight): Promise<boolean> {
    return this.persist({ addPlaylistIds: [], removePlaylistIds: [], weightPlaylistIds: [playlistId], weight });
  }

  async removeFromPlaylist(playlistId: string): Promise<boolean> {
    return this.persist({ addPlaylistIds: [], removePlaylistIds: [playlistId], weightPlaylistIds: [], weight: null });
  }

  async setSignals(signals: MusicItemSignal[]): Promise<boolean> {
    if (this.saving || this.itemIds.length === 0) return false;
    this.saving = true;
    this.error = null;
    try {
      const prior = await Promise.all(this.itemIds.map(async (itemId) => ({ itemId, signals: (await getMusicInspectorDetail(itemId)).signals })));
      await this.library.runOptimistic({
        key: `bulk-signals:${this.itemIds.join(":")}`,
        label: "Describe selected tracks",
        apply: () => undefined,
        rollback: () => undefined,
        persist: () => setMusicItemSignals({ itemIds: this.itemIds, signals, updatedAt: this.now() }),
        undo: async () => {
          for (const entry of prior) await setMusicItemSignals({ itemIds: [entry.itemId], signals: entry.signals, updatedAt: this.now() });
          await this.library.refreshAfterMutation();
        },
      });
      await this.library.refreshAfterMutation();
      return true;
    } catch (error) {
      this.error = error instanceof Error ? error.message : String(error);
      return false;
    } finally {
      this.saving = false;
    }
  }

  async setReviewState(reviewState: MusicReviewState): Promise<boolean> {
    if (this.saving || this.itemIds.length === 0) return false;
    const versions = new Map(this.library.currentWindow.items.map((item) => [item.id, item.version]));
    const items = this.itemIds.flatMap((itemId) => {
      const expectedVersion = versions.get(itemId);
      return expectedVersion ? [{ itemId, expectedVersion }] : [];
    });
    if (items.length !== this.itemIds.length) {
      this.selectionStale = true;
      return false;
    }
    this.saving = true;
    this.error = null;
    try {
      await bulkSetMusicReviewState({ items, reviewState, deferredUntil: null, updatedAt: this.now() });
      await this.library.refreshAfterMutation();
      return true;
    } catch (error) {
      this.error = error instanceof Error ? error.message : String(error);
      return false;
    } finally {
      this.saving = false;
    }
  }

  async snooze(scope: MusicSnoozeScope, playlistId: string | null, endsAt: number | null): Promise<boolean> {
    if (this.saving || this.itemIds.length === 0) return false;
    const now = this.now();
    this.saving = true;
    this.error = null;
    try {
      await bulkSnoozeMusicItems({
        actionId: this.id(), itemIds: this.itemIds, scope, playlistId,
        startsAt: now, endsAt, reason: "", createdAt: now,
      });
      await this.library.refreshAfterMutation();
      return true;
    } catch (error) {
      this.error = error instanceof Error ? error.message : String(error);
      return false;
    } finally {
      this.saving = false;
    }
  }

  clear(): void {
    this.itemIds = [];
    this.states = {};
    this.initialCounts = {};
    this.search = "";
    this.error = null;
    this.selectionStale = false;
  }

  private async persist(edit: {
    addPlaylistIds: string[];
    removePlaylistIds: string[];
    weightPlaylistIds: string[];
    weight: MusicWeight | null;
  }): Promise<boolean> {
    if (this.saving || this.itemIds.length === 0) return false;
    this.saving = true;
    this.error = null;
    try {
      await bulkEditMusicMemberships({
        actionId: this.id(),
        itemIds: this.itemIds,
        ...edit,
        updatedAt: this.now(),
      });
      await this.library.refreshAfterMutation();
      return true;
    } catch (error) {
      this.error = error instanceof Error ? error.message : String(error);
      return false;
    } finally {
      this.saving = false;
    }
  }
}

export function createMusicBulkEditController(library: MusicLibraryController): MusicBulkEditController {
  return new MusicBulkEditController(library);
}
