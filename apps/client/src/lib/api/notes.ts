import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";
import {
  invalidateAssetUrlKind,
  invalidateNotesAssetUrls,
} from "$lib/api/asset-url-cache";
import { invalidateNotesNotificationSchedule } from "$lib/notes/notification-schedule.svelte";
import { createRichText } from "$lib/notes/block-factory";
import { NOTES_PAGE_PROJECT_ID_PROPERTY } from "$lib/notes/project-membership";
import {
  applyNotesProjectHistoryMutationDeadline,
  notifyNotesProjectHistoryMutation,
} from "$lib/notes/project-history-scheduler";
import {
  mapNotesBacklinkDto,
  mapNotesBlockDto,
  mapNotesBlockListDto,
  mapNotesCommentThreadDto,
  mapNotesCreatedDatabaseDto,
  mapNotesDataSourceDto,
  mapNotesDataSourceBoardViewDto,
  mapNotesDataSourceCalendarViewDto,
  mapNotesDataSourceCsvExportDto,
  mapNotesDataSourceCsvExportSaveDto,
  mapNotesDataSourceCsvImportDto,
  mapNotesDataSourceGalleryViewDto,
  mapNotesDataSourceListViewDto,
  mapNotesDataSourceSchemaDto,
  mapNotesDataSourceTableViewDto,
  mapNotesDataSourceTemplateDto,
  mapNotesDataSourceTimelineViewDto,
  mapNotesFolderDto,
  mapNotesHtmlArchiveSaveDto,
  mapNotesHtmlExportDto,
  mapNotesHtmlImportDto,
  mapNotesAgentBridgeExportDto,
  mapNotesAgentBridgeExportSaveDto,
  mapNotesJsonGraphExportDto,
  mapNotesJsonGraphExportSaveDto,
  mapNotesLocalUserDto,
  mapNotesLoadedPageDto,
  mapNotesNotionApiImportDto,
  mapNotesNotionExportImportDto,
  mapNotesMarkdownExportDto,
  mapNotesMarkdownImportDto,
  mapNotesMentionNotificationDto,
  mapNotesPageBreadcrumbItemDto,
  mapNotesPageAliasDto,
  mapNotesPageHistorySettingsDto,
  mapNotesPageHistorySnapshotDto,
  mapNotesPageDto,
  mapNotesPageTemplateDto,
  mapNotesSearchResultDto,
  mapNotesSidebarPageListDto,
  mapNotesSuggestionDto,
  mapNotesUnresolvedLinkDto,
} from "$lib/notes/notion-mappers";
import type {
  NotesAppendBlockChildrenRequest,
  NotesBacklink,
  NotesBlock,
  NotesChildPageFromBlockCreate,
  NotesCommentCreate,
  NotesCommentThread,
  NotesCommentThreadReadUpdate,
  NotesCommentUpdate,
  NotesCreatedDatabase,
  NotesDatabaseCreateRequest,
  NotesDataSourceBoardRowMove,
  NotesDataSourceBoardView,
  NotesDataSourceBoardViewUpdate,
  NotesDataSourceButtonClickRequest,
  NotesDataSourceCalendarView,
  NotesDataSourceCalendarViewUpdate,
  NotesDataSourceCsvExportRequest,
  NotesDataSourceCsvExportResult,
  NotesDataSourceCsvExportSaveResult,
  NotesDataSourceCsvImportRequest,
  NotesDataSourceCsvImportResult,
  NotesDataSourceGalleryView,
  NotesDataSourceGalleryViewUpdate,
  NotesDataSourceListView,
  NotesDataSourceListViewUpdate,
  NotesDatabaseViewScope,
  NotesDataSource,
  NotesDataSourceRowPageCreateRequest,
  NotesDataSourceRowPropertyUpdate,
  NotesDataSourceSchema,
  NotesDataSourceSchemaUpdate,
  NotesDataSourceTableView,
  NotesDataSourceTableViewUpdate,
  NotesDataSourceTemplate,
  NotesDataSourceTemplateApplyRequest,
  NotesDataSourceTemplateCreateFromRowRequest,
  NotesDataSourceTemplateDuplicateRequest,
  NotesDataSourceTemplateUpdateRequest,
  NotesDataSourceTimelineView,
  NotesDataSourceTimelineViewUpdate,
  NotesDuplicatePageRequest,
  NotesDuplicateBlockRequest,
  NotesDuplicateBlocksRequest,
  NotesBlockUpdate,
  NotesFolder,
  NotesFolderCreate,
  NotesFolderUpdate,
  NotesHtmlArchiveSaveResult,
  NotesHtmlExportRequest,
  NotesHtmlExportResult,
  NotesAgentBridgeExportRequest,
  NotesAgentBridgeExportResult,
  NotesAgentBridgeExportSaveResult,
  NotesJsonGraphExportRequest,
  NotesJsonGraphExportResult,
  NotesJsonGraphExportSaveResult,
  NotesLinkedDatabaseCreateRequest,
  NotesLocalUser,
  NotesLocalUserUpdate,
  NotesLoadedPage,
  NotesHtmlImportRequest,
  NotesHtmlImportResult,
  NotesNotionApiImportRequest,
  NotesNotionApiImportResult,
  NotesNotionExportImportRequest,
  NotesNotionExportImportResult,
  NotesMarkdownExportRequest,
  NotesMarkdownExportResult,
  NotesMarkdownImportRequest,
  NotesMarkdownImportResult,
  NotesMentionNotification,
  NotesMentionNotificationDeliveryUpdate,
  NotesMovePageRequest,
  NotesMoveBlockRequest,
  NotesMoveBlocksRequest,
  NotesPage,
  NotesPageAlias,
  NotesPageAliasCreate,
  NotesPageBreadcrumbItem,
  NotesPageCreate,
  NotesPageHistoryCopyBlocksRequest,
  NotesPageHistorySettings,
  NotesPageHistorySettingsUpdate,
  NotesPageHistorySnapshot,
  NotesPageTemplate,
  NotesPageTemplateApplyRequest,
  NotesPageTemplateCreateFromPageRequest,
  NotesPageTemplateDuplicateRequest,
  NotesPageTemplateUpdateRequest,
  NotesPageUpdate,
  NotesPaginatedBlockList,
  NotesSearchResult,
  NotesSidebarPageList,
  NotesSidebarPagesRequest,
  NotesSuggestion,
  NotesSuggestionCreate,
  NotesTrashBlocksRequest,
  NotesUnresolvedLink,
  NotesUnresolvedLinkResolve,
  NotesWorkspaceShell,
  NotesWorkspaceShellRequest,
} from "$lib/notes/types";

