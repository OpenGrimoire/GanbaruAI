import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";
import {
  mapNotesHtmlArchiveSaveDto,
  mapNotesHtmlExportDto,
  mapNotesHtmlImportDto,
  mapNotesAgentBridgeExportDto,
  mapNotesAgentBridgeExportSaveDto,
  mapNotesJsonGraphExportDto,
  mapNotesJsonGraphExportSaveDto,
  mapNotesNotionApiImportDto,
  mapNotesNotionExportImportDto,
  mapNotesMarkdownExportDto,
  mapNotesMarkdownImportDto,
} from "$lib/notes/notion-mappers";
import type {
  NotesHtmlArchiveSaveResult,
  NotesHtmlExportRequest,
  NotesHtmlExportResult,
  NotesAgentBridgeExportRequest,
  NotesAgentBridgeExportResult,
  NotesAgentBridgeExportSaveResult,
  NotesJsonGraphExportRequest,
  NotesJsonGraphExportResult,
  NotesJsonGraphExportSaveResult,
  NotesHtmlImportRequest,
  NotesHtmlImportResult,
  NotesNotionApiImportRequest,
  NotesNotionApiImportResult,
  NotesNotionExportImportRequest,
  NotesNotionExportImportResult,
  NotesMarkdownExportRequest,
  NotesMarkdownExportResult,
  NotesMarkdownImportRequest,
  NotesMarkdownImportResult,
} from "$lib/notes/types";
import { invokeNotesMutation } from "./mutation";

export async function importNotesMarkdownPage(
  request: NotesMarkdownImportRequest,
): Promise<NotesMarkdownImportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesMarkdownImportDto(
    await invokeNotesMutation("notes_import_markdown_page", { dbUrl, request }, true),
  );
}

export async function importNotesHtmlPage(
  request: NotesHtmlImportRequest,
): Promise<NotesHtmlImportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesHtmlImportDto(
    await invokeNotesMutation("notes_import_html_page", { dbUrl, request }, true),
  );
}

export async function importNotesNotionApi(
  request: NotesNotionApiImportRequest,
): Promise<NotesNotionApiImportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesNotionApiImportDto(
    await invokeNotesMutation("notes_import_notion_api", { dbUrl, request }, true),
  );
}

export async function importNotesNotionExportFolder(
  request: NotesNotionExportImportRequest,
): Promise<NotesNotionExportImportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesNotionExportImportDto(
    await invokeNotesMutation("notes_import_notion_export_folder", { dbUrl, request }, true),
  );
}

export async function exportNotesMarkdownPage(
  request: NotesMarkdownExportRequest,
): Promise<NotesMarkdownExportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesMarkdownExportDto(
    await invoke<unknown>("notes_export_markdown_page", { dbUrl, request }),
  );
}

export async function exportNotesHtmlPage(
  request: NotesHtmlExportRequest,
): Promise<NotesHtmlExportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesHtmlExportDto(await invoke<unknown>("notes_export_html_page", { dbUrl, request }));
}

export async function saveNotesHtmlArchive(
  request: NotesHtmlExportRequest,
): Promise<NotesHtmlArchiveSaveResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesHtmlArchiveSaveDto(
    await invoke<unknown>("notes_pick_and_write_html_archive", { dbUrl, request }),
  );
}

export async function exportNotesJsonGraph(
  request: NotesJsonGraphExportRequest,
): Promise<NotesJsonGraphExportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesJsonGraphExportDto(
    await invoke<unknown>("notes_export_json_graph", { dbUrl, request }),
  );
}

export async function saveNotesJsonGraph(
  request: NotesJsonGraphExportRequest,
): Promise<NotesJsonGraphExportSaveResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesJsonGraphExportSaveDto(
    await invoke<unknown>("notes_pick_and_write_json_graph", { dbUrl, request }),
  );
}

export async function exportNotesAgentBridge(
  request: NotesAgentBridgeExportRequest,
): Promise<NotesAgentBridgeExportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesAgentBridgeExportDto(
    await invoke<unknown>("notes_export_agent_bridge", { dbUrl, request }),
  );
}

export async function saveNotesAgentBridge(
  request: NotesAgentBridgeExportRequest,
): Promise<NotesAgentBridgeExportSaveResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesAgentBridgeExportSaveDto(
    await invoke<unknown>("notes_pick_and_write_agent_bridge", { dbUrl, request }),
  );
}
