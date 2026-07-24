import type { ProviderModel } from "$lib/chat/contracts";

export type ModelCompanyId =
  | "openai"
  | "anthropic"
  | "google"
  | "xai"
  | "meta"
  | "mistral"
  | "deepseek"
  | "alibaba"
  | "moonshot"
  | "minimax"
  | "cursor"
  | "opencode";

export interface ModelCompanyIdentity {
  id: ModelCompanyId;
  name: string;
  iconFamilyId: string;
  order: number;
}

const COMPANIES: Record<ModelCompanyId, ModelCompanyIdentity> = {
  openai: { id: "openai", name: "OpenAI", iconFamilyId: "codex", order: 0 },
  anthropic: { id: "anthropic", name: "Anthropic", iconFamilyId: "claude", order: 1 },
  google: { id: "google", name: "Google", iconFamilyId: "google", order: 2 },
  xai: { id: "xai", name: "xAI", iconFamilyId: "grok", order: 3 },
  meta: { id: "meta", name: "Meta", iconFamilyId: "meta", order: 4 },
  mistral: { id: "mistral", name: "Mistral AI", iconFamilyId: "mistral", order: 5 },
  deepseek: { id: "deepseek", name: "DeepSeek", iconFamilyId: "deepseek", order: 6 },
  alibaba: { id: "alibaba", name: "Alibaba", iconFamilyId: "alibaba", order: 7 },
  moonshot: { id: "moonshot", name: "Moonshot AI", iconFamilyId: "moonshot", order: 8 },
  minimax: { id: "minimax", name: "MiniMax", iconFamilyId: "minimax", order: 9 },
  cursor: { id: "cursor", name: "Cursor", iconFamilyId: "cursor", order: 10 },
  opencode: { id: "opencode", name: "OpenCode", iconFamilyId: "opencode", order: 11 },
};

const FAMILY_DEFAULTS: Record<string, ModelCompanyId> = {
  codex: "openai",
  claude: "anthropic",
  cursor: "cursor",
  grok: "xai",
  opencode: "opencode",
};

const MODEL_MATCHERS: readonly [ModelCompanyId, RegExp][] = [
  ["anthropic", /(?:^|[\s/_.-])(anthropic|claude|sonnet|opus|haiku)(?:$|[\s/_.-])/i],
  ["openai", /(?:^|[\s/_.-])(openai|chatgpt|gpt|codex|o[134])(?:$|[\s/_.-]|\d)/i],
  ["google", /(?:^|[\s/_.-])(google|gemini|gemma)(?:$|[\s/_.-]|\d)/i],
  ["xai", /(?:^|[\s/_.-])(xai|grok)(?:$|[\s/_.-]|\d)/i],
  ["meta", /(?:^|[\s/_.-])(meta|llama)(?:$|[\s/_.-]|\d)/i],
  ["mistral", /(?:^|[\s/_.-])(mistral|codestral|ministral|mixtral)(?:$|[\s/_.-]|\d)/i],
  ["deepseek", /(?:^|[\s/_.-])deepseek(?:$|[\s/_.-])/i],
  ["alibaba", /(?:^|[\s/_.-])(alibaba|qwen)(?:$|[\s/_.-]|\d)/i],
  ["moonshot", /(?:^|[\s/_.-])(moonshot|kimi)(?:$|[\s/_.-]|\d)/i],
  ["minimax", /(?:^|[\s/_.-])minimax(?:$|[\s/_.-])/i],
];

/**
 * Resolves the company presented for a model while preserving its execution integration.
 *
 * @param familyId - Family of the installed execution integration.
 * @param model - Provider model when one is available.
 * @returns Stable company identity used to group the model picker.
 */
export function modelCompany(familyId: string, model: ProviderModel | null): ModelCompanyIdentity {
  if (model) {
    const identity = `${model.id} ${model.displayName}`;
    for (const [companyId, matcher] of MODEL_MATCHERS) {
      if (matcher.test(identity)) return COMPANIES[companyId];
    }
  }
  return COMPANIES[FAMILY_DEFAULTS[familyId] ?? "opencode"];
}

/**
 * Returns the default company represented by an execution integration.
 *
 * @param familyId - Execution integration family identifier.
 * @returns Default company identity for setup and provider-managed model states.
 */
export function integrationCompany(familyId: string): ModelCompanyIdentity {
  return COMPANIES[FAMILY_DEFAULTS[familyId] ?? "opencode"];
}
