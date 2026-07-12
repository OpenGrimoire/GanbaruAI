import { describe, expect, it, vi } from "vitest";
import type { NotesBlock } from "$lib/notes/types";

const backend = vi.hoisted(() => {
  let resolveHydration: (blocks: NotesBlock[]) => void = () => {};
  return {
    hydrateNotesBlocks: vi.fn(() => new Promise<NotesBlock[]>((resolve) => {
      resolveHydration = resolve;
    })),
    resolve(blocks: NotesBlock[]): void {
      resolveHydration(blocks);
    },
  };
});

vi.mock("$lib/api/notes", () => ({
  getNotesBlockFrontier: vi.fn(),
  getNotesBlockOutlineFrontier: vi.fn(),
  hydrateNotesBlocks: backend.hydrateNotesBlocks,
}));

describe("Notes hydration controller", () => {
  it("does not apply hydration from a stale page generation", async () => {
    const { createNotesHydrationController } = await import("./notes-store-hydration");
    let generation = 1;
    let pageId: string | null = "page-a";
    const replaceHydratedBlocks = vi.fn();
    const controller = createNotesHydrationController({
      readPageGeneration: () => generation,
      readSelectedPageId: () => pageId,
      readBlockOutlines: () => [{
        id: "block-a",
        page_id: "page-a",
        parent: { type: "page_id", page_id: "page-a" },
        type: "paragraph",
        has_children: false,
        sort_order: 1_000,
        retained_height: 36,
      }],
      readFlatBlockOutlines: () => [],
      readBlocksById: () => ({}),
      readFocusRequest: () => ({ blockId: null, requestId: 0, selection: null }),
      mergeBlockOutlines: vi.fn(),
      replaceHydratedBlocks,
      setLoadError: vi.fn(),
      reloadOpenComments: vi.fn(),
    });

    const hydration = controller.hydrateBlockRange(["block-a"], generation);
    generation = 2;
    pageId = "page-b";
    backend.resolve([]);
    await hydration;

    expect(replaceHydratedBlocks).not.toHaveBeenCalled();
  });
});
