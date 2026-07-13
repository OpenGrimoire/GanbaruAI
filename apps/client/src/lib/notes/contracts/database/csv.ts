import type { NotesDataSourcePropertyType } from "./base";

export interface NotesDataSourceCsvImportRequest {
  csv: string;
  has_header?: boolean | null;
  dry_run?: boolean | null;
}

export interface NotesDataSourceCsvImportColumn {
  source_index: number;
  source_name: string;
  property_id: string | null;
  property_name: string | null;
  property_type: NotesDataSourcePropertyType | string | null;
  mapped: boolean;
  read_only: boolean;
  warning: string | null;
}

export type NotesDataSourceCsvImportDiagnosticSeverity = "info" | "warning" | "error";

export interface NotesDataSourceCsvImportDiagnostic {
  code: string;
  severity: NotesDataSourceCsvImportDiagnosticSeverity;
  row_number: number | null;
  column_index: number | null;
  column_name: string | null;
  property_id: string | null;
  message: string;
}

export interface NotesDataSourceCsvImportRow {
  row_number: number;
  title: string;
  valid: boolean;
  mapped_cell_count: number;
  error_count: number;
}

export interface NotesDataSourceCsvImportResult {
  object: "notes_data_source_csv_import";
  data_source_id: string;
  dry_run: boolean;
  total_row_count: number;
  valid_row_count: number;
  skipped_row_count: number;
  imported_row_count: number;
  imported_page_ids: string[];
  columns: NotesDataSourceCsvImportColumn[];
  rows: NotesDataSourceCsvImportRow[];
  diagnostics: NotesDataSourceCsvImportDiagnostic[];
}

export type NotesDataSourceCsvExportScope = "view" | "all";

export interface NotesDataSourceCsvExportRequest {
  database_id?: string | null;
  view_id?: string | null;
  scope?: NotesDataSourceCsvExportScope | null;
}

export type NotesDataSourceCsvExportDiagnosticSeverity = "info" | "warning" | "error";

export interface NotesDataSourceCsvExportDiagnostic {
  code: string;
  severity: NotesDataSourceCsvExportDiagnosticSeverity;
  property_id: string | null;
  property_name: string | null;
  message: string;
}

export interface NotesDataSourceCsvExportResult {
  object: "notes_data_source_csv_export";
  data_source_id: string;
  database_id: string;
  view_id: string;
  scope: NotesDataSourceCsvExportScope;
  file_name: string;
  csv: string;
  exported_row_count: number;
  exported_property_count: number;
  diagnostics: NotesDataSourceCsvExportDiagnostic[];
}

export interface NotesDataSourceCsvExportSaveResult {
  saved: boolean;
  export: NotesDataSourceCsvExportResult | null;
}
