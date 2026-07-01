import { buildNotesPageTree } from "./page-tree";
import { notesPageTitle } from "./page-title";
import type { NotesPage, NotesParent } from "./types";

export interface NotesPageMoveTarget {
  parent: NotesParent;
  title: string;
  depth: number;
}

/** Return valid page move destinations for the source page. */
export function notesPageMoveTargets(
  pages: readonly NotesPage[],
  sourcePageId: string,
  workspaceTitle: string,
  titleForPage: (page: NotesPage) => string = notesPageTitle,
): NotesPageMoveTarget[] {
  const sourcePage = pages.find((page) => page.id === sourcePageId);
  if (!sourcePage) return [];
  const descendantIds = descendantPageIds(pages, sourcePageId);
  const targets: NotesPageMoveTarget[] = [];
  if (sourcePage.parent.type !== "workspace") {
    targets.push({
      parent: { type: "workspace", workspace: true },
      title: workspaceTitle,
      depth: 0,
    });
  }
  const tree = buildNotesPageTree(
    pages.filter((page) => page.id !== sourcePageId && !descendantIds.has(page.id)),
    { titleForPage },
  );
  for (const item of tree) {
    if (sourcePage.parent.type === "page_id" && sourcePage.parent.page_id === item.page.id) {
      continue;
    }
    targets.push({
      parent: { type: "page_id", page_id: item.page.id },
      title: titleForPage(item.page),
      depth: item.depth,
    });
  }
  return targets;
}

function descendantPageIds(pages: readonly NotesPage[], sourcePageId: string): Set<string> {
  const childrenByParentId = new Map<string, NotesPage[]>();
  for (const page of pages) {
    if (page.parent.type !== "page_id") continue;
    const children = childrenByParentId.get(page.parent.page_id) ?? [];
    children.push(page);
    childrenByParentId.set(page.parent.page_id, children);
  }
  const descendants = new Set<string>();
  const queue = [...(childrenByParentId.get(sourcePageId) ?? [])];
  while (queue.length > 0) {
    const page = queue.shift();
    if (!page || descendants.has(page.id)) continue;
    descendants.add(page.id);
    queue.push(...(childrenByParentId.get(page.id) ?? []));
  }
  return descendants;
}
