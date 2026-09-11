import { notesPageTitle } from "./page-title";
import type { NotesFolder, NotesPage } from "./types";

export type NotesHierarchyParent =
  | { kind: "root" }
  | { kind: "folder"; id: string }
  | { kind: "page"; id: string };

export type NotesHierarchyNode =
  | { kind: "folder"; key: string; folder: NotesFolder; hasChildren: boolean }
  | { kind: "page"; key: string; page: NotesPage; hasChildren: boolean };

function pageBelongsToParent(page: NotesPage, parent: NotesHierarchyParent): boolean {
  if (parent.kind === "page") {
    return page.parent.type === "page_id" && page.parent.page_id === parent.id;
  }
  if (page.parent.type !== "workspace") return false;
  if (parent.kind === "folder") return page.folder_id === parent.id;
  return page.folder_id === null;
}

function folderBelongsToParent(folder: NotesFolder, parent: NotesHierarchyParent): boolean {
  if (parent.kind === "page") return false;
  if (parent.kind === "folder") return folder.parent_folder_id === parent.id;
  return folder.parent_folder_id === null;
}

function nodeHasChildren(
  node: { kind: "folder"; id: string } | { kind: "page"; id: string },
  pages: readonly NotesPage[],
  pageIdsWithChildren: ReadonlySet<string>,
): boolean {
  if (node.kind === "folder") return true;
  if (pageIdsWithChildren.has(node.id)) return true;
  const parent: NotesHierarchyParent = { kind: "page", id: node.id };
  return pages.some((page) => pageBelongsToParent(page, parent));
}

/** Return only the direct folder and note children for one hierarchy level. */
export function notesHierarchyChildren(
  pages: readonly NotesPage[],
  folders: readonly NotesFolder[],
  parent: NotesHierarchyParent,
  untitledTitle = "Untitled",
  pageIdsWithChildren: readonly string[] = [],
): NotesHierarchyNode[] {
  const pageIdsWithChildrenSet = new Set(pageIdsWithChildren);
  const folderNodes: NotesHierarchyNode[] = folders
    .filter((folder) => folderBelongsToParent(folder, parent))
    .map((folder) => ({
      kind: "folder",
      key: `folder:${folder.id}`,
      folder,
      hasChildren: nodeHasChildren(
        { kind: "folder", id: folder.id },
        pages,
        pageIdsWithChildrenSet,
      ),
    }));
  const pageNodes: NotesHierarchyNode[] = pages
    .filter((page) => pageBelongsToParent(page, parent))
    .map((page) => ({
      kind: "page",
      key: `page:${page.id}`,
      page,
      hasChildren: nodeHasChildren(
        { kind: "page", id: page.id },
        pages,
        pageIdsWithChildrenSet,
      ),
    }));

  folderNodes.sort((left, right) => {
    if (left.kind !== "folder" || right.kind !== "folder") return 0;
    return left.folder.name.localeCompare(right.folder.name, undefined, {
      numeric: true,
      sensitivity: "base",
    });
  });
  pageNodes.sort((left, right) => {
    if (left.kind !== "page" || right.kind !== "page") return 0;
    return notesPageTitle(left.page, untitledTitle).localeCompare(
      notesPageTitle(right.page, untitledTitle),
      undefined,
      { numeric: true, sensitivity: "base" },
    );
  });
  return [...folderNodes, ...pageNodes];
}

/** Return the valid folder and note ancestry ending at a selected note. */
export function notesHierarchyPath(
  selectedPageId: string | null,
  pages: readonly NotesPage[],
  folders: readonly NotesFolder[],
  pageIdsWithChildren: readonly string[] = [],
): NotesHierarchyNode[] {
  if (!selectedPageId) return [];
  const pageById = new Map(pages.map((page) => [page.id, page]));
  const folderById = new Map(folders.map((folder) => [folder.id, folder]));
  const pagePath: NotesPage[] = [];
  const visitedPageIds = new Set<string>();
  let page = pageById.get(selectedPageId);

  while (page && !visitedPageIds.has(page.id)) {
    visitedPageIds.add(page.id);
    pagePath.unshift(page);
    page = page.parent.type === "page_id" ? pageById.get(page.parent.page_id) : undefined;
  }

  const folderPath: NotesFolder[] = [];
  const pageIdsWithChildrenSet = new Set(pageIdsWithChildren);
  const rootPage = pagePath[0];
  const visitedFolderIds = new Set<string>();
  let folder = rootPage?.parent.type === "workspace" && rootPage.folder_id
    ? folderById.get(rootPage.folder_id)
    : undefined;
  while (folder && !visitedFolderIds.has(folder.id)) {
    visitedFolderIds.add(folder.id);
    folderPath.unshift(folder);
    folder = folder.parent_folder_id ? folderById.get(folder.parent_folder_id) : undefined;
  }

  return [
    ...folderPath.map((item): NotesHierarchyNode => ({
      kind: "folder",
      key: `folder:${item.id}`,
      folder: item,
      hasChildren: nodeHasChildren(
        { kind: "folder", id: item.id },
        pages,
        pageIdsWithChildrenSet,
      ),
    })),
    ...pagePath.map((item): NotesHierarchyNode => ({
      kind: "page",
      key: `page:${item.id}`,
      page: item,
      hasChildren: nodeHasChildren(
        { kind: "page", id: item.id },
        pages,
        pageIdsWithChildrenSet,
      ),
    })),
  ];
}

/** Return the hierarchy level containing the provided folder or note. */
export function notesHierarchyNodeParent(node: NotesHierarchyNode): NotesHierarchyParent {
  if (node.kind === "folder") {
    return node.folder.parent_folder_id
      ? { kind: "folder", id: node.folder.parent_folder_id }
      : { kind: "root" };
  }
  if (node.page.parent.type === "page_id") {
    return { kind: "page", id: node.page.parent.page_id };
  }
  if (node.page.parent.type === "workspace" && node.page.folder_id) {
    return { kind: "folder", id: node.page.folder_id };
  }
  return { kind: "root" };
}

/** Return the folder containing a note or its root note ancestor. */
export function notesPageContainingFolderId(
  selectedPageId: string | null,
  pages: readonly NotesPage[],
): string | null {
  if (!selectedPageId) return null;
  const pageById = new Map(pages.map((page) => [page.id, page]));
  const visitedPageIds = new Set<string>();
  let page = pageById.get(selectedPageId);
  while (page && !visitedPageIds.has(page.id)) {
    visitedPageIds.add(page.id);
    if (page.parent.type !== "page_id") {
      return page.parent.type === "workspace" ? page.folder_id : null;
    }
    page = pageById.get(page.parent.page_id);
  }
  return null;
}
