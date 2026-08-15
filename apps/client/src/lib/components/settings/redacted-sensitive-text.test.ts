// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { describe, expect, it } from "vitest";
import RedactedSensitiveText from "./RedactedSensitiveText.svelte";
import { redactedSensitiveText } from "./redacted-sensitive-text";

describe("redacted sensitive text", () => {
  it("preserves account structure without retaining visible characters", () => {
    const value = "person.name+work@example-domain.com";
    const redacted = redactedSensitiveText(value);

    expect(redacted).toHaveLength(value.length);
    expect(redacted).not.toBe(value);
    expect(redacted[6]).toBe(".");
    expect(redacted[16]).toBe("@");
    expect(redacted[24]).toBe("-");
    expect(redacted[31]).toBe(".");
    for (let index = 0; index < value.length; index += 1) {
      if (["@", ".", "-", "_"].includes(value[index] ?? "")) continue;
      expect(redacted[index]?.toLowerCase()).not.toBe(value[index]?.toLowerCase());
    }
  });

  it("returns the same placeholder for the same account", () => {
    expect(redactedSensitiveText("person@example.com")).toBe(
      redactedSensitiveText("person@example.com"),
    );
  });

  it("reveals the real value and updates its action affordance", async () => {
    const target = document.createElement("div");
    document.body.append(target);
    const component = mount(RedactedSensitiveText, {
      target,
      props: {
        value: "person@example.com",
        revealLabel: "Reveal account identity",
        hideLabel: "Hide account identity",
      },
    });
    const button = target.querySelector("button");

    expect(button?.textContent).not.toContain("person@example.com");
    expect(button?.dataset.appTooltip).toBe("Reveal account identity");
    expect(button?.getAttribute("aria-label")).toBe("Reveal account identity");

    button?.click();
    await tick();

    expect(button?.textContent).toContain("person@example.com");
    expect(button?.dataset.appTooltip).toBe("Hide account identity");
    expect(button?.getAttribute("aria-label")).toBe("Hide account identity: person@example.com");

    await unmount(component);
    target.remove();
  });
});
