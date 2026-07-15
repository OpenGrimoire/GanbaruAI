import { getMusicInspectorDetail, saveMusicAdvancedMembership, setMusicItemSignals, setMusicMetadataOverrides } from "$lib/api/music-library";
import type { MusicInspectorDetail, MusicItemSignal, MusicMembershipSkipRange, MusicPlaylistMembership } from "$lib/music/library-contracts";

export interface MusicBuilderInspectorApi {
  detail(itemId: string): Promise<MusicInspectorDetail>;
  setSignals(itemId: string, signals: MusicItemSignal[], updatedAt: number): Promise<number>;
}

const defaultApi: MusicBuilderInspectorApi = {
  detail: getMusicInspectorDetail,
  async setSignals(itemId, signals, updatedAt) {
    const [receipt] = await setMusicItemSignals({ itemIds: [itemId], signals, updatedAt });
    if (!receipt) throw new Error("The signal update did not return a receipt.");
    return receipt.version;
  },
};

export class MusicBuilderInspectorController {
  itemId = $state<string | null>(null);
  detail = $state<MusicInspectorDetail | null>(null);
  busy = $state(false);
  error = $state<Error | null>(null);
  saving = $state(false);
  expandedSections = $state<Set<string>>(new Set(["details", "memberships"]));
  signalUndo = $state<MusicItemSignal[] | null>(null);

  private generation = 0;
  private readonly api: MusicBuilderInspectorApi;

  constructor(api: MusicBuilderInspectorApi = defaultApi) {
    this.api = api;
  }

  async select(itemId: string | null): Promise<boolean> {
    if (!itemId) {
      this.clear();
      return true;
    }
    if (this.itemId === itemId && this.detail) return true;
    const generation = ++this.generation;
    this.itemId = itemId;
    this.busy = true;
    this.error = null;
    try {
      const detail = await this.api.detail(itemId);
      if (generation !== this.generation || this.itemId !== itemId) return false;
      this.detail = detail;
      return true;
    } catch (error) {
      if (generation !== this.generation || this.itemId !== itemId) return false;
      this.detail = null;
      this.error = error instanceof Error ? error : new Error(String(error));
      return false;
    } finally {
      if (generation === this.generation) this.busy = false;
    }
  }

  clear(): void {
    this.generation += 1;
    this.itemId = null;
    this.detail = null;
    this.error = null;
    this.busy = false;
  }

  toggleSection(section: string): void {
    const next = new Set(this.expandedSections);
    if (next.has(section)) next.delete(section);
    else next.add(section);
    this.expandedSections = next;
  }

  async saveMetadataOverrides(overrides: {
    titleOverride: string | null;
    artistOverride: string | null;
    albumOverride: string | null;
    artworkOverride: string | null;
  }): Promise<boolean> {
    const detail = this.detail;
    if (!detail || this.saving) return false;
    this.saving = true;
    this.error = null;
    try {
      const receipt = await setMusicMetadataOverrides({
        itemId: detail.item.id,
        ...overrides,
        expectedVersion: detail.item.version,
        updatedAt: Date.now(),
      });
      Object.assign(detail.item, overrides, { version: receipt.version });
      return true;
    } catch (error) {
      this.error = error instanceof Error ? error : new Error(String(error));
      return false;
    } finally {
      this.saving = false;
    }
  }

  async saveSignals(signals: MusicItemSignal[]): Promise<boolean> {
    const detail = this.detail;
    if (!detail || this.saving) return false;
    const previousSignals = [...detail.signals];
    this.saving = true;
    this.error = null;
    detail.signals = [...signals];
    try {
      detail.item.version = await this.api.setSignals(detail.item.id, signals, Date.now());
      this.signalUndo = previousSignals;
      return true;
    } catch (error) {
      detail.signals = previousSignals;
      this.error = error instanceof Error ? error : new Error(String(error));
      return false;
    } finally {
      this.saving = false;
    }
  }

  async undoSignals(): Promise<boolean> {
    const detail = this.detail;
    const previous = this.signalUndo;
    if (!detail || !previous || this.saving) return false;
    this.saving = true;
    this.error = null;
    try {
      detail.item.version = await this.api.setSignals(detail.item.id, previous, Date.now());
      detail.signals = [...previous];
      this.signalUndo = null;
      return true;
    } catch (error) {
      this.error = error instanceof Error ? error : new Error(String(error));
      return false;
    } finally {
      this.saving = false;
    }
  }

  async saveAdvancedMembership(
    membership: MusicPlaylistMembership,
    skipRanges: MusicMembershipSkipRange[],
  ): Promise<boolean> {
    if (this.saving) return false;
    this.saving = true;
    this.error = null;
    try {
      const receipt = await saveMusicAdvancedMembership({
        membership: { ...membership, expectedVersion: membership.version, updatedAt: Date.now() },
        skipRanges,
      });
      membership.version = receipt.version;
      if (this.detail) {
        this.detail.membershipSkipRanges = [
          ...this.detail.membershipSkipRanges.filter((range) => range.membershipId !== membership.id),
          ...skipRanges,
        ];
      }
      return true;
    } catch (error) {
      this.error = error instanceof Error ? error : new Error(String(error));
      return false;
    } finally {
      this.saving = false;
    }
  }
}

export function createMusicBuilderInspectorController(
  api: MusicBuilderInspectorApi = defaultApi,
): MusicBuilderInspectorController {
  return new MusicBuilderInspectorController(api);
}
