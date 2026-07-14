import type { NotesPage } from "../core";
import type { NotesDataSource, NotesDatabaseView } from "./base";
import type { NotesDatabaseTableFilter, NotesDatabaseTableSort } from "./table";

export type NotesDatabaseGalleryRowOpenMode = "full_page" | "side_panel";

export type NotesDatabaseGalleryCoverSource = "page_cover" | "files_property" | "none";

export type NotesDatabaseGalleryCardSize = "small" | "medium" | "large";

export interface NotesDatabaseGalleryConfiguration {
  cover_source: NotesDatabaseGalleryCoverSource;
  cover_property_id: string | null;
  visible_property_ids: string[];
  card_size: NotesDatabaseGalleryCardSize;
  fit_image: boolean;
  row_open_mode: NotesDatabaseGalleryRowOpenMode;
}

export interface NotesDataSourceGalleryViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseGalleryConfiguration;
}

export interface NotesDataSourceGalleryView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  rows: NotesPage[];
  total_row_count: number;
  next_cursor: string | null;
  has_more: boolean;
}
