import { beforeEach, describe, expect, it, vi } from "vitest";

const { loadArtworkDataUrl } = vi.hoisted(() => ({
  loadArtworkDataUrl: vi.fn(async (path: string) => `data:image/mock,${path}`),
}));

vi.mock("$lib/api/music", () => ({ loadArtworkDataUrl }));

import { clearMusicArtworkCache, musicArtworkDataUrl } from "./music-artwork-cache";

describe("music artwork cache", () => {
  beforeEach(() => {
    clearMusicArtworkCache();
    loadArtworkDataUrl.mockClear();
  });

  it("retains at most 96 artwork requests and evicts the least recently used entry", async () => {
    for (let index = 0; index < 97; index += 1) {
      await musicArtworkDataUrl(`/artwork/${index}.jpg`);
    }
    expect(loadArtworkDataUrl).toHaveBeenCalledTimes(97);

    await musicArtworkDataUrl("/artwork/96.jpg");
    expect(loadArtworkDataUrl).toHaveBeenCalledTimes(97);

    await musicArtworkDataUrl("/artwork/0.jpg");
    expect(loadArtworkDataUrl).toHaveBeenCalledTimes(98);
  });
});
