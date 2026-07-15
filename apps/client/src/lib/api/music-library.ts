import { invoke } from "@tauri-apps/api/core";
import { dbUrl } from "$lib/api/db";
import {
  parseBindingResult,
  parseBindings,
  parseCollections,
  parseDeleteImpact,
  parseInspectorDetail,
  parseIssues,
  parseItemWindow,
  parsePlaylist,
  parsePlaylistSummaries,
  parseRefreshJobProgress,
  parseRelinkPlanSummary,
  parseRelinkPlanWindow,
  parseRoots,
  parseSearchRebuild,
  parseSourceSummaries,
  parseSourceRemovalImpact,
  parseWriteReceipt,
  parseYouTubeSnapshotResult,
  type LocalRootBinding,
  type MusicBulkMembershipWrite,
  type MusicCollectionWrite,
  type MusicInspectorDetail,
  type MusicIssue,
  type MusicItemWindow,
  type MusicItemWindowRequest,
  type MusicLibraryItemWrite,
  type MusicLocalRefreshRequest,
  type MusicLocalLocationWrite,
  type MusicLocalRoot,
  type MusicMembershipRemove,
  type MusicPlaylist,
  type MusicPlaylistCreate,
  type MusicPlaylistDelete,
  type MusicPlaylistDeleteImpact,
  type MusicPlaylistDuplicate,
  type MusicPlaylistSummary,
  type MusicPlaylistUpdate,
  type MusicReviewWrite,
  type MusicRelinkApplyRequest,
  type MusicRelinkPlanRequest,
  type MusicRelinkPlanSummary,
  type MusicRelinkPlanWindow,
  type MusicRefreshJobProgress,
  type MusicSearchRebuildResult,
  type MusicSnoozeWrite,
  type MusicSourceCollection,
  type MusicSourceSummary,
  type MusicSourceRemovalImpact,
  type MusicSourceRemovalRequest,
  type MusicStatisticsReset,
  type MusicWriteReceipt,
  type MusicYouTubePlaylistSnapshotWrite,
  type MusicYouTubeSnapshotResult,
  type MusicYouTubeSourceFailureWrite,
  type MusicYouTubeVideoWrite,
} from "$lib/music/library-contracts";

export type MusicLibraryApiErrorCode = "validation" | "not-found" | "conflict" | "stale-write" | "database" | "unknown";

export class MusicLibraryApiError extends Error {
  readonly code: MusicLibraryApiErrorCode;
  readonly field: string | null;

