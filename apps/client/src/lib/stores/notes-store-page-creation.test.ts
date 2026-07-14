import { beforeEach, describe, expect, it, vi } from "vitest";
import type { NotesLoadedPage, NotesPageCreate } from "$lib/notes/types";

const backend = vi.hoisted(() => ({
  createNotesPage: vi.fn<(request: NotesPageCreate) => Promise<NotesLoadedPage>>(),
}));

vi.mock("$lib/api/notes", () => ({
  createNotesPage: backend.createNotesPage,
}));

const request: NotesPageCreate = {
  id: "11111111-1111-4111-8111-111111111111",
  title: "",
  parent: { type: "workspace", workspace: true },
  folder_id: null,
  first_block_id: "22222222-2222-4222-8222-222222222222",
};

const loaded = { page: { id: request.id } } as NotesLoadedPage;

describe("Notes page creation controller", () => {
  beforeEach(() => backend.createNotesPage.mockReset());

  it("gates mutations until creation reconciles", async () => {
    let resolveCreate: (value: NotesLoadedPage) => void = () => {};
    backend.createNotesPage.mockReturnValue(new Promise((resolve) => {
      resolveCreate = resolve;
    }));
    const reconcile = vi.fn<() => Promise<void>>().mockResolvedValue();
    const { createNotesPageCreationController } = await import("./notes-store-page-creation.svelte");
    const controller = createNotesPageCreationController({ reconcile });

    controller.begin(request);
    const ready = controller.awaitReady(request.id);
    expect(controller.isPending(request.id)).toBe(true);
    expect(reconcile).not.toHaveBeenCalled();

    resolveCreate(loaded);
    await ready;

    expect(reconcile).toHaveBeenCalledWith(loaded);
    expect(controller.isPending(request.id)).toBe(false);
  });

  it("releases buffered mutations after persistence completes", async () => {
    backend.createNotesPage.mockResolvedValue(loaded);
    const afterPersisted = vi.fn<() => Promise<void>>().mockResolvedValue();
    const { createNotesPageCreationController } = await import("./notes-store-page-creation.svelte");
    const controller = createNotesPageCreationController({
      reconcile: () => Promise.resolve(),
      afterPersisted,
    });

    controller.begin(request);
    await vi.waitFor(() => expect(afterPersisted).toHaveBeenCalledWith(request.id));
  });

  it("keeps a failed draft and retries the same request", async () => {
    backend.createNotesPage
      .mockRejectedValueOnce(new Error("disk full"))
      .mockResolvedValueOnce(loaded);
    const reconcile = vi.fn<() => Promise<void>>().mockResolvedValue();
    const { createNotesPageCreationController } = await import("./notes-store-page-creation.svelte");
    const controller = createNotesPageCreationController({ reconcile });

    controller.begin(request);
    let mutationReleased = false;
    const bufferedMutation = controller.awaitReady(request.id).then(() => {
      mutationReleased = true;
    });
    await vi.waitFor(() => expect(controller.errorFor(request.id)).toBe("disk full"));
    expect(controller.isPending(request.id)).toBe(true);
    expect(mutationReleased).toBe(false);

    controller.retry(request.id);
    await vi.waitFor(() => expect(controller.isPending(request.id)).toBe(false));
    await bufferedMutation;

    expect(backend.createNotesPage).toHaveBeenNthCalledWith(2, request);
    expect(reconcile).toHaveBeenCalledWith(loaded);
    expect(mutationReleased).toBe(true);
  });
});
