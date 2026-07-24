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

export interface ProviderRefreshResult {
  familiesScanned: number;
  providersChecked: number;
  providersDiscovered: number;
  issues: number;
}

export const PROVIDER_FILE_KINDS = ["configuration", "instructions"] as const;
export type ProviderFileKind = (typeof PROVIDER_FILE_KINDS)[number];

export const PROVIDER_FILE_FORMATS = ["toml", "json", "jsonc", "markdown"] as const;
export type ProviderFileFormat = (typeof PROVIDER_FILE_FORMATS)[number];

export interface ProviderFileRead {
  fileId: string;
  name: string;
  kind: ProviderFileKind;
  format: ProviderFileFormat;
  path: string;
  exists: boolean;
  contents: string;
  revision: string;
}
