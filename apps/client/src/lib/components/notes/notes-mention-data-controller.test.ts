import { describe, expect, it, vi } from "vitest";
import { createNotesMentionDataController } from "./notes-mention-data-controller.svelte";
import type { NotesDatabaseMentionData } from "./notes-block-mention-targets";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((next) => { resolve = next; });
  return { promise, resolve };
}

describe("Notes mention data controller", () => {
  it("ignores an older aggregate result after a newer reload finishes", async () => {
    const first = deferred<NotesDatabaseMentionData>();
    const second = deferred<NotesDatabaseMentionData>();
    const load = vi.fn()
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const controller = createNotesMentionDataController(load);

    const firstReload = controller.reload();
    const secondReload = controller.reload();
    second.resolve({ dataSources: [], rowPages: [] });
    await secondReload;
    first.resolve({
      dataSources: [],
      rowPages: [{ id: "stale" } as NotesDatabaseMentionData["rowPages"][number]],
    });
    await firstReload;

    expect(controller.rowPages).toEqual([]);
  });

  it("clears current data when the latest aggregate load fails", async () => {
    const successful: NotesDatabaseMentionData = { dataSources: [], rowPages: [] };
    const load = vi.fn()
      .mockResolvedValueOnce(successful)
      .mockRejectedValueOnce(new Error("failed"));
    const controller = createNotesMentionDataController(load);
    await controller.reload();
    await controller.reload();
    expect(controller.dataSources).toEqual([]);
    expect(controller.rowPages).toEqual([]);
  });
});
