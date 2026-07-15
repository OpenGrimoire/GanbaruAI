export type MusicLibrarySourceKind = "local-file" | "youtube-video";
export type MusicMediaKind = "audio" | "video" | "unknown";
export type MusicReviewState = "unreviewed" | "reviewed" | "deferred" | "ignored";
export type MusicItemAvailability = "available" | "missing" | "unavailable" | "ambiguous" | "unknown";
export type MusicLocationAvailability = "available" | "missing" | "ambiguous" | "unsupported" | "unknown";
export type MusicCollectionKind = "local-root" | "youtube-playlist";
export type MusicRefreshState = "idle" | "queued" | "running" | "partial" | "failed";
export type MusicWeight = "rarely" | "less-often" | "normal" | "more-often" | "much-more-often";
export type MusicFocusFit = "helpful" | "neutral" | "potentially-distracting" | "unknown";
export type MusicIntendedUse = "general" | "focus" | "reading" | "relaxation" | "energizing";
export type MusicItemSignal = "lyrics" | "sudden-changes" | "high-intensity" | "calm" | "repetitive" | "energizing";
export type MusicSnoozeScope = "playlist" | "all-playlists";
export type MusicRepeatMode = "off" | "all" | "one";
export type MusicListDestination = "review" | "library" | "playlist";
export type MusicItemSort = "title" | "artist" | "album" | "discovered-at" | "last-played-at" | "play-count" | "manual-position";
export type MusicSortDirection = "ascending" | "descending";
export type MusicGroupBy = "none" | "source-kind" | "review-state" | "availability" | "album";
export type LocalRootBindingStatus = "available" | "missing" | "needs-relink";

export interface MusicWriteReceipt { id: string; version: number }
export interface MusicPlaylistDeleteImpact {
  membershipCount: number;
  projectFocusAssignmentCount: number;
  projectBreakAssignmentCount: number;
  calendarAssignmentCount: number;
}
export interface MusicPlaylistCreate {
  id: string;
  name: string;
  description: string;
  shuffleEnabled: boolean;
  repeatMode: MusicRepeatMode;
  intendedUses: MusicIntendedUse[];
  createdAt: number;
}
export interface MusicPlaylistUpdate extends Omit<MusicPlaylistCreate, "createdAt"> {
  expectedVersion: number;
  updatedAt: number;
}
export interface MusicPlaylistDuplicate {
  sourcePlaylistId: string;
  newPlaylistId: string;
  name: string;
  createdAt: number;
}
export interface MusicPlaylistDelete {
  playlistId: string;
  expectedVersion: number;
  expectedImpact: MusicPlaylistDeleteImpact;
}
export interface MusicReviewWrite {
  itemId: string;
  reviewState: MusicReviewState;
  expectedVersion: number;
  updatedAt: number;
}
export interface MusicMembershipWrite {
  id: string;
  playlistId: string;
  itemId: string;
  position: number;
  weight: MusicWeight;
  enabled: boolean;
  focusFit: MusicFocusFit;
  startMs: number | null;
  endMs: number | null;
  volume: number | null;
  rate: number | null;
  expectedVersion: number | null;
  updatedAt: number;
}
export interface MusicBulkMembershipWrite { memberships: MusicMembershipWrite[] }
export interface MusicMembershipRemove { membershipIds: string[] }
export interface MusicSnoozeWrite {
  id: string;
  itemId: string;
  scope: MusicSnoozeScope;
  playlistId: string | null;
  startsAt: number;
  endsAt: number | null;
  reason: string;
  createdAt: number;
}
export interface MusicStatisticsReset { itemIds: string[]; resetRecentSelections: boolean }
export interface MusicCollectionWrite {
  id: string;
  kind: MusicCollectionKind;
  identityKey: string;
  name: string;
  localRootId: string | null;
  youtubePlaylistId: string | null;
  updatedAt: number;
}
export interface MusicLibraryItemWrite {
  id: string;
  identityKey: string;
  sourceKind: MusicLibrarySourceKind;
  mediaKind: MusicMediaKind;
  youtubeVideoId: string | null;
  originalTitle: string;
  originalArtist: string;
  originalAlbum: string;
  durationMs: number | null;
  availability: MusicItemAvailability;
  discoveredAt: number;
  updatedAt: number;
}
export interface MusicLocalLocationWrite {
  id: string;
  itemId: string;
  rootId: string;
  relativePath: string;
  fileSizeBytes: number | null;
  modifiedAtMs: number | null;
  lightweightFingerprint: string | null;
  strongFingerprint: string | null;
  availability: MusicLocationAvailability;
  lastSeenGeneration: number | null;
  firstSeenAt: number;
  updatedAt: number;
}
export interface MusicItemWindowRequest {
  destination: MusicListDestination;
  playlistId: string | null;
  search: string;
  sourceKind: MusicLibrarySourceKind | null;
  availability: MusicItemAvailability | null;
  reviewState: MusicReviewState | null;
  sourceCollectionId: string | null;
  snoozed: boolean | null;
  sort: MusicItemSort;
  direction: MusicSortDirection;
  groupBy: MusicGroupBy;
  nowMs: number;
  offset: number;
  limit: number;
}

