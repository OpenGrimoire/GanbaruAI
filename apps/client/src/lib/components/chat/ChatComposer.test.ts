// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ChatComposerSnapshot } from "$lib/chat/composer-controller";
import type { ChatAttachmentRead, ChatSettingsRead } from "$lib/chat/contracts";
import { composerModelSelection, readComposerModelSelection } from "$lib/chat/composer-model";
import { getChat } from "$lib/stores/chat.svelte";
import ChatComposer from "./ChatComposer.svelte";

const api = vi.hoisted(() => ({
  attachmentUrl: vi.fn(async () => "data:image/png;base64,iVBORw0KGgo="),
}));

class ResizeObserverMock implements ResizeObserver {
  constructor(_callback: ResizeObserverCallback) {}

  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

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
    vi.stubGlobal("ResizeObserver", ResizeObserverMock);
    const chat = getChat();
    chat.composer = composer();
    chat.composerAttachments = [];
    chat.interaction = null;
    chat.sendError = null;
    chat.settings = null;
    chat.activeThreads = [];
    chat.archivedThreads = [];
    chat.workspaces = [];
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
    vi.unstubAllGlobals();
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
    const safetyTrigger = target.querySelector<HTMLElement>('[data-chat-field="safety"]');
    expect(safetyTrigger?.tagName).toBe("BUTTON");
    expect(safetyTrigger?.hasAttribute("title")).toBe(false);
    expect(safetyTrigger?.dataset.appTooltipDisabled).toBe("true");
    expect(target.querySelector('[data-chat-field="interaction"]')).toBeNull();
    expect(target.querySelector(".context-ring svg")?.getAttribute("role")).toBe("img");
    expect(target.querySelector("select")).toBeNull();
    expect(target.querySelector(".attachment-menu summary")?.getAttribute("aria-label")).toBe("Attach images");
    expect(target.querySelector("button.primary-action")?.getAttribute("aria-label")).toBe("Send");
    expect(target.querySelector('[aria-label*="microphone" i]')).toBeNull();
  });

  it("centers broad permission confirmation at the app root", async () => {
    const chat = getChat();
    chat.settings = modelSettings();
    chat.composer = {
      ...composer(),
      providerInstanceId: "codex-local",
      safetyMode: "ask_for_approval",
      interactionMode: "build",
    };
    chat.workspaces = [{
      workspace: {
        id: "workspace-1",
        projectId: null,
        displayName: "Example",
        repositoryKind: "git",
        repositoryIdentity: "example-repository",
        createdAt: "2026-07-24T12:00:00.000Z",
        updatedAt: "2026-07-24T12:00:00.000Z",
        archivedAt: null,
        revision: 1,
      },
      bindingStatus: "available",
      canonicalPath: "/workspace/example",
      lastVerifiedAt: "2026-07-24T12:00:00.000Z",
      currentBranch: "feat/chat",
    }];
    vi.spyOn(chat, "setComposerModes").mockImplementation((safetyMode, interactionMode) => {
      chat.composer = { ...chat.composer, safetyMode, interactionMode };
    });

    const { target } = setup(false);
    target.querySelector<HTMLButtonElement>('[data-chat-field="safety"]')?.click();
    await tick();
    [...document.body.querySelectorAll<HTMLButtonElement>('[role="listbox"] button')]
      .find((button) => button.textContent?.includes("Full access"))?.click();
    await tick();

    expect(target.querySelector('[role="dialog"]')).toBeNull();
    let dialog = document.body.querySelector<HTMLElement>('.confirm-dialog[role="dialog"]');
    expect(dialog?.closest(".fixed.inset-0")).not.toBeNull();
    expect(dialog?.textContent).toContain("Allow Full access?");
    expect(dialog?.textContent).toContain("Unrestricted access to the internet and any file");
    expect(dialog?.textContent).not.toContain("Example");

    [...dialog?.querySelectorAll<HTMLButtonElement>("button") ?? []]
      .find((button) => button.textContent?.trim() === "Cancel")?.click();
    await tick();
    target.querySelector<HTMLButtonElement>('[data-chat-field="safety"]')?.click();
    await tick();
    [...document.body.querySelectorAll<HTMLButtonElement>('[role="listbox"] button')]
      .find((button) => button.textContent?.includes("Custom"))?.click();
    await tick();

    dialog = document.body.querySelector<HTMLElement>('.confirm-dialog[role="dialog"]');
    expect(dialog?.textContent).toContain("Use custom permissions?");
    expect(dialog?.textContent).toContain("Uses permissions from config.toml, which may grant Full access");
    expect(dialog?.textContent).not.toContain("Example");
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
      safetyMode: "ask_for_approval",
      interactionMode: "build",
    };
    vi.spyOn(chat, "setComposerModel").mockImplementation((modelSelection) => {
      chat.composer = { ...chat.composer, modelSelection };
    });
    const { target } = setup(false);
    const trigger = target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]");
    expect(trigger?.querySelector(".provider-icon")).toBeNull();
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
    expect(effortChoices[0]?.style.left).toContain("0% + 0.875rem");
    expect(effortChoices.at(-1)?.style.left).toContain("100% - 0.875rem");
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
    expect(effortKnob?.style.left).toContain("100% - 0.875rem");
    effortChoices[0]?.click();
    await tick();
    expect(trigger?.textContent).toContain("5.6 Terra");
    expect(trigger?.textContent).toContain("Light");
    expect(target.querySelectorAll<HTMLButtonElement>(".effort-options button")[0]?.getAttribute("aria-pressed")).toBe("true");
    expect(effortKnob?.style.left).toContain("0% + 0.875rem");
    const advanced = target.querySelector<HTMLButtonElement>(".advanced-toggle");
    expect(advanced?.textContent).toContain("Advanced");
    advanced?.click();
    await tick();
    expect(target.querySelector(".advanced-view.active")).not.toBeNull();
    expect(target.querySelector<HTMLElement>(".overview-view")?.inert).toBe(true);
    const advancedRows = [...target.querySelectorAll<HTMLButtonElement>(".advanced-list button")];
    expect(advancedRows.some((button) => button.textContent?.includes("EffortLight"))).toBe(true);
    expect(advancedRows.some((button) => button.textContent?.includes("SpeedStandard"))).toBe(true);
    expect(advancedRows.some((button) => button.textContent?.includes("ProviderCodex"))).toBe(false);
    const speedRow = advancedRows.find((button) => button.textContent?.includes("SpeedStandard"));
    speedRow?.dispatchEvent(new MouseEvent("pointerenter"));
    await tick();
    expect(target.querySelector(".model-flyout")?.textContent).toContain("Standard");
    expect(target.querySelector(".model-flyout")?.textContent).toContain("Fast");
    [...target.querySelectorAll<HTMLButtonElement>(".model-flyout .selection-list > button")]
      .find((button) => button.textContent?.startsWith("Standard"))?.click();
    await tick();
    expect(target.querySelector(".model-flyout")).not.toBeNull();
    const effortRow = advancedRows.find((button) => button.textContent?.includes("EffortLight"));
    effortRow?.dispatchEvent(new MouseEvent("pointerenter"));
    await tick();
    expect(target.querySelector(".model-flyout")?.textContent).toContain("High");
    expect(target.querySelector(".model-flyout")?.textContent).not.toContain("Uses less reasoning");
    expect(target.querySelector(".flyout-heading")).toBeNull();
    effortRow?.dispatchEvent(new MouseEvent("pointerleave", { clientX: 999, clientY: 999 }));
    await new Promise((resolve) => setTimeout(resolve, 90));
    await tick();
    expect(target.querySelector(".model-flyout.positioned")).not.toBeNull();
    const modelRow = advancedRows.find((button) => button.textContent?.startsWith("Model"));
    modelRow?.dispatchEvent(new MouseEvent("pointerenter"));
    await tick();
    expect(target.querySelector(".provider-rail")).toBeNull();
    expect(target.querySelector('[data-model-company="openai"] .model-company-heading svg')).not.toBeNull();
    expect(target.querySelector('[data-model-company="openai"] .model-company-heading')?.textContent).toContain("OpenAI");
    expect(target.querySelector('[data-model-company="anthropic"] .model-company-heading')?.textContent).toContain("Anthropic");
    const openAiHeading = target.querySelector<HTMLButtonElement>('[data-model-company="openai"] .model-company-heading');
    openAiHeading?.click();
    await tick();
    expect(openAiHeading?.getAttribute("aria-expanded")).toBe("false");
    expect(target.querySelector<HTMLElement>('[data-model-company="openai"] .model-company-content')?.inert).toBe(true);
    openAiHeading?.click();
    await tick();
    expect(openAiHeading?.getAttribute("aria-expanded")).toBe("true");
    expect(target.querySelector(".model-picker-provider")).toBeNull();
    const modelList = target.querySelector<HTMLElement>(".model-list");
    if (!modelList) throw new Error("Model list did not render");
    expect(modelList.querySelector(".model-search-row")).not.toBeNull();
    Object.defineProperties(modelList, {
      clientHeight: { configurable: true, value: 200 },
      scrollHeight: { configurable: true, value: 600 },
      scrollTop: { configurable: true, writable: true, value: 0 },
    });
    modelList.dispatchEvent(new Event("scroll"));
    await tick();
    expect(modelList.classList.contains("model-list-scroll-bottom")).toBe(true);
    modelList.scrollTop = 200;
    modelList.dispatchEvent(new Event("scroll"));
    await tick();
    expect(modelList.classList.contains("model-list-scroll-both")).toBe(true);
    modelList.scrollTop = 400;
    modelList.dispatchEvent(new Event("scroll"));
    await tick();
    expect(modelList.classList.contains("model-list-scroll-top")).toBe(true);
    expect(target.querySelector<HTMLInputElement>(".model-search input")?.placeholder).toBe("Search models...");
    expect(target.querySelector(".model-flyout")?.textContent).not.toContain("gpt-5.6-sol");
    expect(target.querySelector(".model-flyout")?.textContent).not.toContain("GPT-5.4 Deprecated");
    expect(target.querySelector(".favorite-company-section .model-company-heading")?.textContent).toContain("Favorites");
    expect(target.querySelector(".favorite-company-section .model-company-content-inner > p")?.textContent).toBe("No favorites yet.");
    const setupAction = target.querySelector<HTMLElement>('[data-model-company="anthropic"] .company-setup');
    expect(setupAction?.textContent).toBe("Configure Anthropic");
    expect(setupAction?.querySelector("small")).toBeNull();
    expect(target.querySelector(".model-flyout")?.textContent).not.toContain("ProviderCodex");
    expect(advancedRows.some((button) => button.textContent?.startsWith("Interaction"))).toBe(false);
    expect(target.querySelector('[data-chat-field="interaction"]')).toBeNull();
    trigger?.click();
    await tick();
    expect(target.querySelector(".model-popover")).toBeNull();
    trigger?.click();
    await tick();
    expect(target.querySelector(".advanced-view.active")).not.toBeNull();
    expect(target.querySelector<HTMLElement>(".overview-view")?.inert).toBe(true);
  });

  it("switches provider and model together from the model picker", async () => {
    const chat = getChat();
    chat.settings = multiProviderSettings();
    chat.composer = {
      ...composer(),
      providerInstanceId: "codex-local",
      modelSelection: composerModelSelection("gpt-5.6-sol", false, []),
      safetyMode: "ask_for_approval",
      interactionMode: "build",
    };
    vi.spyOn(chat, "setComposerProvider").mockImplementation((providerInstanceId) => {
      chat.composer = { ...chat.composer, providerInstanceId };
    });
    vi.spyOn(chat, "setComposerModel").mockImplementation((modelSelection) => {
      chat.composer = { ...chat.composer, modelSelection };
    });

    const { target } = setup(false);
    target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]")?.click();
    await tick();
    target.querySelector<HTMLButtonElement>(".advanced-toggle")?.click();
    await tick();
    const modelRow = [...target.querySelectorAll<HTMLButtonElement>(".advanced-list button")]
      .find((button) => button.textContent?.startsWith("Model"));
    modelRow?.click();
    await tick();
    expect(target.querySelector('[data-model-company="openai"]')).not.toBeNull();
    expect(target.querySelector('[data-model-company="anthropic"]')).not.toBeNull();
    const claudeModel = [...target.querySelectorAll<HTMLButtonElement>('[data-model-company="anthropic"] .model-choice')]
      .find((button) => button.textContent?.includes("Opus 4.8"));
    claudeModel?.click();
    await tick();

    expect(chat.composer.providerInstanceId).toBe("claude");
    expect(readComposerModelSelection(chat.composer.modelSelection).modelId).toBe("default");
  });

  it("shows and updates favorites across providers without exposing raw model IDs", async () => {
    const chat = getChat();
    const settings = multiProviderSettings();
    const codex = settings.providerInstances.find((entry) => entry.configuration.instanceId === "codex-local");
    const claude = settings.providerInstances.find((entry) => entry.configuration.instanceId === "claude");
    if (!codex || !claude) throw new Error("Favorite fixtures require Codex and Claude");
    codex.configuration.favoriteModelIds = ["gpt-5.6-sol"];
    claude.configuration.favoriteModelIds = ["default"];
    chat.settings = settings;
    chat.composer = {
      ...composer(),
      providerInstanceId: "codex-local",
      modelSelection: composerModelSelection("gpt-5.6-sol", false, []),
      safetyMode: "ask_for_approval",
      interactionMode: "build",
    };
    const updateModels = vi.spyOn(chat, "updateModels").mockResolvedValue();

    const { target } = setup(false);
    target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]")?.click();
    await tick();
    target.querySelector<HTMLButtonElement>(".advanced-toggle")?.click();
    await tick();
    [...target.querySelectorAll<HTMLButtonElement>(".advanced-list button")]
      .find((button) => button.textContent?.startsWith("Model"))?.click();
    await tick();

    const providerModelRow = [...target.querySelectorAll<HTMLElement>('[data-model-company="openai"] .model-row')]
      .find((row) => row.textContent?.includes("5.6 Sol"));
    expect(providerModelRow?.querySelector(".model-provider-caption")).toBeNull();
    const favoriteText = target.querySelector(".favorite-company-section")?.textContent ?? "";
    expect(favoriteText).toContain("5.6 Sol");
    expect(favoriteText).toContain("Opus 4.8");
    expect(favoriteText).not.toContain("gpt-5.6-sol");
    expect(target.querySelectorAll(".model-provider-caption")).toHaveLength(0);
    expect(target.querySelectorAll(".favorite-source-section")).toHaveLength(0);
    expect(target.querySelectorAll(".favorite-model-label")).toHaveLength(2);

    target.querySelector<HTMLButtonElement>('.favorite-company-section .model-favorite[aria-label="Favorite: 5.6 Sol"]')?.click();
    await tick();
    expect(updateModels).toHaveBeenCalledWith("codex-local", [], []);
  });

  it("selects a provider's recommended model and effort for a fresh composer", async () => {
    const chat = getChat();
    chat.settings = claudeModelSettings();
    chat.composer = {
      ...composer(),
      providerInstanceId: "claude",
      safetyMode: "ask_for_approval",
      interactionMode: "build",
    };
    vi.spyOn(chat, "setComposerModel").mockImplementation((modelSelection) => {
      chat.composer = { ...chat.composer, modelSelection };
    });

    const { target } = setup(false);
    await tick();
    await tick();

    const selection = readComposerModelSelection(chat.composer.modelSelection);
    expect(selection.modelId).toBe("default");
    expect(selection.providerManaged).toBe(false);
    expect(selection.options).toEqual([{
      key: "effort",
      value: { kind: "choice", value: "high" },
    }, {
      key: "fastMode",
      value: { kind: "boolean", value: false },
    }]);
    const trigger = target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]");
    expect(trigger?.textContent).toContain("Opus 4.8");
    expect(trigger?.textContent).toContain("High");

    trigger?.click();
    await tick();
    expect(target.querySelector(".fast-button")).not.toBeNull();
    const choices = [...target.querySelectorAll<HTMLButtonElement>(".effort-options button")];
    expect(choices).toHaveLength(5);
    expect(choices[2]?.getAttribute("aria-pressed")).toBe("true");
    expect(choices[0]?.style.left).toContain("0% + 0.875rem");
    expect(choices.at(-1)?.style.left).toContain("100% - 0.875rem");
    const knob = target.querySelector<HTMLElement>(".effort-knob");
    choices[0]?.click();
    await tick();
    expect(knob?.style.left).toContain("0% + 0.875rem");
    choices.at(-1)?.click();
    await tick();
    expect(knob?.style.left).toContain("100% - 0.875rem");

    target.querySelector<HTMLButtonElement>(".advanced-toggle")?.click();
    await tick();
    const speedRow = [...target.querySelectorAll<HTMLButtonElement>(".advanced-list button")]
      .find((button) => button.textContent?.includes("SpeedStandard"));
    speedRow?.click();
    await tick();
    const speedChoices = [...target.querySelectorAll<HTMLButtonElement>(".model-flyout .option-list > button")];
    expect(speedChoices.map((button) => button.querySelector("strong")?.textContent)).toEqual(["Standard", "Fast"]);
    expect(speedChoices[0]?.textContent).toContain("Default speed and usage");
    expect(speedChoices[1]?.textContent).toContain("Lower latency with higher usage cost");
    expect(speedChoices[0]?.querySelector("svg")).not.toBeNull();
    speedChoices[1]?.click();
    await tick();
    expect(readComposerModelSelection(chat.composer.modelSelection).options).toContainEqual({
      key: "fastMode",
      value: { kind: "boolean", value: true },
    });
    expect(speedChoices[1]?.querySelector("svg")).not.toBeNull();
  });

  it("selects the strongest model from the first healthy provider and defaults permissions", async () => {
    const chat = getChat();
    const settings = modelSettings();
    const firstProvider = settings.providerInstances[0];
    if (!firstProvider?.modelCatalog) throw new Error("Model settings require a discovered catalog");
    firstProvider.modelCatalog.models.reverse();
    chat.settings = settings;
    vi.spyOn(chat, "setComposerProvider").mockImplementation((providerInstanceId) => {
      chat.composer = { ...chat.composer, providerInstanceId };
    });
    vi.spyOn(chat, "setComposerModel").mockImplementation((modelSelection) => {
      chat.composer = { ...chat.composer, modelSelection };
    });
    vi.spyOn(chat, "setComposerModes").mockImplementation((safetyMode, interactionMode) => {
      chat.composer = { ...chat.composer, safetyMode, interactionMode };
    });

    const { target } = setup(false);
    await tick();
    await tick();
    await tick();

    expect(chat.composer.providerInstanceId).toBe("codex-local");
    expect(readComposerModelSelection(chat.composer.modelSelection).modelId).toBe("gpt-5.6-sol");
    expect(chat.composer.safetyMode).toBe("ask_for_approval");
    expect(chat.composer.interactionMode).toBe("build");
    expect(target.querySelector('[data-chat-field="safety"]')?.textContent).toContain("Ask for approval");
    expect(target.querySelector("[data-chat-model-trigger]")?.textContent).toContain("5.6 Sol");
  });

  it("previews disabled model controls when no provider is configured", async () => {
    const chat = getChat();
    const settings = modelSettings();
    settings.providerInstances = [];
    chat.settings = settings;
    vi.spyOn(chat, "setComposerModes").mockImplementation((safetyMode, interactionMode) => {
      chat.composer = { ...chat.composer, safetyMode, interactionMode };
    });

    const { target } = setup(false);
    await tick();
    await tick();

    expect(chat.composer.providerInstanceId).toBeNull();
    expect(chat.composer.safetyMode).toBe("ask_for_approval");
    const trigger = target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]");
    expect(trigger?.textContent).toContain("Choose provider");
    trigger?.click();
    await tick();

    expect(target.querySelector(".advanced-view.active")).not.toBeNull();
    const rows = [...target.querySelectorAll<HTMLButtonElement>(".advanced-list button")];
    expect(rows.map((row) => row.textContent?.trim())).toEqual(["ModelChoose provider", "Effort", "Speed"]);
    expect(rows[0]?.disabled).toBe(false);
    expect(rows[1]?.disabled).toBe(true);
    expect(rows[2]?.disabled).toBe(true);

    target.querySelector<HTMLButtonElement>(".advanced-heading")?.click();
    await tick();
    const dummyLadder = target.querySelector<HTMLElement>(".effort-ladder.dummy");
    expect(dummyLadder?.getAttribute("aria-disabled")).toBe("true");
    expect(dummyLadder?.querySelectorAll("button:disabled")).toHaveLength(6);
    expect(target.querySelector<HTMLButtonElement>(".fast-button.dummy")?.disabled).toBe(true);
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
      familyId: "codex",
      displayName: "OpenAI",
      configurationSchemaVersion: 1,
      supportedPlatforms: ["linux", "windows", "macos"],
      minimumTestedCliVersion: null,
      defaultExecutableCandidates: ["codex"],
      implementationStatus: "available",
      potentialCapabilities: [],
      unavailableReason: null,
    }, {
      familyId: "claude",
      displayName: "Anthropic",
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
        label: "OpenAI",
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
              { value: "low", label: "low", description: "Uses less reasoning" },
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
        }, {
          id: "gpt-5.4",
          displayName: "GPT-5.4 Deprecated",
          description: null,
          contextLimit: 200_000,
          availability: "deprecated",
          capabilities: [],
          custom: false,
          options: [],
        }],
      },
    }],
    credentialStoreAvailability: "available",
    lastSelectedThreadId: null,
  };
}

