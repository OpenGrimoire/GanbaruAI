import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";
import type {
  ChatBehaviorPreferences,
  ChatPanelPreferences,
  ChatProjectShellRead,
  ChatSettingsRead,
  ChatThreadId,
  ChatThreadShellRead,
  ChatTimelinePageRead,
  ChatWorkspaceId,
  ChatWorkspaceRead,
  CreateChatWorkspaceRequest,
  CredentialReferenceId,
  ModelId,
  ProviderInstanceConfig,
  ProviderInstanceId,
  ProviderInstanceRead,
  ProviderModelCatalog,
  ProviderProbeResult,
  ProviderSetupTestRead,
  RemoveProviderResult,
} from "$lib/chat/contracts";
import {
  parseChatProjectShells,
  parseChatSettingsRead,
  parseChatThreadShell,
  parseChatThreadShells,
  parseChatTimelinePage,
  parseChatVaultConfig,
  parseChatWorkspaceRead,
  parseChatWorkspaceReads,
  parseProviderInstanceRead,
  parseProviderModelCatalog,
  parseProviderProbeResult,
  parseProviderSetupTestRead,
  parseRemoveProviderResult,
} from "$lib/chat/validation";

export async function listChatWorkspaces(): Promise<ChatWorkspaceRead[]> {
  return parseChatWorkspaceReads(await invoke<unknown>("chat_list_workspaces", { dbUrl: await ensureDbUrl() }));
}

export async function createChatWorkspace(
  request: CreateChatWorkspaceRequest,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_create_workspace", { dbUrl: await ensureDbUrl(), request }));
}

export async function renameChatWorkspace(
  workspaceId: ChatWorkspaceId,
  displayName: string,
  expectedRevision: number,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_rename_workspace", {
    dbUrl: await ensureDbUrl(),
    workspaceId,
    displayName,
    expectedRevision,
  }));
}

export async function bindChatWorkspace(
  workspaceId: ChatWorkspaceId,
  title: string,
): Promise<ChatWorkspaceRead | null> {
  const value = await invoke<unknown>("chat_bind_workspace", { dbUrl: await ensureDbUrl(), workspaceId, title });
  return value === null ? null : parseChatWorkspaceRead(value);
}

export async function rebindChatWorkspace(
  workspaceId: ChatWorkspaceId,
  title: string,
): Promise<ChatWorkspaceRead | null> {
  const value = await invoke<unknown>("chat_rebind_workspace", { dbUrl: await ensureDbUrl(), workspaceId, title });
  return value === null ? null : parseChatWorkspaceRead(value);
}

export async function removeChatWorkspaceBinding(
  workspaceId: ChatWorkspaceId,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_remove_workspace_binding", { dbUrl: await ensureDbUrl(), workspaceId }));
}

export async function archiveChatWorkspace(
  workspaceId: ChatWorkspaceId,
  expectedRevision: number,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_archive_workspace", { dbUrl: await ensureDbUrl(), workspaceId, expectedRevision }));
}

export async function restoreChatWorkspace(
  workspaceId: ChatWorkspaceId,
  expectedRevision: number,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_restore_workspace", { dbUrl: await ensureDbUrl(), workspaceId, expectedRevision }));
}

export async function openChatWorkspaceFolder(workspaceId: ChatWorkspaceId): Promise<void> {
  await invoke("chat_open_workspace_folder", { dbUrl: await ensureDbUrl(), workspaceId });
}

export async function readChatSettings(): Promise<ChatSettingsRead> {
  return parseChatSettingsRead(await invoke<unknown>("chat_read_settings"));
}

export async function setLastSelectedChatThread(threadId: ChatThreadId | null): Promise<void> {
  await invoke("chat_set_last_selected_thread", { threadId });
}

export async function saveChatProvider(configuration: ProviderInstanceConfig): Promise<ProviderInstanceRead> {
  return parseProviderInstanceRead(await invoke<unknown>("chat_save_provider", {
    request: { configuration },
  }));
}

export async function setChatProviderEnabled(instanceId: ProviderInstanceId, enabled: boolean): Promise<ProviderInstanceRead> {
  return parseProviderInstanceRead(await invoke<unknown>("chat_set_provider_enabled", { instanceId, enabled }));
}

export async function removeChatProvider(instanceId: ProviderInstanceId): Promise<RemoveProviderResult> {
  return parseRemoveProviderResult(await invoke<unknown>("chat_remove_provider", { instanceId }));
}

export async function testChatProvider(configuration: ProviderInstanceConfig): Promise<ProviderSetupTestRead> {
  return parseProviderSetupTestRead(await invoke<unknown>("chat_test_provider", {
    request: { configuration },
  }));
}

export async function probeChatProvider(instanceId: ProviderInstanceId): Promise<ProviderProbeResult> {
  return parseProviderProbeResult(await invoke<unknown>("chat_probe_provider", { instanceId }));
}