export interface MusicItemListEntry {
  id: string;
  identityKey: string;
  sourceKind: MusicLibrarySourceKind;
  mediaKind: MusicMediaKind;
  title: string;
  artist: string;
  album: string;
  durationMs: number | null;
  availability: MusicItemAvailability;
  reviewState: MusicReviewState;
  discoveredAt: number;
  updatedAt: number;
  version: number;
  playlistCount: number;
  activeSnoozeCount: number;
  lastPlayedAt: number | null;
  playCount: number;
  membershipId: string | null;
  membershipPosition: number | null;
  membershipWeight: MusicWeight | null;
  membershipEnabled: boolean | null;
  membershipVersion: number | null;
}
export interface MusicGroupCount { key: string; count: number }
export interface MusicItemWindow {
  items: MusicItemListEntry[];
  groups: MusicGroupCount[];
  totalCount: number;
  offset: number;
  limit: number;
}
export interface MusicPlaylistSummary {
  id: string;
  name: string;
  description: string;
  shuffleEnabled: boolean;
  repeatMode: MusicRepeatMode;
  totalCount: number;
  eligibleCount: number;
  unavailableCount: number;
  snoozedCount: number;
  localCount: number;
  onlineCount: number;
  version: number;
}
export interface MusicSourceSummary {
  id: string;
  kind: MusicCollectionKind;
  name: string;
  refreshState: MusicRefreshState;
  lastSuccessfulRefreshAt: number | null;
  itemCount: number;
  missingCount: number;
  newCount: number;
  openIssueCount: number;
  version: number;
}
export interface MusicIssue {
  id: string;
  issueKind: string;
  itemId: string | null;
  playlistId: string | null;
  message: string;
  createdAt: number;
}
export interface MusicLibraryItem {
  id: string;
  identityKey: string;
  sourceKind: MusicLibrarySourceKind;
  mediaKind: MusicMediaKind;
  youtubeVideoId: string | null;
  originalTitle: string;
  originalArtist: string;
  originalAlbum: string;
  titleOverride: string | null;
  artistOverride: string | null;
  albumOverride: string | null;
  artworkOverride: string | null;
  durationMs: number | null;
  availability: MusicItemAvailability;
  reviewState: MusicReviewState;
  reviewChangedAt: number | null;
  discoveredAt: number;
  updatedAt: number;
  version: number;
}
export interface MusicLocalLocation {
  id: string;
  itemId: string;
  rootId: string;
  relativePath: string;
  fileSizeBytes: number | null;
  modifiedAtMs: number | null;
  lightweightFingerprint: string | null;
  strongFingerprint: string | null;
  availability: MusicLocationAvailability;
  lastSeenGeneration: number | null;
  firstSeenAt: number;
  updatedAt: number;
}
export interface MusicPlaylistMembership extends Omit<MusicMembershipWrite, "expectedVersion"> {
  createdAt: number;
  version: number;
}
export interface MusicSnooze extends MusicSnoozeWrite {}
export interface MusicListeningStatistics {
  itemId: string;
  lastPlayedAt: number | null;
  playCount: number;
  completionCount: number;
  skipCount: number;
  updatedAt: number;
}
export interface MusicInspectorDetail {
  item: MusicLibraryItem;
  locations: MusicLocalLocation[];
  memberships: MusicPlaylistMembership[];
  snoozes: MusicSnooze[];
  signals: MusicItemSignal[];
  statistics: MusicListeningStatistics | null;
  sourceCollectionIds: string[];
}
export interface MusicLocalRoot { id: string; name: string; createdAt: number; updatedAt: number; version: number }
export interface MusicSourceCollection {
  id: string;
  kind: MusicCollectionKind;
  identityKey: string;
  name: string;
  localRootId: string | null;
  youtubePlaylistId: string | null;
  refreshState: MusicRefreshState;
  lastSuccessfulRefreshAt: number | null;
  lastRefreshErrorCode: string | null;
  snapshotGeneration: number;
  createdAt: number;
  updatedAt: number;
  version: number;
}
export interface MusicPlaylist {
  id: string;
  name: string;
  description: string;
  shuffleEnabled: boolean;
  repeatMode: MusicRepeatMode;
  intendedUses: MusicIntendedUse[];
  createdAt: number;
  updatedAt: number;
  version: number;
}
export interface MusicSearchRebuildResult { indexedItemCount: number; schemaVersion: number; fingerprint: string; rebuiltAt: number }
export interface LocalRootBinding { rootId: string; folderPath: string | null; status: LocalRootBindingStatus }

