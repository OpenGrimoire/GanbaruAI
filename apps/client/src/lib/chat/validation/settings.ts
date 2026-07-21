import {
  CREDENTIAL_STORE_AVAILABILITIES,
  type ChatSettingsRead,
  type ProviderInstanceRead,
  type ProviderSetupTestRead,
  type RemoveProviderResult,
} from "../contracts";
import { parseChatVaultConfig } from "./config";
import {
  parseProviderFamilyMetadata,
  parseProviderInstanceConfig,
  parseProviderModelCatalog,
  parseProviderProbeResult,
} from "./provider";
import { readArray, readBoolean, readEnum, readIdentifier, readNullable, readRecord, readUtcTimestamp } from "./readers";

export function parseProviderInstanceRead(value: unknown, label = "provider instance"): ProviderInstanceRead {
  const record = readRecord(value, label);
  return {
    configuration: parseProviderInstanceConfig(record.configuration, `${label}.configuration`),
    lastProbe: readNullable(record.lastProbe, `${label}.lastProbe`, parseProviderProbeResult),
    lastSuccessfulProbeAt: readNullable(record.lastSuccessfulProbeAt, `${label}.lastSuccessfulProbeAt`, readUtcTimestamp),
    modelCatalog: readNullable(record.modelCatalog, `${label}.modelCatalog`, parseProviderModelCatalog),
  };
}

export function parseChatSettingsRead(value: unknown): ChatSettingsRead {
  const record = readRecord(value, "Chat settings");
  return {
    configuration: parseChatVaultConfig(record.configuration, "Chat settings.configuration"),
    providerFamilies: readArray(record.providerFamilies, "Chat settings.providerFamilies", parseProviderFamilyMetadata),
    providerInstances: readArray(record.providerInstances, "Chat settings.providerInstances", parseProviderInstanceRead),
    credentialStoreAvailability: readEnum(
      record.credentialStoreAvailability,
      CREDENTIAL_STORE_AVAILABILITIES,
      "Chat settings.credentialStoreAvailability",
    ),
    lastSelectedThreadId: readNullable(
      record.lastSelectedThreadId,
      "Chat settings.lastSelectedThreadId",
      readIdentifier,
    ),
  };
}

export function parseRemoveProviderResult(value: unknown): RemoveProviderResult {
  const record = readRecord(value, "remove provider result");
  return {
    removed: readBoolean(record.removed, "remove provider result.removed"),
    credentialCleanupFailed: readBoolean(
      record.credentialCleanupFailed,
      "remove provider result.credentialCleanupFailed",
    ),
  };
}

export function parseProviderSetupTestRead(value: unknown): ProviderSetupTestRead {
  const record = readRecord(value, "provider setup test");
  return {
    probe: parseProviderProbeResult(record.probe, "provider setup test.probe"),
    modelCatalog: readNullable(
      record.modelCatalog,
      "provider setup test.modelCatalog",
      parseProviderModelCatalog,
    ),
  };
}
