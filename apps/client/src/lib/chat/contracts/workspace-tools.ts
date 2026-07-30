import type {
  ChatAttachmentId,
  ChatCheckpointId,
  ChatThreadId,
  ChatTurnId,
  ProjectWorkingFolderId,
  UtcTimestamp,
} from "./common";

export type ChatInspectorTab = "changes" | "plan" | "files" | "sourceControl" | "browser" | "review" | "terminal";
export type ChatChangeScope = "current_turn" | "entire_thread";
export type ChatChangedFileStatus = "added" | "modified" | "deleted" | "renamed" | "type_changed" | "unknown";

export interface GitChangedPathRead {
  relativePath: string;
  originalRelativePath: string | null;
  indexStatus: string;
  worktreeStatus: string;
  untracked: boolean;
  ignored: boolean;
}

export interface GitStatusRead {
  branch: string | null;
  detached: boolean;
  upstream: string | null;
  ahead: number;
  behind: number;
  files: GitChangedPathRead[];
}

export interface GitRemoteRead {
  name: string;
  fetchUrl: string | null;
  pushUrl: string | null;
}

export interface GitBranchRead {
  name: string;
  current: boolean;
  upstream: string | null;
  ahead: number;
  behind: number;
}

export interface GitWorktreeRead {
  path: string;
  head: string;
  branch: string | null;
  bare: boolean;
  detached: boolean;
  locked: boolean;
  prunable: boolean;
}

export interface ChatExecutionEnvironmentRead {
  id: string;
  workingFolderId: ProjectWorkingFolderId;
  kind: "current_folder" | "worktree";
  displayName: string;
  lifecycleState: string;
  branchName: string | null;
  baseReference: string | null;
  remoteName: string | null;
  cleanupState: string | null;
  localPath: string | null;
  createdAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
}

export interface CreateChatWorktreeRequest {
  environmentId: string;
  workingFolderId: ProjectWorkingFolderId;
  displayName: string;
  branchName: string;
  baseReference: string;
  remoteName: string | null;
  fetchRemote: boolean;
}

export interface HostedSourceControlRead {
  kind: "github" | "gitlab" | "azure_devops" | "bitbucket";
  label: string;
  detectedForRepository: boolean;
  remoteName: string | null;
  repositorySlug: string | null;
  status: string;
  version: string | null;
  unavailableReason: string | null;
  configurationHint: string | null;
}

export interface HostedChangeRequestRead {
  providerKind: HostedSourceControlRead["kind"];
  number: number;
  title: string;
  url: string;
  state: string;
  baseBranch: string;
  headBranch: string;
  author: string | null;
  draft: boolean;
}

export interface CreateHostedChangeRequest {
  workingFolderId: ProjectWorkingFolderId;
  executionEnvironmentId: string | null;
  providerKind: HostedSourceControlRead["kind"];
  repositorySlug: string;
  title: string;
  body: string;
  baseBranch: string;
  headBranch: string;
  draft: boolean;
}

export interface PreviewBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface PreviewTabRead {
  threadId: ChatThreadId;
  tabId: string;
  currentUrl: string;
  title: string;
  visible: boolean;
  loading: boolean;
  viewportWidth: number;
  viewportHeight: number;
  externalOrigin: boolean;
}

export type ChatReviewCommentState = "open" | "resolved";

export interface ChatReviewCommentRead {
  id: string;
  threadId: ChatThreadId;
  relativePath: string;
  contentRevision: string;
  startLine: number;
  startColumn: number;
  endLine: number;
  endColumn: number;
  selectedText: string;
  commentText: string;
  state: ChatReviewCommentState;
  createdAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
  resolvedAt: UtcTimestamp | null;
}

export interface CreateChatReviewCommentRequest {
  id: string;
  threadId: ChatThreadId;
  relativePath: string;
  contentRevision: string;
  startLine: number;
  startColumn: number;
  endLine: number;
  endColumn: number;
  selectedText: string;
  commentText: string;
}

export interface ProjectWorkingFolderFileEntry {
  relativePath: string;
  displayName: string;
  kind: "file" | "directory";
  ignored: boolean;
  byteSize: number | null;
}

export interface ProjectWorkingFolderDirectoryRead {
  relativePath: string;
  entries: ProjectWorkingFolderFileEntry[];
  truncated: boolean;
}

export interface ProjectWorkingFolderFilePreview {
  relativePath: string;
  displayName: string;
  language: string | null;
  text: string | null;
  lineCount: number | null;
  byteSize: number;
  binary: boolean;
  oversized: boolean;
  contentRevision: string | null;
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
  workingFolderId: ProjectWorkingFolderId;
  name: string;
  shell: string;
  columns: number;
  rows: number;
  running: boolean;
  exitCode: number | null;
  generation: number;
  lastSequence: number;
}

export interface ChatTerminalPanelLayout {
  placement: "inspector" | "bottom";
  terminalNames: string[];
  selectedIndex: number | null;
  splitDirection: "horizontal" | "vertical";
  splitSizes: number[];
}

export interface ChatTerminalLayoutRead {
  threadId: ChatThreadId;
  schemaVersion: number;
  groups: ChatTerminalPanelLayout[];
  updatedAt: UtcTimestamp | null;
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
