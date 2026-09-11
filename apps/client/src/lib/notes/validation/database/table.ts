import type { NotesDataSourceTableView, NotesDatabaseTableFilterCondition, NotesDatabaseTableSortDirection } from "../../contracts/database";
import { isNotesTableRowOpenMode } from ".././blocks";
import { readBoolean, readInteger, readNullableString, readRecord, readRecordArray, readString, readStringArray } from ".././readers";
import { parseNotesPage } from ".././workspace";
import { parseNotesDataSource, parseNotesDatabaseView } from "./base";

function isNotesTableFilterCondition(value: unknown): value is NotesDatabaseTableFilterCondition {
  return (
    value === "contains"
    || value === "equals"
    || value === "is_empty"
    || value === "is_not_empty"
    || value === "checked"
    || value === "unchecked"
  );
}

function isNotesTableSortDirection(value: unknown): value is NotesDatabaseTableSortDirection {
  return value === "ascending" || value === "descending";
}

function validateNotesTableConfiguration(value: Record<string, unknown> | null): void {
  if (value === null) return;
  if (value.type !== undefined && value.type !== "table") {
    throw new Error("database table configuration.type must be table");
  }
  const table = readRecord(value.table, "database table configuration.table");
  readStringArray(table.property_order, "database table configuration.property_order");
  readStringArray(table.hidden_property_ids, "database table configuration.hidden_property_ids");
  const columnWidths = readRecord(
    table.column_widths ?? {},
    "database table configuration.column_widths",
  );
  for (const [propertyId, width] of Object.entries(columnWidths)) {
    readString(propertyId, "database table configuration.column_widths key");
    const parsedWidth = readInteger(width, `database table configuration.column_widths.${propertyId}`);
    if (parsedWidth < 96 || parsedWidth > 480) {
      throw new Error(`database table configuration.column_widths.${propertyId} is out of range`);
    }
  }
  const rowOpenMode = table.row_open_mode ?? "full_page";
  if (!isNotesTableRowOpenMode(rowOpenMode)) {
    throw new Error("database table configuration.row_open_mode must be supported");
  }
}

export function validateNotesTableFilter(value: Record<string, unknown> | null): void {
  if (value === null) return;
  if (value.type !== undefined && value.type !== "and") {
    throw new Error("database table filter.type must be and");
  }
  const filters = readRecordArray(value.filters ?? [], "database table filter.filters");
  for (const [index, filter] of filters.entries()) {
    readString(filter.property_id, `database table filter.filters[${index}].property_id`);
    if (!isNotesTableFilterCondition(filter.condition)) {
      throw new Error(`database table filter.filters[${index}].condition must be supported`);
    }
    const filterValue = filter.value;
    if (
      filterValue !== undefined
      && filterValue !== null
      && typeof filterValue !== "string"
      && typeof filterValue !== "number"
      && typeof filterValue !== "boolean"
    ) {
      throw new Error(`database table filter.filters[${index}].value must be scalar`);
    }
  }
}

export function validateNotesTableSorts(value: Record<string, unknown>[]): void {
  for (const [index, sort] of value.entries()) {
    readString(sort.property_id, `database table sorts[${index}].property_id`);
    if (!isNotesTableSortDirection(sort.direction)) {
      throw new Error(`database table sorts[${index}].direction must be supported`);
    }
  }
}

export function parseNotesDataSourceTableView(value: unknown): NotesDataSourceTableView {
  const record = readRecord(value, "data source table view");
  const dataSource = parseNotesDataSource(record.data_source);
  const view = parseNotesDatabaseView(record.view);
  if (view.type !== "table") {
    throw new Error("data source table view.view.type must be table");
  }
  if (view.data_source_id !== dataSource.id) {
    throw new Error("data source table view ids must match");
  }
  validateNotesTableConfiguration(view.configuration);
  validateNotesTableFilter(view.filter);
  validateNotesTableSorts(view.sorts);
  if (!Array.isArray(record.rows)) throw new Error("data source table view.rows must be an array");
  return {
    data_source: dataSource,
    view,
    rows: record.rows.map(parseNotesPage),
    total_row_count: readInteger(record.total_row_count, "data source table view.total_row_count"),
    next_cursor: readNullableString(record.next_cursor, "data source table view.next_cursor"),
    has_more: readBoolean(record.has_more, "data source table view.has_more"),
  };
}
