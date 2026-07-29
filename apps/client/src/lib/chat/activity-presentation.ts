import type { TimelineActivityRow } from "./timeline-model";

export interface CommandActivityPresentation {
  command: string | null;
  cwd: string | null;
  output: string | null;
  exitCode: number | null;
  durationMs: number | null;
  running: boolean;
}

export interface FileChangePresentation {
  path: string;
  kind: string;
  diff: string | null;
  additions: number;
  deletions: number;
}

export type ActivitySummaryKind =
  | "thinking"
  | "commands"
  | "file_changes"
  | "file_reads"
  | "web_searches"
  | "image_views"
  | "tools"
  | "collaboration"
  | "review"
  | "compaction"
  | "errors";

export interface ActivitySummaryCount {
  kind: ActivitySummaryKind;
  count: number;
}

const MAX_TRANSIENT_SUMMARY_CHARACTERS = 160;

/** Returns whether a timeline activity represents a shell command lifecycle. */
export function isCommandActivity(activity: TimelineActivityRow): boolean {
  return activity.activityKind === "command_execution" || activity.activityKind === "command_output";
}

/** Returns whether a timeline activity represents a provider file edit lifecycle. */
export function isFileChangeActivity(activity: TimelineActivityRow): boolean {
  return activity.activityKind === "file_change" || activity.activityKind === "file_change_output";
}

/** Returns whether an activity is a provider-neutral file read action. */
export function isFileReadActivity(activity: TimelineActivityRow): boolean {
  const tool = normalizedToolName(activity.title);
  return ["read", "read_file", "read_files", "open_file", "read_mcp_resource"]
    .some((name) => tool === name || tool.endsWith(`_${name}`));
}

/** Returns whether an activity is a provider-neutral image inspection action. */
export function isImageViewActivity(activity: TimelineActivityRow): boolean {
  if (activity.activityKind === "image_view") return true;
  const tool = normalizedToolName(activity.title);
  return ["view_image", "image_view"]
    .some((name) => tool === name || tool.endsWith(`_${name}`));
}

/** Summarizes consecutive actions by semantic kind while preserving first occurrence order. */
export function summarizeActivityKinds(
  activities: readonly TimelineActivityRow[],
): ActivitySummaryCount[] {
  const counts = new Map<ActivitySummaryKind, number>();
  for (const activity of activities) {
    const kind = activitySummaryKind(activity);
    counts.set(kind, (counts.get(kind) ?? 0) + 1);
  }
  return [...counts].map(([kind, count]) => ({ kind, count }));
}

/** Returns a bounded provider-designated summary suitable for a temporary live status row. */
export function transientActivitySummary(activity: TimelineActivityRow): string | null {
  if (activity.activityKind !== "reasoning_summary") return null;
  const lines = activity.detail
    ?.split(/\r?\n/u)
    .map((line) => line.trim().replace(/^(?:#{1,6}|>|\*|-)\s+/u, ""))
    .filter(Boolean);
  const summary = lines?.at(-1)?.replaceAll(/\s+/gu, " ").trim();
  if (!summary) return null;
  if (summary.length <= MAX_TRANSIENT_SUMMARY_CHARACTERS) return summary;
  return `${summary.slice(0, MAX_TRANSIENT_SUMMARY_CHARACTERS - 1).trimEnd()}…`;
}

/** Converts validated timeline data into the command card view model. */
export function commandActivityPresentation(
  activity: TimelineActivityRow,
): CommandActivityPresentation {
  const metadata = record(activity.metadata?.value);
  const rawCommand = activity.title.trim().replace(/^(?:run|running|ran)\s+/i, "");
  const command = rawCommand.length > 0
    && rawCommand !== "command execution"
    && rawCommand !== "command output"
    ? rawCommand
    : null;
  const output = activity.detail?.trim() || null;
  return {
    command,
    cwd: stringValue(metadata?.cwd),
    output,
    exitCode: integerValue(metadata?.exitCode),
    durationMs: nonNegativeNumber(metadata?.durationMs),
    running: activity.status === "pending"
      || activity.status === "active"
      || activity.status === "waiting",
  };
}

/** Converts provider file-change metadata into bounded diff rows. */
export function fileChangePresentation(activity: TimelineActivityRow): FileChangePresentation[] {
  const metadata = record(activity.metadata?.value);
  if (!Array.isArray(metadata?.changes)) return [];
  const changes: FileChangePresentation[] = [];
  for (const candidate of metadata.changes) {
    const change = record(candidate);
    const path = stringValue(change?.path);
    if (!path) continue;
    const diff = stringValue(change?.diff);
    const counts = diffLineCounts(diff);
    changes.push({
      path,
      kind: stringValue(change?.kind) ?? "modified",
      diff,
      additions: counts.additions,
      deletions: counts.deletions,
    });
  }
  return changes;
}

function diffLineCounts(diff: string | null): { additions: number; deletions: number } {
  if (!diff) return { additions: 0, deletions: 0 };
  let additions = 0;
  let deletions = 0;
  for (const line of diff.split(/\r?\n/u)) {
    if (line.startsWith("+") && !line.startsWith("+++")) additions += 1;
    if (line.startsWith("-") && !line.startsWith("---")) deletions += 1;
  }
  return { additions, deletions };
}

function record(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function stringValue(value: unknown): string | null {
  return typeof value === "string" && value.trim().length > 0 ? value.trim() : null;
}

function integerValue(value: unknown): number | null {
  return typeof value === "number" && Number.isSafeInteger(value) ? value : null;
}

function nonNegativeNumber(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) && value >= 0 ? value : null;
}

function activitySummaryKind(activity: TimelineActivityRow): ActivitySummaryKind {
  if (activity.id.startsWith("turn-pending:")
    || activity.activityKind === "reasoning"
    || activity.activityKind === "reasoning_text"
    || activity.activityKind === "reasoning_summary") return "thinking";
  if (isCommandActivity(activity)) return "commands";
  if (isFileChangeActivity(activity)) return "file_changes";
  if (isFileReadActivity(activity)) return "file_reads";
  if (activity.activityKind === "web_search") return "web_searches";
  if (isImageViewActivity(activity)) return "image_views";
  if (activity.activityKind === "collaboration_task") return "collaboration";
  if (activity.activityKind === "review_transition") return "review";
  if (activity.activityKind === "context_compaction") return "compaction";
  if (activity.activityKind === "error" || activity.status === "failed") return "errors";
  return "tools";
}

function normalizedToolName(title: string): string {
  return title.trim().toLowerCase().replaceAll(/[^a-z0-9]+/g, "_");
}
