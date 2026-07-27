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

/** Returns whether a timeline activity represents a shell command lifecycle. */
export function isCommandActivity(activity: TimelineActivityRow): boolean {
  return activity.activityKind === "command_execution" || activity.activityKind === "command_output";
}

/** Returns whether a timeline activity represents a provider file edit lifecycle. */
export function isFileChangeActivity(activity: TimelineActivityRow): boolean {
  return activity.activityKind === "file_change" || activity.activityKind === "file_change_output";
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
