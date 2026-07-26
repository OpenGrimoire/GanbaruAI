import {
  type ChatCheckpointDiffRead,
  type ChatCheckpointFileDiffRead,
  type ChatChangedFileRead,
  type ChatRestorePreviewRead,
  type ChatRestoreResultRead,
  type ChatTerminalContextRead,
  type ChatTerminalCloseResult,
  type ChatTerminalOutputChunk,
  type ChatTerminalRead,
  type ChatTerminalSnapshotRead,
  type ProjectWorkingFolderDirectoryRead,
  type ProjectWorkingFolderFileEntry,
  type ProjectWorkingFolderFilePreview,
} from "../contracts";
import {
  readBoolean,
  readEnum,
  readIdentifier,
  readNonNegativeSafeInteger,
  readNullable,
  readRecord,
  readSafeInteger,
  readString,
  readUtcTimestamp,
} from "./readers";

const FILE_KINDS = ["file", "directory"] as const;
const FILE_STATUSES = ["added", "modified", "deleted", "renamed", "type_changed", "unknown"] as const;
const CHANGE_SCOPES = ["current_turn", "entire_thread"] as const;

function array<T>(value: unknown, label: string, parse: (entry: unknown, label: string) => T): T[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  return value.map((entry, index) => parse(entry, `${label}[${index}]`));
}

function parseWorkspaceFileEntry(value: unknown, label: string): ProjectWorkingFolderFileEntry {
  const record = readRecord(value, label);
  return {
    relativePath: readString(record.relativePath, `${label}.relativePath`),
    displayName: readString(record.displayName, `${label}.displayName`),
    kind: readEnum(record.kind, FILE_KINDS, `${label}.kind`),
    ignored: readBoolean(record.ignored, `${label}.ignored`),
    byteSize: readNullable(record.byteSize, `${label}.byteSize`, readNonNegativeSafeInteger),
  };
}

export function parseProjectWorkingFolderDirectory(value: unknown): ProjectWorkingFolderDirectoryRead {
  const record = readRecord(value, "workspaceDirectory");
  return {
    relativePath: readString(record.relativePath, "workspaceDirectory.relativePath"),
    entries: array(record.entries, "workspaceDirectory.entries", parseWorkspaceFileEntry),
    truncated: readBoolean(record.truncated, "workspaceDirectory.truncated"),
  };
}

export function parseProjectWorkingFolderFilePreview(value: unknown): ProjectWorkingFolderFilePreview {
  const record = readRecord(value, "workspaceFilePreview");
  return {
    relativePath: readString(record.relativePath, "workspaceFilePreview.relativePath"),
    displayName: readString(record.displayName, "workspaceFilePreview.displayName"),
    language: readNullable(record.language, "workspaceFilePreview.language", readString),
    text: readNullable(record.text, "workspaceFilePreview.text", readString),
    lineCount: readNullable(record.lineCount, "workspaceFilePreview.lineCount", readNonNegativeSafeInteger),
    byteSize: readNonNegativeSafeInteger(record.byteSize, "workspaceFilePreview.byteSize"),
    binary: readBoolean(record.binary, "workspaceFilePreview.binary"),
    oversized: readBoolean(record.oversized, "workspaceFilePreview.oversized"),
  };
}

function parseChangedFile(value: unknown, label: string): ChatChangedFileRead {
  const record = readRecord(value, label);
  return {
    relativePath: readString(record.relativePath, `${label}.relativePath`),
    previousRelativePath: readNullable(record.previousRelativePath, `${label}.previousRelativePath`, readString),
    status: readEnum(record.status, FILE_STATUSES, `${label}.status`),
    additions: readNullable(record.additions, `${label}.additions`, readNonNegativeSafeInteger),
    deletions: readNullable(record.deletions, `${label}.deletions`, readNonNegativeSafeInteger),
    binary: readBoolean(record.binary, `${label}.binary`),
    providerReported: readBoolean(record.providerReported, `${label}.providerReported`),
    gitObserved: readBoolean(record.gitObserved, `${label}.gitObserved`),
  };
}

export function parseChatCheckpointDiff(value: unknown): ChatCheckpointDiffRead {
  const record = readRecord(value, "checkpointDiff");
  return {
    scope: readEnum(record.scope, CHANGE_SCOPES, "checkpointDiff.scope"),
    available: readBoolean(record.available, "checkpointDiff.available"),
    unavailableReason: readNullable(record.unavailableReason, "checkpointDiff.unavailableReason", readString),
    preCheckpointId: readNullable(record.preCheckpointId, "checkpointDiff.preCheckpointId", readIdentifier),
    postCheckpointId: readNullable(record.postCheckpointId, "checkpointDiff.postCheckpointId", readIdentifier),
    files: array(record.files, "checkpointDiff.files", parseChangedFile),
    additions: readNonNegativeSafeInteger(record.additions, "checkpointDiff.additions"),
    deletions: readNonNegativeSafeInteger(record.deletions, "checkpointDiff.deletions"),
    providerMismatch: readBoolean(record.providerMismatch, "checkpointDiff.providerMismatch"),
  };
}

