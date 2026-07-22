import type {
  ChatChangedFileRead,
  ChatInspectorTab,
  ChatThreadId,
} from "./contracts";

export interface ChatInspectorThreadState {
  tab: ChatInspectorTab;
  selectedFile: string | null;
  fileBrowserPath: string;
  filePreviewPath: string | null;
  fileTreeVisible: boolean;
  changeScope: "current_turn" | "entire_thread";
  whitespaceIgnored: boolean;
  diffView: "auto" | "unified" | "split";
  maximized: boolean;
}

export interface ChatChangedFileTreeNode {
  name: string;
  relativePath: string;
  kind: "directory" | "file";
  file: ChatChangedFileRead | null;
  children: ChatChangedFileTreeNode[];
}

const DEFAULT_STATE: ChatInspectorThreadState = {
  tab: "changes",
  selectedFile: null,
  fileBrowserPath: "",
  filePreviewPath: null,
  fileTreeVisible: true,
  changeScope: "current_turn",
  whitespaceIgnored: false,
  diffView: "auto",
  maximized: false,
};

export class ChatInspectorSessionState {
  private readonly threads = new Map<ChatThreadId, ChatInspectorThreadState>();

  read(threadId: ChatThreadId | null): ChatInspectorThreadState {
    if (!threadId) return { ...DEFAULT_STATE };
    return { ...(this.threads.get(threadId) ?? DEFAULT_STATE) };
  }

  update(threadId: ChatThreadId, update: Partial<ChatInspectorThreadState>): ChatInspectorThreadState {
    const next = { ...this.read(threadId), ...update };
    this.threads.set(threadId, next);
    return { ...next };
  }
}

export const chatInspectorSession = new ChatInspectorSessionState();

export function inspectorSessionKey(
  threadId: ChatThreadId | null,
  workspaceId: string | null,
): string | null {
  if (threadId) return threadId;
  return workspaceId ? `draft:${workspaceId}` : null;
}

export function buildChangedFileTree(files: readonly ChatChangedFileRead[]): ChatChangedFileTreeNode[] {
  const root: ChatChangedFileTreeNode = {
    name: "",
    relativePath: "",
    kind: "directory",
    file: null,
    children: [],
  };
  for (const file of [...files].sort((left, right) => left.relativePath.localeCompare(right.relativePath))) {
    const parts = file.relativePath.split("/").filter(Boolean);
    let parent = root;
    parts.forEach((part, index) => {
      const relativePath = parts.slice(0, index + 1).join("/");
      const isFile = index === parts.length - 1;
      let child = parent.children.find((entry) => entry.name === part && entry.kind === (isFile ? "file" : "directory"));
      if (!child) {
        child = {
          name: part,
          relativePath,
          kind: isFile ? "file" : "directory",
          file: isFile ? file : null,
          children: [],
        };
        parent.children.push(child);
      }
      parent = child;
    });
  }
  return root.children;
}

export function splitDiffFits(availableWidth: number, fontScale = 1): boolean {
  return availableWidth >= 720 * Math.max(1, fontScale);
}

export type ChatInspectorPresentation = "column" | "sheet" | "full";

export function inspectorPresentation(
  containerWidth: number,
  maximized: boolean,
): ChatInspectorPresentation {
  if (maximized) return "full";
  return containerWidth < 920 ? "sheet" : "column";
}

export function inspectorFocusAction(
  wasOpen: boolean,
  open: boolean,
): "enter" | "restore" | "none" {
  if (open && !wasOpen) return "enter";
  if (!open && wasOpen) return "restore";
  return "none";
}
