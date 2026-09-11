import type { Translate } from "$lib/i18n/translator.svelte";
import type { NotesFileAssetMetadata, NotesMediaBlockType } from "./media";

export const NOTES_IMPORT_FILE_CONTEXTS = [
  "notion_export",
  "html_import",
  "markdown_import",
] as const;

export const NOTES_IMPORT_FILE_CHOICES = [
  "copy_local_file",
  "keep_external_reference",
  "skip",
] as const;

export type NotesImportFileContext = (typeof NOTES_IMPORT_FILE_CONTEXTS)[number];
export type NotesImportFileChoice = (typeof NOTES_IMPORT_FILE_CHOICES)[number];
export type NotesImportFileDiagnosticSeverity = "info" | "warning" | "error";
export type NotesImportFileReferenceKind =
  | "empty"
  | "external_url"
  | "managed_asset"
  | "local_file";

export type NotesImportFileAction =
  | "copied_asset"
  | "external_reference"
  | "skipped"
  | "blocked";

export interface NotesImportFileReferenceRequest {
  importContext: NotesImportFileContext;
  choice: NotesImportFileChoice;
  reference: string;
  importRoot?: string | null;
  blockType?: NotesMediaBlockType | null;
  originalName?: string | null;
}

export interface NotesImportFileDiagnostic {
  code: string;
  severity: NotesImportFileDiagnosticSeverity;
  message: string;
}

export interface NotesImportFileReferenceResult {
  action: NotesImportFileAction;
  asset: NotesFileAssetMetadata | null;
  externalUrl: string | null;
  diagnostics: NotesImportFileDiagnostic[];
}

export function classifyNotesImportFileReference(
  reference: string,
): NotesImportFileReferenceKind {
  const trimmed = reference.trim();
  if (!trimmed) return "empty";
  const lower = trimmed.toLowerCase();
  if (lower.startsWith("ganbaru-asset:")) return "managed_asset";
  if (lower.startsWith("http://") || lower.startsWith("https://")) return "external_url";
  return "local_file";
}

export function notesImportFileChoicesForReference(
  reference: string,
  hasImportRoot: boolean,
): NotesImportFileChoice[] {
  const kind = classifyNotesImportFileReference(reference);
  if (kind === "empty" || kind === "managed_asset") return ["skip"];
  if (kind === "external_url") return ["keep_external_reference", "skip"];
  return hasImportRoot ? ["copy_local_file", "skip"] : ["skip"];
}

export function notesImportFileChoiceLabel(
  t: Translate,
  choice: NotesImportFileChoice,
): string {
  if (choice === "copy_local_file") return t("notes.importFileChoiceCopy");
  if (choice === "keep_external_reference") return t("notes.importFileChoiceKeepExternal");
  return t("notes.importFileChoiceSkip");
}

export function notesImportFileDiagnosticText(
  t: Translate,
  diagnostic: NotesImportFileDiagnostic,
): string {
  if (diagnostic.code === "import_reference_copied") return t("notes.importFileCopied");
  if (diagnostic.code === "import_reference_external_kept") {
    return t("notes.importFileExternalKept");
  }
  if (diagnostic.code === "import_reference_skipped") return t("notes.importFileSkipped");
  if (diagnostic.code === "import_reference_requires_explicit_choice") {
    return t("notes.importFileChoosePolicy");
  }
  if (diagnostic.code === "import_reference_external_requires_https") {
    return t("notes.importFileRequiresHttps");
  }
  if (diagnostic.code === "import_reference_external_unsupported_type") {
    return t("notes.importFileUnsupportedType");
  }
  if (diagnostic.code === "import_reference_external_not_copied") {
    return t("notes.importFileExternalNotCopied");
  }
  if (diagnostic.code === "import_reference_root_required") {
    return t("notes.importFileRootRequired");
  }
  if (diagnostic.code === "import_reference_path_escape") {
    return t("notes.importFilePathEscape");
  }
  if (diagnostic.code === "import_reference_too_large") return t("notes.importFileTooLarge");
  return diagnostic.message;
}
