import { getMusicInspectorDetail } from "$lib/api/music-library";
import type { MusicInspectorDetail } from "$lib/music/library-contracts";

export interface MusicBuilderInspectorApi {
  detail(itemId: string): Promise<MusicInspectorDetail>;
}

const defaultApi: MusicBuilderInspectorApi = { detail: getMusicInspectorDetail };

export class MusicBuilderInspectorController {
  itemId = $state<string | null>(null);
  detail = $state<MusicInspectorDetail | null>(null);
  busy = $state(false);
  error = $state<Error | null>(null);
  expandedSections = $state<Set<string>>(new Set(["details", "memberships"]));

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
}

export function createMusicBuilderInspectorController(
  api: MusicBuilderInspectorApi = defaultApi,
): MusicBuilderInspectorController {
  return new MusicBuilderInspectorController(api);
}
