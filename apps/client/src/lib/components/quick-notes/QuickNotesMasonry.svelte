<script lang="ts">
  import { onMount } from "svelte";
  import type { QuickNote, QuickNotesCollection, QuickNoteTag } from "$lib/quick-notes/types";
  import { quickNoteMasonryLayout, type MasonryPosition } from "$lib/quick-notes/masonry";
  import type { Theme } from "$lib/stores/themes";
  import QuickNoteCard from "./QuickNoteCard.svelte";

  let {
    notes,
    collection,
    theme,
    tags,
    onopen,
    onpin,
    oncolor,
    ontag,
    onarchive,
    onunarchive,
    ontrash,
    onrestore,
    ondelete,
  }: {
    notes: readonly QuickNote[];
    collection: QuickNotesCollection;
    theme: Theme;
    tags: readonly QuickNoteTag[];
    onopen: (note: QuickNote) => void;
    onpin: (note: QuickNote, pinned: boolean) => void;
    oncolor: (note: QuickNote, color: QuickNote["color"]) => void;
    ontag: (note: QuickNote, tagId: string | null) => void;
    onarchive: (note: QuickNote) => void;
    onunarchive: (note: QuickNote) => void;
    ontrash: (note: QuickNote) => void;
    onrestore: (note: QuickNote) => void;
    ondelete: (note: QuickNote) => void;
  } = $props();

  let container = $state<HTMLDivElement | null>(null);
  let width = $state(0);
  let positions = $state<Record<string, MasonryPosition>>({});
  let layoutHeight = $state(0);
  const heights = new Map<string, number>();
  const observers = new Map<string, ResizeObserver>();
  let frame: number | null = null;

  function scheduleLayout(): void {
    if (frame !== null) cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      frame = null;
      const layout = quickNoteMasonryLayout(width, notes.map((note) => heights.get(note.id) ?? 120));
      positions = Object.fromEntries(notes.map((note, index) => [note.id, layout.positions[index]]));
      layoutHeight = layout.height;
    });
  }

  function measure(node: HTMLElement, noteId: string): { update: (id: string) => void; destroy: () => void } {
    let id = noteId;
    const observer = new ResizeObserver(() => {
      heights.set(id, node.getBoundingClientRect().height);
      scheduleLayout();
    });
    observer.observe(node);
    observers.set(id, observer);
    heights.set(id, node.getBoundingClientRect().height);
    scheduleLayout();
    return {
      update(nextId: string) {
        observers.delete(id);
        heights.delete(id);
        id = nextId;
        observers.set(id, observer);
        heights.set(id, node.getBoundingClientRect().height);
        scheduleLayout();
      },
      destroy() {
        observer.disconnect();
        observers.delete(id);
        heights.delete(id);
      },
    };
  }

  onMount(() => {
    if (!container) return;
    const observer = new ResizeObserver(([entry]) => {
      width = entry?.contentRect.width ?? container?.clientWidth ?? 0;
      scheduleLayout();
    });
    observer.observe(container);
    width = container.clientWidth;
    scheduleLayout();
    return () => {
      observer.disconnect();
      for (const cardObserver of observers.values()) cardObserver.disconnect();
      if (frame !== null) cancelAnimationFrame(frame);
    };
  });

  $effect(() => {
    void notes.map((note) => note.id).join("|");
    scheduleLayout();
  });
</script>

<div bind:this={container} class="relative w-full" style="height: {layoutHeight}px;" role="list">
  {#each notes as note (note.id)}
    {@const position = positions[note.id]}
    <div
      use:measure={note.id}
      class="absolute left-0 top-0 will-change-transform motion-safe:transition-transform motion-safe:duration-150"
      class:opacity-0={!position}
      style={position ? `width: ${position.width}px; transform: translate(${position.left}px, ${position.top}px);` : "width: 210px;"}
      role="listitem"
    >
      <QuickNoteCard
        {note}
        {collection}
        {theme}
        {tags}
        onopen={() => onopen(note)}
        onpin={(pinned) => onpin(note, pinned)}
        oncolor={(color) => oncolor(note, color)}
        ontag={(tagId) => ontag(note, tagId)}
        onarchive={() => onarchive(note)}
        onunarchive={() => onunarchive(note)}
        ontrash={() => ontrash(note)}
        onrestore={() => onrestore(note)}
        ondelete={() => ondelete(note)}
      />
    </div>
  {/each}
</div>
