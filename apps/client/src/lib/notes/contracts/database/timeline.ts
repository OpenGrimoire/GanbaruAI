import type { NotesPage } from "../core";
import type { NotesDataSource, NotesDatabaseView } from "./base";
import type { NotesDatabaseTableFilter, NotesDatabaseTableSort } from "./table";

export type NotesDatabaseTimelineRowOpenMode = "full_page" | "side_panel";

export interface NotesDatabaseTimelineConfiguration {
  date_property_id: string | null;
  group_property_id: string | null;
  group_order: string[];
  hidden_group_ids: string[];
  range_start: string;
  range_end: string;
  visible_property_ids: string[];
  row_open_mode: NotesDatabaseTimelineRowOpenMode;
}

export interface NotesDataSourceTimelineViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseTimelineConfiguration;
}

export interface NotesDataSourceTimelineGroup {
  id: string;
  name: string;
  color: string;
  hidden: boolean;
  rows: NotesPage[];
}

export interface NotesDataSourceTimelineView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  rows: NotesPage[];
  total_row_count: number;
  next_cursor: string | null;
  has_more: boolean;
  group_counts: Record<string, number>;
}
