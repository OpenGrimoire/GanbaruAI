import { loadArtworkDataUrl } from "$lib/api/music";

const MAX_CACHED_ARTWORK = 96;
const artworkCache = new Map<string, Promise<string | null>>();

/** Loads a validated artwork image once and bounds retained data URLs. */
export function musicArtworkDataUrl(path: string): Promise<string | null> {
  const cached = artworkCache.get(path);
  if (cached) {
    artworkCache.delete(path);
    artworkCache.set(path, cached);
    return cached;
  }
  const request = loadArtworkDataUrl(path).catch(() => null);
  artworkCache.set(path, request);
  while (artworkCache.size > MAX_CACHED_ARTWORK) {
    const oldest = artworkCache.keys().next().value;
    if (typeof oldest !== "string") break;
    artworkCache.delete(oldest);
  }
  return request;
}

export function clearMusicArtworkCache(): void {
  artworkCache.clear();
}

export function invalidateMusicArtwork(path: string): void {
  artworkCache.delete(path);
}
