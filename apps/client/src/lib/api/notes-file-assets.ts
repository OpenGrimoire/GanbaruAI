import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";
import type { NotesFileAssetMetadata, NotesMediaBlockType } from "$lib/notes/media";
import type {
  NotesImportFileDiagnostic,
  NotesImportFileReferenceRequest,
  NotesImportFileReferenceResult,
  NotesImportFileAction,
} from "$lib/notes/import-file-policy";

interface NotesFileAssetDto {
  relativePath: string;
  originalName: string | null;
  contentType: string;
  byteSize: number;
  sha256: string;
  kind: NotesMediaBlockType;
}

interface NotesImportFileReferenceDto {
  action: NotesImportFileAction;
  asset: NotesFileAssetDto | null;
  externalUrl: string | null;
  diagnostics: NotesImportFileDiagnostic[];
}

const notesFileAssetUrls = new Map<string, string>();

function mapNotesFileAssetDto(value: NotesFileAssetDto): NotesFileAssetMetadata {
  return {
    relativePath: value.relativePath,
    originalName: value.originalName,
    contentType: value.contentType,
    byteSize: value.byteSize,
    sha256: value.sha256,
    kind: value.kind,
  };
}

/** Open a native file picker and copy the selected file into managed Notes assets. */
export async function pickNotesFileAsset(
  blockType: NotesMediaBlockType,
): Promise<NotesFileAssetMetadata | null> {
  const dbUrl = await ensureDbUrl();
  const asset = await invoke<NotesFileAssetDto | null>("notes_pick_file_asset", {
    dbUrl,
    blockType,
  });
  return asset ? mapNotesFileAssetDto(asset) : null;
}

/** Prepare a user-approved import file reference for safe Notes persistence. */
export async function prepareNotesImportFileReference(
  request: NotesImportFileReferenceRequest,
): Promise<NotesImportFileReferenceResult> {
  const dbUrl = await ensureDbUrl();
  const result = await invoke<NotesImportFileReferenceDto>(
    "notes_prepare_import_file_reference",
    {
      dbUrl,
      request,
    },
  );
  return {
    action: result.action,
    asset: result.asset ? mapNotesFileAssetDto(result.asset) : null,
    externalUrl: result.externalUrl,
    diagnostics: result.diagnostics,
  };
}

/** Load a managed Notes file asset as a data URL for local preview rendering. */
export async function notesFileAssetUrl(relativePath: string): Promise<string> {
  const cached = notesFileAssetUrls.get(relativePath);
  if (cached) return cached;
  const dbUrl = await ensureDbUrl();
  const assetUrl = await invoke<string>("notes_file_asset_data_url", { dbUrl, relativePath });
  notesFileAssetUrls.set(relativePath, assetUrl);
  return assetUrl;
}
