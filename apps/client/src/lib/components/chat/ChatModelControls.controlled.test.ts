// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ModelOptionSelection } from "$lib/chat/contracts";
import { getChat } from "$lib/stores/chat.svelte";
import { modelSettings, resetChatComposerTestStore } from "./ChatComposer.test-support";
import ChatModelControls from "./ChatModelControls.svelte";

interface ControlledSelection {
  providerInstanceId: string | null;
  modelId: string | null;
  providerManaged: boolean;
  options: ModelOptionSelection[];
}

function domRect(left: number, top: number, width: number, height: number): DOMRect {
  return {
    x: left,
    y: top,
    left,
    top,
    right: left + width,
    bottom: top + height,
    width,
    height,
    toJSON: () => ({}),
  };
}

describe("ChatModelControls controlled mode", () => {
  const mounted: Array<{
    component: ReturnType<typeof mount>;
    target: HTMLDivElement;
  }> = [];

  beforeEach(() => {
    resetChatComposerTestStore();
    getChat().settings = modelSettings();
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

  it("emits provider option changes without mutating the live composer", async () => {
    const target = document.createElement("div");
    document.body.append(target);
    const changes: ControlledSelection[] = [];
    const component = mount(ChatModelControls, {
      target,
      props: {
        value: {
          providerInstanceId: "codex-local",
          modelId: "gpt-5.6-sol",
          providerManaged: false,
          options: [
            { key: "reasoning_effort", value: { kind: "choice", value: "medium" } },
            { key: "service_tier", value: { kind: "choice", value: "standard" } },
          ],
        },
        onChange: (selection: ControlledSelection) => changes.push(selection),
      },
    });
    mounted.push({ component, target });
    const composerBefore = getChat().composer.modelSelection;

    target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]")?.click();
    await tick();
    target.querySelector<HTMLButtonElement>(".fast-button")?.click();
    await tick();

    expect(changes).toHaveLength(1);
    expect(changes[0]?.options).toContainEqual({
      key: "service_tier",
      value: { kind: "choice", value: "fast" },
    });
    expect(getChat().composer.modelSelection).toBe(composerBefore);
  });

  it("prevents editing when the teammate is archived", async () => {
    const target = document.createElement("div");
    document.body.append(target);
    const component = mount(ChatModelControls, {
      target,
      props: {
        value: {
          providerInstanceId: "codex-local",
          modelId: "gpt-5.6-sol",
          providerManaged: false,
          options: [],
        },
        disabled: true,
      },
    });
    mounted.push({ component, target });

    const trigger = target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]");
    expect(trigger?.disabled).toBe(true);
    trigger?.click();
    await tick();
    expect(target.querySelector(".model-popover")).toBeNull();
  });

  it("ports advanced flyouts above scroll containers and prefers the right side", async () => {
    const target = document.createElement("div");
    document.body.append(target);
    const component = mount(ChatModelControls, {
      target,
      props: {
        value: {
          providerInstanceId: "codex-local",
          modelId: "gpt-5.6-sol",
          providerManaged: false,
          options: [],
        },
      },
    });
    mounted.push({ component, target });

    target.querySelector<HTMLButtonElement>("[data-chat-model-trigger]")?.click();
    await tick();
    target.querySelector<HTMLButtonElement>(".advanced-toggle")?.click();
    await tick();
    const modelRow = [...target.querySelectorAll<HTMLButtonElement>(".advanced-list button")]
      .find((button) => button.textContent?.startsWith("Model"));
    modelRow?.dispatchEvent(new MouseEvent("pointerenter"));
    await tick();

    const picker = target.querySelector<HTMLElement>(".model-popover");
    const flyout = document.querySelector<HTMLElement>(".model-flyout");
    expect(flyout?.parentElement).toBe(document.body);
    if (!picker || !modelRow || !flyout) throw new Error("Advanced model flyout did not render");
    vi.spyOn(picker, "getBoundingClientRect").mockReturnValue(domRect(400, 80, 300, 220));
    vi.spyOn(modelRow, "getBoundingClientRect").mockReturnValue(domRect(410, 100, 280, 32));
    vi.spyOn(flyout, "getBoundingClientRect").mockReturnValue(domRect(0, 0, 200, 180));
    vi.stubGlobal("innerWidth", 1024);
    vi.stubGlobal("innerHeight", 768);

    window.dispatchEvent(new Event("resize"));
    await tick();

    expect(flyout.style.left).toBe("706px");
    expect(flyout.style.top).toBe("96px");
  });
});
