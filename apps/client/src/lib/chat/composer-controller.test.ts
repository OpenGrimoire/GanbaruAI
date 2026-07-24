import { describe, expect, it, vi } from "vitest";
import type { ChatDraftRead } from "./contracts";
import { ChatComposerController, chatDraftId, type ChatDraftApi } from "./composer-controller";

const timestamp = "2026-07-21T12:00:00Z";

function draft(overrides: Partial<ChatDraftRead> = {}): ChatDraftRead {
  return {
    id: chatDraftId("workspace-1", null),
    workspaceId: "workspace-1",
    threadId: null,
    text: "Saved prompt",
    attachmentIds: [],
    mentions: { schemaVersion: 1, value: [] },
    providerInstanceId: "codex-personal",
    modelSelection: null,
    safetyMode: "ask_for_approval",
    interactionMode: "build",
    sentSnapshot: null,
    updatedAt: timestamp,
    ...overrides,
  };
}

function fakeApi(overrides: Partial<ChatDraftApi> = {}): ChatDraftApi {
  return {
    read: vi.fn(async () => null),
    save: vi.fn(async (value) => draft({ ...value, updatedAt: timestamp })),
    delete: vi.fn(async () => true),
    ...overrides,
  };
}

describe("ChatComposerController", () => {
  it("hydrates the same draft for the hero and docked composer", async () => {
    const api = fakeApi({ read: vi.fn(async () => draft()) });
    const controller = new ChatComposerController(api);

    await controller.bind("workspace-1", null);

    expect(controller.snapshot()).toMatchObject({
      text: "Saved prompt",
      providerInstanceId: "codex-personal",
      dirty: false,
    });
  });

  it("debounces edits and flushes before changing conversations", async () => {
    vi.useFakeTimers();
    const api = fakeApi();
    const controller = new ChatComposerController(api, 100);
    await controller.bind("workspace-1", null);
    controller.setText("First prompt");
    controller.setText("Latest prompt");

    await vi.advanceTimersByTimeAsync(99);
    expect(api.save).not.toHaveBeenCalled();
    const bind = controller.bind("workspace-1", "thread-2");
    await bind;

    expect(api.save).toHaveBeenCalledTimes(1);
    expect(api.save).toHaveBeenCalledWith(expect.objectContaining({ text: "Latest prompt" }));
    expect(controller.snapshot().threadId).toBe("thread-2");
    vi.useRealTimers();
  });

  it("keeps a recoverable sent snapshot when clearing submitted content", async () => {
    const api = fakeApi();
    const controller = new ChatComposerController(api);
    await controller.bind("workspace-1", null);
    controller.setText("Please update the calendar");
    controller.setAttachments(["attachment-1"]);
    controller.setMentions([{ relativePath: "src/calendar.ts", kind: "file", ignored: false }]);

    controller.markSent();
    await controller.flush();

    const saved = vi.mocked(api.save).mock.calls.at(-1)?.[0];
    expect(saved).toMatchObject({ text: "", mentions: { value: [] } });
    expect(saved?.sentSnapshot?.value).toMatchObject({
      text: "Please update the calendar",
      attachmentIds: ["attachment-1"],
      mentions: [{ relativePath: "src/calendar.ts", kind: "file", ignored: false }],
    });
    expect(controller.restoreSentSnapshot()).toBe(true);
    expect(controller.snapshot()).toMatchObject({
      text: "Please update the calendar",
      attachmentIds: ["attachment-1"],
      mentions: [{ relativePath: "src/calendar.ts" }],
    });
  });

  it("ignores a stale load after another draft is selected", async () => {
    let resolveFirst: ((value: ChatDraftRead | null) => void) | undefined;
    const first = new Promise<ChatDraftRead | null>((resolve) => { resolveFirst = resolve; });
    const api = fakeApi({
      read: vi.fn()
        .mockReturnValueOnce(first)
        .mockResolvedValueOnce(draft({ id: chatDraftId("workspace-2", null), workspaceId: "workspace-2", text: "New" })),
    });
    const controller = new ChatComposerController(api);

    const stale = controller.bind("workspace-1", null);
    const current = controller.bind("workspace-2", null);
    await current;
    resolveFirst?.(draft({ text: "Stale" }));
    await stale;

    expect(controller.snapshot()).toMatchObject({ workspaceId: "workspace-2", text: "New" });
  });
});
