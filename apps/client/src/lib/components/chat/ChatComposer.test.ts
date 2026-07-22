// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ChatComposerSnapshot } from "$lib/chat/composer-controller";
import type { ChatAttachmentRead } from "$lib/chat/contracts";
import { getChat } from "$lib/stores/chat.svelte";
import ChatComposer from "./ChatComposer.svelte";

const api = vi.hoisted(() => ({
  attachmentUrl: vi.fn(async () => "data:image/png;base64,iVBORw0KGgo="),
}));

vi.mock("$lib/api/chat", async (importOriginal) => ({
  ...await importOriginal<typeof import("$lib/api/chat")>(),
  chatAttachmentDataUrl: api.attachmentUrl,
  hasChatFullAccessTrust: vi.fn(async () => false),
  listChatPromptCatalog: vi.fn(async () => []),
  searchChatWorkspacePaths: vi.fn(async () => ({ entries: [], nextCursor: null })),
}));

describe("ChatComposer", () => {
  const mounted: { target: HTMLDivElement; component: ReturnType<typeof mount> }[] = [];

  beforeEach(() => {
    const chat = getChat();
    chat.composer = composer();
    chat.composerAttachments = [];
    chat.interaction = null;
    chat.sendError = null;
    chat.settings = null;
    chat.activeThreads = [];
    chat.archivedThreads = [];
    chat.selectedThreadId = null;
    chat.selectedWorkspaceId = "workspace-1";
    chat.timelinePages = [];
    api.attachmentUrl.mockClear();
  });

  afterEach(async () => {
    while (mounted.length > 0) {
      const entry = mounted.pop();
      if (!entry) continue;
      await unmount(entry.component);
      entry.target.remove();
    }
    vi.restoreAllMocks();
  });

  function setup(hero = false): { target: HTMLDivElement; textarea: HTMLTextAreaElement } {
    const target = document.createElement("div");
    document.body.append(target);
    const component = mount(ChatComposer, { target, props: { hero } });
    mounted.push({ target, component });
    const textarea = target.querySelector("textarea[data-chat-composer]");
    if (!(textarea instanceof HTMLTextAreaElement)) throw new Error("Chat composer did not render");
    return { target, textarea };
  }

  it("restores focus and selection when the shared composer changes layout", async () => {
    const first = setup(true);
    first.textarea.focus();
    first.textarea.setSelectionRange(3, 7);
    first.textarea.dispatchEvent(new Event("select", { bubbles: true }));
    const firstMount = mounted.pop();
    if (!firstMount) throw new Error("First composer mount was not recorded");
    await unmount(firstMount.component);
    firstMount.target.remove();

    const second = setup(false);
    await tick();
    await tick();
    expect(document.activeElement).toBe(second.textarea);
    expect([second.textarea.selectionStart, second.textarea.selectionEnd]).toEqual([3, 7]);
  });

  it("keeps direct provider and model controls discoverable with compact secondary controls", () => {
    const { target } = setup(false);
    expect(target.querySelector('[data-chat-field="provider"]')).not.toBeNull();
    expect(target.querySelector('[data-chat-field="model"]')).not.toBeNull();
    expect(target.querySelector(".compact-controls summary")?.textContent).toContain("Safety");
    expect(target.querySelector("button.primary-action")?.getAttribute("aria-label")).toBe("Send");
  });

  it("opens a managed image preview and preserves a failed import error", async () => {
    const chat = getChat();
    const attachment = imageAttachment();
    chat.composer = { ...composer(), attachmentIds: [attachment.id] };
    chat.composerAttachments = [attachment];
    const importImages = vi.spyOn(chat, "importComposerImages").mockRejectedValue(new Error("Image signature is invalid"));
    const { target } = setup(false);
    await tick();
    await Promise.resolve();
    await tick();
    const previewTrigger = target.querySelector<HTMLButtonElement>(".attachment-preview");
    previewTrigger?.focus();
    previewTrigger?.click();
    await tick();
    await Promise.resolve();
    await tick();
    const previewDialog = target.querySelector<HTMLElement>('[role="dialog"]');
    expect(previewDialog?.textContent).toContain("diagram.png");
    expect(previewDialog?.contains(document.activeElement)).toBe(true);
    previewDialog?.querySelector<HTMLButtonElement>("header button")?.click();
    await tick();
    await Promise.resolve();
    expect(document.activeElement).toBe(previewTrigger);

    const fileInput = target.querySelector<HTMLInputElement>('input[type="file"]');
    if (!fileInput) throw new Error("Image input did not render");
    const file = new File([new Uint8Array([1, 2, 3])], "broken.png", { type: "image/png" });
    Object.defineProperty(fileInput, "files", { configurable: true, value: [file] });
    fileInput.dispatchEvent(new Event("change", { bubbles: true }));
    await tick();
    await Promise.resolve();
    await tick();
    expect(importImages).toHaveBeenCalledWith([file]);
    expect(target.querySelector('[role="alert"]')?.textContent).toContain("Image signature is invalid");
  });
});

function composer(): ChatComposerSnapshot {
  return {
    draftId: "workspace:workspace-1:thread:new",
    workspaceId: "workspace-1",
    threadId: null,
    text: "Review the calendar implementation",
    attachmentIds: [],
    mentions: [],
    providerInstanceId: null,
    modelSelection: null,
    safetyMode: null,
    interactionMode: null,
    sentSnapshot: null,
    loading: false,
    saving: false,
    dirty: false,
    error: null,
  };
}

function imageAttachment(): ChatAttachmentRead {
  return {
    id: "attachment-1",
    workspaceId: "workspace-1",
    kind: "image",
    originalDisplayName: "diagram.png",
    mimeType: "image/png",
    byteSize: 123,
    sha256: "a".repeat(64),
    managedRelativePath: "assets/chat/attachments/diagram.png",
    signatureKind: "png",
    createdAt: "2026-07-21T12:00:00.000Z",
  };
}