function claudeModelSettings(): ChatSettingsRead {
  const settings = modelSettings();
  const provider = settings.providerInstances[0];
  if (!provider) throw new Error("Model settings require a provider fixture");
  provider.configuration.instanceId = "claude";
  provider.configuration.familyId = "claude";
  provider.configuration.label = "Anthropic";
  provider.configuration.executable = "claude";
  provider.lastProbe = provider.lastProbe ? { ...provider.lastProbe, instanceId: "claude" } : null;
  provider.modelCatalog = {
    instanceId: "claude",
    source: "provider",
    discoveredAt: "2026-07-23T12:00:00.000Z",
    stale: false,
    models: [{
      id: "default",
      displayName: "Opus 4.8",
      description: "Use the default model (currently Opus 4.8)",
      contextLimit: null,
      availability: "available",
      capabilities: [],
      custom: false,
      options: [{
        kind: "choice",
        key: "effort",
        label: "Effort",
        description: "Provider-supported reasoning effort",
        defaultValue: "high",
        options: ["low", "medium", "high", "xhigh", "max"].map((value) => ({
          value,
          label: value,
          description: null,
        })),
      }, {
        kind: "boolean",
        key: "fastMode",
        label: "Fast mode",
        description: "Lower latency with higher usage cost",
        defaultValue: false,
      }],
    }],
  };
  return settings;
}

function multiProviderSettings(): ChatSettingsRead {
  const settings = modelSettings();
  const claude = claudeModelSettings().providerInstances[0];
  if (!claude) throw new Error("Claude provider fixture is unavailable");
  settings.providerInstances.push(claude);
  return settings;
}
