import type {
  ChatReviewFileRead,
  ChatReviewPatchHunkRead,
  ChatReviewPatchRead,
  ChatReviewSnapshotRead,
  ReviewDiffSource,
} from "./contracts";

export type ReviewLayoutPreference = "auto" | "continuous" | "file";
export type ReviewResolvedLayout = Exclude<ReviewLayoutPreference, "auto">;
export type ReviewDiffPreference = "auto" | "unified" | "split";
export type ReviewResolvedDiffStyle = Exclude<ReviewDiffPreference, "auto">;

export interface ReviewOpenIntent {
  source: ReviewDiffSource;
  relativePath: string | null;
}

export interface ReviewSearchMatch {
  fileId: string;
  relativePath: string;
  lineNumber: number;
  side: "deletions" | "additions";
  text: string;
}

/** Appends unseen patch pages while preserving the backend page order for every file. */
export function appendReviewPatchPages(
  current: readonly ChatReviewPatchRead[],
  incoming: readonly ChatReviewPatchRead[],
): ChatReviewPatchRead[] {
  if (incoming.length === 0) return [...current];
  const seen = new Set(current.map(reviewPatchPageKey));
  const next = [...current];
  for (const patch of incoming) {
    const key = reviewPatchPageKey(patch);
    if (seen.has(key)) continue;
    seen.add(key);
    next.push(patch);
  }
  return next;
}

function reviewPatchPageKey(patch: ChatReviewPatchRead): string {
  return `${patch.fileId}\u0000${patch.continuationCursor ?? "<complete>"}`;
}

export const REVIEW_CONTINUOUS_MIN_WIDTH_PX = 720;
export const REVIEW_CONTINUOUS_MAX_FILES = 200;
export const REVIEW_CONTINUOUS_MAX_CHANGED_LINES = 50_000;
export const REVIEW_SPLIT_MIN_WIDTH_PX = 720;

/** Resolves the review flow without relying on viewport classes. */
export function resolveReviewLayout(
  preference: ReviewLayoutPreference,
  availableWidth: number,
  fileCount: number,
  changedLines: number,
): ReviewResolvedLayout {
  if (preference === "file") return "file";
  if (fileCount > REVIEW_CONTINUOUS_MAX_FILES || changedLines > REVIEW_CONTINUOUS_MAX_CHANGED_LINES) {
    return "file";
  }
  if (preference === "continuous") return "continuous";
  return availableWidth >= REVIEW_CONTINUOUS_MIN_WIDTH_PX
    ? "continuous"
    : "file";
}

/** Resolves split diff rendering from the measured review content width. */
export function resolveReviewDiffStyle(
  preference: ReviewDiffPreference,
  availableWidth: number,
): ReviewResolvedDiffStyle {
  if (preference !== "auto") return preference;
  return availableWidth >= REVIEW_SPLIT_MIN_WIDTH_PX ? "split" : "unified";
}

/** Creates the initial source used when an older Changes tab opens Review. */
export function legacyReviewSource(
  scope: "current_turn" | "entire_thread",
  turnId: string | null,
): ReviewDiffSource {
  return {
    kind: "checkpoint",
    range: scope === "entire_thread" ? "thread" : "turn",
    turnId: scope === "entire_thread" ? null : turnId,
  };
}

/** Selects the file retained across immutable snapshot refreshes. */
export function retainReviewFile(
  snapshot: ChatReviewSnapshotRead,
  fileId: string | null,
  relativePath: string | null,
): ChatReviewFileRead | null {
  return snapshot.files.find((file) => file.fileId === fileId)
    ?? snapshot.files.find((file) => file.relativePath === relativePath)
    ?? snapshot.files[0]
    ?? null;
}

/** Returns the reviewed hunk identifiers intersecting the current line selection. */
export function selectedReviewHunkIds(
  patch: ChatReviewPatchRead | null,
  range: { start: number; end: number; side?: "deletions" | "additions"; endSide?: "deletions" | "additions" } | null,
): string[] {
  if (!patch || !range) return [];
  const start = Math.min(range.start, range.end);
  const end = Math.max(range.start, range.end);
  const sides = new Set([range.side ?? "additions", range.endSide ?? range.side ?? "additions"]);
  return patch.hunks
    .filter((hunk) => hunk.state === "complete" && hunkIntersects(hunk, sides, start, end))
    .map((hunk) => hunk.hunkId);
}

function hunkIntersects(
  hunk: ChatReviewPatchHunkRead,
  sides: ReadonlySet<"deletions" | "additions">,
  start: number,
  end: number,
): boolean {
  return (sides.has("deletions") && rangesIntersect(start, end, hunk.oldStart, hunk.oldCount))
    || (sides.has("additions") && rangesIntersect(start, end, hunk.newStart, hunk.newCount));
}

function rangesIntersect(start: number, end: number, hunkStart: number, hunkCount: number): boolean {
  if (hunkCount <= 0) return false;
  return start <= hunkStart + hunkCount - 1 && end >= hunkStart;
}

/** Finds patch text without constructing rendered DOM nodes. */
export function findReviewSearchMatches(
  files: readonly ChatReviewFileRead[],
  patches: readonly ChatReviewPatchRead[],
  query: string,
  limit = 1_000,
): ReviewSearchMatch[] {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return [];
  const filesById = new Map(files.map((file) => [file.fileId, file]));
  const matches: ReviewSearchMatch[] = [];
  for (const patch of patches) {
    const file = filesById.get(patch.fileId);
    if (!file || !patch.patch) continue;
    let oldLine = 0;
    let newLine = 0;
    for (const rawLine of patch.patch.split("\n")) {
      const hunk = /^@@ -(?<old>\d+)(?:,\d+)? \+(?<next>\d+)(?:,\d+)? @@/.exec(rawLine);
      if (hunk?.groups) {
        oldLine = Number.parseInt(hunk.groups.old ?? "0", 10);
        newLine = Number.parseInt(hunk.groups.next ?? "0", 10);
        continue;
      }
      const marker = rawLine[0];
      if (rawLine.startsWith("+++") || rawLine.startsWith("---")) continue;
      if (marker !== "+" && marker !== "-" && marker !== " ") continue;
      const text = rawLine.slice(1);
      if (text.toLocaleLowerCase().includes(needle)) {
        matches.push({
          fileId: patch.fileId,
          relativePath: file.relativePath,
          lineNumber: marker === "-" ? oldLine : newLine,
          side: marker === "-" ? "deletions" : "additions",
          text,
        });
        if (matches.length >= limit) return matches;
      }
      if (marker !== "+") oldLine += 1;
      if (marker !== "-") newLine += 1;
    }
  }
  return matches;
}

/** Returns a stable source key for request cancellation and local state. */
export function reviewSourceKey(source: ReviewDiffSource): string {
  switch (source.kind) {
    case "working_tree": return `${source.kind}:${source.mode}`;
    case "checkpoint": return `${source.kind}:${source.range}:${source.turnId ?? ""}`;
    case "commit": return `${source.kind}:${source.revision}`;
    case "branch": return `${source.kind}:${source.baseRef ?? ""}:${source.headRef}:${source.comparison}`;
    case "provider_turn": return `${source.kind}:${source.turnId}`;
    case "change_request": return `${source.kind}:${source.provider}:${source.repositorySlug}:${source.number}`;
  }
}
