import { describe, expect, it } from "vitest";
import type { ProviderModel } from "$lib/chat/contracts";
import { integrationCompany, modelCompany } from "./model-company";

function model(id: string, displayName = id): ProviderModel {
  return {
    id,
    displayName,
    description: null,
    contextLimit: null,
    availability: "available",
    capabilities: [],
    options: [],
    custom: false,
  };
}

describe("Chat model companies", () => {
  it("maps single-company execution integrations to their companies", () => {
    expect(integrationCompany("codex").name).toBe("OpenAI");
    expect(integrationCompany("claude").name).toBe("Anthropic");
    expect(integrationCompany("grok").name).toBe("xAI");
    expect(integrationCompany("cursor").name).toBe("Cursor");
    expect(integrationCompany("opencode").name).toBe("OpenCode");
  });

  it("classifies models exposed through multi-vendor integrations", () => {
    expect(modelCompany("cursor", model("anthropic/claude-opus-4-1")).name).toBe("Anthropic");
    expect(modelCompany("cursor", model("gpt-5.4")).name).toBe("OpenAI");
    expect(modelCompany("opencode", model("google/gemini-2.5-pro")).name).toBe("Google");
    expect(modelCompany("opencode", model("qwen3-coder")).name).toBe("Alibaba");
    expect(modelCompany("cursor", model("composer-2")).name).toBe("Cursor");
  });
});
