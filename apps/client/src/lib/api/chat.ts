import { invoke } from "@tauri-apps/api/core";
import type {
  ChatWorkspaceId,
  ChatWorkspaceRead,
  CreateChatWorkspaceRequest,
} from "$lib/chat/contracts";
import { parseChatWorkspaceRead, parseChatWorkspaceReads } from "$lib/chat/validation";

export async function listChatWorkspaces(): Promise<ChatWorkspaceRead[]> {
  return parseChatWorkspaceReads(await invoke<unknown>("chat_list_workspaces"));
}

export async function createChatWorkspace(
  request: CreateChatWorkspaceRequest,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_create_workspace", { request }));
}

export async function renameChatWorkspace(
  workspaceId: ChatWorkspaceId,
  displayName: string,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_rename_workspace", {
    workspaceId,
    displayName,
  }));
}

export async function bindChatWorkspace(
  workspaceId: ChatWorkspaceId,
): Promise<ChatWorkspaceRead | null> {
  const value = await invoke<unknown>("chat_bind_workspace", { workspaceId });
  return value === null ? null : parseChatWorkspaceRead(value);
}

export async function rebindChatWorkspace(
  workspaceId: ChatWorkspaceId,
): Promise<ChatWorkspaceRead | null> {
  const value = await invoke<unknown>("chat_rebind_workspace", { workspaceId });
  return value === null ? null : parseChatWorkspaceRead(value);
}

export async function removeChatWorkspaceBinding(
  workspaceId: ChatWorkspaceId,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_remove_workspace_binding", { workspaceId }));
}

export async function archiveChatWorkspace(
  workspaceId: ChatWorkspaceId,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_archive_workspace", { workspaceId }));
}

export async function restoreChatWorkspace(
  workspaceId: ChatWorkspaceId,
): Promise<ChatWorkspaceRead> {
  return parseChatWorkspaceRead(await invoke<unknown>("chat_restore_workspace", { workspaceId }));
}

export async function openChatWorkspaceFolder(workspaceId: ChatWorkspaceId): Promise<void> {
  await invoke("chat_open_workspace_folder", { workspaceId });
}
