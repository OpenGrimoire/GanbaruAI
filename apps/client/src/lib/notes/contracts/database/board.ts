import type { NotesPage } from "../core";
import type { NotesDataSource, NotesDatabaseView } from "./base";
import type { NotesDatabaseTableFilter, NotesDatabaseTableSort } from "./table";

export type NotesDatabaseBoardRowOpenMode = "full_page" | "side_panel";

export interface NotesDatabaseBoardConfiguration {
  group_property_id: string | null;
  group_order: string[];
  hidden_group_ids: string[];
  visible_property_ids: string[];
  row_open_mode: NotesDatabaseBoardRowOpenMode;
}

export interface NotesDataSourceBoardViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseBoardConfiguration;
}

export interface NotesDataSourceBoardRowMove {
  page_id: string;
  group_id: string;
}

export interface NotesDataSourceBoardGroup {
  id: string;
  name: string;
  color: string;
  hidden: boolean;
  rows: NotesPage[];
}

export interface NotesDataSourceBoardView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  groups: NotesDataSourceBoardGroup[];
  total_row_count: number;
  next_cursor: string | null;
  has_more: boolean;
  group_counts: Record<string, number>;
}
