import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";
import type {
  CreateProjectWorkingFolderRequest,
  ProjectWorkingFolderId,
  ProjectWorkingFolderRead,
} from "$lib/chat/contracts";
import {
  parseProjectWorkingFolderRead,
  parseProjectWorkingFolderReads,
} from "$lib/chat/validation";

export async function listProjectWorkingFolders(): Promise<ProjectWorkingFolderRead[]> {
  return parseProjectWorkingFolderReads(await invoke<unknown>(
    "projects_list_working_folders",
    { dbUrl: await ensureDbUrl() },
  ));
}

export async function listCachedProjectWorkingFolders(): Promise<ProjectWorkingFolderRead[]> {
  return parseProjectWorkingFolderReads(await invoke<unknown>(
    "projects_list_working_folders_cached",
    { dbUrl: await ensureDbUrl() },
  ));
}

export async function addExternalProjectWorkingFolder(
  request: CreateProjectWorkingFolderRequest,
  title: string,
): Promise<ProjectWorkingFolderRead | null> {
  const value = await invoke<unknown>("projects_add_external_working_folder", {
    dbUrl: await ensureDbUrl(),
    request,
    title,
  });
  return value === null ? null : parseProjectWorkingFolderRead(value);
}

export async function renameProjectWorkingFolder(
  workingFolderId: ProjectWorkingFolderId,
  displayName: string,
  expectedRevision: number,
): Promise<ProjectWorkingFolderRead> {
  return parseProjectWorkingFolderRead(await invoke<unknown>(
    "projects_rename_working_folder",
    { dbUrl: await ensureDbUrl(), workingFolderId, displayName, expectedRevision },
  ));
}

export async function locateProjectWorkingFolder(
  workingFolderId: ProjectWorkingFolderId,
  title: string,
): Promise<ProjectWorkingFolderRead | null> {
  const value = await invoke<unknown>("projects_locate_working_folder", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    title,
  });
  return value === null ? null : parseProjectWorkingFolderRead(value);
}

export async function rebindProjectWorkingFolder(
  workingFolderId: ProjectWorkingFolderId,
  title: string,
): Promise<ProjectWorkingFolderRead | null> {
  const value = await invoke<unknown>("projects_rebind_working_folder", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    title,
  });
  return value === null ? null : parseProjectWorkingFolderRead(value);
}

export async function unbindProjectWorkingFolder(
  workingFolderId: ProjectWorkingFolderId,
): Promise<ProjectWorkingFolderRead> {
  return parseProjectWorkingFolderRead(await invoke<unknown>(
    "projects_unbind_working_folder",
    { dbUrl: await ensureDbUrl(), workingFolderId },
  ));
}

export async function archiveProjectWorkingFolder(
  workingFolderId: ProjectWorkingFolderId,
  expectedRevision: number,
): Promise<ProjectWorkingFolderRead> {
  return parseProjectWorkingFolderRead(await invoke<unknown>(
    "projects_archive_working_folder",
    { dbUrl: await ensureDbUrl(), workingFolderId, expectedRevision },
  ));
}

export async function restoreProjectWorkingFolder(
  workingFolderId: ProjectWorkingFolderId,
  expectedRevision: number,
): Promise<ProjectWorkingFolderRead> {
  return parseProjectWorkingFolderRead(await invoke<unknown>(
    "projects_restore_working_folder",
    { dbUrl: await ensureDbUrl(), workingFolderId, expectedRevision },
  ));
}

export async function openProjectWorkingFolder(
  workingFolderId: ProjectWorkingFolderId,
): Promise<void> {
  await invoke("projects_open_working_folder", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
  });
}

export async function removeProjectWorkingFolder(
  workingFolderId: ProjectWorkingFolderId,
): Promise<void> {
  await invoke("projects_remove_working_folder", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
  });
}

export async function recreateManagedProjectWorkingFolder(
  workingFolderId: ProjectWorkingFolderId,
): Promise<ProjectWorkingFolderRead> {
  return parseProjectWorkingFolderRead(await invoke<unknown>(
    "projects_recreate_managed_working_folder",
    { dbUrl: await ensureDbUrl(), workingFolderId },
  ));
}

export async function rememberProjectWorkingFolder(
  projectId: string,
  workingFolderId: ProjectWorkingFolderId,
): Promise<void> {
  await invoke("projects_remember_working_folder", {
    dbUrl: await ensureDbUrl(),
    projectId,
    workingFolderId,
  });
}

export async function lastProjectWorkingFolder(
  projectId: string,
): Promise<ProjectWorkingFolderId | null> {
  return invoke<ProjectWorkingFolderId | null>("projects_last_working_folder", { projectId });
}
