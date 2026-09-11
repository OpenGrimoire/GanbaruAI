import { describe, expect, it, vi } from "vitest";
import { createBlockWrite } from "$lib/notes/block-factory";
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
  it("does not publish a replacement when requested blocks are already loaded", async () => {
    const { createNotesHydrationController } = await import("./notes-store-hydration");
    const write = createBlockWrite("block-a", "paragraph");
    if (write.type !== "paragraph") throw new Error("expected paragraph test block");
    const block: NotesBlock = {
      object: "block",
      id: write.id,
      parent: { type: "page_id", page_id: "page-a" },
      created_time: "2026-07-12T00:00:00.000Z",
      last_edited_time: "2026-07-12T00:00:00.000Z",
      has_children: false,
      in_trash: false,
      archived: false,
      source_provider: null,
      source_object_id: null,
      source_last_edited_time: null,
      type: "paragraph",
      paragraph: write.paragraph,
    };
    const replaceHydratedBlocks = vi.fn();
    const reloadOpenComments = vi.fn();
    const controller = createNotesHydrationController({
      readPageGeneration: () => 1,
      readSelectedPageId: () => "page-a",
      readBlockOutlines: () => [{
        id: block.id,
        page_id: "page-a",
        parent: { type: "page_id", page_id: "page-a" },
        type: block.type,
        has_children: false,
        sort_order: 1_000,
        retained_height: 36,
      }],
      readFlatBlockOutlines: () => [],
      readBlocksById: () => ({ [block.id]: block }),
      readFocusRequest: () => ({ blockId: null, requestId: 0, selection: null }),
      mergeBlockOutlines: vi.fn(),
      replaceHydratedBlocks,
      setLoadError: vi.fn(),
      reloadOpenComments,
    });

    await controller.hydrateBlockRange([block.id], 1);

    expect(backend.hydrateNotesBlocks).not.toHaveBeenCalled();
    expect(replaceHydratedBlocks).not.toHaveBeenCalled();
    expect(reloadOpenComments).not.toHaveBeenCalled();
  });

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
