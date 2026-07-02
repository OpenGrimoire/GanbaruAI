import {
  parseNotesBacklink,
  parseNotesBlock,
  parseNotesCommentThread,
  parseNotesCreatedDatabase,
  parseNotesDataSourceBoardView,
  parseNotesDataSourceCalendarView,
  parseNotesDataSourceGalleryView,
  parseNotesDataSourceListView,
  parseNotesDataSourceSchema,
  parseNotesDataSourceTableView,
  parseNotesLoadedPage,
  parseNotesPage,
  parseNotesPageBreadcrumbItem,
  parseNotesPageHistorySettings,
  parseNotesPageHistorySnapshot,
  parseNotesPageTemplate,
  parseNotesPaginatedBlockList,
  parseNotesSearchResult,
  parseNotesSidebarPageList,
} from "./block-validation";
import type {
  NotesBacklink,
  NotesBlock,
  NotesCommentThread,
  NotesCreatedDatabase,
  NotesDataSourceBoardView,
  NotesDataSourceCalendarView,
  NotesDataSourceGalleryView,
  NotesDataSourceListView,
  NotesDataSourceSchema,
  NotesDataSourceTableView,
  NotesLoadedPage,
  NotesPage,
  NotesPageBreadcrumbItem,
  NotesPageHistorySettings,
  NotesPageHistorySnapshot,
  NotesPageTemplate,
  NotesPaginatedBlockList,
  NotesSearchResult,
  NotesSidebarPageList,
} from "./types";

/** Validate and map an unknown page DTO from the Tauri boundary. */
export function mapNotesPageDto(value: unknown): NotesPage {
  return parseNotesPage(value);
}

/** Validate and map an unknown block DTO from the Tauri boundary. */
export function mapNotesBlockDto(value: unknown): NotesBlock {
  return parseNotesBlock(value);
}

/** Validate and map an unknown paginated block list from the Tauri boundary. */
export function mapNotesBlockListDto(value: unknown): NotesPaginatedBlockList {
  return parseNotesPaginatedBlockList(value);
}

/** Validate and map an unknown loaded page DTO from the Tauri boundary. */
export function mapNotesLoadedPageDto(value: unknown): NotesLoadedPage {
  return parseNotesLoadedPage(value);
}

/** Validate and map an unknown sidebar page-list DTO from the Tauri boundary. */
export function mapNotesSidebarPageListDto(value: unknown): NotesSidebarPageList {
  return parseNotesSidebarPageList(value);
}

/** Validate and map an unknown page breadcrumb item DTO from the Tauri boundary. */
export function mapNotesPageBreadcrumbItemDto(value: unknown): NotesPageBreadcrumbItem {
  return parseNotesPageBreadcrumbItem(value);
}

/** Validate and map an unknown page template DTO from the Tauri boundary. */
export function mapNotesPageTemplateDto(value: unknown): NotesPageTemplate {
  return parseNotesPageTemplate(value);
}

/** Validate and map an unknown page history snapshot DTO from the Tauri boundary. */
export function mapNotesPageHistorySnapshotDto(value: unknown): NotesPageHistorySnapshot {
  return parseNotesPageHistorySnapshot(value);
}

/** Validate and map an unknown page history settings DTO from the Tauri boundary. */
export function mapNotesPageHistorySettingsDto(value: unknown): NotesPageHistorySettings {
  return parseNotesPageHistorySettings(value);
}

/** Validate and map an unknown backlink DTO from the Tauri boundary. */
export function mapNotesBacklinkDto(value: unknown): NotesBacklink {
  return parseNotesBacklink(value);
}

/** Validate and map an unknown Notes search result DTO from the Tauri boundary. */
export function mapNotesSearchResultDto(value: unknown): NotesSearchResult {
  return parseNotesSearchResult(value);
}

/** Validate and map an unknown comment thread DTO from the Tauri boundary. */
export function mapNotesCommentThreadDto(value: unknown): NotesCommentThread {
  return parseNotesCommentThread(value);
}

/** Validate and map an unknown local database create DTO from the Tauri boundary. */
export function mapNotesCreatedDatabaseDto(value: unknown): NotesCreatedDatabase {
  return parseNotesCreatedDatabase(value);
}

/** Validate and map an unknown local data source schema DTO from the Tauri boundary. */
export function mapNotesDataSourceSchemaDto(value: unknown): NotesDataSourceSchema {
  return parseNotesDataSourceSchema(value);
}

/** Validate and map an unknown local data source table view DTO from the Tauri boundary. */
export function mapNotesDataSourceTableViewDto(value: unknown): NotesDataSourceTableView {
  return parseNotesDataSourceTableView(value);
}

/** Validate and map an unknown local data source board view DTO from the Tauri boundary. */
export function mapNotesDataSourceBoardViewDto(value: unknown): NotesDataSourceBoardView {
  return parseNotesDataSourceBoardView(value);
}

/** Validate and map an unknown local data source gallery view DTO from the Tauri boundary. */
export function mapNotesDataSourceGalleryViewDto(value: unknown): NotesDataSourceGalleryView {
  return parseNotesDataSourceGalleryView(value);
}

/** Validate and map an unknown local data source list view DTO from the Tauri boundary. */
export function mapNotesDataSourceListViewDto(value: unknown): NotesDataSourceListView {
  return parseNotesDataSourceListView(value);
}

/** Validate and map an unknown local data source calendar view DTO from the Tauri boundary. */
export function mapNotesDataSourceCalendarViewDto(value: unknown): NotesDataSourceCalendarView {
  return parseNotesDataSourceCalendarView(value);
}