export async function refreshChatProviderModels(instanceId: ProviderInstanceId): Promise<ProviderModelCatalog> {
  return parseProviderModelCatalog(await invoke<unknown>("chat_refresh_provider_models", { instanceId }));
}

export async function updateChatProviderModels(
  instanceId: ProviderInstanceId,
  visibleModelIds: ModelId[],
  favoriteModelIds: ModelId[],
): Promise<ProviderInstanceRead> {
  return parseProviderInstanceRead(await invoke<unknown>("chat_update_provider_models", {
    instanceId,
    visibleModelIds,
    favoriteModelIds,
  }));
}

export async function updateChatBehavior(behavior: ChatBehaviorPreferences): Promise<ChatSettingsRead["configuration"]> {
  return parseChatVaultConfig(await invoke<unknown>("chat_update_behavior", { behavior }));
}

export async function updateChatPanels(panels: ChatPanelPreferences): Promise<ChatSettingsRead["configuration"]> {
  return parseChatVaultConfig(await invoke<unknown>("chat_update_panels", { panels }));
}

export async function setChatWorkspaceProviderPreference(
  workspaceId: ChatWorkspaceId,
  instanceId: ProviderInstanceId | null,
): Promise<ChatSettingsRead["configuration"]> {
  return parseChatVaultConfig(await invoke<unknown>("chat_set_workspace_provider_preference", { workspaceId, instanceId }));
}

export async function replaceChatCredential(referenceId: CredentialReferenceId, secret: string): Promise<void> {
  await invoke("chat_replace_credential", { referenceId, secret });
}

export async function removeChatCredential(referenceId: CredentialReferenceId): Promise<boolean> {
  return invoke<boolean>("chat_remove_credential", { referenceId });
}

export async function pickChatProviderExecutable(title: string): Promise<string | null> {
  return invoke<string | null>("chat_pick_provider_executable", { title });
}

export async function pickChatProviderHome(title: string): Promise<string | null> {
  return invoke<string | null>("chat_pick_provider_home", { title });
}

export async function listChatProjectShells(): Promise<ChatProjectShellRead[]> {
  return parseChatProjectShells(await invoke<unknown>("chat_list_project_shells", { dbUrl: await ensureDbUrl() }));
}

export async function listChatThreads(
  workspaceId: ChatWorkspaceId | null,
  archived: boolean,
): Promise<ChatThreadShellRead[]> {
  return parseChatThreadShells(await invoke<unknown>("chat_list_threads", {
    dbUrl: await ensureDbUrl(),
    workspaceId,
    archived,
  }));
}

export async function searchChatThreadTitles(query: string, archived: boolean | null, limit = 100): Promise<ChatThreadShellRead[]> {
  return parseChatThreadShells(await invoke<unknown>("chat_search_thread_titles", {
    dbUrl: await ensureDbUrl(),
    query,
    archived,
    limit,
  }));
}

export async function readChatTimelinePage(
  threadId: ChatThreadId,
  cursor: string | null = null,
  limit = 100,
): Promise<ChatTimelinePageRead> {
  return parseChatTimelinePage(await invoke<unknown>("chat_read_timeline_page", {
    dbUrl: await ensureDbUrl(),
    threadId,
    cursor,
    limit,
  }));
}

export async function openChatExternalUrl(url: string): Promise<void> {
  await invoke("chat_open_external_url", { url });
}

async function threadMutation(command: string, args: Record<string, unknown>): Promise<ChatThreadShellRead> {
  return parseChatThreadShell(await invoke<unknown>(command, { dbUrl: await ensureDbUrl(), ...args }));
}

export function renameChatThread(threadId: ChatThreadId, title: string, expectedRevision: number): Promise<ChatThreadShellRead> {
  return threadMutation("chat_rename_thread", { threadId, title, expectedRevision });
}

export function setChatThreadRead(threadId: ChatThreadId, read: boolean, expectedRevision: number): Promise<ChatThreadShellRead> {
  return threadMutation("chat_set_thread_read", { threadId, read, expectedRevision });
}

export function archiveChatThread(threadId: ChatThreadId, expectedRevision: number): Promise<ChatThreadShellRead> {
  return threadMutation("chat_archive_thread", { threadId, expectedRevision });
}

export function restoreChatThread(threadId: ChatThreadId, expectedRevision: number): Promise<ChatThreadShellRead> {
  return threadMutation("chat_restore_thread", { threadId, expectedRevision });
}

export async function deleteChatThreadPermanently(
  threadId: ChatThreadId,
  expectedRevision: number,
  confirmedTitle: string,
): Promise<void> {
  await invoke("chat_delete_thread_permanently", {
    dbUrl: await ensureDbUrl(),
    threadId,
    expectedRevision,
    confirmedTitle,
  });
}