const sourceKinds = ["local-file", "youtube-video"] as const;
const mediaKinds = ["audio", "video", "unknown"] as const;
const reviewStates = ["unreviewed", "reviewed", "deferred", "ignored"] as const;
const itemAvailability = ["available", "missing", "unavailable", "ambiguous", "unknown"] as const;
const locationAvailability = ["available", "missing", "ambiguous", "unsupported", "unknown"] as const;
const collectionKinds = ["local-root", "youtube-playlist"] as const;
const refreshStates = ["idle", "queued", "running", "partial", "failed"] as const;
const weights = ["rarely", "less-often", "normal", "more-often", "much-more-often"] as const;
const focusFits = ["helpful", "neutral", "potentially-distracting", "unknown"] as const;
const intendedUses = ["general", "focus", "reading", "relaxation", "energizing"] as const;
const signals = ["lyrics", "sudden-changes", "high-intensity", "calm", "repetitive", "energizing"] as const;
const snoozeScopes = ["playlist", "all-playlists"] as const;
const repeatModes = ["off", "all", "one"] as const;
const rootStatuses = ["available", "missing", "needs-relink"] as const;

function object(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error(`${label} must be an object`);
  return value as Record<string, unknown>;
}
function string(value: unknown, label: string): string {
  if (typeof value !== "string") throw new Error(`${label} must be a string`);
  return value;
}
function number(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value)) throw new Error(`${label} must be a safe integer`);
  return value;
}
function finite(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) throw new Error(`${label} must be finite`);
  return value;
}
function boolean(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") throw new Error(`${label} must be a boolean`);
  return value;
}
function nullable<T>(value: unknown, parse: (value: unknown, label: string) => T, label: string): T | null {
  return value === null ? null : parse(value, label);
}
function array<T>(value: unknown, parse: (value: unknown, label: string) => T, label: string): T[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  return value.map((entry, index) => parse(entry, `${label}[${index}]`));
}
function enumeration<const T extends readonly string[]>(value: unknown, values: T, label: string): T[number] {
  if (typeof value !== "string" || !values.includes(value)) throw new Error(`${label} is not supported`);
  return value as T[number];
}

