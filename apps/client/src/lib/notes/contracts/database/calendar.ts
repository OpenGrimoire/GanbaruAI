import type { NotesPage } from "../core";
import type { NotesDataSource, NotesDatabaseView } from "./base";
import type { NotesDatabaseTableFilter, NotesDatabaseTableSort } from "./table";

export type NotesDatabaseCalendarRowOpenMode = "full_page" | "side_panel";

export interface NotesDatabaseCalendarConfiguration {
  date_property_id: string | null;
  range_start: string;
  range_end: string;
  visible_property_ids: string[];
  row_open_mode: NotesDatabaseCalendarRowOpenMode;
}

export interface NotesDataSourceCalendarViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseCalendarConfiguration;
}

export interface NotesDataSourceCalendarDay {
  date: string;
  in_month: boolean;
  rows: NotesPage[];
}

export interface NotesDataSourceCalendarView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  rows: NotesPage[];
  total_row_count: number;
  next_cursor: string | null;
  has_more: boolean;
}
