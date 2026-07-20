import {
  getDeviceId,
  getMusicSoundscapes,
  getMusicSoundscapeState,
  pauseSoundscape,
  recoverSoundscape,
  removeMusicSoundscape,
  resumeSoundscape,
  setSoundscapeVolume,
  startSoundscape,
  stopSoundscape,
  updateMusicSoundscapeState,
  upsertMusicSoundscape,
} from "$lib/api/soundscape";
import type {
  MusicSoundscapeDefinition,
  MusicSoundscapeSnapshot,
  MusicSoundscapeState,
  MusicSoundscapeWrite,
} from "$lib/music/soundscape-contracts";

class SoundscapeStore {
  definitions = $state<MusicSoundscapeDefinition[]>([]);
  persisted = $state<MusicSoundscapeState | null>(null);
  snapshot = $state<MusicSoundscapeSnapshot>({ status: "idle", sourceId: null, volume: 0.35, errorCode: null });
  loading = $state(false);
  saving = $state(false);
  error = $state<string | null>(null);
  deviceId = $state<string | null>(null);
  private generation = 0;
  private volumeGeneration = 0;
  private volumePersistTimer: number | null = null;
  private stateWriteQueue: Promise<void> = Promise.resolve();

  get activeDefinition(): MusicSoundscapeDefinition | null {
    return this.definitions.find((definition) => definition.id === this.persisted?.activeSoundscapeId) ?? null;
  }

  async initialize(): Promise<void> {
    const generation = ++this.generation;
    this.loading = true;
    this.error = null;
    try {
      const deviceId = await getDeviceId();
      const [definitions, persisted] = await Promise.all([
        getMusicSoundscapes(deviceId),
        getMusicSoundscapeState(),
      ]);
      if (generation !== this.generation) return;
      this.deviceId = deviceId;
      this.definitions = definitions;
      this.persisted = persisted;
      this.snapshot = { status: "idle", sourceId: null, volume: persisted.volume, errorCode: null };
      if (persisted.desiredPlaying) {
        const active = definitions.find((definition) => definition.id === persisted.activeSoundscapeId);
        if (active?.availability === "available") await this.play(active.id, false);
      }
    } catch (error) {
      if (generation === this.generation) this.error = message(error);
    } finally {
      if (generation === this.generation) this.loading = false;
    }
  }

  async play(id: string, persist = true): Promise<void> {
    const definition = this.definitions.find((entry) => entry.id === id);
    if (!definition || definition.availability !== "available") {
      this.error = "This background sound needs to be repaired before it can play.";
      return;
    }
    this.saving = true;
    this.error = null;
    try {
      this.snapshot = await startSoundscape({
        sourceId: definition.id,
        generatedKind: definition.generatedKind,
        localPath: definition.localPath,
        volume: this.persisted?.volume ?? this.snapshot.volume,
      });
      if (persist) await this.persistState(definition.id, true, this.snapshot.volume);
    } catch (error) {
      this.error = message(error);
      this.snapshot = { ...this.snapshot, status: "error", sourceId: id };
    } finally {
      this.saving = false;
    }
  }

  async pause(): Promise<void> {
    try { this.snapshot = await pauseSoundscape(); await this.persistState(this.snapshot.sourceId, false, this.snapshot.volume); }
    catch (error) { this.error = message(error); }
  }

  async resume(): Promise<void> {
    try { this.snapshot = await resumeSoundscape(); await this.persistState(this.snapshot.sourceId, true, this.snapshot.volume); }
    catch (error) { this.error = message(error); }
  }

  async stop(): Promise<void> {
    try { this.snapshot = await stopSoundscape(); await this.persistState(null, false, this.snapshot.volume); }
    catch (error) { this.error = message(error); }
  }

  async setVolume(volume: number): Promise<void> {
    const next = Math.min(1, Math.max(0, volume));
    const generation = ++this.volumeGeneration;
    this.snapshot = { ...this.snapshot, volume: next };
    try {
      const snapshot = await setSoundscapeVolume(next);
      if (generation === this.volumeGeneration) this.snapshot = snapshot;
      if (this.volumePersistTimer !== null) window.clearTimeout(this.volumePersistTimer);
      this.volumePersistTimer = window.setTimeout(() => {
        this.volumePersistTimer = null;
        void this.persistState(this.persisted?.activeSoundscapeId ?? null, this.persisted?.desiredPlaying ?? false, next);
      }, 180);
    } catch (error) { this.error = message(error); }
  }

  async recover(): Promise<void> {
    try { this.snapshot = await recoverSoundscape(); this.error = null; }
    catch (error) { this.error = message(error); }
  }

  async switchVault(nextVaultId: string | null): Promise<void> {
    ++this.generation;
    if (this.volumePersistTimer !== null) {
      window.clearTimeout(this.volumePersistTimer);
      this.volumePersistTimer = null;
    }
    try { this.snapshot = await stopSoundscape(); } catch { this.snapshot = { status: "idle", sourceId: null, volume: 0.35, errorCode: null }; }
    this.definitions = [];
    this.persisted = null;
    this.error = null;
    if (nextVaultId) await this.initialize();
  }

  async saveDefinition(write: Omit<MusicSoundscapeWrite, "deviceId">): Promise<void> {
    if (!this.deviceId) return;
    this.saving = true;
    this.error = null;
    try {
      const saved = await upsertMusicSoundscape({ ...write, deviceId: this.deviceId });
      this.definitions = [...this.definitions.filter((entry) => entry.id !== saved.id), saved]
        .sort((left, right) => left.createdAt - right.createdAt || left.name.localeCompare(right.name));
    } catch (error) { this.error = message(error); throw error; }
    finally { this.saving = false; }
  }

  async removeDefinition(definition: MusicSoundscapeDefinition): Promise<void> {
    await removeMusicSoundscape(definition.id, definition.version);
    if (this.persisted?.activeSoundscapeId === definition.id) await this.stop();
    this.definitions = this.definitions.filter((entry) => entry.id !== definition.id);
  }

  private async persistState(activeSoundscapeId: string | null, desiredPlaying: boolean, volume: number): Promise<void> {
    this.stateWriteQueue = this.stateWriteQueue.catch(() => undefined).then(async () => {
      const current = this.persisted;
      if (!current) return;
      this.persisted = await updateMusicSoundscapeState({
        activeSoundscapeId,
        desiredPlaying,
        volume,
        expectedVersion: current.version,
        updatedAt: Date.now(),
      });
    });
    await this.stateWriteQueue;
  }
}

function message(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error && typeof error.message === "string") return error.message;
  return error instanceof Error ? error.message : String(error);
}

let instance: SoundscapeStore | null = null;
export function getSoundscapeStore(): SoundscapeStore {
  instance ??= new SoundscapeStore();
  return instance;
}
