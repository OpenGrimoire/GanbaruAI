import type { NotesAgentBridgeExportDiagnostic, NotesAgentBridgeExportDiagnosticSeverity, NotesAgentBridgeExportResult, NotesAgentBridgeExportSaveResult, NotesHtmlArchiveSaveResult, NotesHtmlExportAsset, NotesHtmlExportDiagnostic, NotesHtmlExportDiagnosticSeverity, NotesHtmlExportFile, NotesHtmlExportResult, NotesJsonGraphExportDiagnostic, NotesJsonGraphExportDiagnosticSeverity, NotesJsonGraphExportResult, NotesJsonGraphExportSaveResult } from "../contracts/transfers";
import { readBoolean, readNonNegativeInteger, readNullableString, readNullableUuidString, readRecord, readString, readUuidString } from "./readers";

export function parseNotesHtmlExportResult(value: unknown): NotesHtmlExportResult {
  const record = readRecord(value, "HTML export result");
  if (record.object !== "notes_html_archive_export") {
    throw new Error("HTML export result.object must be notes_html_archive_export");
  }
  if (!Array.isArray(record.files)) throw new Error("HTML export result.files must be an array");
  if (!Array.isArray(record.assets)) throw new Error("HTML export result.assets must be an array");
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("HTML export result.diagnostics must be an array");
  }
  return {
    object: "notes_html_archive_export",
    root_page_id: readUuidString(record.root_page_id, "HTML export result.root_page_id"),
    files: record.files.map(parseNotesHtmlExportFile),
    assets: record.assets.map(parseNotesHtmlExportAsset),
    diagnostics: record.diagnostics.map(parseNotesHtmlExportDiagnostic),
    manifest_json: readString(record.manifest_json, "HTML export result.manifest_json"),
    exported_page_count: readNonNegativeInteger(
      record.exported_page_count,
      "HTML export result.exported_page_count",
    ),
    exported_block_count: readNonNegativeInteger(
      record.exported_block_count,
      "HTML export result.exported_block_count",
    ),
    exported_asset_count: readNonNegativeInteger(
      record.exported_asset_count,
      "HTML export result.exported_asset_count",
    ),
    exported_comment_count: readNonNegativeInteger(
      record.exported_comment_count,
      "HTML export result.exported_comment_count",
    ),
    exported_database_view_count: readNonNegativeInteger(
      record.exported_database_view_count,
      "HTML export result.exported_database_view_count",
    ),
  };
}

export function parseNotesHtmlArchiveSaveResult(value: unknown): NotesHtmlArchiveSaveResult {
  const record = readRecord(value, "HTML archive save result");
  if (record.object !== "notes_html_archive_save") {
    throw new Error("HTML archive save result.object must be notes_html_archive_save");
  }
  return {
    object: "notes_html_archive_save",
    saved: readBoolean(record.saved, "HTML archive save result.saved"),
    export: record.export === null
      ? null
      : parseNotesHtmlExportResult(record.export),
  };
}

export function parseNotesJsonGraphExportResult(value: unknown): NotesJsonGraphExportResult {
  const record = readRecord(value, "JSON graph export result");
  if (record.object !== "notes_json_graph_export") {
    throw new Error("JSON graph export result.object must be notes_json_graph_export");
  }
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("JSON graph export result.diagnostics must be an array");
  }
  return {
    object: "notes_json_graph_export",
    export_version: readNonNegativeInteger(
      record.export_version,
      "JSON graph export result.export_version",
    ),
    schema_version: readString(record.schema_version, "JSON graph export result.schema_version"),
    generated_at: readString(record.generated_at, "JSON graph export result.generated_at"),
    file_name: readString(record.file_name, "JSON graph export result.file_name"),
    content_type: readString(record.content_type, "JSON graph export result.content_type"),
    json: readString(record.json, "JSON graph export result.json"),
    byte_size: readNonNegativeInteger(record.byte_size, "JSON graph export result.byte_size"),
    counts: readRecord(record.counts, "JSON graph export result.counts"),
    diagnostics: record.diagnostics.map(parseNotesJsonGraphExportDiagnostic),
    exported_page_count: readNonNegativeInteger(
      record.exported_page_count,
      "JSON graph export result.exported_page_count",
    ),
    exported_block_count: readNonNegativeInteger(
      record.exported_block_count,
      "JSON graph export result.exported_block_count",
    ),
    exported_comment_count: readNonNegativeInteger(
      record.exported_comment_count,
      "JSON graph export result.exported_comment_count",
    ),
    exported_data_source_count: readNonNegativeInteger(
      record.exported_data_source_count,
      "JSON graph export result.exported_data_source_count",
    ),
    exported_file_count: readNonNegativeInteger(
      record.exported_file_count,
      "JSON graph export result.exported_file_count",
    ),
    exported_index_record_count: readNonNegativeInteger(
      record.exported_index_record_count,
      "JSON graph export result.exported_index_record_count",
    ),
    exported_property_schema_count: readNonNegativeInteger(
      record.exported_property_schema_count,
      "JSON graph export result.exported_property_schema_count",
    ),
    exported_table_count: readNonNegativeInteger(
      record.exported_table_count,
      "JSON graph export result.exported_table_count",
    ),
    exported_record_count: readNonNegativeInteger(
      record.exported_record_count,
      "JSON graph export result.exported_record_count",
    ),
    warning_count: readNonNegativeInteger(
      record.warning_count,
      "JSON graph export result.warning_count",
    ),
  };
}

