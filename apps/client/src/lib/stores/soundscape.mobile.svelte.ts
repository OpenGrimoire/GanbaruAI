import type {
  MusicSoundscapeDefinition,
  MusicSoundscapeSnapshot,
  MusicSoundscapeState,
  MusicSoundscapeWrite,
} from "$lib/music/soundscape-contracts";

class MobileSoundscapeStore {
  definitions = $state<MusicSoundscapeDefinition[]>([]);
  persisted = $state<MusicSoundscapeState | null>(null);
  snapshot = $state<MusicSoundscapeSnapshot>({
    status: "idle",
    sourceId: null,
    volume: 0.35,
    errorCode: null,
  });
  loading = $state(false);
  saving = $state(false);
  error = $state<string | null>(null);
  deviceId = $state<string | null>(null);

  get activeDefinition(): MusicSoundscapeDefinition | null {
    return null;
  }

  async initialize(): Promise<void> {}
  async play(_id: string, _persist = true): Promise<void> {}
  async pause(): Promise<void> {}
  async resume(): Promise<void> {}
  async stop(): Promise<void> {}
  async setVolume(_volume: number): Promise<void> {}
  async recover(): Promise<void> {}
  async switchVault(_nextVaultId: string | null): Promise<void> {}
  async saveDefinition(_write: Omit<MusicSoundscapeWrite, "deviceId">): Promise<void> {}
  async removeDefinition(_definition: MusicSoundscapeDefinition): Promise<void> {}
}

let instance: MobileSoundscapeStore | null = null;

/** Return the inert soundscape store used on platforms without a soundscape engine. */
export function getSoundscapeStore(): MobileSoundscapeStore {
  instance ??= new MobileSoundscapeStore();
  return instance;
}
