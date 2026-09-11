import type { NotesHtmlImportDiagnostic, NotesHtmlImportDiagnosticSeverity, NotesHtmlImportResult, NotesMarkdownExportDiagnostic, NotesMarkdownExportDiagnosticSeverity, NotesMarkdownExportResult, NotesMarkdownImportDiagnostic, NotesMarkdownImportDiagnosticSeverity, NotesMarkdownImportResult, NotesNotionApiImportDiagnostic, NotesNotionApiImportDiagnosticSeverity, NotesNotionApiImportResult, NotesNotionApiImportedObject, NotesNotionApiImportedUser, NotesNotionExportImportDiagnostic, NotesNotionExportImportDiagnosticSeverity, NotesNotionExportImportResult } from "../contracts/transfers";
import { readInteger, readNonNegativeInteger, readNullableString, readNullableUuidString, readRecord, readString, readUuidString } from "./readers";
import { parseNotesLoadedPage } from "./workspace";

export function parseNotesMarkdownImportResult(value: unknown): NotesMarkdownImportResult {
  const record = readRecord(value, "markdown import result");
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("markdown import result.diagnostics must be an array");
  }
  const importedBlockCount = readInteger(
    record.imported_block_count,
    "markdown import result.imported_block_count",
  );
  if (importedBlockCount < 0) {
    throw new Error("markdown import result.imported_block_count must not be negative");
  }
  return {
    page: parseNotesLoadedPage(record.page),
    diagnostics: record.diagnostics.map(parseNotesMarkdownImportDiagnostic),
    imported_block_count: importedBlockCount,
  };
}

function parseNotesMarkdownImportDiagnostic(
  value: unknown,
  index: number,
): NotesMarkdownImportDiagnostic {
  const record = readRecord(value, `markdown import result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `markdown import result.diagnostics[${index}].severity`,
  );
  if (!isMarkdownImportDiagnosticSeverity(severity)) {
    throw new Error(`markdown import result.diagnostics[${index}].severity is unsupported`);
  }
  const line = record.line === null
    ? null
    : readInteger(record.line, `markdown import result.diagnostics[${index}].line`);
  if (line !== null && line < 1) {
    throw new Error(`markdown import result.diagnostics[${index}].line must be positive`);
  }
  return {
    code: readString(record.code, `markdown import result.diagnostics[${index}].code`),
    severity,
    line,
    message: readString(record.message, `markdown import result.diagnostics[${index}].message`),
  };
}

function isMarkdownImportDiagnosticSeverity(
  value: string,
): value is NotesMarkdownImportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesHtmlImportResult(value: unknown): NotesHtmlImportResult {
  const record = readRecord(value, "HTML import result");
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("HTML import result.diagnostics must be an array");
  }
  const importedBlockCount = readInteger(
    record.imported_block_count,
    "HTML import result.imported_block_count",
  );
  if (importedBlockCount < 0) {
    throw new Error("HTML import result.imported_block_count must not be negative");
  }
  return {
    page: parseNotesLoadedPage(record.page),
    diagnostics: record.diagnostics.map(parseNotesHtmlImportDiagnostic),
    imported_block_count: importedBlockCount,
  };
}

function parseNotesHtmlImportDiagnostic(
  value: unknown,
  index: number,
): NotesHtmlImportDiagnostic {
  const record = readRecord(value, `HTML import result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `HTML import result.diagnostics[${index}].severity`,
  );
  if (!isHtmlImportDiagnosticSeverity(severity)) {
    throw new Error(`HTML import result.diagnostics[${index}].severity is unsupported`);
  }
  const line = record.line === null
    ? null
    : readInteger(record.line, `HTML import result.diagnostics[${index}].line`);
  if (line !== null && line < 1) {
    throw new Error(`HTML import result.diagnostics[${index}].line must be positive`);
  }
  return {
    code: readString(record.code, `HTML import result.diagnostics[${index}].code`),
    severity,
    line,
    message: readString(record.message, `HTML import result.diagnostics[${index}].message`),
  };
}

