import type { ChatVaultConfig } from "./config";
import type { ChatThreadId } from "./common";
import type {
  ProviderFamilyMetadataRead,
  ProviderInstanceConfig,
  ProviderModelCatalog,
  ProviderProbeResult,
} from "./provider";

export const CREDENTIAL_STORE_AVAILABILITIES = ["available", "unavailable"] as const;
export type CredentialStoreAvailability = (typeof CREDENTIAL_STORE_AVAILABILITIES)[number];

export interface ProviderInstanceRead {
  configuration: ProviderInstanceConfig;
  lastProbe: ProviderProbeResult | null;
  lastSuccessfulProbeAt: string | null;
  modelCatalog: ProviderModelCatalog | null;
}

export interface ChatSettingsRead {
  configuration: ChatVaultConfig;
  providerFamilies: ProviderFamilyMetadataRead[];
  providerInstances: ProviderInstanceRead[];
  credentialStoreAvailability: CredentialStoreAvailability;
  lastSelectedThreadId: ChatThreadId | null;
}

export interface RemoveProviderResult {
  removed: boolean;
  credentialCleanupFailed: boolean;
}

export interface ProviderSetupTestRead {
  probe: ProviderProbeResult;
  modelCatalog: ProviderModelCatalog | null;
}
