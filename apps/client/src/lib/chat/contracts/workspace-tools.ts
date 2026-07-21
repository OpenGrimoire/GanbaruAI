import type {
  ChatAttachmentId,
  ChatCheckpointId,
  ChatThreadId,
  ChatTurnId,
  ChatWorkspaceId,
  UtcTimestamp,
} from "./common";

export type ChatInspectorTab = "changes" | "plan" | "files" | "terminal";
export type ChatChangeScope = "current_turn" | "entire_thread";
export type ChatChangedFileStatus = "added" | "modified" | "deleted" | "renamed" | "type_changed" | "unknown";

export interface ChatWorkspaceFileEntry {
  relativePath: string;
  displayName: string;
  kind: "file" | "directory";
  ignored: boolean;
  byteSize: number | null;
}

export interface ChatWorkspaceDirectoryRead {
  relativePath: string;
  entries: ChatWorkspaceFileEntry[];
  truncated: boolean;
}

export interface ChatWorkspaceFilePreview {
  relativePath: string;
  displayName: string;
  language: string | null;
  text: string | null;
  lineCount: number | null;
  byteSize: number;
  binary: boolean;
  oversized: boolean;
}

export interface ChatChangedFileRead {
  relativePath: string;
  previousRelativePath: string | null;
  status: ChatChangedFileStatus;
  additions: number | null;
  deletions: number | null;
  binary: boolean;
  providerReported: boolean;
  gitObserved: boolean;
}

export interface ChatCheckpointDiffRead {
  scope: ChatChangeScope;
  available: boolean;
  unavailableReason: string | null;
  preCheckpointId: ChatCheckpointId | null;
  postCheckpointId: ChatCheckpointId | null;
  files: ChatChangedFileRead[];
  additions: number;
  deletions: number;
  providerMismatch: boolean;
}

export interface ChatCheckpointFileDiffRead {
  relativePath: string;
  patch: string | null;
  binary: boolean;
  truncated: boolean;
  byteSize: number;
}

export interface ChatRestorePreviewRead {
  previewId: string;
  checkpointId: ChatCheckpointId;
  expiresAt: UtcTimestamp;
  files: ChatChangedFileRead[];
  stagedChanges: boolean;
  providerRollback: "supported" | "unsupported";
  warnings: string[];
}

export interface ChatRestoreResultRead {
  checkpointId: ChatCheckpointId;
  revertedTurnIds: ChatTurnId[];
  providerHistoryAction: "rolled_back" | "fork_required";
  recoveryState: "complete" | "recovered" | "recovery_required";
  threadRevision: number;
}

export interface ChatTerminalRead {
  id: string;
  threadId: ChatThreadId;
  workspaceId: ChatWorkspaceId;
  name: string;
  shell: string;
  columns: number;
  rows: number;
  running: boolean;
  exitCode: number | null;
  generation: number;
  lastSequence: number;
}

export interface ChatTerminalOutputChunk {
  terminalId: string;
  generation: number;
  sequence: number;
  dataBase64: string;
  replay: boolean;
}

export interface ChatTerminalSnapshotRead {
  terminal: ChatTerminalRead;
  scrollback: ChatTerminalOutputChunk[];
}

export interface ChatTerminalCloseResult {
  closed: boolean;
  confirmationRequired: boolean;
}

export interface ChatTerminalContextRead {
  attachmentId: ChatAttachmentId;
  terminalId: string;
  terminalName: string;
  capturedAt: UtcTimestamp;
  byteSize: number;
  lineCount: number;
  preview: string;
  truncated: boolean;
}