function databaseViewScopeArgs(scope?: NotesDatabaseViewScope | null): {
  databaseId: string | null;
  viewId: string | null;
} {
  return {
    databaseId: scope?.databaseId ?? null,
    viewId: scope?.viewId ?? null,
  };
}

async function invokeNotesMutation(
  command: string,
  args: Record<string, unknown>,
  forceCheckpoint = false,
): Promise<unknown> {
  const result = await invoke<unknown>(command, args);
  if (typeof result === "object" && result !== null && !Array.isArray(result)) {
    const record = result as Record<string, unknown>;
    if (Object.hasOwn(record, "value") && Object.hasOwn(record, "nextHistoryCheckpointAt")) {
      const deadline = record.nextHistoryCheckpointAt;
      if (deadline !== null && typeof deadline !== "string") {
        throw new Error(`${command} returned an invalid Notes history deadline`);
      }
      applyNotesProjectHistoryMutationDeadline(deadline as string | null);
      return record.value;
    }
  }
  notifyNotesProjectHistoryMutation(forceCheckpoint);
  return result;
}

function schemaViewScopeArgs(scope?: NotesDatabaseViewScope | null): { viewId: string | null } {
  return { viewId: scope?.viewId ?? null };
}

export async function listNotesPages(): Promise<NotesPage[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_pages", { dbUrl });
  if (!Array.isArray(rows)) throw new Error("notes_list_pages returned a non-array payload");
  return rows.map(mapNotesPageDto);
}

export async function listNotesFolders(): Promise<NotesFolder[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_folders", { dbUrl });
  if (!Array.isArray(rows)) throw new Error("notes_list_folders returned a non-array payload");
  return rows.map(mapNotesFolderDto);
}

function notesWorkspaceShellRecord(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("notes_load_workspace_shell returned an invalid payload");
  }
  return value as Record<string, unknown>;
}

function shellString(value: unknown, field: string): string {
  if (typeof value !== "string") throw new Error(`${field} must be a string`);
  return value;
}

function shellNullableString(value: unknown, field: string): string | null {
  if (value === null) return null;
  return shellString(value, field);
}

function shellStringArray(value: unknown, field: string): string[] {
  if (!Array.isArray(value)) throw new Error(`${field} must be an array`);
  return value.map((item, index) => shellString(item, `${field}[${index}]`));
}

