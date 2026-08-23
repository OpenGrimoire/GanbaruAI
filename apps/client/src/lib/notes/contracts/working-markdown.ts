import type { ProjectWorkingFolderId } from "$lib/chat/contracts";

export type NotesWorkingMarkdownNodeKind = "directory" | "file";

export interface NotesWorkingMarkdownNode {
  kind: NotesWorkingMarkdownNodeKind;
  name: string;
  relativePath: string;
  children: NotesWorkingMarkdownNode[];
}

export interface NotesWorkingMarkdownRoot {
  workingFolderId: ProjectWorkingFolderId;
  displayName: string;
  sourceKind: "managed" | "external";
  displayPath: string;
  nodes: NotesWorkingMarkdownNode[];
  truncated: boolean;
}

export interface NotesWorkingMarkdownTreeRead {
  roots: NotesWorkingMarkdownRoot[];
  unavailableWorkingFolderIds: ProjectWorkingFolderId[];
}

export interface NotesWorkingMarkdownFileRef {
  workingFolderId: ProjectWorkingFolderId;
  workingFolderName: string;
  relativePath: string;
}

export interface NotesWorkingMarkdownFileRead {
  workingFolderId: ProjectWorkingFolderId;
  relativePath: string;
  content: string;
  revision: string;
  byteSize: number;
}
