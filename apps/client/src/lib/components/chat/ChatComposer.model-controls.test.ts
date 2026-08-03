// @vitest-environment jsdom

import { tick } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { composerModelSelection, readComposerModelSelection } from "$lib/chat/composer-model";
import { getChat } from "$lib/stores/chat.svelte";
import {
  claudeModelSettings,
  composer,
  createChatComposerHarness,
  modelSettings,
  multiProviderSettings,
  pointerEvent,
  resetChatComposerTestStore,
} from "./ChatComposer.test-support";

vi.mock("$lib/api/chat", async (importOriginal) => ({
  ...await importOriginal<typeof import("$lib/api/chat")>(),
  listChatPromptCatalog: vi.fn(async () => []),
}));

describe("ChatComposer model controls", () => {
  const { setup, cleanup } = createChatComposerHarness();

  beforeEach(() => {
    resetChatComposerTestStore();
  });

  afterEach(async () => {
    await cleanup();
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it("keeps model, effort, and speed inside the compact model control", async () => {
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
    const modelButton = target.querySelector<HTMLButtonElement>(".model-picker-toggle");
    expect(modelButton?.textContent).toContain("Model");
    expect(target.querySelector(".advanced-toggle")).toBeNull();
    expect(target.querySelector(".advanced-list")).toBeNull();
    expect(modelButton?.closest(".effort-footer")?.nextElementSibling).toBe(effortLadder);
    modelButton?.dispatchEvent(new MouseEvent("pointerenter"));
    await tick();
    expect(target.querySelector(".model-flyout.positioned")).not.toBeNull();
    const modelSearch = target.querySelector<HTMLInputElement>(".model-search input");
    modelButton?.click();
    await tick();
    expect(document.activeElement).not.toBe(modelSearch);
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
    expect(target.querySelector('[data-chat-field="interaction"]')).toBeNull();
    trigger?.click();
    await tick();
    expect(target.querySelector(".model-popover")).toBeNull();
    trigger?.click();
    await tick();
    expect(target.querySelector(".model-picker-toggle")?.textContent).toContain("Model");
    expect(target.querySelector(".model-flyout")).toBeNull();
  });

  it("hides Fast when the provider catalog does not support speed selection", async () => {
    const chat = getChat();
    const settings = modelSettings();
    const provider = settings.providerInstances[0];
    if (!provider?.modelCatalog) throw new Error("Model settings require a discovered catalog");
    provider.modelCatalog.models = provider.modelCatalog.models.map((model) => ({
      ...model,
      options: model.options.filter((definition) => !["service_tier", "fastMode"].includes(definition.key)),
    }));
    chat.settings = settings;
    chat.composer = {
      ...composer(),
      providerInstanceId: provider.configuration.instanceId,
      modelSelection: composerModelSelection("gpt-5.6-sol", false, [
        { key: "reasoning_effort", value: { kind: "choice", value: "medium" } },
      ]),
      safetyMode: "ask_for_approval",
      interactionMode: "build",
    };

    const { target } = setup(false);
    target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]")?.click();
    await tick();

    expect(target.querySelector(".model-picker-toggle")?.textContent).toContain("Model");
    expect(target.querySelector(".fast-button")).toBeNull();
  });

  it("keeps the model picker open only while the pointer aims toward it", async () => {
    const chat = getChat();
    chat.settings = modelSettings();
    chat.composer = {
      ...composer(),
      providerInstanceId: "codex-local",
      modelSelection: composerModelSelection("gpt-5.6-sol", false, []),
      safetyMode: "ask_for_approval",
      interactionMode: "build",
    };

    const { target } = setup(false);
    target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]")?.click();
    await tick();
    const modelButton = target.querySelector<HTMLButtonElement>(".model-picker-toggle");
    if (!modelButton) throw new Error("Model button did not render");
    vi.spyOn(modelButton, "getBoundingClientRect").mockReturnValue(new DOMRect(400, 100, 120, 32));
    modelButton.dispatchEvent(pointerEvent("pointerenter", 410, 116));
    await tick();
    const flyout = target.querySelector<HTMLElement>(".model-flyout");
    if (!flyout) throw new Error("Model flyout did not render");
    vi.spyOn(flyout, "getBoundingClientRect").mockReturnValue(new DOMRect(120, 80, 260, 300));
    window.dispatchEvent(new Event("resize"));

    modelButton.dispatchEvent(pointerEvent("pointerleave", 400, 116));
    window.dispatchEvent(pointerEvent("pointermove", 390, 130));
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(target.querySelector(".model-flyout")).not.toBeNull();

    flyout.dispatchEvent(pointerEvent("pointerenter", 379, 130));
    await new Promise((resolve) => setTimeout(resolve, 260));
    expect(target.querySelector(".model-flyout")).not.toBeNull();

    flyout.dispatchEvent(pointerEvent("pointerleave", 120, 400));
    await tick();
    expect(target.querySelector(".model-flyout")).not.toBeNull();
    await new Promise((resolve) => setTimeout(resolve, 130));
    expect(target.querySelector(".model-flyout")).toBeNull();
    expect(target.querySelector(".model-popover")).not.toBeNull();

    modelButton.dispatchEvent(pointerEvent("pointerenter", 410, 116));
    await tick();
    const reopenedFlyout = target.querySelector<HTMLElement>(".model-flyout");
    if (!reopenedFlyout) throw new Error("Model flyout did not reopen");
    vi.spyOn(reopenedFlyout, "getBoundingClientRect").mockReturnValue(new DOMRect(120, 80, 260, 300));
    window.dispatchEvent(new Event("resize"));
    modelButton.dispatchEvent(pointerEvent("pointerleave", 400, 116));
    window.dispatchEvent(pointerEvent("pointermove", 410, 170));
    await tick();
    expect(target.querySelector(".model-flyout")).toBeNull();
    expect(target.querySelector(".model-popover")).not.toBeNull();
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
    target.querySelector<HTMLButtonElement>(".model-picker-toggle")?.click();
    await tick();
    expect(target.querySelector('[data-model-company="openai"]')).not.toBeNull();
    expect(target.querySelector('[data-model-company="anthropic"]')).not.toBeNull();
    const claudeModel = [...target.querySelectorAll<HTMLButtonElement>('[data-model-company="anthropic"] .model-choice')]
      .find((button) => button.textContent?.includes("Opus 4.8"));
    claudeModel?.click();
    await tick();

    expect(chat.composer.providerInstanceId).toBe("claude");
    expect(readComposerModelSelection(chat.composer.modelSelection).modelId).toBe("default");
    expect(target.querySelector(".model-flyout")).toBeNull();
    expect(target.querySelector(".model-popover")).not.toBeNull();
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
    target.querySelector<HTMLButtonElement>(".model-picker-toggle")?.click();
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

  it("selects a provider's recommended model with medium effort for a fresh composer", async () => {
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
      value: { kind: "choice", value: "medium" },
    }, {
      key: "fastMode",
      value: { kind: "boolean", value: false },
    }]);
    const trigger = target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]");
    expect(trigger?.textContent).toContain("Opus 4.8");
    expect(trigger?.textContent).toContain("Medium");

    trigger?.click();
    await tick();
    expect(target.querySelector(".fast-button")).not.toBeNull();
    const choices = [...target.querySelectorAll<HTMLButtonElement>(".effort-options button")];
    expect(choices).toHaveLength(5);
    expect(choices[1]?.getAttribute("aria-pressed")).toBe("true");
    expect(choices[0]?.style.left).toContain("0% + 0.875rem");
    expect(choices.at(-1)?.style.left).toContain("100% - 0.875rem");
    const knob = target.querySelector<HTMLElement>(".effort-knob");
    choices[0]?.click();
    await tick();
    expect(knob?.style.left).toContain("0% + 0.875rem");
    choices.at(-1)?.click();
    await tick();
    expect(knob?.style.left).toContain("100% - 0.875rem");

    const fastButton = target.querySelector<HTMLButtonElement>(".fast-button");
    expect(fastButton?.getAttribute("aria-pressed")).toBe("false");
    fastButton?.click();
    await tick();
    expect(readComposerModelSelection(chat.composer.modelSelection).options).toContainEqual({
      key: "fastMode",
      value: { kind: "boolean", value: true },
    });
    expect(fastButton?.getAttribute("aria-pressed")).toBe("true");
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
    await tick();

    expect(target.querySelector(".model-picker-toggle")?.textContent).toContain("Model");
    expect(target.querySelector(".model-flyout.positioned")).not.toBeNull();
    const dummyLadder = target.querySelector<HTMLElement>(".effort-ladder.dummy");
    expect(dummyLadder?.getAttribute("aria-disabled")).toBe("true");
    expect(dummyLadder?.querySelectorAll("button:disabled")).toHaveLength(6);
    expect(target.querySelector<HTMLButtonElement>(".fast-button.dummy")?.disabled).toBe(true);
  });
});
