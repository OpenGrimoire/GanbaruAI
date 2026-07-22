import type {
  ChatChangedFileRead,
  ChatInspectorTab,
  ChatThreadId,
} from "./contracts";

export interface ChatInspectorThreadState {
  tab: ChatInspectorTab;
  openTabs: ChatInspectorTab[];
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
  tab: "files",
  openTabs: ["files"],
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

  constructor(private readonly initialTab: ChatInspectorTab = "files") {}

  private initialState(): ChatInspectorThreadState {
    return {
      ...DEFAULT_STATE,
      tab: this.initialTab,
      openTabs: [this.initialTab],
      fileTreeVisible: this.initialTab === "files",
    };
  }

  read(threadId: ChatThreadId | null): ChatInspectorThreadState {
    const state = threadId ? this.threads.get(threadId) ?? this.initialState() : this.initialState();
    return { ...state, openTabs: [...state.openTabs] };
  }

  update(threadId: ChatThreadId, update: Partial<ChatInspectorThreadState>): ChatInspectorThreadState {
    const current = this.read(threadId);
    const next = {
      ...current,
      ...update,
      openTabs: [...(update.openTabs ?? current.openTabs)],
    };
    this.threads.set(threadId, next);
    return { ...next, openTabs: [...next.openTabs] };
  }
}

export const chatInspectorSession = new ChatInspectorSessionState();

/**
 * Opens a workspace panel without duplicating an existing tab.
 *
 * @param tabs Currently open panel tabs.
 * @param tab Panel to open or select.
 * @returns Tabs with the requested panel present once.
 */
export function openInspectorTab(
  tabs: readonly ChatInspectorTab[],
  tab: ChatInspectorTab,
): ChatInspectorTab[] {
  return tabs.includes(tab) ? [...tabs] : [...tabs, tab];
}

/**
 * Closes a workspace panel and selects its nearest remaining neighbor.
 *
 * @param tabs Currently open panel tabs.
 * @param tab Panel to close.
 * @param selectedTab Currently selected panel.
 * @returns Remaining tabs and the next selected panel, if one exists.
 */
export function closeInspectorTab(
  tabs: readonly ChatInspectorTab[],
  tab: ChatInspectorTab,
  selectedTab: ChatInspectorTab,
): { tabs: ChatInspectorTab[]; selectedTab: ChatInspectorTab | null } {
  const closedIndex = tabs.indexOf(tab);
  const nextTabs = tabs.filter((entry) => entry !== tab);
  if (selectedTab !== tab) return { tabs: nextTabs, selectedTab };
  if (nextTabs.length === 0) return { tabs: [], selectedTab: null };
  return {
    tabs: nextTabs,
    selectedTab: nextTabs[Math.min(Math.max(0, closedIndex), nextTabs.length - 1)] ?? null,
  };
}

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
