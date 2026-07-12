import type { NotesPageCover } from "./assets";
import type { NotesCommentDisplayName, NotesPartialUser } from "./collaboration";
import type { NotesPageIcon } from "./core";

export interface NotesPageHistorySnapshot {
  object: "page_history_snapshot";
  id: string;
  page_id: string;
  title: string;
  icon: NotesPageIcon | null;
  cover: NotesPageCover | null;
  block_count: number;
  reason: string;
  created_by: NotesPartialUser;
  created_time: string;
  page_last_edited_time: string;
}

export interface NotesPageHistorySettings {
  object: "page_history_settings";
  retention_days: 0 | 7 | 30 | 90 | 180 | 365;
  updated_at: string;
}

export interface NotesProjectHistoryVersion {
  id: string;
  projectId: string;
  manifestHash: string;
  reason: string;
  createdBy: string;
  displayName: NotesCommentDisplayName;
  changedNoteSummary: string;
  pageCount: number;
  activePageCount: number;
  archivedPageCount: number;
  deletedPageCount: number;
  createdTime: string;
}

export interface NotesProjectHistoryVersionList {
  versions: NotesProjectHistoryVersion[];
  nextCursorTime: string | null;
  nextCursorId: string | null;
}

export interface NotesHistoryRetentionImpact {
  versionCount: number;
  storedBytes: number;
}

export interface NotesHistoricalPageSummary {
  id: string;
  title: string;
  parentPageId: string | null;
  parentDataSourceId: string | null;
  inTrash: boolean;
  archived: boolean;
  icon: NotesPageIcon | null;
}

export interface NotesProjectHistoryTree {
  version: NotesProjectHistoryVersion;
  pages: NotesHistoricalPageSummary[];
}

export interface NotesHistoricalPage {
  id: string;
  title: string;
  properties: Record<string, unknown>;
  icon: NotesPageIcon | null;
  cover: NotesPageCover | null;
  inTrash: boolean;
  archived: boolean;
  blocks: Record<string, unknown>[];
  databases: Record<string, unknown>[];
  dataSources: Record<string, unknown>[];
  databaseViews: Record<string, unknown>[];
}

export interface NotesProjectHistoryRestorePlan {
  versionId: string;
  removeCount: number;
  recreateCount: number;
  changeCount: number;
  copyCount: number;
  safetyVersionWillBeCreated: boolean;
}

export interface NotesPageHistorySettingsUpdate {
  retention_days: 0 | 7 | 30 | 90 | 180 | 365;
}

export interface NotesPageHistoryCopyBlocksRequest {
  after_block_id?: string | null;
}
