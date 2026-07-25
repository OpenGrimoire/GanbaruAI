import type {
  ChatChangedFileRead,
  ChatInspectorTab,
  ChatThreadId,
} from "./contracts";

export interface ChatInspectorThreadState {
  tab: ChatInspectorTab;
  openTabs: ChatInspectorTab[];
  tabOrder: ChatWorkspacePanelTabKey[];
  selectedFile: string | null;
  fileBrowserPath: string;
  filePreviewPath: string | null;
  fileTreeVisible: boolean;
  fileTreeWidthPx: number;
  changeScope: "current_turn" | "entire_thread";
  changedFileListHeightPx: number;
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
  tabOrder: ["files"],
  selectedFile: null,
  fileBrowserPath: "",
  filePreviewPath: null,
  fileTreeVisible: true,
  fileTreeWidthPx: 220,
  changeScope: "current_turn",
  changedFileListHeightPx: 160,
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
      tabOrder: [this.initialTab],
      fileTreeVisible: this.initialTab === "files",
    };
  }

  read(threadId: ChatThreadId | null): ChatInspectorThreadState {
    const state = threadId ? this.threads.get(threadId) ?? this.initialState() : this.initialState();
    return { ...state, openTabs: [...state.openTabs], tabOrder: [...state.tabOrder] };
  }

  update(threadId: ChatThreadId, update: Partial<ChatInspectorThreadState>): ChatInspectorThreadState {
    const current = this.read(threadId);
    const next = {
      ...current,
      ...update,
      openTabs: [...(update.openTabs ?? current.openTabs)],
      tabOrder: [...(update.tabOrder ?? current.tabOrder)],
    };
    this.threads.set(threadId, next);
    return { ...next, openTabs: [...next.openTabs], tabOrder: [...next.tabOrder] };
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

export type ChatWorkspacePanelTabKey = ChatInspectorTab | `terminal:${string}`;

/**
 * Creates the stable tab key used for one terminal session.
 *
 * @param terminalId Terminal session identifier.
 * @returns A key that cannot collide with a workspace tool tab.
 */
export function terminalWorkspacePanelTabKey(terminalId: string): `terminal:${string}` {
  return `terminal:${terminalId}`;
}

/**
 * Reads a terminal identifier from a workspace panel tab key.
 *
 * @param key Workspace panel tab key.
 * @returns The terminal identifier, or null for a tool tab and the loading placeholder.
 */
export function workspacePanelTerminalId(key: ChatWorkspacePanelTabKey): string | null {
  return key.startsWith("terminal:") ? key.slice("terminal:".length) : null;
}

/**
 * Resolves a stored tab order against the panels and terminal sessions that still exist.
 *
 * The generic terminal key acts as a loading placeholder. Once terminal sessions are
 * known, they replace that placeholder without moving the surrounding tool tabs.
 *
 * @param storedOrder Last user-defined physical tab order.
 * @param openTabs Open tool families in their fallback order.
 * @param terminalIds Terminal sessions owned by this panel.
 * @returns A complete, duplicate-free physical tab order.
 */
export function reconcileWorkspacePanelTabOrder(
  storedOrder: readonly ChatWorkspacePanelTabKey[],
  openTabs: readonly ChatInspectorTab[],
  terminalIds: readonly string[],
): ChatWorkspacePanelTabKey[] {
  const terminalKeys = terminalIds.map(terminalWorkspacePanelTabKey);
  const fallback = openTabs.flatMap<ChatWorkspacePanelTabKey>((tab) => (
    tab === "terminal" ? terminalKeys.length > 0 ? terminalKeys : ["terminal"] : [tab]
  ));
  const available = new Set(fallback);
  const resolved: ChatWorkspacePanelTabKey[] = [];
  const append = (key: ChatWorkspacePanelTabKey): void => {
    if (available.has(key) && !resolved.includes(key)) resolved.push(key);
  };

  for (const key of storedOrder) {
    if (key === "terminal" && terminalKeys.length > 0) terminalKeys.forEach(append);
    else append(key);
  }
  fallback.forEach(append);
  return resolved;
}

/**
 * Moves one physical workspace tab to an insertion index.
 *
 * @param tabs Current physical tab order.
 * @param tab Tab being moved.
 * @param insertionIndex Index in the order after removing the moving tab.
 * @returns The reordered tabs, or a copy of the original order for an unknown tab.
 */
export function moveWorkspacePanelTab(
  tabs: readonly ChatWorkspacePanelTabKey[],
  tab: ChatWorkspacePanelTabKey,
  insertionIndex: number,
): ChatWorkspacePanelTabKey[] {
  if (!tabs.includes(tab)) return [...tabs];
  const remaining = tabs.filter((entry) => entry !== tab);
  const boundedIndex = Math.min(Math.max(0, insertionIndex), remaining.length);
  remaining.splice(boundedIndex, 0, tab);
  return remaining;
}

/**
 * Finds the insertion point for a dragged tab among the remaining tab centers.
 *
 * @param draggedCenter Horizontal center of the dragged tab.
 * @param remainingCenters Ordered horizontal centers after removing the dragged tab.
 * @returns Insertion index in the remaining tab order.
 */
export function workspacePanelTabInsertionIndex(
  draggedCenter: number,
  remainingCenters: readonly number[],
): number {
  const index = remainingCenters.findIndex((center) => draggedCenter < center);
  return index < 0 ? remainingCenters.length : index;
}

/**
 * Calculates the temporary sibling displacement for an uncommitted tab drag.
 *
 * @param index Original tab index.
 * @param sourceIndex Original index of the dragged tab.
 * @param targetIndex Pending insertion index after removing the dragged tab.
 * @param sourceSpan Dragged tab width plus the tab-strip gap.
 * @returns Horizontal displacement in pixels.
 */
export function workspacePanelTabShift(
  index: number,
  sourceIndex: number,
  targetIndex: number,
  sourceSpan: number,
): number {
  if (targetIndex > sourceIndex && index > sourceIndex && index <= targetIndex) {
    return -sourceSpan;
  }
  if (targetIndex < sourceIndex && index >= targetIndex && index < sourceIndex) {
    return sourceSpan;
  }
  return 0;
}

/**
 * Collapses physical terminal tabs back into the panel families used for selection.
 *
 * @param tabs Physical workspace tab order.
 * @returns Unique panel families in first-appearance order.
 */
export function workspacePanelKinds(
  tabs: readonly ChatWorkspacePanelTabKey[],
): ChatInspectorTab[] {
  const kinds: ChatInspectorTab[] = [];
  for (const key of tabs) {
    const kind: ChatInspectorTab = key === "changes" || key === "plan" || key === "files"
      ? key
      : "terminal";
    if (!kinds.includes(kind)) kinds.push(kind);
  }
  return kinds;
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

export interface SplitPaneResizeBounds {
  minimum: number;
  maximum: number;
}

/**
 * Keeps both sides of an internal pane split reachable at constrained sizes.
 *
 * @param availableSize Total width or height available to the split.
 * @param primaryMinimum Comfortable minimum for the resizable first pane.
 * @param secondaryMinimum Reserved space for the second pane.
 * @param primaryMaximum Largest useful size for the first pane.
 * @returns Whole-pixel resize bounds adapted to the available space.
 */
export function splitPaneResizeBounds(
  availableSize: number,
  primaryMinimum: number,
  secondaryMinimum: number,
  primaryMaximum: number,
): SplitPaneResizeBounds {
  const available = Math.max(0, Math.floor(availableSize));
  const maximum = Math.max(0, Math.min(primaryMaximum, available - secondaryMinimum));
  return {
    minimum: Math.min(primaryMinimum, maximum),
    maximum,
  };
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
