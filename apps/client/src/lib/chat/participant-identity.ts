import type { ModelId, ProviderModelCatalog } from "$lib/chat/contracts";
import {
  modelCompany,
  modelCompanyForIdentity,
  type ModelCompanyIdentity,
} from "$lib/chat/model-company";

export interface ChatModelParticipant {
  displayName: string;
  company: ModelCompanyIdentity;
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
  };
}