  constructor(code: MusicLibraryApiErrorCode, message: string, field: string | null = null) {
    super(message);
    this.name = "MusicLibraryApiError";
    this.code = code;
    this.field = field;
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

const errorCodes = new Set<MusicLibraryApiErrorCode>([
  "validation",
  "not-found",
  "conflict",
  "stale-write",
  "database",
  "unknown",
]);

export function normalizeMusicLibraryError(error: unknown): MusicLibraryApiError {
  if (error instanceof MusicLibraryApiError) return error;
  if (isRecord(error)) {
    const code = typeof error.code === "string" && errorCodes.has(error.code as MusicLibraryApiErrorCode)
      ? error.code as MusicLibraryApiErrorCode
      : "unknown";
    const message = typeof error.message === "string" && error.message.trim()
      ? error.message
      : "Music library operation failed.";
    const field = typeof error.field === "string" && error.field.trim() ? error.field : null;
    return new MusicLibraryApiError(code, message, field);
  }
  const message = error instanceof Error ? error.message : String(error);
  return new MusicLibraryApiError("unknown", message.trim() || "Music library operation failed.");
}

async function call<T>(command: string, args: Record<string, unknown>, parse: (value: unknown) => T): Promise<T> {
  try {
    return parse(await invoke<unknown>(command, args));
  } catch (error) {
    if (error instanceof Error && !(error instanceof MusicLibraryApiError)) {
      throw new MusicLibraryApiError("database", `Invalid response from ${command}: ${error.message}`);
    }
    throw normalizeMusicLibraryError(error);
  }
}

function databaseArgs(extra: Record<string, unknown> = {}): Record<string, unknown> {
  return { dbUrl: dbUrl(), ...extra };
}

const parseVoid = (_value: unknown): void => undefined;

export const upsertMusicLibraryItem = (request: MusicLibraryItemWrite): Promise<MusicWriteReceipt> =>
  call("music_library_upsert_item", databaseArgs({ request }), parseWriteReceipt);
export const upsertMusicLocalLocation = (request: MusicLocalLocationWrite): Promise<void> =>
  call("music_library_upsert_local_location", databaseArgs({ request }), parseVoid);
export const startMusicLocalRefresh = (request: MusicLocalRefreshRequest): Promise<MusicRefreshJobProgress> =>
  call("music_library_start_local_refresh", databaseArgs({ request }), parseRefreshJobProgress);
export const getMusicRefreshProgress = (jobId: string): Promise<MusicRefreshJobProgress> =>
  call("music_library_refresh_progress", databaseArgs({ jobId }), parseRefreshJobProgress);
export const cancelMusicRefresh = (jobId: string, cancelledAt: number): Promise<MusicRefreshJobProgress> =>
  call("music_library_cancel_refresh", databaseArgs({ jobId, cancelledAt }), parseRefreshJobProgress);
export const upsertMusicYouTubeVideo = (request: MusicYouTubeVideoWrite): Promise<MusicWriteReceipt> =>
  call("music_library_upsert_youtube_video", databaseArgs({ request }), parseWriteReceipt);
export const applyMusicYouTubePlaylistSnapshot = (
  request: MusicYouTubePlaylistSnapshotWrite,
): Promise<MusicYouTubeSnapshotResult> =>
  call("music_library_apply_youtube_playlist_snapshot", databaseArgs({ request }), parseYouTubeSnapshotResult);
export const reportMusicYouTubeSourceFailure = (request: MusicYouTubeSourceFailureWrite): Promise<void> =>
  call("music_library_report_youtube_source_failure", databaseArgs({ request }), parseVoid);
export const createMusicRelinkPlan = (request: MusicRelinkPlanRequest): Promise<MusicRelinkPlanSummary> =>
  call("music_library_create_relink_plan", databaseArgs({ request }), parseRelinkPlanSummary);
export const getMusicRelinkPlanEntries = (
  planId: string,
  offset: number,
  limit: number,
): Promise<MusicRelinkPlanWindow> =>
  call("music_library_relink_plan_entries", databaseArgs({ planId, offset, limit }), parseRelinkPlanWindow);
export const applyMusicRelinkPlan = (request: MusicRelinkApplyRequest): Promise<MusicRelinkPlanSummary> =>
  call("music_library_apply_relink_plan", databaseArgs({ request }), parseRelinkPlanSummary);
export const cancelMusicRelinkPlan = (planId: string, cancelledAt: number): Promise<MusicRelinkPlanSummary> =>
  call("music_library_cancel_relink_plan", databaseArgs({ planId, cancelledAt }), parseRelinkPlanSummary);
export const getMusicSourceRemovalImpact = (collectionId: string): Promise<MusicSourceRemovalImpact> =>
  call("music_library_source_removal_impact", databaseArgs({ collectionId }), parseSourceRemovalImpact);
export const removeMusicSource = (request: MusicSourceRemovalRequest): Promise<MusicSourceRemovalImpact> =>
  call("music_library_remove_source", databaseArgs({ request }), parseSourceRemovalImpact);
export const restoreMusicSource = (
  collectionId: string,
  expectedVersion: number,
  restoredAt: number,
): Promise<MusicWriteReceipt> =>
  call("music_library_restore_source", databaseArgs({ collectionId, expectedVersion, restoredAt }), parseWriteReceipt);
export const createMusicPlaylist = (request: MusicPlaylistCreate): Promise<MusicWriteReceipt> =>
  call("music_library_create_playlist", databaseArgs({ request }), parseWriteReceipt);
export const updateMusicPlaylist = (request: MusicPlaylistUpdate): Promise<MusicWriteReceipt> =>
  call("music_library_update_playlist", databaseArgs({ request }), parseWriteReceipt);
export const duplicateMusicPlaylist = (request: MusicPlaylistDuplicate): Promise<MusicWriteReceipt> =>
  call("music_library_duplicate_playlist", databaseArgs({ request }), parseWriteReceipt);
export const getMusicPlaylistDeleteImpact = (playlistId: string): Promise<MusicPlaylistDeleteImpact> =>
  call("music_library_playlist_delete_impact", databaseArgs({ playlistId }), parseDeleteImpact);
export const deleteMusicPlaylist = (request: MusicPlaylistDelete): Promise<MusicPlaylistDeleteImpact> =>
  call("music_library_delete_playlist", databaseArgs({ request }), parseDeleteImpact);
export const setMusicReviewState = (request: MusicReviewWrite): Promise<MusicWriteReceipt> =>
  call("music_library_set_review_state", databaseArgs({ request }), parseWriteReceipt);
export const upsertMusicMemberships = (request: MusicBulkMembershipWrite): Promise<MusicWriteReceipt[]> =>
  call("music_library_upsert_memberships", databaseArgs({ request }), (value) => {
    if (!Array.isArray(value)) throw new Error("membership receipts must be an array");
    return value.map((entry, index) => parseWriteReceipt(entry, `membership receipts[${index}]`));
  });
export const removeMusicMemberships = (request: MusicMembershipRemove): Promise<void> =>
  call("music_library_remove_memberships", databaseArgs({ request }), parseVoid);
export const upsertMusicSnooze = (request: MusicSnoozeWrite): Promise<void> =>
  call("music_library_upsert_snooze", databaseArgs({ request }), parseVoid);
export const removeMusicSnooze = (snoozeId: string): Promise<void> =>
  call("music_library_remove_snooze", databaseArgs({ request: { snoozeId } }), parseVoid);
export const resetMusicStatistics = (request: MusicStatisticsReset): Promise<void> =>
  call("music_library_reset_statistics", databaseArgs({ request }), parseVoid);
export const getMusicItemWindow = (request: MusicItemWindowRequest): Promise<MusicItemWindow> =>
  call("music_library_item_window", databaseArgs({ request }), parseItemWindow);
export const getMusicPlaylistSummaries = (nowMs: number, offset: number, limit: number): Promise<MusicPlaylistSummary[]> =>
  call("music_library_playlist_summaries", databaseArgs({ nowMs, offset, limit }), parsePlaylistSummaries);
export const getMusicSourceSummaries = (nowMs: number, offset: number, limit: number): Promise<MusicSourceSummary[]> =>
  call("music_library_source_summaries", databaseArgs({ nowMs, offset, limit }), parseSourceSummaries);
export const getMusicIssues = (offset: number, limit: number): Promise<MusicIssue[]> =>
  call("music_library_issues", databaseArgs({ offset, limit }), parseIssues);
export const getMusicInspectorDetail = (itemId: string): Promise<MusicInspectorDetail> =>
  call("music_library_inspector_detail", databaseArgs({ itemId }), parseInspectorDetail);
export const rebuildMusicSearchIndex = (rebuiltAt: number): Promise<MusicSearchRebuildResult> =>
  call("music_library_rebuild_search_index", databaseArgs({ rebuiltAt }), parseSearchRebuild);
export const getMusicLocalRoots = (offset: number, limit: number): Promise<MusicLocalRoot[]> =>
  call("music_library_local_roots", databaseArgs({ offset, limit }), parseRoots);
export const getMusicSourceCollections = (offset: number, limit: number): Promise<MusicSourceCollection[]> =>
  call("music_library_source_collections", databaseArgs({ offset, limit }), parseCollections);
export const getMusicPlaylist = (playlistId: string): Promise<MusicPlaylist> =>
  call("music_library_playlist_detail", databaseArgs({ playlistId }), parsePlaylist);
export const upsertMusicSourceCollection = (request: MusicCollectionWrite): Promise<MusicWriteReceipt> =>
  call("music_library_upsert_source_collection", databaseArgs({ request }), parseWriteReceipt);

export const getLocalRootBindings = (vaultId: string, rootIds: string[]): Promise<LocalRootBinding[]> =>
  call("music_get_local_root_bindings", { vaultId, rootIds }, parseBindings);
export const setLocalRootBinding = (vaultId: string, rootId: string, folderPath: string): Promise<LocalRootBinding> =>
  call("music_set_local_root_binding", { vaultId, rootId, folderPath }, parseBindingResult);
export const clearLocalRootBinding = (vaultId: string, rootId: string): Promise<LocalRootBinding> =>
  call("music_clear_local_root_binding", { vaultId, rootId }, parseBindingResult);
