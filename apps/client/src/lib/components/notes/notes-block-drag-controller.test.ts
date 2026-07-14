// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import { getActiveNotesBlockDragId } from "$lib/notes/block-drag";
import { createNotesBlockDragController } from "./notes-block-drag-controller.svelte";

describe("Notes block drag controller", () => {
  it("clears local and shared drag state after drag end", () => {
    const setData = vi.fn();
    const controller = createNotesBlockDragController({
      readTreeState: () => ({ blocksById: {}, childIdsByParentId: {} }),
      dropBlock: vi.fn(),
    });
    controller.start("block-1", {
      dataTransfer: { effectAllowed: "none", setData },
    } as unknown as DragEvent);
    expect(controller.draggingBlockId).toBe("block-1");
    expect(getActiveNotesBlockDragId()).toBe("block-1");
    controller.end();
    expect(controller.draggingBlockId).toBeNull();
    expect(getActiveNotesBlockDragId()).toBeNull();
  });
});
