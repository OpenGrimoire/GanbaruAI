import type { NotesPage } from "../core";
import type { NotesDataSource, NotesDatabaseView } from "./base";
import type { NotesDatabaseTableFilter, NotesDatabaseTableSort } from "./table";

export type NotesDatabaseListRowOpenMode = "full_page" | "side_panel";

export interface NotesDatabaseListConfiguration {
  group_property_id: string | null;
  group_order: string[];
  hidden_group_ids: string[];
  visible_property_ids: string[];
  row_open_mode: NotesDatabaseListRowOpenMode;
}

export interface NotesDataSourceListViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseListConfiguration;
}

export interface NotesDataSourceListGroup {
  id: string;
  name: string;
  color: string;
  hidden: boolean;
  rows: NotesPage[];
}

export interface NotesDataSourceListView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  rows: NotesPage[];
  total_row_count: number;
  next_cursor: string | null;
  has_more: boolean;
  group_counts: Record<string, number>;
}
