import type { MusicItemListEntry } from "$lib/music/library-contracts";

export interface MusicReviewTreeNode {
  id: string;
  name: string;
  path: string;
  itemIds: string[];
  directItems: MusicItemListEntry[];
  children: MusicReviewTreeNode[];
}

export type MusicReviewTreeRow =
  | { kind: "folder"; depth: number; node: MusicReviewTreeNode }
  | { kind: "item"; depth: number; item: MusicItemListEntry };

interface MutableNode {
  id: string;
  name: string;
  path: string;
  directItems: MusicItemListEntry[];
  children: Map<string, MutableNode>;
}

function folder(id: string, name: string, path: string): MutableNode {
  return { id, name, path, directItems: [], children: new Map() };
}

function normalizedSegments(relativePath: string | null | undefined): string[] {
  if (!relativePath) return [];
  return relativePath.replaceAll("\\", "/").split("/").map((part) => part.trim()).filter(Boolean);
}

function freezeNode(node: MutableNode): MusicReviewTreeNode {
  const children = [...node.children.values()]
    .sort((left, right) => left.name.localeCompare(right.name, undefined, { sensitivity: "base" }))
    .map(freezeNode);
  const directItems = [...node.directItems].sort((left, right) => left.title.localeCompare(right.title, undefined, { sensitivity: "base" }));
  return {
    id: node.id,
    name: node.name,
    path: node.path,
    directItems,
    children,
    itemIds: [...directItems.map((item) => item.id), ...children.flatMap((child) => child.itemIds)],
  };
}

/** Projects review items into stable local-folder and online-source trees. */
export function buildMusicReviewTree(items: readonly MusicItemListEntry[]): MusicReviewTreeNode[] {
  const local = folder("review-root:local", "Music", "");
  const online = folder("review-root:online", "Online", "online");
  for (const item of items) {
    if (item.sourceKind === "youtube-video") {
      online.directItems.push(item);
      continue;
    }
    const segments = normalizedSegments(item.relativePath);
    segments.pop();
    let parent = local;
    let path = "";
    for (const segment of segments) {
      path = path ? `${path}/${segment}` : segment;
      let child = parent.children.get(segment);
      if (!child) {
        child = folder(`review-folder:${path}`, segment, path);
        parent.children.set(segment, child);
      }
      parent = child;
    }
    parent.directItems.push(item);
  }
  return [local, online].filter((node) => node.directItems.length > 0 || node.children.size > 0).map(freezeNode);
}

/** Flattens expanded tree nodes into keyboard-friendly visual rows. */
export function flattenMusicReviewTree(
  nodes: readonly MusicReviewTreeNode[],
  expandedIds: ReadonlySet<string>,
  depth = 0,
): MusicReviewTreeRow[] {
  const rows: MusicReviewTreeRow[] = [];
  for (const node of nodes) {
    rows.push({ kind: "folder", depth, node });
    if (!expandedIds.has(node.id)) continue;
    rows.push(...node.directItems.map((item) => ({ kind: "item" as const, depth: depth + 1, item })));
    rows.push(...flattenMusicReviewTree(node.children, expandedIds, depth + 1));
  }
  return rows;
}

/** Toggles every descendant of a folder while preserving unrelated selections. */
export function toggleMusicReviewTreeSelection(
  selectedIds: ReadonlySet<string>,
  itemIds: readonly string[],
): Set<string> {
  const next = new Set(selectedIds);
  const allSelected = itemIds.length > 0 && itemIds.every((itemId) => next.has(itemId));
  for (const itemId of itemIds) {
    if (allSelected) next.delete(itemId);
    else next.add(itemId);
  }
  return next;
}
