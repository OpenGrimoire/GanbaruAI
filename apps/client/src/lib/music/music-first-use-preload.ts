import { musicBuilderLoader } from "$lib/music/music-builder-loader";
import { getMusicSourcesController } from "$lib/music/music-sources-controller.svelte";
import {
  onActiveVaultIdentityChange,
  requireActiveVaultIdentity,
} from "$lib/vault/active-vault";

let activeLoad: Promise<void> | null = null;
let queuedVaultId: string | null = null;

async function preloadVault(vaultId: string): Promise<void> {
  const sources = getMusicSourcesController();
  sources.setVault(vaultId);
  await sources.load();
  if (sources.firstUseSession) await musicBuilderLoader.load();
}

/** Starts first-use music discovery at app startup and follows active-vault changes. */
export function startMusicFirstUsePreload(): () => void {
  const start = (vaultId: string): void => {
    queuedVaultId = vaultId;
    if (activeLoad) return;
    activeLoad = (async () => {
      while (queuedVaultId) {
        const nextVaultId = queuedVaultId;
        queuedVaultId = null;
        try {
          await preloadVault(nextVaultId);
        } catch (error) {
          console.warn("Music first-use preload failed", error);
        }
      }
    })().finally(() => {
      activeLoad = null;
    });
  };
  try {
    start(requireActiveVaultIdentity());
  } catch {
    // Startup can mount before vault validation publishes the active identity.
  }
  return onActiveVaultIdentityChange((_previous, next) => {
    if (next) start(next);
  });
}