export function parseChatCheckpointFileDiff(value: unknown): ChatCheckpointFileDiffRead {
  const record = readRecord(value, "checkpointFileDiff");
  return {
    relativePath: readString(record.relativePath, "checkpointFileDiff.relativePath"),
    patch: readNullable(record.patch, "checkpointFileDiff.patch", readString),
    binary: readBoolean(record.binary, "checkpointFileDiff.binary"),
    truncated: readBoolean(record.truncated, "checkpointFileDiff.truncated"),
    byteSize: readNonNegativeSafeInteger(record.byteSize, "checkpointFileDiff.byteSize"),
  };
}

export function parseChatTerminal(value: unknown, label = "terminal"): ChatTerminalRead {
  const record = readRecord(value, label);
  return {
    id: readIdentifier(record.id, `${label}.id`),
    threadId: readIdentifier(record.threadId, `${label}.threadId`),
    workingFolderId: readIdentifier(record.workingFolderId, `${label}.workingFolderId`),
    name: readString(record.name, `${label}.name`),
    shell: readString(record.shell, `${label}.shell`),
    columns: readNonNegativeSafeInteger(record.columns, `${label}.columns`),
    rows: readNonNegativeSafeInteger(record.rows, `${label}.rows`),
    running: readBoolean(record.running, `${label}.running`),
    exitCode: readNullable(record.exitCode, `${label}.exitCode`, readSafeInteger),
    generation: readNonNegativeSafeInteger(record.generation, `${label}.generation`),
    lastSequence: readNonNegativeSafeInteger(record.lastSequence, `${label}.lastSequence`),
  };
}

export function parseChatTerminalOutput(value: unknown, label = "terminalOutput"): ChatTerminalOutputChunk {
  const record = readRecord(value, label);
  return {
    terminalId: readIdentifier(record.terminalId, `${label}.terminalId`),
    generation: readNonNegativeSafeInteger(record.generation, `${label}.generation`),
    sequence: readNonNegativeSafeInteger(record.sequence, `${label}.sequence`),
    dataBase64: readString(record.dataBase64, `${label}.dataBase64`),
    replay: readBoolean(record.replay, `${label}.replay`),
  };
}

export function parseChatTerminalSnapshot(value: unknown): ChatTerminalSnapshotRead {
  const record = readRecord(value, "terminalSnapshot");
  return {
    terminal: parseChatTerminal(record.terminal, "terminalSnapshot.terminal"),
    scrollback: array(record.scrollback, "terminalSnapshot.scrollback", parseChatTerminalOutput),
  };
}

export function parseChatTerminalCloseResult(value: unknown): ChatTerminalCloseResult {
  const record = readRecord(value, "terminalClose");
  return {
    closed: readBoolean(record.closed, "terminalClose.closed"),
    confirmationRequired: readBoolean(record.confirmationRequired, "terminalClose.confirmationRequired"),
  };
}

export function parseChatTerminals(value: unknown): ChatTerminalRead[] {
  return array(value, "terminals", parseChatTerminal);
}

export function parseChatRestorePreview(value: unknown): ChatRestorePreviewRead {
  const record = readRecord(value, "restorePreview");
  return {
    previewId: readIdentifier(record.previewId, "restorePreview.previewId"),
    checkpointId: readIdentifier(record.checkpointId, "restorePreview.checkpointId"),
    expiresAt: readUtcTimestamp(record.expiresAt, "restorePreview.expiresAt"),
    files: array(record.files, "restorePreview.files", parseChangedFile),
    stagedChanges: readBoolean(record.stagedChanges, "restorePreview.stagedChanges"),
    providerRollback: readEnum(record.providerRollback, ["supported", "unsupported"] as const, "restorePreview.providerRollback"),
    warnings: array(record.warnings, "restorePreview.warnings", readString),
  };
}

export function parseChatRestoreResult(value: unknown): ChatRestoreResultRead {
  const record = readRecord(value, "restoreResult");
  return {
    checkpointId: readIdentifier(record.checkpointId, "restoreResult.checkpointId"),
    revertedTurnIds: array(record.revertedTurnIds, "restoreResult.revertedTurnIds", readIdentifier),
    providerHistoryAction: readEnum(record.providerHistoryAction, ["rolled_back", "fork_required"] as const, "restoreResult.providerHistoryAction"),
    recoveryState: readEnum(record.recoveryState, ["complete", "recovered", "recovery_required"] as const, "restoreResult.recoveryState"),
    threadRevision: readNonNegativeSafeInteger(record.threadRevision, "restoreResult.threadRevision"),
  };
}

export function parseChatTerminalContext(value: unknown): ChatTerminalContextRead {
  const record = readRecord(value, "terminalContext");
  return {
    attachmentId: readIdentifier(record.attachmentId, "terminalContext.attachmentId"),
    terminalId: readIdentifier(record.terminalId, "terminalContext.terminalId"),
    terminalName: readString(record.terminalName, "terminalContext.terminalName"),
    capturedAt: readUtcTimestamp(record.capturedAt, "terminalContext.capturedAt"),
    byteSize: readNonNegativeSafeInteger(record.byteSize, "terminalContext.byteSize"),
    lineCount: readNonNegativeSafeInteger(record.lineCount, "terminalContext.lineCount"),
    preview: readString(record.preview, "terminalContext.preview"),
    truncated: readBoolean(record.truncated, "terminalContext.truncated"),
  };
}