export function parseWriteReceipt(value: unknown, label = "music write receipt"): MusicWriteReceipt {
  const row = object(value, label); return { id: string(row.id, `${label}.id`), version: number(row.version, `${label}.version`) };
}
export function parseDeleteImpact(value: unknown, label = "playlist delete impact"): MusicPlaylistDeleteImpact {
  const row = object(value, label); return {
    membershipCount: number(row.membershipCount, `${label}.membershipCount`),
    projectFocusAssignmentCount: number(row.projectFocusAssignmentCount, `${label}.projectFocusAssignmentCount`),
    projectBreakAssignmentCount: number(row.projectBreakAssignmentCount, `${label}.projectBreakAssignmentCount`),
    calendarAssignmentCount: number(row.calendarAssignmentCount, `${label}.calendarAssignmentCount`),
  };
}
function parseItemEntry(value: unknown, label: string): MusicItemListEntry {
  const row = object(value, label); return {
    id: string(row.id, `${label}.id`), identityKey: string(row.identityKey, `${label}.identityKey`),
    sourceKind: enumeration(row.sourceKind, sourceKinds, `${label}.sourceKind`), mediaKind: enumeration(row.mediaKind, mediaKinds, `${label}.mediaKind`),
    title: string(row.title, `${label}.title`), artist: string(row.artist, `${label}.artist`), album: string(row.album, `${label}.album`),
    durationMs: nullable(row.durationMs, number, `${label}.durationMs`), availability: enumeration(row.availability, itemAvailability, `${label}.availability`),
    reviewState: enumeration(row.reviewState, reviewStates, `${label}.reviewState`), discoveredAt: number(row.discoveredAt, `${label}.discoveredAt`),
    updatedAt: number(row.updatedAt, `${label}.updatedAt`), version: number(row.version, `${label}.version`), playlistCount: number(row.playlistCount, `${label}.playlistCount`),
    activeSnoozeCount: number(row.activeSnoozeCount, `${label}.activeSnoozeCount`), lastPlayedAt: nullable(row.lastPlayedAt, number, `${label}.lastPlayedAt`),
    playCount: number(row.playCount, `${label}.playCount`), membershipId: nullable(row.membershipId, string, `${label}.membershipId`),
    membershipPosition: nullable(row.membershipPosition, number, `${label}.membershipPosition`),
    membershipWeight: row.membershipWeight === null ? null : enumeration(row.membershipWeight, weights, `${label}.membershipWeight`),
    membershipEnabled: nullable(row.membershipEnabled, boolean, `${label}.membershipEnabled`), membershipVersion: nullable(row.membershipVersion, number, `${label}.membershipVersion`),
  };
}
export function parseItemWindow(value: unknown, label = "music item window"): MusicItemWindow {
  const row = object(value, label); return {
    items: array(row.items, parseItemEntry, `${label}.items`),
    groups: array(row.groups, (entry, entryLabel) => { const group = object(entry, entryLabel); return { key: string(group.key, `${entryLabel}.key`), count: number(group.count, `${entryLabel}.count`) }; }, `${label}.groups`),
    totalCount: number(row.totalCount, `${label}.totalCount`), offset: number(row.offset, `${label}.offset`), limit: number(row.limit, `${label}.limit`),
  };
}
function parsePlaylistSummary(value: unknown, label: string): MusicPlaylistSummary {
  const row = object(value, label); return {
    id: string(row.id, `${label}.id`), name: string(row.name, `${label}.name`), description: string(row.description, `${label}.description`),
    shuffleEnabled: boolean(row.shuffleEnabled, `${label}.shuffleEnabled`), repeatMode: enumeration(row.repeatMode, repeatModes, `${label}.repeatMode`),
    totalCount: number(row.totalCount, `${label}.totalCount`), eligibleCount: number(row.eligibleCount, `${label}.eligibleCount`), unavailableCount: number(row.unavailableCount, `${label}.unavailableCount`),
    snoozedCount: number(row.snoozedCount, `${label}.snoozedCount`), localCount: number(row.localCount, `${label}.localCount`), onlineCount: number(row.onlineCount, `${label}.onlineCount`), version: number(row.version, `${label}.version`),
  };
}
export const parsePlaylistSummaries = (value: unknown): MusicPlaylistSummary[] => array(value, parsePlaylistSummary, "playlist summaries");
function parseSourceSummary(value: unknown, label: string): MusicSourceSummary {
  const row = object(value, label); return {
    id: string(row.id, `${label}.id`), kind: enumeration(row.kind, collectionKinds, `${label}.kind`), name: string(row.name, `${label}.name`),
    refreshState: enumeration(row.refreshState, refreshStates, `${label}.refreshState`), lastSuccessfulRefreshAt: nullable(row.lastSuccessfulRefreshAt, number, `${label}.lastSuccessfulRefreshAt`),
    itemCount: number(row.itemCount, `${label}.itemCount`), missingCount: number(row.missingCount, `${label}.missingCount`), newCount: number(row.newCount, `${label}.newCount`), openIssueCount: number(row.openIssueCount, `${label}.openIssueCount`), version: number(row.version, `${label}.version`),
  };
}
export const parseSourceSummaries = (value: unknown): MusicSourceSummary[] => array(value, parseSourceSummary, "source summaries");
function parseIssue(value: unknown, label: string): MusicIssue {
  const row = object(value, label); return { id: string(row.id, `${label}.id`), issueKind: string(row.issueKind, `${label}.issueKind`), itemId: nullable(row.itemId, string, `${label}.itemId`), playlistId: nullable(row.playlistId, string, `${label}.playlistId`), message: string(row.message, `${label}.message`), createdAt: number(row.createdAt, `${label}.createdAt`) };
}
export const parseIssues = (value: unknown): MusicIssue[] => array(value, parseIssue, "music issues");
function parseLibraryItem(value: unknown, label: string): MusicLibraryItem {
  const row = object(value, label); return {
    id: string(row.id, `${label}.id`), identityKey: string(row.identityKey, `${label}.identityKey`), sourceKind: enumeration(row.sourceKind, sourceKinds, `${label}.sourceKind`), mediaKind: enumeration(row.mediaKind, mediaKinds, `${label}.mediaKind`),
    youtubeVideoId: nullable(row.youtubeVideoId, string, `${label}.youtubeVideoId`), originalTitle: string(row.originalTitle, `${label}.originalTitle`), originalArtist: string(row.originalArtist, `${label}.originalArtist`), originalAlbum: string(row.originalAlbum, `${label}.originalAlbum`),
    titleOverride: nullable(row.titleOverride, string, `${label}.titleOverride`), artistOverride: nullable(row.artistOverride, string, `${label}.artistOverride`), albumOverride: nullable(row.albumOverride, string, `${label}.albumOverride`), artworkOverride: nullable(row.artworkOverride, string, `${label}.artworkOverride`),
    durationMs: nullable(row.durationMs, number, `${label}.durationMs`), availability: enumeration(row.availability, itemAvailability, `${label}.availability`), reviewState: enumeration(row.reviewState, reviewStates, `${label}.reviewState`), reviewChangedAt: nullable(row.reviewChangedAt, number, `${label}.reviewChangedAt`), discoveredAt: number(row.discoveredAt, `${label}.discoveredAt`), updatedAt: number(row.updatedAt, `${label}.updatedAt`), version: number(row.version, `${label}.version`),
  };
}
function parseLocation(value: unknown, label: string): MusicLocalLocation {
  const row = object(value, label); return { id: string(row.id, `${label}.id`), itemId: string(row.itemId, `${label}.itemId`), rootId: string(row.rootId, `${label}.rootId`), relativePath: string(row.relativePath, `${label}.relativePath`), fileSizeBytes: nullable(row.fileSizeBytes, number, `${label}.fileSizeBytes`), modifiedAtMs: nullable(row.modifiedAtMs, number, `${label}.modifiedAtMs`), lightweightFingerprint: nullable(row.lightweightFingerprint, string, `${label}.lightweightFingerprint`), strongFingerprint: nullable(row.strongFingerprint, string, `${label}.strongFingerprint`), availability: enumeration(row.availability, locationAvailability, `${label}.availability`), lastSeenGeneration: nullable(row.lastSeenGeneration, number, `${label}.lastSeenGeneration`), firstSeenAt: number(row.firstSeenAt, `${label}.firstSeenAt`), updatedAt: number(row.updatedAt, `${label}.updatedAt`) };
}
function parseMembership(value: unknown, label: string): MusicPlaylistMembership {
  const row = object(value, label); return { id: string(row.id, `${label}.id`), playlistId: string(row.playlistId, `${label}.playlistId`), itemId: string(row.itemId, `${label}.itemId`), position: number(row.position, `${label}.position`), weight: enumeration(row.weight, weights, `${label}.weight`), enabled: boolean(row.enabled, `${label}.enabled`), focusFit: enumeration(row.focusFit, focusFits, `${label}.focusFit`), startMs: nullable(row.startMs, number, `${label}.startMs`), endMs: nullable(row.endMs, number, `${label}.endMs`), volume: nullable(row.volume, finite, `${label}.volume`), rate: nullable(row.rate, finite, `${label}.rate`), updatedAt: number(row.updatedAt, `${label}.updatedAt`), createdAt: number(row.createdAt, `${label}.createdAt`), version: number(row.version, `${label}.version`) };
}
function parseSnooze(value: unknown, label: string): MusicSnooze {
  const row = object(value, label); return { id: string(row.id, `${label}.id`), itemId: string(row.itemId, `${label}.itemId`), scope: enumeration(row.scope, snoozeScopes, `${label}.scope`), playlistId: nullable(row.playlistId, string, `${label}.playlistId`), startsAt: number(row.startsAt, `${label}.startsAt`), endsAt: nullable(row.endsAt, number, `${label}.endsAt`), reason: string(row.reason, `${label}.reason`), createdAt: number(row.createdAt, `${label}.createdAt`) };
}
function parseStatistics(value: unknown, label: string): MusicListeningStatistics {
  const row = object(value, label); return { itemId: string(row.itemId, `${label}.itemId`), lastPlayedAt: nullable(row.lastPlayedAt, number, `${label}.lastPlayedAt`), playCount: number(row.playCount, `${label}.playCount`), completionCount: number(row.completionCount, `${label}.completionCount`), skipCount: number(row.skipCount, `${label}.skipCount`), updatedAt: number(row.updatedAt, `${label}.updatedAt`) };
}
export function parseInspectorDetail(value: unknown): MusicInspectorDetail {
  const row = object(value, "music inspector"); return { item: parseLibraryItem(row.item, "music inspector.item"), locations: array(row.locations, parseLocation, "music inspector.locations"), memberships: array(row.memberships, parseMembership, "music inspector.memberships"), snoozes: array(row.snoozes, parseSnooze, "music inspector.snoozes"), signals: array(row.signals, (entry, label) => enumeration(entry, signals, label), "music inspector.signals"), statistics: row.statistics === null ? null : parseStatistics(row.statistics, "music inspector.statistics"), sourceCollectionIds: array(row.sourceCollectionIds, string, "music inspector.sourceCollectionIds") };
}
function parseRoot(value: unknown, label: string): MusicLocalRoot { const row = object(value, label); return { id: string(row.id, `${label}.id`), name: string(row.name, `${label}.name`), createdAt: number(row.createdAt, `${label}.createdAt`), updatedAt: number(row.updatedAt, `${label}.updatedAt`), version: number(row.version, `${label}.version`) }; }
export const parseRoots = (value: unknown): MusicLocalRoot[] => array(value, parseRoot, "music roots");
function parseCollection(value: unknown, label: string): MusicSourceCollection { const row = object(value, label); return { id: string(row.id, `${label}.id`), kind: enumeration(row.kind, collectionKinds, `${label}.kind`), identityKey: string(row.identityKey, `${label}.identityKey`), name: string(row.name, `${label}.name`), localRootId: nullable(row.localRootId, string, `${label}.localRootId`), youtubePlaylistId: nullable(row.youtubePlaylistId, string, `${label}.youtubePlaylistId`), refreshState: enumeration(row.refreshState, refreshStates, `${label}.refreshState`), lastSuccessfulRefreshAt: nullable(row.lastSuccessfulRefreshAt, number, `${label}.lastSuccessfulRefreshAt`), lastRefreshErrorCode: nullable(row.lastRefreshErrorCode, string, `${label}.lastRefreshErrorCode`), snapshotGeneration: number(row.snapshotGeneration, `${label}.snapshotGeneration`), createdAt: number(row.createdAt, `${label}.createdAt`), updatedAt: number(row.updatedAt, `${label}.updatedAt`), version: number(row.version, `${label}.version`) }; }
export const parseCollections = (value: unknown): MusicSourceCollection[] => array(value, parseCollection, "music collections");
export function parsePlaylist(value: unknown): MusicPlaylist { const row = object(value, "music playlist"); return { id: string(row.id, "music playlist.id"), name: string(row.name, "music playlist.name"), description: string(row.description, "music playlist.description"), shuffleEnabled: boolean(row.shuffleEnabled, "music playlist.shuffleEnabled"), repeatMode: enumeration(row.repeatMode, repeatModes, "music playlist.repeatMode"), intendedUses: array(row.intendedUses, (entry, label) => enumeration(entry, intendedUses, label), "music playlist.intendedUses"), createdAt: number(row.createdAt, "music playlist.createdAt"), updatedAt: number(row.updatedAt, "music playlist.updatedAt"), version: number(row.version, "music playlist.version") }; }
export function parseSearchRebuild(value: unknown): MusicSearchRebuildResult { const row = object(value, "music search rebuild"); return { indexedItemCount: number(row.indexedItemCount, "music search rebuild.indexedItemCount"), schemaVersion: number(row.schemaVersion, "music search rebuild.schemaVersion"), fingerprint: string(row.fingerprint, "music search rebuild.fingerprint"), rebuiltAt: number(row.rebuiltAt, "music search rebuild.rebuiltAt") }; }
function parseBinding(value: unknown, label: string): LocalRootBinding { const row = object(value, label); return { rootId: string(row.rootId, `${label}.rootId`), folderPath: nullable(row.folderPath, string, `${label}.folderPath`), status: enumeration(row.status, rootStatuses, `${label}.status`) }; }
export const parseBindings = (value: unknown): LocalRootBinding[] => array(value, parseBinding, "music root bindings");
export const parseBindingResult = (value: unknown): LocalRootBinding => parseBinding(value, "music root binding");