function shellNumber(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${field} must be a non-negative integer`);
  }
  return value;
}

function mapWorkspaceShellPage(value: unknown): NotesPage {
  const row = notesWorkspaceShellRecord(value);
  const parentType = shellString(row.parent_type, "page.parent_type");
  let parent: NotesPage["parent"];
  if (parentType === "workspace") {
    parent = { type: "workspace", workspace: true };
  } else if (parentType === "page_id") {
    parent = { type: "page_id", page_id: shellString(row.parent_page_id, "page.parent_page_id") };
  } else if (parentType === "block_id") {
    parent = { type: "block_id", block_id: shellString(row.parent_block_id, "page.parent_block_id") };
  } else if (parentType === "data_source_id") {
    parent = {
      type: "data_source_id",
      data_source_id: shellString(row.parent_data_source_id, "page.parent_data_source_id"),
    };
  } else {
    throw new Error("page.parent_type is invalid");
  }
  const title = shellString(row.title, "page.title");
  const projectId = shellNullableString(row.project_id, "page.project_id");
  const rawIcon = shellNullableString(row.icon, "page.icon");
  return mapNotesPageDto({
    object: "page",
    id: shellString(row.id, "page.id"),
    created_time: shellString(row.created_time, "page.created_time"),
    last_edited_time: shellString(row.last_edited_time, "page.last_edited_time"),
    parent,
    folder_id: shellNullableString(row.folder_id, "page.folder_id"),
    in_trash: false,
    archived: false,
    icon: rawIcon ? JSON.parse(rawIcon) : null,
    cover: null,
    properties: {
      title: { id: "title", type: "title", title: title ? [createRichText(title)] : [] },
      ...(projectId ? { [NOTES_PAGE_PROJECT_ID_PROPERTY]: projectId } : {}),
    },
    url: null,
    public_url: null,
    source_provider: null,
    source_object_id: null,
    source_workspace_id: null,
    source_last_edited_time: null,
  });
}

export async function loadNotesWorkspaceShell(
  request: NotesWorkspaceShellRequest,
): Promise<NotesWorkspaceShell> {
  const dbUrl = await ensureDbUrl();
  const value = await invoke<unknown>("notes_load_workspace_shell", { dbUrl, request });
  const record = notesWorkspaceShellRecord(value);
  if (!Array.isArray(record.pages) || !Array.isArray(record.folders)) {
    throw new Error("notes_load_workspace_shell returned invalid collections");
  }
  return {
    pages: record.pages.map(mapWorkspaceShellPage),
    folders: record.folders.map(mapNotesFolderDto),
    page_ids_with_children: shellStringArray(record.page_ids_with_children, "page_ids_with_children"),
    missing_parent_page_ids: shellStringArray(record.missing_parent_page_ids, "missing_parent_page_ids"),
    trashed_parent_page_ids: shellStringArray(record.trashed_parent_page_ids, "trashed_parent_page_ids"),
    resolved_selected_page_id: shellNullableString(
      record.resolved_selected_page_id,
      "resolved_selected_page_id",
    ),
    total_page_count: shellNumber(record.total_page_count, "total_page_count"),
    total_folder_count: shellNumber(record.total_folder_count, "total_folder_count"),
    next_page_cursor: shellNullableString(record.next_page_cursor, "next_page_cursor"),
    next_folder_cursor: shellNullableString(record.next_folder_cursor, "next_folder_cursor"),
  };
}

export async function listNotesDestinationCandidates(
  projectId: string | null,
  cursor: string | null = null,
): Promise<NotesWorkspaceShell> {
  return loadNotesWorkspaceShell({
    project_id: projectId,
    expanded_page_ids: [],
    seed_page_ids: [],
    selected_page_id: null,
    page_cursor: cursor,
    destination_candidates: true,
  });
}

export async function createNotesFolder(folder: NotesFolderCreate): Promise<NotesFolder> {
  const dbUrl = await ensureDbUrl();
  return mapNotesFolderDto(await invokeNotesMutation("notes_create_folder", { dbUrl, folder }));
}

export async function updateNotesFolder(
  folderId: string,
  update: NotesFolderUpdate,
): Promise<NotesFolder> {
  const dbUrl = await ensureDbUrl();
  return mapNotesFolderDto(
    await invokeNotesMutation("notes_update_folder", { dbUrl, folderId, update }),
  );
}

export async function deleteNotesFolder(folderId: string): Promise<string> {
  const dbUrl = await ensureDbUrl();
  const deletedFolderId = await invokeNotesMutation("notes_delete_folder", { dbUrl, folderId });
  if (typeof deletedFolderId !== "string") {
    throw new Error("notes_delete_folder returned a non-string payload");
  }
  return deletedFolderId;
}

export async function listTrashedNotesPages(): Promise<NotesPage[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_trashed_pages", { dbUrl });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_trashed_pages returned a non-array payload");
  }
  return rows.map(mapNotesPageDto);
}

export async function listArchivedNotesPages(): Promise<NotesPage[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_archived_pages", { dbUrl });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_archived_pages returned a non-array payload");
  }
  return rows.map(mapNotesPageDto);
}

export async function listNotesSidebarPages(
  request: NotesSidebarPagesRequest,
): Promise<NotesSidebarPageList> {
  const dbUrl = await ensureDbUrl();
  return mapNotesSidebarPageListDto(
    await invoke<unknown>("notes_list_sidebar_pages", { dbUrl, request }),
  );
}

export async function listNotesBacklinks(pageId: string): Promise<NotesBacklink[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_backlinks", { dbUrl, pageId });
  if (!Array.isArray(rows)) throw new Error("notes_list_backlinks returned a non-array payload");
  return rows.map(mapNotesBacklinkDto);
}

export async function rebuildNotesBacklinkIndex(): Promise<number> {
  const dbUrl = await ensureDbUrl();
  const count = await invoke<unknown>("notes_rebuild_backlink_index", { dbUrl });
  if (typeof count !== "number") {
    throw new Error("notes_rebuild_backlink_index returned a non-number payload");
  }
  return count;
}

export async function rebuildNotesLinkFacts(): Promise<number> {
  const dbUrl = await ensureDbUrl();
  const count = await invoke<unknown>("notes_rebuild_link_facts", { dbUrl });
  if (typeof count !== "number") {
    throw new Error("notes_rebuild_link_facts returned a non-number payload");
  }
  return count;
}

export async function listNotesPageAliases(pageId: string): Promise<NotesPageAlias[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_page_aliases", { dbUrl, pageId });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_page_aliases returned a non-array payload");
  }
  return rows.map(mapNotesPageAliasDto);
}

export async function addNotesPageAlias(
  pageId: string,
  request: NotesPageAliasCreate,
): Promise<NotesPageAlias[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invokeNotesMutation("notes_add_page_alias", { dbUrl, pageId, request });
  if (!Array.isArray(rows)) {
    throw new Error("notes_add_page_alias returned a non-array payload");
  }
  return rows.map(mapNotesPageAliasDto);
}

export async function deleteNotesPageAlias(
  pageId: string,
  aliasId: string,
): Promise<NotesPageAlias[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invokeNotesMutation("notes_delete_page_alias", { dbUrl, pageId, aliasId });
  if (!Array.isArray(rows)) {
    throw new Error("notes_delete_page_alias returned a non-array payload");
  }
  return rows.map(mapNotesPageAliasDto);
}

export async function listNotesUnresolvedLinks(pageId: string): Promise<NotesUnresolvedLink[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_unresolved_links", { dbUrl, pageId });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_unresolved_links returned a non-array payload");
  }
  return rows.map(mapNotesUnresolvedLinkDto);
}

export async function resolveNotesUnresolvedLink(
  linkId: string,
  request: NotesUnresolvedLinkResolve,
): Promise<NotesUnresolvedLink[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invokeNotesMutation("notes_resolve_unresolved_link", {
    dbUrl,
    linkId,
    request,
  });
  if (!Array.isArray(rows)) {
    throw new Error("notes_resolve_unresolved_link returned a non-array payload");
  }
  return rows.map(mapNotesUnresolvedLinkDto);
}

export async function getNotesPageBreadcrumb(pageId: string): Promise<NotesPageBreadcrumbItem[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_get_page_breadcrumb", { dbUrl, pageId });
  if (!Array.isArray(rows)) {
    throw new Error("notes_get_page_breadcrumb returned a non-array payload");
  }
  return rows.map(mapNotesPageBreadcrumbItemDto);
}

export async function searchNotes(
  query: string,
  pageSize = 20,
  includeResolvedComments = false,
): Promise<NotesSearchResult[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_search", {
    dbUrl,
    query,
    pageSize,
    includeResolvedComments,
  });
  if (!Array.isArray(rows)) throw new Error("notes_search returned a non-array payload");
  return rows.map(mapNotesSearchResultDto);
}

export async function rebuildNotesSearchIndex(): Promise<number> {
  const dbUrl = await ensureDbUrl();
  const count = await invoke<unknown>("notes_rebuild_search_index", { dbUrl });
  if (typeof count !== "number") {
    throw new Error("notes_rebuild_search_index returned a non-number payload");
  }
  return count;
}

export async function getNotesLocalUser(): Promise<NotesLocalUser> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLocalUserDto(await invoke<unknown>("notes_get_local_user", { dbUrl }));
}

export async function updateNotesLocalUser(
  update: NotesLocalUserUpdate,
): Promise<NotesLocalUser> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLocalUserDto(
    await invoke<unknown>("notes_update_local_user", { dbUrl, update }),
  );
}

export async function refreshNotesMentionNotifications(): Promise<number> {
  const dbUrl = await ensureDbUrl();
  const refreshed = await invoke<unknown>("notes_refresh_mention_notifications", { dbUrl });
  if (typeof refreshed !== "number" || !Number.isInteger(refreshed)) {
    throw new Error("notes_refresh_mention_notifications returned an invalid count");
  }
  invalidateNotesNotificationSchedule();
  return refreshed;
}

export async function listPendingNotesMentionNotifications():
  Promise<NotesMentionNotification[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_pending_mention_notifications", { dbUrl });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_pending_mention_notifications returned a non-array payload");
  }
  return rows.map(mapNotesMentionNotificationDto);
}

export async function markNotesMentionNotificationsDelivered(
  request: NotesMentionNotificationDeliveryUpdate,
): Promise<NotesMentionNotification[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_mark_mention_notifications_delivered", {
    dbUrl,
    request,
  });
  if (!Array.isArray(rows)) {
    throw new Error("notes_mark_mention_notifications_delivered returned a non-array payload");
  }
  return rows.map(mapNotesMentionNotificationDto);
}

export async function listNotesPageTemplates(): Promise<NotesPageTemplate[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_page_templates", { dbUrl });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_page_templates returned a non-array payload");
  }
  return rows.map(mapNotesPageTemplateDto);
}

export async function createNotesPageTemplateFromPage(
  request: NotesPageTemplateCreateFromPageRequest,
): Promise<NotesPageTemplate> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageTemplateDto(
    await invoke<unknown>("notes_create_page_template_from_page", { dbUrl, request }),
  );
}

export async function applyNotesPageTemplate(
  templateId: string,
  request: NotesPageTemplateApplyRequest,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(
    await invoke<unknown>("notes_apply_page_template", { dbUrl, templateId, request }),
  );
}

export async function updateNotesPageTemplate(
  templateId: string,
  update: NotesPageTemplateUpdateRequest,
): Promise<NotesPageTemplate> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageTemplateDto(
    await invoke<unknown>("notes_update_page_template", { dbUrl, templateId, update }),
  );
}

export async function duplicateNotesPageTemplate(
  templateId: string,
  request: NotesPageTemplateDuplicateRequest,
): Promise<NotesPageTemplate> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageTemplateDto(
    await invoke<unknown>("notes_duplicate_page_template", { dbUrl, templateId, request }),
  );
}

export async function deleteNotesPageTemplate(templateId: string): Promise<string> {
  const dbUrl = await ensureDbUrl();
  const deletedTemplateId = await invoke<unknown>("notes_delete_page_template", { dbUrl, templateId });
  if (typeof deletedTemplateId !== "string") {
    throw new Error("notes_delete_page_template returned an invalid template id");
  }
  return deletedTemplateId;
}

export async function getNotesPageHistorySettings(): Promise<NotesPageHistorySettings> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageHistorySettingsDto(
    await invoke<unknown>("notes_get_page_history_settings", { dbUrl }),
  );
}

export async function updateNotesPageHistorySettings(
  update: NotesPageHistorySettingsUpdate,
): Promise<NotesPageHistorySettings> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageHistorySettingsDto(
    await invoke<unknown>("notes_update_page_history_settings", { dbUrl, update }),
  );
}

export async function listNotesPageHistorySnapshots(
  pageId: string,
): Promise<NotesPageHistorySnapshot[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_page_history_snapshots", { dbUrl, pageId });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_page_history_snapshots returned a non-array payload");
  }
  return rows.map(mapNotesPageHistorySnapshotDto);
}

export async function loadNotesPageHistorySnapshot(
  pageId: string,
  snapshotId: string,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(
    await invoke<unknown>("notes_load_page_history_snapshot", { dbUrl, pageId, snapshotId }),
  );
}

export async function restoreNotesPageHistorySnapshot(
  pageId: string,
  snapshotId: string,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(
    await invokeNotesMutation(
      "notes_restore_page_history_snapshot",
      { dbUrl, pageId, snapshotId },
      true,
    ),
  );
}

export async function copyNotesPageHistoryBlocks(
  pageId: string,
  snapshotId: string,
  request: NotesPageHistoryCopyBlocksRequest,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockListDto(
    await invokeNotesMutation(
      "notes_copy_page_history_blocks",
      { dbUrl, pageId, snapshotId, request },
      true,
    ),
  );
}

export async function listNotesComments(
  pageId: string,
  includeResolved = false,
): Promise<NotesCommentThread[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_comments", { dbUrl, pageId, includeResolved });
  if (!Array.isArray(rows)) throw new Error("notes_list_comments returned a non-array payload");
  return rows.map(mapNotesCommentThreadDto);
}

export async function markNotesCommentThreadsRead(
  request: NotesCommentThreadReadUpdate,
): Promise<NotesCommentThread[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_mark_comment_threads_read", { dbUrl, request });
  if (!Array.isArray(rows)) {
    throw new Error("notes_mark_comment_threads_read returned a non-array payload");
  }
  return rows.map(mapNotesCommentThreadDto);
}

export async function createNotesComment(
  request: NotesCommentCreate,
): Promise<NotesCommentThread> {
  const dbUrl = await ensureDbUrl();
  const thread = mapNotesCommentThreadDto(
    await invokeNotesMutation("notes_create_comment", { dbUrl, request }),
  );
  invalidateNotesNotificationSchedule();
  return thread;
}

export async function updateNotesComment(
  commentId: string,
  update: NotesCommentUpdate,
): Promise<NotesCommentThread> {
  const dbUrl = await ensureDbUrl();
  const thread = mapNotesCommentThreadDto(
    await invokeNotesMutation("notes_update_comment", { dbUrl, commentId, update }),
  );
  invalidateNotesNotificationSchedule();
  return thread;
}

export async function deleteNotesComment(commentId: string): Promise<NotesCommentThread> {
  const dbUrl = await ensureDbUrl();
  const thread = mapNotesCommentThreadDto(
    await invokeNotesMutation("notes_delete_comment", { dbUrl, commentId }),
  );
  invalidateNotesNotificationSchedule();
  return thread;
}

export async function resolveNotesCommentThread(
  discussionId: string,
  resolved = true,
): Promise<NotesCommentThread> {
  const dbUrl = await ensureDbUrl();
  const thread = mapNotesCommentThreadDto(
    await invokeNotesMutation("notes_resolve_comment_thread", { dbUrl, discussionId, resolved }),
  );
  invalidateNotesNotificationSchedule();
  return thread;
}

export async function listNotesSuggestions(
  pageId: string,
  includeDecided = false,
): Promise<NotesSuggestion[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_suggestions", { dbUrl, pageId, includeDecided });
  if (!Array.isArray(rows)) throw new Error("notes_list_suggestions returned a non-array payload");
  return rows.map(mapNotesSuggestionDto);
}

export async function createNotesSuggestion(
  request: NotesSuggestionCreate,
): Promise<NotesSuggestion> {
  const dbUrl = await ensureDbUrl();
  return mapNotesSuggestionDto(
    await invokeNotesMutation("notes_create_suggestion", { dbUrl, request }),
  );
}

export async function acceptNotesSuggestion(suggestionId: string): Promise<NotesSuggestion> {
  const dbUrl = await ensureDbUrl();
  return mapNotesSuggestionDto(
    await invokeNotesMutation("notes_accept_suggestion", { dbUrl, suggestionId }),
  );
}

export async function rejectNotesSuggestion(suggestionId: string): Promise<NotesSuggestion> {
  const dbUrl = await ensureDbUrl();
  return mapNotesSuggestionDto(
    await invokeNotesMutation("notes_reject_suggestion", { dbUrl, suggestionId }),
  );
}

export async function createNotesPage(page: NotesPageCreate): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(await invokeNotesMutation("notes_create_page", { dbUrl, page }));
}

export async function importNotesMarkdownPage(
  request: NotesMarkdownImportRequest,
): Promise<NotesMarkdownImportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesMarkdownImportDto(
    await invokeNotesMutation("notes_import_markdown_page", { dbUrl, request }, true),
  );
}

export async function importNotesHtmlPage(
  request: NotesHtmlImportRequest,
): Promise<NotesHtmlImportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesHtmlImportDto(
    await invokeNotesMutation("notes_import_html_page", { dbUrl, request }, true),
  );
}

export async function importNotesNotionApi(
  request: NotesNotionApiImportRequest,
): Promise<NotesNotionApiImportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesNotionApiImportDto(
    await invokeNotesMutation("notes_import_notion_api", { dbUrl, request }, true),
  );
}

export async function importNotesNotionExportFolder(
  request: NotesNotionExportImportRequest,
): Promise<NotesNotionExportImportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesNotionExportImportDto(
    await invokeNotesMutation("notes_import_notion_export_folder", { dbUrl, request }, true),
  );
}

export async function exportNotesMarkdownPage(
  request: NotesMarkdownExportRequest,
): Promise<NotesMarkdownExportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesMarkdownExportDto(
    await invoke<unknown>("notes_export_markdown_page", { dbUrl, request }),
  );
}

export async function exportNotesHtmlPage(
  request: NotesHtmlExportRequest,
): Promise<NotesHtmlExportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesHtmlExportDto(await invoke<unknown>("notes_export_html_page", { dbUrl, request }));
}

export async function saveNotesHtmlArchive(
  request: NotesHtmlExportRequest,
): Promise<NotesHtmlArchiveSaveResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesHtmlArchiveSaveDto(
    await invoke<unknown>("notes_pick_and_write_html_archive", { dbUrl, request }),
  );
}

export async function exportNotesJsonGraph(
  request: NotesJsonGraphExportRequest,
): Promise<NotesJsonGraphExportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesJsonGraphExportDto(
    await invoke<unknown>("notes_export_json_graph", { dbUrl, request }),
  );
}

export async function saveNotesJsonGraph(
  request: NotesJsonGraphExportRequest,
): Promise<NotesJsonGraphExportSaveResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesJsonGraphExportSaveDto(
    await invoke<unknown>("notes_pick_and_write_json_graph", { dbUrl, request }),
  );
}

export async function exportNotesAgentBridge(
  request: NotesAgentBridgeExportRequest,
): Promise<NotesAgentBridgeExportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesAgentBridgeExportDto(
    await invoke<unknown>("notes_export_agent_bridge", { dbUrl, request }),
  );
}

export async function saveNotesAgentBridge(
  request: NotesAgentBridgeExportRequest,
): Promise<NotesAgentBridgeExportSaveResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesAgentBridgeExportSaveDto(
    await invoke<unknown>("notes_pick_and_write_agent_bridge", { dbUrl, request }),
  );
}

export async function createNotesChildPageFromBlock(
  blockId: string,
  request: NotesChildPageFromBlockCreate,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(
    await invokeNotesMutation("notes_create_child_page_from_block", { dbUrl, blockId, request }),
  );
}

export async function createNotesDatabase(
  request: NotesDatabaseCreateRequest,
): Promise<NotesCreatedDatabase> {
  const dbUrl = await ensureDbUrl();
  return mapNotesCreatedDatabaseDto(
    await invokeNotesMutation("notes_create_database", { dbUrl, request }),
  );
}

export async function createNotesLinkedDatabaseView(
  request: NotesLinkedDatabaseCreateRequest,
): Promise<NotesCreatedDatabase> {
  const dbUrl = await ensureDbUrl();
  return mapNotesCreatedDatabaseDto(
    await invokeNotesMutation("notes_create_linked_database_view", { dbUrl, request }),
  );
}

export async function getNotesDataSourceSchema(
  dataSourceId: string,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceSchema> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceSchemaDto(
    await invoke<unknown>("notes_get_data_source_schema", {
      dbUrl,
      dataSourceId,
      ...schemaViewScopeArgs(scope),
    }),
  );
}

export async function updateNotesDataSourceSchema(
  dataSourceId: string,
  update: NotesDataSourceSchemaUpdate,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceSchema> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceSchemaDto(
    await invokeNotesMutation("notes_update_data_source_schema", {
      dbUrl,
      dataSourceId,
      update,
      ...schemaViewScopeArgs(scope),
    }),
  );
}

export async function listNotesDataSources(): Promise<NotesDataSource[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_data_sources", { dbUrl });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_data_sources returned a non-array payload");
  }
  return rows.map(mapNotesDataSourceDto);
}

export async function listNotesDataSourceRowPages(
  dataSourceId: string,
): Promise<NotesPage[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_data_source_row_pages", { dbUrl, dataSourceId });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_data_source_row_pages returned a non-array payload");
  }
  return rows.map(mapNotesPageDto);
}

export async function createNotesDataSourceRowPage(
  dataSourceId: string,
  request: NotesDataSourceRowPageCreateRequest,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(
    await invokeNotesMutation("notes_create_data_source_row_page", {
      dbUrl,
      dataSourceId,
      request,
    }),
  );
}

export async function importNotesDataSourceCsv(
  dataSourceId: string,
  request: NotesDataSourceCsvImportRequest,
): Promise<NotesDataSourceCsvImportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceCsvImportDto(
    await invokeNotesMutation(
      "notes_import_data_source_csv",
      { dbUrl, dataSourceId, request },
      request.dry_run === false,
    ),
  );
}

export async function exportNotesDataSourceCsv(
  dataSourceId: string,
  request: NotesDataSourceCsvExportRequest,
): Promise<NotesDataSourceCsvExportResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceCsvExportDto(
    await invoke<unknown>("notes_export_data_source_csv", { dbUrl, dataSourceId, request }),
  );
}

export async function saveNotesDataSourceCsv(
  dataSourceId: string,
  request: NotesDataSourceCsvExportRequest,
): Promise<NotesDataSourceCsvExportSaveResult> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceCsvExportSaveDto(
    await invoke<unknown>("notes_pick_and_write_data_source_csv", { dbUrl, dataSourceId, request }),
  );
}

export async function listNotesDataSourceTemplates(
  dataSourceId: string,
): Promise<NotesDataSourceTemplate[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_data_source_templates", { dbUrl, dataSourceId });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_data_source_templates returned a non-array payload");
  }
  return rows.map(mapNotesDataSourceTemplateDto);
}

export async function createNotesDataSourceTemplateFromRow(
  dataSourceId: string,
  request: NotesDataSourceTemplateCreateFromRowRequest,
): Promise<NotesDataSourceTemplate> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceTemplateDto(
    await invokeNotesMutation("notes_create_data_source_template_from_row", {
      dbUrl,
      dataSourceId,
      request,
    }),
  );
}

export async function applyNotesDataSourceTemplate(
  dataSourceId: string,
  templateId: string,
  request: NotesDataSourceTemplateApplyRequest,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(
    await invokeNotesMutation("notes_apply_data_source_template", {
      dbUrl,
      dataSourceId,
      templateId,
      request,
    }),
  );
}

export async function updateNotesDataSourceTemplate(
  dataSourceId: string,
  templateId: string,
  update: NotesDataSourceTemplateUpdateRequest,
): Promise<NotesDataSourceTemplate> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceTemplateDto(
    await invokeNotesMutation("notes_update_data_source_template", {
      dbUrl,
      dataSourceId,
      templateId,
      update,
    }),
  );
}

export async function duplicateNotesDataSourceTemplate(
  dataSourceId: string,
  templateId: string,
  request: NotesDataSourceTemplateDuplicateRequest,
): Promise<NotesDataSourceTemplate> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceTemplateDto(
    await invokeNotesMutation("notes_duplicate_data_source_template", {
      dbUrl,
      dataSourceId,
      templateId,
      request,
    }),
  );
}

export async function deleteNotesDataSourceTemplate(
  dataSourceId: string,
  templateId: string,
): Promise<string> {
  const dbUrl = await ensureDbUrl();
  const deletedTemplateId = await invokeNotesMutation("notes_delete_data_source_template", {
    dbUrl,
    dataSourceId,
    templateId,
  });
  if (typeof deletedTemplateId !== "string") {
    throw new Error("notes_delete_data_source_template returned an invalid template id");
  }
  return deletedTemplateId;
}

export async function getNotesDataSourceTableView(
  dataSourceId: string,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceTableView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceTableViewDto(
    await invoke<unknown>("notes_get_data_source_table_view", {
      dbUrl,
      dataSourceId,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function updateNotesDataSourceTableView(
  dataSourceId: string,
  update: NotesDataSourceTableViewUpdate,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceTableView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceTableViewDto(
    await invokeNotesMutation("notes_update_data_source_table_view", {
      dbUrl,
      dataSourceId,
      update,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function getNotesDataSourceBoardView(
  dataSourceId: string,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceBoardView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceBoardViewDto(
    await invoke<unknown>("notes_get_data_source_board_view", {
      dbUrl,
      dataSourceId,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function updateNotesDataSourceBoardView(
  dataSourceId: string,
  update: NotesDataSourceBoardViewUpdate,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceBoardView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceBoardViewDto(
    await invokeNotesMutation("notes_update_data_source_board_view", {
      dbUrl,
      dataSourceId,
      update,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function moveNotesDataSourceBoardRow(
  dataSourceId: string,
  request: NotesDataSourceBoardRowMove,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceBoardView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceBoardViewDto(
    await invokeNotesMutation("notes_move_data_source_board_row", {
      dbUrl,
      dataSourceId,
      request,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function getNotesDataSourceGalleryView(
  dataSourceId: string,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceGalleryView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceGalleryViewDto(
    await invoke<unknown>("notes_get_data_source_gallery_view", {
      dbUrl,
      dataSourceId,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function updateNotesDataSourceGalleryView(
  dataSourceId: string,
  update: NotesDataSourceGalleryViewUpdate,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceGalleryView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceGalleryViewDto(
    await invokeNotesMutation("notes_update_data_source_gallery_view", {
      dbUrl,
      dataSourceId,
      update,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function updateNotesDataSourceRowProperty(
  dataSourceId: string,
  pageId: string,
  update: NotesDataSourceRowPropertyUpdate,
): Promise<NotesPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageDto(
    await invokeNotesMutation("notes_update_data_source_row_property", {
      dbUrl,
      dataSourceId,
      pageId,
      update,
    }),
  );
}

export async function clickNotesDataSourceButton(
  dataSourceId: string,
  pageId: string,
  request: NotesDataSourceButtonClickRequest,
): Promise<NotesPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageDto(
    await invokeNotesMutation("notes_click_data_source_button", {
      dbUrl,
      dataSourceId,
      pageId,
      request,
    }),
  );
}

export async function getNotesDataSourceListView(
  dataSourceId: string,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceListView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceListViewDto(
    await invoke<unknown>("notes_get_data_source_list_view", {
      dbUrl,
      dataSourceId,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function updateNotesDataSourceListView(
  dataSourceId: string,
  update: NotesDataSourceListViewUpdate,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceListView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceListViewDto(
    await invokeNotesMutation("notes_update_data_source_list_view", {
      dbUrl,
      dataSourceId,
      update,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function getNotesDataSourceCalendarView(
  dataSourceId: string,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceCalendarView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceCalendarViewDto(
    await invoke<unknown>("notes_get_data_source_calendar_view", {
      dbUrl,
      dataSourceId,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function updateNotesDataSourceCalendarView(
  dataSourceId: string,
  update: NotesDataSourceCalendarViewUpdate,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceCalendarView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceCalendarViewDto(
    await invokeNotesMutation("notes_update_data_source_calendar_view", {
      dbUrl,
      dataSourceId,
      update,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function getNotesDataSourceTimelineView(
  dataSourceId: string,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceTimelineView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceTimelineViewDto(
    await invoke<unknown>("notes_get_data_source_timeline_view", {
      dbUrl,
      dataSourceId,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function updateNotesDataSourceTimelineView(
  dataSourceId: string,
  update: NotesDataSourceTimelineViewUpdate,
  scope?: NotesDatabaseViewScope | null,
): Promise<NotesDataSourceTimelineView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceTimelineViewDto(
    await invokeNotesMutation("notes_update_data_source_timeline_view", {
      dbUrl,
      dataSourceId,
      update,
      ...databaseViewScopeArgs(scope),
    }),
  );
}

export async function duplicateNotesPage(
  pageId: string,
  request: NotesDuplicatePageRequest,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(await invokeNotesMutation("notes_duplicate_page", { dbUrl, pageId, request }));
}

export async function moveNotesPage(
  pageId: string,
  request: NotesMovePageRequest,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(await invokeNotesMutation("notes_move_page", { dbUrl, pageId, request }, true));
}

export async function updateNotesPage(
  pageId: string,
  update: NotesPageUpdate,
): Promise<NotesPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageDto(await invokeNotesMutation("notes_update_page", { dbUrl, pageId, update }));
}

export async function trashNotesPage(
  pageId: string,
  inTrash = true,
): Promise<NotesPage> {
  const dbUrl = await ensureDbUrl();
  const page = mapNotesPageDto(
    await invokeNotesMutation("notes_trash_page", { dbUrl, pageId, inTrash }, true),
  );
  if (inTrash) invalidateNotesAssetUrls();
  return page;
}

export async function archiveNotesPage(
  pageId: string,
  archived = true,
): Promise<NotesPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageDto(await invokeNotesMutation("notes_archive_page", { dbUrl, pageId, archived }, true));
}

export async function permanentlyDeleteNotesPage(pageId: string): Promise<string[]> {
  const dbUrl = await ensureDbUrl();
  const deletedPageIds = await invokeNotesMutation(
    "notes_permanently_delete_page",
    { dbUrl, pageId },
    true,
  );
  if (!Array.isArray(deletedPageIds)) {
    throw new Error("notes_permanently_delete_page returned a non-array payload");
  }
  if (!deletedPageIds.every((deletedPageId): deletedPageId is string => typeof deletedPageId === "string")) {
    throw new Error("notes_permanently_delete_page returned invalid page ids");
  }
  invalidateNotesAssetUrls();
  return deletedPageIds;
}

export async function loadNotesPage(pageId: string): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(await invoke<unknown>("notes_load_page", { dbUrl, pageId }));
}

export async function getNotesBlockChildren(
  parentId: string,
  startCursor: string | null = null,
  pageSize = 50,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockListDto(
    await invoke<unknown>("notes_get_block_children", {
      dbUrl,
      parentId,
      startCursor,
      pageSize,
    }),
  );
}

export async function appendNotesBlockChildren(
  request: NotesAppendBlockChildrenRequest,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  const blocks = mapNotesBlockListDto(
    await invokeNotesMutation("notes_append_block_children", { dbUrl, request }),
  );
  invalidateNotesNotificationSchedule();
  return blocks;
}

export async function updateNotesBlock(
  blockId: string,
  update: NotesBlockUpdate,
): Promise<NotesBlock> {
  const dbUrl = await ensureDbUrl();
  const block = mapNotesBlockDto(
    await invokeNotesMutation("notes_update_block", { dbUrl, blockId, update }),
  );
  invalidateNotesNotificationSchedule();
  return block;
}

export async function trashNotesBlock(
  blockId: string,
  inTrash = true,
): Promise<NotesBlock> {
  const dbUrl = await ensureDbUrl();
  const block = mapNotesBlockDto(
    await invokeNotesMutation("notes_trash_block", { dbUrl, blockId, inTrash }),
  );
  if (inTrash) invalidateAssetUrlKind("notes-file");
  invalidateNotesNotificationSchedule();
  return block;
}

export async function trashNotesBlocks(
  request: NotesTrashBlocksRequest,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  const blocks = mapNotesBlockListDto(
    await invokeNotesMutation("notes_trash_blocks", { dbUrl, request }),
  );
  if (request.in_trash) invalidateAssetUrlKind("notes-file");
  invalidateNotesNotificationSchedule();
  return blocks;
}

export async function moveNotesBlock(
  blockId: string,
  request: NotesMoveBlockRequest,
): Promise<NotesBlock> {
  const dbUrl = await ensureDbUrl();
  const block = mapNotesBlockDto(
    await invokeNotesMutation("notes_move_block", { dbUrl, blockId, request }),
  );
  invalidateNotesNotificationSchedule();
  return block;
}

export async function moveNotesBlocks(
  request: NotesMoveBlocksRequest,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  const blocks = mapNotesBlockListDto(
    await invokeNotesMutation("notes_move_blocks", { dbUrl, request }),
  );
  invalidateNotesNotificationSchedule();
  return blocks;
}

export async function duplicateNotesBlock(
  blockId: string,
  request: NotesDuplicateBlockRequest,
): Promise<NotesBlock> {
  const dbUrl = await ensureDbUrl();
  const block = mapNotesBlockDto(
    await invokeNotesMutation("notes_duplicate_block", { dbUrl, blockId, request }),
  );
  invalidateNotesNotificationSchedule();
  return block;
}

export async function duplicateNotesBlocks(
  request: NotesDuplicateBlocksRequest,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  const blocks = mapNotesBlockListDto(
    await invokeNotesMutation("notes_duplicate_blocks", { dbUrl, request }),
  );
  invalidateNotesNotificationSchedule();
  return blocks;
}

export async function loadNotesUndoState(pageId: string): Promise<string | null> {
  const dbUrl = await ensureDbUrl();
  const state = await invoke<unknown>("notes_load_undo_state", { dbUrl, pageId });
  if (state === null) return null;
  if (typeof state !== "string") throw new Error("notes_load_undo_state returned invalid state");
  return state;
}

export async function saveNotesUndoState(
  pageId: string,
  stateJson: string,
): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("notes_save_undo_state", { dbUrl, pageId, stateJson });
}

export async function clearNotesUndoState(pageId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("notes_clear_undo_state", { dbUrl, pageId });
}