export function parseNotesJsonGraphExportSaveResult(
  value: unknown,
): NotesJsonGraphExportSaveResult {
  const record = readRecord(value, "JSON graph export save result");
  if (record.object !== "notes_json_graph_export_save") {
    throw new Error("JSON graph export save result.object must be notes_json_graph_export_save");
  }
  return {
    object: "notes_json_graph_export_save",
    saved: readBoolean(record.saved, "JSON graph export save result.saved"),
    export: record.export === null
      ? null
      : parseNotesJsonGraphExportResult(record.export),
  };
}

function parseNotesJsonGraphExportDiagnostic(
  value: unknown,
  index: number,
): NotesJsonGraphExportDiagnostic {
  const record = readRecord(value, `JSON graph export result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `JSON graph export result.diagnostics[${index}].severity`,
  );
  if (!isJsonGraphExportDiagnosticSeverity(severity)) {
    throw new Error(`JSON graph export result.diagnostics[${index}].severity is unsupported`);
  }
  return {
    code: readString(record.code, `JSON graph export result.diagnostics[${index}].code`),
    severity,
    table_name: readNullableString(
      record.table_name,
      `JSON graph export result.diagnostics[${index}].table_name`,
    ),
    row_id: readNullableString(
      record.row_id,
      `JSON graph export result.diagnostics[${index}].row_id`,
    ),
    message: readString(record.message, `JSON graph export result.diagnostics[${index}].message`),
  };
}

function isJsonGraphExportDiagnosticSeverity(
  value: string,
): value is NotesJsonGraphExportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesAgentBridgeExportResult(value: unknown): NotesAgentBridgeExportResult {
  const record = readRecord(value, "agent bridge export result");
  if (record.object !== "notes_agent_bridge_export") {
    throw new Error("agent bridge export result.object must be notes_agent_bridge_export");
  }
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("agent bridge export result.diagnostics must be an array");
  }
  return {
    object: "notes_agent_bridge_export",
    export_version: readNonNegativeInteger(
      record.export_version,
      "agent bridge export result.export_version",
    ),
    schema_version: readString(record.schema_version, "agent bridge export result.schema_version"),
    file_name: readString(record.file_name, "agent bridge export result.file_name"),
    content_type: readString(record.content_type, "agent bridge export result.content_type"),
    markdown: readString(record.markdown, "agent bridge export result.markdown"),
    byte_size: readNonNegativeInteger(record.byte_size, "agent bridge export result.byte_size"),
    diagnostics: record.diagnostics.map(parseNotesAgentBridgeExportDiagnostic),
    exported_page_count: readNonNegativeInteger(
      record.exported_page_count,
      "agent bridge export result.exported_page_count",
    ),
    exported_project_count: readNonNegativeInteger(
      record.exported_project_count,
      "agent bridge export result.exported_project_count",
    ),
    exported_task_count: readNonNegativeInteger(
      record.exported_task_count,
      "agent bridge export result.exported_task_count",
    ),
    exported_database_view_count: readNonNegativeInteger(
      record.exported_database_view_count,
      "agent bridge export result.exported_database_view_count",
    ),
    exported_backlink_count: readNonNegativeInteger(
      record.exported_backlink_count,
      "agent bridge export result.exported_backlink_count",
    ),
    warning_count: readNonNegativeInteger(
      record.warning_count,
      "agent bridge export result.warning_count",
    ),
  };
}

export function parseNotesAgentBridgeExportSaveResult(
  value: unknown,
): NotesAgentBridgeExportSaveResult {
  const record = readRecord(value, "agent bridge export save result");
  if (record.object !== "notes_agent_bridge_export_save") {
    throw new Error(
      "agent bridge export save result.object must be notes_agent_bridge_export_save",
    );
  }
  return {
    object: "notes_agent_bridge_export_save",
    saved: readBoolean(record.saved, "agent bridge export save result.saved"),
    export: record.export === null
      ? null
      : parseNotesAgentBridgeExportResult(record.export),
  };
}

function parseNotesAgentBridgeExportDiagnostic(
  value: unknown,
  index: number,
): NotesAgentBridgeExportDiagnostic {
  const record = readRecord(value, `agent bridge export result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `agent bridge export result.diagnostics[${index}].severity`,
  );
  if (!isAgentBridgeExportDiagnosticSeverity(severity)) {
    throw new Error(`agent bridge export result.diagnostics[${index}].severity is unsupported`);
  }
  return {
    code: readString(record.code, `agent bridge export result.diagnostics[${index}].code`),
    severity,
    source_type: readNullableString(
      record.source_type,
      `agent bridge export result.diagnostics[${index}].source_type`,
    ),
    source_id: readNullableString(
      record.source_id,
      `agent bridge export result.diagnostics[${index}].source_id`,
    ),
    message: readString(record.message, `agent bridge export result.diagnostics[${index}].message`),
  };
}

