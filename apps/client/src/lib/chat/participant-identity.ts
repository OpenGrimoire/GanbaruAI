import type { ModelId, ProviderModel, ProviderModelCatalog } from "$lib/chat/contracts";
import {
  modelCompany,
  modelCompanyForIdentity,
  type ModelCompanyIdentity,
} from "$lib/chat/model-company";

export interface ChatModelParticipant {
  displayName: string;
  company: ModelCompanyIdentity;
  modelId: ModelId | null;
  description: string | null;
  contextLimit: number | null;
  defaultReasoning: string | null;
}

/** Returns the human-readable reasoning level Ganbaru selects for new work. */
export function modelDefaultReasoning(model: ProviderModel | null): string | null {
  const definition = model?.options.find((option) => (
    option.kind === "choice" && /effort|reasoning/iu.test(`${option.key} ${option.label}`)
  ));
  if (definition?.kind !== "choice") return null;
  const medium = definition.options.find((option) => option.value.toLowerCase() === "medium") ?? null;
  if (medium) return medium.label;
  if (definition.defaultValue === null) return null;
  return definition.options.find((option) => option.value === definition.defaultValue)?.label ?? definition.defaultValue;
}

/** Resolves the visible model identity for one historical Chat turn. */
export function chatModelParticipant(
  providerFamilyId: string,
  modelId: ModelId | null,
  catalog: ProviderModelCatalog | null,
): ChatModelParticipant {
  const model = modelId
    ? catalog?.models.find((candidate) => candidate.id === modelId) ?? null
    : null;
  const company = model
    ? modelCompany(providerFamilyId, model)
    : modelCompanyForIdentity(providerFamilyId, modelId);
  return {
    displayName: model?.displayName ?? modelId ?? company.name,
    company,
    modelId: model?.id ?? modelId,
    description: model?.description ?? null,
    contextLimit: model?.contextLimit ?? null,
    defaultReasoning: modelDefaultReasoning(model),
  };
}
