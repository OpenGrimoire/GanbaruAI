import { afterEach, describe, expect, it, vi } from "vitest";
import { setActiveVaultIdentity } from "$lib/vault/active-vault";
import {
  assetUrlCacheKey,
  ByteAwareLruCache,
  clearAssetUrlCache,
  invalidateAssetUrl,
  loadAssetUrl,
} from "./asset-url-cache";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

function stringCache(maxEntries = 2, maxBytes = 10) {
  return new ByteAwareLruCache<string, string>({
    maxEntries,
    maxBytes,
    sizeOf: (value) => value.length,
  });
}

afterEach(() => {
  clearAssetUrlCache();
  setActiveVaultIdentity(null);
});

describe("byte-aware asset URL cache", () => {
  it("enforces independent entry and encoded-byte bounds", async () => {
    const entryBounded = stringCache(2, 100);
    await entryBounded.load("a", async () => "a");
    await entryBounded.load("b", async () => "b");
    await entryBounded.load("c", async () => "c");
    expect(entryBounded.stats()).toEqual({ entries: 2, bytes: 2, inFlight: 0 });
    expect(entryBounded.get("a")).toBeUndefined();

    const byteBounded = stringCache(10, 5);
    await byteBounded.load("a", async () => "aaa");
    await byteBounded.load("b", async () => "bbb");
    expect(byteBounded.get("a")).toBeUndefined();
    expect(byteBounded.stats()).toEqual({ entries: 1, bytes: 3, inFlight: 0 });

    await byteBounded.load("oversized", async () => "123456");
    expect(byteBounded.get("oversized")).toBeUndefined();
    expect(byteBounded.stats().bytes).toBeLessThanOrEqual(5);
  });

  it("evicts the least recently used entry", async () => {
    const cache = stringCache(2, 100);
    await cache.load("a", async () => "A");
    await cache.load("b", async () => "B");
    expect(cache.get("a")).toBe("A");
    await cache.load("c", async () => "C");

    expect(cache.get("a")).toBe("A");
    expect(cache.get("b")).toBeUndefined();
    expect(cache.get("c")).toBe("C");
  });

  it("deduplicates concurrent loads", async () => {
    const cache = stringCache();
    const result = deferred<string>();
    const loader = vi.fn(() => result.promise);

    const first = cache.load("asset", loader);
    const second = cache.load("asset", loader);
    await Promise.resolve();
    expect(loader).toHaveBeenCalledOnce();

    result.resolve("data:image/png;base64,AA==");
    await expect(first).resolves.toBe("data:image/png;base64,AA==");
    await expect(second).resolves.toBe("data:image/png;base64,AA==");
    expect(loader).toHaveBeenCalledOnce();
  });

  it("does not cache failures and retries the next request", async () => {
    const cache = stringCache();
    const loader = vi.fn()
      .mockRejectedValueOnce(new Error("read failed"))
      .mockResolvedValue("recovered");

    await expect(cache.load("asset", loader)).rejects.toThrow("read failed");
    await expect(cache.load("asset", loader)).resolves.toBe("recovered");
    expect(loader).toHaveBeenCalledTimes(2);
  });

  it("invalidates cached and in-flight values without retaining stale results", async () => {
    const cache = stringCache(4, 100);
    await cache.load("kept", async () => "kept");
    const stale = deferred<string>();
    const staleLoad = cache.load("stale", () => stale.promise);
    await Promise.resolve();

    cache.invalidate("stale");
    stale.resolve("old");
    await expect(staleLoad).resolves.toBe("old");
    expect(cache.get("stale")).toBeUndefined();
    expect(cache.get("kept")).toBe("kept");

    cache.clear();
    expect(cache.stats()).toEqual({ entries: 0, bytes: 0, inFlight: 0 });
  });

  it("isolates identical paths by vault and clears on an active-vault switch", async () => {
    expect(assetUrlCacheKey("vault-a", "notes-file", "notes/files/a.png"))
      .not.toBe(assetUrlCacheKey("vault-b", "notes-file", "notes/files/a.png"));
    const loaderA = vi.fn(async () => "vault-a-url");
    const loaderB = vi.fn(async () => "vault-b-url");

    setActiveVaultIdentity("vault-a");
    await expect(loadAssetUrl("notes-file", "notes/files/a.png", loaderA))
      .resolves.toBe("vault-a-url");
    await loadAssetUrl("notes-file", "notes/files/a.png", loaderA);
    expect(loaderA).toHaveBeenCalledOnce();

    setActiveVaultIdentity("vault-b");
    await expect(loadAssetUrl("notes-file", "notes/files/a.png", loaderB))
      .resolves.toBe("vault-b-url");
    expect(loaderB).toHaveBeenCalledOnce();
  });

  it("rejects an in-flight URL returned after its vault becomes inactive", async () => {
    const pending = deferred<string>();
    setActiveVaultIdentity("vault-a");
    const load = loadAssetUrl("notes-file", "notes/files/a.png", () => pending.promise);
    await Promise.resolve();

    setActiveVaultIdentity("vault-b");
    pending.resolve("stale-vault-a-url");

    await expect(load).rejects.toThrow("inactive vault");
  });

  it("supports targeted invalidation without clearing unrelated asset kinds", async () => {
    setActiveVaultIdentity("vault-a");
    const fileLoader = vi.fn(async () => "file-url");
    const iconLoader = vi.fn(async () => "icon-url");
    await loadAssetUrl("notes-file", "notes/files/a.png", fileLoader);
    await loadAssetUrl("notes-page-icon", "notes/page-icons/a.png", iconLoader);

    invalidateAssetUrl("notes-file", "notes/files/a.png");
    await loadAssetUrl("notes-file", "notes/files/a.png", fileLoader);
    await loadAssetUrl("notes-page-icon", "notes/page-icons/a.png", iconLoader);

    expect(fileLoader).toHaveBeenCalledTimes(2);
    expect(iconLoader).toHaveBeenCalledOnce();
  });
});
