import { loadArtworkDataUrl, loadEmbeddedArtworkDataUrl } from "$lib/api/music";

const MAX_CACHED_ARTWORK = 96;
const MAX_CACHED_EMBEDDED_ARTWORK = 12;
const artworkCache = new Map<string, Promise<string | null>>();
const embeddedArtworkCache = new Map<string, Promise<string | null>>();

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
  embeddedArtworkCache.clear();
}

/** Extracts embedded artwork once and bounds retained builder preview data URLs. */
export function musicEmbeddedArtworkDataUrl(path: string, identity = path): Promise<string | null> {
  const cached = embeddedArtworkCache.get(identity);
  if (cached) {
    embeddedArtworkCache.delete(identity);
    embeddedArtworkCache.set(identity, cached);
    return cached;
  }
  const request = loadEmbeddedArtworkDataUrl(path).catch(() => null);
  embeddedArtworkCache.set(identity, request);
  while (embeddedArtworkCache.size > MAX_CACHED_EMBEDDED_ARTWORK) {
    const oldest = embeddedArtworkCache.keys().next().value;
    if (typeof oldest !== "string") break;
    embeddedArtworkCache.delete(oldest);
  }
  return request;
}

export function invalidateMusicArtwork(path: string): void {
  artworkCache.delete(path);
}
