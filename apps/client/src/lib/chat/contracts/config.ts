import type {
  ProjectWorkingFolderId,
  CredentialReferenceId,
  InteractionMode,
  JsonValue,
  ModelId,
  ProviderFamilyId,
  ProviderInstanceId,
  SafetyMode,
  VersionedJson,
} from "./common";
import type { ModelOptionSelection } from "./provider";

export const CHAT_VAULT_CONFIG_SCHEMA_VERSION = 1;
export const CHAT_SEND_KEYS = ["enter", "mod_enter"] as const;
export type ChatSendKey = (typeof CHAT_SEND_KEYS)[number];

export interface ChatPortableProviderConfig {
  schemaVersion: number;
  instanceId: ProviderInstanceId;
  familyId: ProviderFamilyId;
  label: string;
  accentColor: string | null;
  enabled: boolean;
  launchArguments: string[];
  environment: Record<string, string>;
  credentialReferences: Record<string, CredentialReferenceId>;
  visibleModelIds: ModelId[];
  favoriteModelIds: ModelId[];
  providerConfig: VersionedJson;
  [unknownField: string]: unknown;
}

export interface RememberedComposerSelection {
  workingFolderId: ProjectWorkingFolderId;
  providerInstanceId: ProviderInstanceId;
  modelId: ModelId | null;
  providerManagedModel: boolean;
  modelOptions: ModelOptionSelection[];
  safetyMode: SafetyMode;
  interactionMode: InteractionMode;
}

export interface ChatPanelPreferences {
  inspectorWidthPx: number;
}

export interface ChatBehaviorPreferences {
  sendKey: ChatSendKey;
  restoreLastSelectedThread: boolean;
  showReasoningSummaries: boolean;
  automaticallyFoldSettledWork: boolean;
  terminalScrollbackLines: number;
  idleSessionTimeoutSeconds: number;
  confirmMultilineTerminalPaste: boolean;
}

export interface ChatVaultConfig {
  schemaVersion: number;
  providers: ChatPortableProviderConfig[];
  automaticProviderSetupDisabled: ProviderFamilyId[];
  rememberedSelections: RememberedComposerSelection[];
  workingFolderProviderPreferences: Record<ProjectWorkingFolderId, ProviderInstanceId>;
  panels: ChatPanelPreferences;
  behavior: ChatBehaviorPreferences;
  [unknownField: string]: JsonValue | unknown;
}
