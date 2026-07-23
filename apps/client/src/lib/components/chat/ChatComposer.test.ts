// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ChatComposerSnapshot } from "$lib/chat/composer-controller";
import type { ChatAttachmentRead, ChatSettingsRead } from "$lib/chat/contracts";
import { composerModelSelection } from "$lib/chat/composer-model";
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

  it("keeps the simplified composer actions directly discoverable", () => {
    const { target } = setup(false);
    expect(target.querySelector("[data-chat-model-trigger]")).not.toBeNull();
    expect(target.querySelector('[data-chat-field="safety"]')?.tagName).toBe("BUTTON");
    expect(target.querySelector('[data-chat-field="interaction"]')).toBeNull();
    expect(target.querySelector(".context-ring svg")?.getAttribute("role")).toBe("img");
    expect(target.querySelector("select")).toBeNull();
    expect(target.querySelector(".attachment-menu summary")?.getAttribute("aria-label")).toBe("Attach images");
    expect(target.querySelector("button.primary-action")?.getAttribute("aria-label")).toBe("Send");
    expect(target.querySelector('[aria-label*="microphone" i]')).toBeNull();
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

  it("keeps effort and speed inside the compact model control", async () => {
    const chat = getChat();
    chat.settings = modelSettings();
    chat.composer = {
      ...composer(),
      providerInstanceId: "codex-local",
      modelSelection: composerModelSelection("gpt-5.6-sol", false, [
        { key: "reasoning_effort", value: { kind: "choice", value: "medium" } },
        { key: "service_tier", value: { kind: "choice", value: "standard" } },
      ]),
      safetyMode: "supervised",
      interactionMode: "build",
    };
    vi.spyOn(chat, "setComposerModel").mockImplementation((modelSelection) => {
      chat.composer = { ...chat.composer, modelSelection };
    });
    const { target } = setup(false);
    const trigger = target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]");
    expect(trigger?.textContent).toContain("5.6 Sol");
    expect(trigger?.textContent).toContain("Medium");
    trigger?.click();
    await tick();
    expect(target.querySelector(".model-control.measured")).not.toBeNull();
    expect(trigger?.querySelector(".model-chevron")?.classList.contains("open")).toBe(false);
    const effortChoices = [...target.querySelectorAll<HTMLButtonElement>(".effort-options button")];
    expect(effortChoices).toHaveLength(7);
    expect(effortChoices[0]?.getAttribute("aria-label")).toBe("5.6 Terra Light");
    expect(effortChoices[2]?.getAttribute("aria-pressed")).toBe("true");
    expect(effortChoices.every((choice) => !choice.hasAttribute("title"))).toBe(true);
    expect(effortChoices.every((choice) => choice.dataset.appTooltipDisabled === "true")).toBe(true);
    const effortKnob = target.querySelector<HTMLElement>(".effort-knob");
    expect(effortKnob).not.toBeNull();
    expect(effortChoices[0]?.style.left).toContain("0% + 1.075rem");
    expect(effortChoices.at(-1)?.style.left).toContain("100% - 1.075rem");
    effortChoices.forEach((choice, index) => {
      vi.spyOn(choice, "getBoundingClientRect").mockReturnValue(new DOMRect(index * 40, 0, 20, 20));
    });
    const effortLadder = target.querySelector<HTMLElement>(".effort-ladder");
    effortLadder?.dispatchEvent(pointerEvent("pointerdown", 90));
    await tick();
    expect(target.querySelector(".effort-footer.holding")).toBeNull();
    effortLadder?.dispatchEvent(pointerEvent("pointermove", 130));
    await tick();
    expect(target.querySelector(".effort-footer.holding")?.textContent).toContain("Faster");
    expect(target.querySelector(".effort-footer.holding")?.textContent).toContain("Smarter");
    effortLadder?.dispatchEvent(pointerEvent("pointermove", 250));
    await tick();
    expect(trigger?.textContent).toContain("Ultra");
    effortLadder?.dispatchEvent(pointerEvent("pointerup", 250));
    await tick();
    expect(target.querySelector(".effort-footer.holding")).toBeNull();
    expect(effortLadder?.classList.contains("handle-hovered")).toBe(true);
    effortLadder?.dispatchEvent(pointerEvent("pointermove", 10));
    await tick();
    expect(effortLadder?.classList.contains("handle-hovered")).toBe(false);
    effortChoices[2]?.click();
    await tick();
    expect(trigger?.textContent).toContain("Medium");
    const fastButton = target.querySelector<HTMLButtonElement>(".fast-button");
    expect(fastButton?.getAttribute("aria-label")).toBe("Enable Fast mode");
    fastButton?.click();
    await tick();
    expect(target.querySelector(".effort-ladder.fast")).not.toBeNull();
    expect(fastButton?.getAttribute("aria-pressed")).toBe("true");
    effortChoices.at(-1)?.click();
    await tick();
    expect(trigger?.textContent).toContain("Ultra");
    expect(target.querySelector(".effort-name.ultra")).not.toBeNull();
    expect(target.querySelector(".effort-ladder.fast.ultra")).not.toBeNull();
    expect(target.querySelector(".fast-button.active.ultra")).not.toBeNull();
    expect(effortKnob?.style.left).toContain("100% - 1.075rem");
    effortChoices[0]?.click();
    await tick();
    expect(trigger?.textContent).toContain("5.6 Terra");
    expect(trigger?.textContent).toContain("Light");
    expect(target.querySelectorAll<HTMLButtonElement>(".effort-options button")[0]?.getAttribute("aria-pressed")).toBe("true");
    expect(effortKnob?.style.left).toContain("0% + 1.075rem");
    const advanced = target.querySelector<HTMLButtonElement>(".advanced-toggle");
    expect(advanced?.textContent).toContain("Advanced");
    advanced?.click();
    await tick();
    expect(target.querySelector(".advanced-view.active")).not.toBeNull();
    expect(target.querySelector<HTMLElement>(".overview-view")?.inert).toBe(true);
    const advancedRows = [...target.querySelectorAll<HTMLButtonElement>(".advanced-list button")];
    expect(advancedRows.some((button) => button.textContent?.includes("EffortLight"))).toBe(true);
    expect(advancedRows.some((button) => button.textContent?.includes("SpeedStandard"))).toBe(true);
    expect(advancedRows.some((button) => button.textContent?.includes("ProviderCodex"))).toBe(true);
    const speedRow = advancedRows.find((button) => button.textContent?.includes("SpeedStandard"));
    speedRow?.click();
    await tick();
    expect(target.querySelector(".model-flyout")?.textContent).toContain("Standard");
    expect(target.querySelector(".model-flyout")?.textContent).toContain("Fast");
    const effortRow = advancedRows.find((button) => button.textContent?.includes("EffortLight"));
    effortRow?.click();
    await tick();
    expect(target.querySelector(".model-flyout")?.textContent).toContain("High");
    effortRow?.dispatchEvent(new MouseEvent("pointerleave", { clientX: 999, clientY: 999 }));
    await new Promise((resolve) => setTimeout(resolve, 90));
    await tick();
    expect(target.querySelector(".model-flyout")).toBeNull();
    const providerRow = advancedRows.find((button) => button.textContent?.includes("ProviderCodex"));
    providerRow?.click();
    await tick();
    expect(target.querySelector(".model-flyout")?.textContent).toContain("ClaudeNot configured");
    expect(target.querySelector('[data-chat-field="interaction"]')).toBeNull();
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

function pointerEvent(type: string, clientX: number): PointerEvent {
  const event = new MouseEvent(type, { bubbles: true, button: 0, clientX }) as PointerEvent;
  Object.defineProperties(event, {
    isPrimary: { value: true },
    pointerId: { value: 1 },
  });
  return event;
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

function modelSettings(): ChatSettingsRead {
  return {
    configuration: {
      schemaVersion: 1,
      providers: [],
      automaticProviderSetupDisabled: [],
      rememberedSelections: [],
      workspaceProviderPreferences: {},
      panels: { railWidthPx: 320, inspectorWidthPx: 520 },
      behavior: {
        sendKey: "enter",
        restoreLastSelectedThread: true,
        showReasoningSummaries: true,
        automaticallyFoldSettledWork: true,
        terminalScrollbackLines: 10_000,
        idleSessionTimeoutSeconds: 1_800,
        confirmMultilineTerminalPaste: true,
      },
    },
    providerFamilies: [{
      familyId: "claude",
      displayName: "Claude",
      configurationSchemaVersion: 1,
      supportedPlatforms: ["linux", "windows", "macos"],
      minimumTestedCliVersion: null,
      defaultExecutableCandidates: ["claude"],
      implementationStatus: "available",
      potentialCapabilities: [],
      unavailableReason: null,
    }],
    providerInstances: [{
      configuration: {
        schemaVersion: 1,
        instanceId: "codex-local",
        familyId: "codex",
        label: "Codex",
        accentColor: null,
        enabled: true,
        executable: "codex",
        providerHome: null,
        launchArguments: [],
        environment: {},
        credentialReferences: {},
        visibleModelIds: [],
        favoriteModelIds: [],
        providerConfig: { schemaVersion: 1, value: {} },
      },
      lastProbe: {
        instanceId: "codex-local",
        state: "healthy",
        version: "1.0.0",
        accountLabel: null,
        capabilities: { entries: [{ capability: "native_plan", supported: true, explanation: null }] },
        checkedAt: "2026-07-22T12:00:00.000Z",
        detail: null,
      },
      lastSuccessfulProbeAt: "2026-07-22T12:00:00.000Z",
      modelCatalog: {
        instanceId: "codex-local",
        source: "provider",
        discoveredAt: "2026-07-22T12:00:00.000Z",
        stale: false,
        models: [{
          id: "gpt-5.6-sol",
          displayName: "5.6 Sol",
          description: null,
          contextLimit: 258_000,
          availability: "available",
          capabilities: [],
          custom: false,
          options: [{
            kind: "choice",
            key: "reasoning_effort",
            label: "Reasoning effort",
            description: null,
            defaultValue: "medium",
            options: [
              { value: "low", label: "low", description: null },
              { value: "medium", label: "Medium", description: null },
              { value: "high", label: "High", description: null },
              { value: "xhigh", label: "xhigh", description: null },
              { value: "max", label: "max", description: null },
              { value: "ultra", label: "ultra", description: null },
            ],
          }],
        }, {
          id: "gpt-5.6-terra",
          displayName: "GPT-5.6-Terra",
          description: null,
          contextLimit: 258_000,
          availability: "available",
          capabilities: [],
          custom: false,
          options: [{
            kind: "choice",
            key: "reasoning_effort",
            label: "Reasoning effort",
            description: null,
            defaultValue: "medium",
            options: [
              { value: "low", label: "low", description: null },
              { value: "medium", label: "Medium", description: null },
              { value: "high", label: "High", description: null },
              { value: "xhigh", label: "xhigh", description: null },
              { value: "max", label: "max", description: null },
              { value: "ultra", label: "ultra", description: null },
            ],
          }, {
            kind: "choice",
            key: "service_tier",
            label: "Service tier",
            description: null,
            defaultValue: "standard",
            options: [
              { value: "standard", label: "Standard", description: "Default speed" },
              { value: "fast", label: "Fast", description: "Faster responses" },
            ],
          }],
        }],
      },
    }],
    credentialStoreAvailability: "available",
    lastSelectedThreadId: null,
  };
}
