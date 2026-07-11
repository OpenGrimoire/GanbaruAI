import { describe, expect, it, vi } from "vitest";
import { createLocaleCatalogLoader, type RuntimeMessageCatalog } from "./catalog-loader";
import { en } from "./messages/en";

function deferredCatalog() {
  let resolve!: (catalog: RuntimeMessageCatalog) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<RuntimeMessageCatalog>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

describe("locale catalog loader", () => {
  it("keeps English resident and single-flights a non-default catalog", async () => {
    const pending = deferredCatalog();
    const importSpanish = vi.fn(() => pending.promise);
    const loader = createLocaleCatalogLoader({ es: importSpanish });

    await expect(loader.load("en")).resolves.toBe(en);
    expect(importSpanish).not.toHaveBeenCalled();

    const first = loader.load("es");
    const second = loader.load("es");
    expect(second).toBe(first);
    expect(importSpanish).toHaveBeenCalledTimes(1);
    expect(loader.hasLoaded("es")).toBe(false);

    pending.resolve(en);
    await expect(first).resolves.toBe(en);
    await expect(loader.load("es")).resolves.toBe(en);
    expect(importSpanish).toHaveBeenCalledTimes(1);
    expect(loader.hasLoaded("es")).toBe(true);
  });

  it("does not cache a failed catalog promise", async () => {
    const importSpanish = vi.fn()
      .mockRejectedValueOnce(new Error("chunk unavailable"))
      .mockResolvedValueOnce(en);
    const loader = createLocaleCatalogLoader({ es: importSpanish });

    await expect(loader.load("es")).rejects.toThrow("chunk unavailable");
    await expect(loader.load("es")).resolves.toBe(en);
    expect(importSpanish).toHaveBeenCalledTimes(2);
  });
});
