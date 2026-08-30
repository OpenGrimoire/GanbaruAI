import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";
import type {
  NotesWorkingMarkdownFileRead,
  NotesWorkingMarkdownNode,
  NotesWorkingMarkdownRoot,
  NotesWorkingMarkdownTreeRead,
} from "$lib/notes/types";

export async function listNotesWorkingMarkdown(
  projectId: string,
): Promise<NotesWorkingMarkdownTreeRead> {
  return parseWorkingMarkdownTree(await invoke<unknown>("notes_list_working_markdown", {
    dbUrl: await ensureDbUrl(),
    projectId,
  }));
}

export async function readNotesWorkingMarkdown(
  workingFolderId: string,
  relativePath: string,
): Promise<NotesWorkingMarkdownFileRead> {
  return parseWorkingMarkdownFile(await invoke<unknown>("notes_read_working_markdown", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    relativePath,
  }));
}

export async function saveNotesWorkingMarkdown(
  workingFolderId: string,
  relativePath: string,
  content: string,
  expectedRevision: string,
): Promise<NotesWorkingMarkdownFileRead> {
  return parseWorkingMarkdownFile(await invoke<unknown>("notes_save_working_markdown", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    relativePath,
    content,
    expectedRevision,
  }));
}

export async function openNotesWorkingMarkdown(
  workingFolderId: string,
  relativePath: string,
): Promise<void> {
  await invoke("notes_open_working_markdown", {
    dbUrl: await ensureDbUrl(),
    workingFolderId,
    relativePath,
  });
}

function parseWorkingMarkdownTree(value: unknown): NotesWorkingMarkdownTreeRead {
  const record = workingMarkdownRecord(value, "working Markdown tree");
  if (!Array.isArray(record.roots) || !Array.isArray(record.unavailableWorkingFolderIds)) {
    throw new Error("notes_list_working_markdown returned an invalid tree");
  }
  return {
    roots: record.roots.map(parseWorkingMarkdownRoot),
    unavailableWorkingFolderIds: record.unavailableWorkingFolderIds.map((id) =>
      workingMarkdownString(id, "unavailable working folder ID")
    ),
  };
}

function parseWorkingMarkdownRoot(value: unknown): NotesWorkingMarkdownRoot {
  const record = workingMarkdownRecord(value, "working Markdown root");
  const sourceKind = workingMarkdownString(record.sourceKind, "source kind");
  if (sourceKind !== "managed" && sourceKind !== "external") {
    throw new Error("notes_list_working_markdown returned an invalid source kind");
  }
  if (!Array.isArray(record.nodes) || typeof record.truncated !== "boolean") {
    throw new Error("notes_list_working_markdown returned an invalid root");
  }
  return {
    workingFolderId: workingMarkdownString(record.workingFolderId, "working folder ID"),
    displayName: workingMarkdownString(record.displayName, "working folder name"),
    sourceKind,
    displayPath: workingMarkdownString(record.displayPath, "working folder path"),
    nodes: record.nodes.map(parseWorkingMarkdownNode),
    truncated: record.truncated,
  };
}

function parseWorkingMarkdownNode(value: unknown): NotesWorkingMarkdownNode {
  const record = workingMarkdownRecord(value, "working Markdown node");
  const kind = workingMarkdownString(record.kind, "node kind");
  if ((kind !== "directory" && kind !== "file") || !Array.isArray(record.children)) {
    throw new Error("notes_list_working_markdown returned an invalid node");
  }
  return {
    kind,
    name: workingMarkdownString(record.name, "node name"),
    relativePath: workingMarkdownString(record.relativePath, "relative path"),
    children: record.children.map(parseWorkingMarkdownNode),
  };
}

function parseWorkingMarkdownFile(value: unknown): NotesWorkingMarkdownFileRead {
  const record = workingMarkdownRecord(value, "working Markdown file");
  if (typeof record.byteSize !== "number" || !Number.isSafeInteger(record.byteSize) || record.byteSize < 0) {
    throw new Error("working Markdown command returned an invalid byte size");
  }
  return {
    workingFolderId: workingMarkdownString(record.workingFolderId, "working folder ID"),
    relativePath: workingMarkdownString(record.relativePath, "relative path"),
    content: workingMarkdownString(record.content, "content", true),
    revision: workingMarkdownString(record.revision, "revision"),
    byteSize: record.byteSize,
  };
}

function workingMarkdownRecord(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`Notes returned an invalid ${label}`);
  }
  return value as Record<string, unknown>;
}

function workingMarkdownString(value: unknown, label: string, allowEmpty = false): string {
  if (typeof value !== "string" || (!allowEmpty && value.length === 0)) {
    throw new Error(`Notes returned an invalid ${label}`);
  }
  return value;
}