function isHtmlImportDiagnosticSeverity(
  value: string,
): value is NotesHtmlImportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesNotionApiImportResult(value: unknown): NotesNotionApiImportResult {
  const record = readRecord(value, "Notion API import result");
  const object = readString(record.object, "Notion API import result.object");
  if (object !== "notes_notion_api_import") {
    throw new Error("Notion API import result.object is unsupported");
  }
  if (!Array.isArray(record.imported_pages)) {
    throw new Error("Notion API import result.imported_pages must be an array");
  }
  if (!Array.isArray(record.imported_data_sources)) {
    throw new Error("Notion API import result.imported_data_sources must be an array");
  }
  if (!Array.isArray(record.imported_users)) {
    throw new Error("Notion API import result.imported_users must be an array");
  }
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("Notion API import result.diagnostics must be an array");
  }
  return {
    object,
    imported_pages: record.imported_pages.map(parseNotesLoadedPage),
    imported_data_sources: record.imported_data_sources.map(parseNotesNotionApiImportedObject),
    imported_users: record.imported_users.map(parseNotesNotionApiImportedUser),
    diagnostics: record.diagnostics.map(parseNotesNotionApiImportDiagnostic),
    request_count: readNonNegativeInteger(record.request_count, "Notion API import result.request_count"),
    retry_count: readNonNegativeInteger(record.retry_count, "Notion API import result.retry_count"),
    rate_limit_count: readNonNegativeInteger(
      record.rate_limit_count,
      "Notion API import result.rate_limit_count",
    ),
    imported_page_count: readNonNegativeInteger(
      record.imported_page_count,
      "Notion API import result.imported_page_count",
    ),
    imported_block_count: readNonNegativeInteger(
      record.imported_block_count,
      "Notion API import result.imported_block_count",
    ),
    imported_data_source_count: readNonNegativeInteger(
      record.imported_data_source_count,
      "Notion API import result.imported_data_source_count",
    ),
    imported_comment_count: readNonNegativeInteger(
      record.imported_comment_count,
      "Notion API import result.imported_comment_count",
    ),
    imported_user_count: readNonNegativeInteger(
      record.imported_user_count,
      "Notion API import result.imported_user_count",
    ),
    imported_file_count: readNonNegativeInteger(
      record.imported_file_count,
      "Notion API import result.imported_file_count",
    ),
    unsupported_block_count: readNonNegativeInteger(
      record.unsupported_block_count,
      "Notion API import result.unsupported_block_count",
    ),
  };
}

function parseNotesNotionApiImportDiagnostic(
  value: unknown,
  index: number,
): NotesNotionApiImportDiagnostic {
  const record = readRecord(value, `Notion API import result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `Notion API import result.diagnostics[${index}].severity`,
  );
  if (!isNotionApiImportDiagnosticSeverity(severity)) {
    throw new Error(`Notion API import result.diagnostics[${index}].severity is unsupported`);
  }
  return {
    code: readString(record.code, `Notion API import result.diagnostics[${index}].code`),
    severity,
    source_object_id: readNullableString(
      record.source_object_id,
      `Notion API import result.diagnostics[${index}].source_object_id`,
    ),
    message: readString(record.message, `Notion API import result.diagnostics[${index}].message`),
  };
}

function parseNotesNotionApiImportedObject(
  value: unknown,
  index: number,
): NotesNotionApiImportedObject {
  const record = readRecord(value, `Notion API import result.imported_data_sources[${index}]`);
  return {
    object_type: readString(record.object_type, `Notion API import result.imported_data_sources[${index}].object_type`),
    source_object_id: readString(
      record.source_object_id,
      `Notion API import result.imported_data_sources[${index}].source_object_id`,
    ),
    local_id: readUuidString(record.local_id, `Notion API import result.imported_data_sources[${index}].local_id`),
    title: readString(record.title, `Notion API import result.imported_data_sources[${index}].title`),
  };
}

function parseNotesNotionApiImportedUser(value: unknown, index: number): NotesNotionApiImportedUser {
  const record = readRecord(value, `Notion API import result.imported_users[${index}]`);
  return {
    source_user_id: readString(record.source_user_id, `Notion API import result.imported_users[${index}].source_user_id`),
    name: readString(record.name, `Notion API import result.imported_users[${index}].name`),
    user_type: readString(record.user_type, `Notion API import result.imported_users[${index}].user_type`),
  };
}

function isNotionApiImportDiagnosticSeverity(
  value: string,
): value is NotesNotionApiImportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesNotionExportImportResult(
  value: unknown,
): NotesNotionExportImportResult {
  const record = readRecord(value, "Notion export import result");
  const object = readString(record.object, "Notion export import result.object");
  if (object !== "notes_notion_export_import") {
    throw new Error("Notion export import result.object must be notes_notion_export_import");
  }
  if (!Array.isArray(record.imported_pages)) {
    throw new Error("Notion export import result.imported_pages must be an array");
  }
  if (!Array.isArray(record.imported_data_sources)) {
    throw new Error("Notion export import result.imported_data_sources must be an array");
  }
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("Notion export import result.diagnostics must be an array");
  }
  return {
    object,
    imported_pages: record.imported_pages.map(parseNotesLoadedPage),
    imported_data_sources: record.imported_data_sources.map(parseNotesNotionApiImportedObject),
    diagnostics: record.diagnostics.map(parseNotesNotionExportImportDiagnostic),
    imported_page_count: readNonNegativeInteger(
      record.imported_page_count,
      "Notion export import result.imported_page_count",
    ),
    imported_block_count: readNonNegativeInteger(
      record.imported_block_count,
      "Notion export import result.imported_block_count",
    ),
    imported_data_source_count: readNonNegativeInteger(
      record.imported_data_source_count,
      "Notion export import result.imported_data_source_count",
    ),
    imported_file_count: readNonNegativeInteger(
      record.imported_file_count,
      "Notion export import result.imported_file_count",
    ),
    skipped_file_count: readNonNegativeInteger(
      record.skipped_file_count,
      "Notion export import result.skipped_file_count",
    ),
    unsupported_block_count: readNonNegativeInteger(
      record.unsupported_block_count,
      "Notion export import result.unsupported_block_count",
    ),
  };
}

function parseNotesNotionExportImportDiagnostic(
  value: unknown,
  index: number,
): NotesNotionExportImportDiagnostic {
  const record = readRecord(value, `Notion export import result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `Notion export import result.diagnostics[${index}].severity`,
  );
  if (!isNotionExportImportDiagnosticSeverity(severity)) {
    throw new Error(`Notion export import result.diagnostics[${index}].severity is unsupported`);
  }
  return {
    code: readString(record.code, `Notion export import result.diagnostics[${index}].code`),
    severity,
    source_path: readNullableString(
      record.source_path,
      `Notion export import result.diagnostics[${index}].source_path`,
    ),
    message: readString(record.message, `Notion export import result.diagnostics[${index}].message`),
  };
}

