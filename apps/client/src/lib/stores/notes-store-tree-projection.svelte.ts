import { flattenNotesBlockOutlines, notesBlockOutlineFromBlock, type NotesBlockOutlineItem } from "$lib/notes/block-outline";
import { buildNotesChildIdsByParent, parentIdForBlock, type NotesTreeState } from "$lib/notes/block-tree";
import type { NotesUndoSnapshot } from "$lib/notes/undo-history";
import { applyNotesPostMutationToTree, type NotesPostMutationResult } from "$lib/notes/post-mutation";
import type { NotesBlock, NotesBlockOutline, NotesLoadedPage, NotesPage } from "$lib/notes/types";
import { notesTreeState, notesTreeStateWithoutLeafBlock, type NotesBlockTreeSnapshot } from "./notes-store-block-tree";

export interface NotesTreeProjectionOptions {
  readSelectedPageId: () => string | null;
}

/** Owns the reactive projection of the currently loaded Notes block tree. */
export class NotesTreeProjectionController {
  loadedPage = $state<NotesPage | null>(null);
  blocksById = $state<Record<string, NotesBlock>>({});
  childIdsByParentId = $state<Record<string, string[]>>({});
  blockOutlines = $state<NotesBlockOutline[]>([]);
  flatBlockOutlines = $state<NotesBlockOutlineItem[]>([]);
  primaryContentReady = $state(false);

  private markLocallyChanged: (blockId: string) => void = () => {};

  constructor(private readonly options: NotesTreeProjectionOptions) {}

  setLocalChangeMarker(marker: (blockId: string) => void): void {
    this.markLocallyChanged = marker;
  }

  snapshot(): NotesBlockTreeSnapshot {
    return {
      selectedPageId: this.options.readSelectedPageId(),
      blocksById: this.blocksById,
      childIdsByParentId: this.childIdsByParentId,
    };
  }

  treeState(): NotesTreeState {
    return notesTreeState(this.snapshot());
  }

  replaceBlock(block: NotesBlock): void {
    this.blocksById = { ...this.blocksById, [block.id]: block };
  }

  applyLocalUndoSnapshot(target: NotesUndoSnapshot, source: NotesUndoSnapshot): void {
    const targetIds = new Set(target.blocks.map((block) => block.id));
    const sourceIds = new Set(source.blocks.map((block) => block.id));
    const affectedIds = new Set([...targetIds, ...sourceIds]);
    const nextBlocksById = { ...this.blocksById };
    for (const blockId of sourceIds) {
      if (!targetIds.has(blockId)) delete nextBlocksById[blockId];
    }
    for (const block of target.blocks) nextBlocksById[block.id] = block;
    for (const blockId of affectedIds) this.markLocallyChanged(blockId);
    this.blocksById = nextBlocksById;
    this.childIdsByParentId = buildNotesChildIdsByParent(Object.values(nextBlocksById));
  }

  insertBlockAfter(block: NotesBlock, afterBlockId: string | null): void {
    const parentId = parentIdForBlock(block);
    const current = (this.childIdsByParentId[parentId] ?? []).filter((id) => id !== block.id);
    const afterIndex = afterBlockId ? current.indexOf(afterBlockId) : -1;
    const insertIndex = afterIndex >= 0 ? afterIndex + 1 : current.length;
    this.childIdsByParentId = {
      ...this.childIdsByParentId,
      [parentId]: [...current.slice(0, insertIndex), block.id, ...current.slice(insertIndex)],
    };
    this.replaceBlock(block);
  }

  removeLeafBlock(blockId: string): boolean {
    const next = notesTreeStateWithoutLeafBlock(this.treeState(), blockId);
    if (!next) return false;
    this.markLocallyChanged(blockId);
    this.blocksById = next.blocksById;
    this.childIdsByParentId = next.childIdsByParentId;
    return true;
  }

  setLoadedPage(loaded: NotesLoadedPage): void {
    this.loadedPage = loaded.page;
    const blocks = loaded.blocks.results;
    this.blocksById = Object.fromEntries(blocks.map((block) => [block.id, block]));
    this.childIdsByParentId = buildNotesChildIdsByParent(blocks);
    this.primaryContentReady = true;
  }

  replaceOutlines(outlines: readonly NotesBlockOutline[], pageId: string): void {
    this.blockOutlines = [...outlines];
    this.flatBlockOutlines = flattenNotesBlockOutlines(this.blockOutlines, pageId);
  }

  mergeOutlines(outlines: readonly NotesBlockOutline[], pageId: string): void {
    const next = new Map(this.blockOutlines.map((outline) => [outline.id, outline]));
    for (const outline of outlines) next.set(outline.id, outline);
    this.replaceOutlines([...next.values()], pageId);
  }

  syncHydratedOutlines(pageId: string): void {
    const next = new Map(this.blockOutlines.map((outline) => [outline.id, outline]));
    for (const [parentId, childIds] of Object.entries(this.childIdsByParentId)) {
      childIds.forEach((blockId, index) => {
        const block = this.blocksById[blockId];
        if (!block) return;
        const outline = notesBlockOutlineFromBlock(block, pageId, (index + 1) * 1_000);
        next.set(blockId, {
          ...outline,
          parent: parentId === pageId
            ? { type: "page_id", page_id: pageId }
            : { type: "block_id", block_id: parentId },
        });
      });
    }
    this.replaceOutlines([...next.values()], pageId);
  }

  applyPostMutation(result: NotesPostMutationResult): void {
    if (result.loadedPage !== undefined) {
      if (result.loadedPage) {
        this.setLoadedPage(result.loadedPage);
        this.blockOutlines = [];
        this.syncHydratedOutlines(result.loadedPage.page.id);
      } else {
        this.clearLoadedTree();
      }
    }
    if (result.blocks || result.placements || result.removedBlockIds) {
      const next = applyNotesPostMutationToTree(this.treeState(), result);
      this.blocksById = { ...next.blocksById };
      this.childIdsByParentId = Object.fromEntries(
        Object.entries(next.childIdsByParentId).map(([parentId, childIds]) => [parentId, [...childIds]]),
      );
      for (const block of result.blocks ?? []) this.markLocallyChanged(block.id);
      for (const blockId of result.removedBlockIds ?? []) this.markLocallyChanged(blockId);
    }
    if (this.loadedPage) {
      const returned = result.pages?.find((page) => page.id === this.loadedPage?.id);
      if (returned) this.loadedPage = returned;
      if (result.blocks || result.placements || result.removedBlockIds) {
        const removed = new Set(result.removedBlockIds ?? []);
        this.blockOutlines = this.blockOutlines.filter((outline) => !removed.has(outline.id));
        this.syncHydratedOutlines(this.loadedPage.id);
      }
    }
  }

  resetOutlines(): void {
    this.blockOutlines = [];
    this.flatBlockOutlines = [];
  }

  clearLoadedTree(): void {
    this.loadedPage = null;
    this.blocksById = {};
    this.childIdsByParentId = {};
  }

  clearSelection(): void {
    this.clearLoadedTree();
    this.resetOutlines();
    this.primaryContentReady = false;
  }
}
