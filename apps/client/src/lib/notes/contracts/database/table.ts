import type { NotesPage } from "../core";
import type { NotesDataSource, NotesDatabaseView } from "./base";

export type NotesDatabaseTableRowOpenMode = "full_page" | "side_panel";

export type NotesDatabaseTableFilterCondition =
  | "contains"
  | "equals"
  | "is_empty"
  | "is_not_empty"
  | "checked"
  | "unchecked";

export type NotesDatabaseTableSortDirection = "ascending" | "descending";

export interface NotesDatabaseTableFilter {
  property_id: string;
  condition: NotesDatabaseTableFilterCondition;
  value?: string | number | boolean | null;
}

export interface NotesDatabaseTableSort {
  property_id: string;
  direction: NotesDatabaseTableSortDirection;
}

export interface NotesDatabaseTableConfiguration {
  property_order: string[];
  hidden_property_ids: string[];
  column_widths: Record<string, number>;
  row_open_mode: NotesDatabaseTableRowOpenMode;
}

export interface NotesDataSourceTableViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseTableConfiguration;
}

export interface NotesDataSourceTableView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  rows: NotesPage[];
  total_row_count: number;
  next_cursor: string | null;
  has_more: boolean;
}