function isNotionExportImportDiagnosticSeverity(
  value: string,
): value is NotesNotionExportImportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesMarkdownExportResult(value: unknown): NotesMarkdownExportResult {
  const record = readRecord(value, "markdown export result");
  if (record.object !== "notes_markdown_export") {
    throw new Error("markdown export result.object must be notes_markdown_export");
  }
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("markdown export result.diagnostics must be an array");
  }
  const exportedBlockCount = readInteger(
    record.exported_block_count,
    "markdown export result.exported_block_count",
  );
  if (exportedBlockCount < 0) {
    throw new Error("markdown export result.exported_block_count must not be negative");
  }
  const exportedCommentCount = readInteger(
    record.exported_comment_count,
    "markdown export result.exported_comment_count",
  );
  if (exportedCommentCount < 0) {
    throw new Error("markdown export result.exported_comment_count must not be negative");
  }
  return {
    object: "notes_markdown_export",
    page_id: readUuidString(record.page_id, "markdown export result.page_id"),
    markdown: readString(record.markdown, "markdown export result.markdown"),
    diagnostics: record.diagnostics.map(parseNotesMarkdownExportDiagnostic),
    exported_block_count: exportedBlockCount,
    exported_comment_count: exportedCommentCount,
  };
}

function parseNotesMarkdownExportDiagnostic(
  value: unknown,
  index: number,
): NotesMarkdownExportDiagnostic {
  const record = readRecord(value, `markdown export result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `markdown export result.diagnostics[${index}].severity`,
  );
  if (!isMarkdownExportDiagnosticSeverity(severity)) {
    throw new Error(`markdown export result.diagnostics[${index}].severity is unsupported`);
  }
  return {
    code: readString(record.code, `markdown export result.diagnostics[${index}].code`),
    severity,
    block_id: readNullableUuidString(
      record.block_id,
      `markdown export result.diagnostics[${index}].block_id`,
    ),
    comment_id: readNullableUuidString(
      record.comment_id,
      `markdown export result.diagnostics[${index}].comment_id`,
    ),
    message: readString(record.message, `markdown export result.diagnostics[${index}].message`),
  };
}

function isMarkdownExportDiagnosticSeverity(
  value: string,
): value is NotesMarkdownExportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}