function isAgentBridgeExportDiagnosticSeverity(
  value: string,
): value is NotesAgentBridgeExportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

function parseNotesHtmlExportFile(value: unknown, index: number): NotesHtmlExportFile {
  const record = readRecord(value, `HTML export result.files[${index}]`);
  return {
    path: readString(record.path, `HTML export result.files[${index}].path`),
    content_type: readString(
      record.content_type,
      `HTML export result.files[${index}].content_type`,
    ),
    contents: readString(record.contents, `HTML export result.files[${index}].contents`),
    byte_size: readNonNegativeInteger(
      record.byte_size,
      `HTML export result.files[${index}].byte_size`,
    ),
  };
}

function parseNotesHtmlExportAsset(value: unknown, index: number): NotesHtmlExportAsset {
  const record = readRecord(value, `HTML export result.assets[${index}]`);
  return {
    id: readString(record.id, `HTML export result.assets[${index}].id`),
    archive_path: readString(
      record.archive_path,
      `HTML export result.assets[${index}].archive_path`,
    ),
    source_path: readString(
      record.source_path,
      `HTML export result.assets[${index}].source_path`,
    ),
    content_type: readString(
      record.content_type,
      `HTML export result.assets[${index}].content_type`,
    ),
    byte_size: readNonNegativeInteger(
      record.byte_size,
      `HTML export result.assets[${index}].byte_size`,
    ),
    sha256: readString(record.sha256, `HTML export result.assets[${index}].sha256`),
    storage_state: readString(
      record.storage_state,
      `HTML export result.assets[${index}].storage_state`,
    ),
    exported: readBoolean(record.exported, `HTML export result.assets[${index}].exported`),
  };
}

function parseNotesHtmlExportDiagnostic(
  value: unknown,
  index: number,
): NotesHtmlExportDiagnostic {
  const record = readRecord(value, `HTML export result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `HTML export result.diagnostics[${index}].severity`,
  );
  if (!isHtmlExportDiagnosticSeverity(severity)) {
    throw new Error(`HTML export result.diagnostics[${index}].severity is unsupported`);
  }
  return {
    code: readString(record.code, `HTML export result.diagnostics[${index}].code`),
    severity,
    page_id: readNullableUuidString(
      record.page_id,
      `HTML export result.diagnostics[${index}].page_id`,
    ),
    block_id: readNullableUuidString(
      record.block_id,
      `HTML export result.diagnostics[${index}].block_id`,
    ),
    asset_id: readNullableString(
      record.asset_id,
      `HTML export result.diagnostics[${index}].asset_id`,
    ),
    comment_id: readNullableUuidString(
      record.comment_id,
      `HTML export result.diagnostics[${index}].comment_id`,
    ),
    message: readString(record.message, `HTML export result.diagnostics[${index}].message`),
  };
}

function isHtmlExportDiagnosticSeverity(
  value: string,
): value is NotesHtmlExportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}
