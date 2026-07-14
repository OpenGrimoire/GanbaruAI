export interface NotesInlineToolbarRect {
  top: number;
  right: number;
  bottom: number;
  left: number;
  width: number;
  height: number;
}

export interface NotesInlineToolbarViewport {
  width: number;
  height: number;
}

export type NotesInlineToolbarPlacement =
  | {
    mode: "floating";
    top: number;
    left: number;
    maxWidth: number;
  }
  | {
    mode: "docked";
    maxWidth: number;
  };

export function notesInlineToolbarWrapperClass(
  placement: NotesInlineToolbarPlacement | null,
): string {
  if (!placement) return "pointer-events-none fixed z-40 flex justify-center";
  return placement.mode === "floating"
    ? "fixed z-40 flex justify-center"
    : "mb-1 flex justify-end";
}

export function notesInlineToolbarWrapperStyle(
  placement: NotesInlineToolbarPlacement | null,
): string {
  if (!placement) return "visibility: hidden; left: 0px; top: 0px;";
  const maxWidth = `max-width: ${placement.maxWidth}px;`;
  if (placement.mode === "docked") return maxWidth;
  return `${maxWidth} left: ${placement.left}px; top: ${placement.top}px;`;
}

interface NotesInlineToolbarPlacementInput {
  selectionRect: NotesInlineToolbarRect;
  toolbarWidth: number;
  toolbarHeight: number;
  viewport: NotesInlineToolbarViewport;
  margin?: number;
  gap?: number;
  minFloatingViewportWidth?: number;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function planNotesInlineToolbarPlacement(
  input: NotesInlineToolbarPlacementInput,
): NotesInlineToolbarPlacement {
  const margin = input.margin ?? 8;
  const gap = input.gap ?? 6;
  const minFloatingViewportWidth = input.minFloatingViewportWidth ?? 360;
  const maxWidth = Math.max(0, input.viewport.width - margin * 2);
  if (
    input.viewport.width < minFloatingViewportWidth
    || input.toolbarHeight + margin * 2 > input.viewport.height
  ) {
    return { mode: "docked", maxWidth };
  }

  const toolbarWidth = Math.min(input.toolbarWidth, maxWidth);
  const maxLeft = Math.max(margin, input.viewport.width - margin - toolbarWidth);
  const selectionCenter = input.selectionRect.left + input.selectionRect.width / 2;
  const left = clamp(selectionCenter - toolbarWidth / 2, margin, maxLeft);
  const aboveTop = input.selectionRect.top - gap - input.toolbarHeight;
  const belowTop = input.selectionRect.bottom + gap;
  const top = aboveTop >= margin
    ? aboveTop
    : belowTop + input.toolbarHeight <= input.viewport.height - margin
      ? belowTop
      : clamp(aboveTop, margin, Math.max(margin, input.viewport.height - margin - input.toolbarHeight));

  return {
    mode: "floating",
    top,
    left,
    maxWidth,
  };
}

export function shouldPreventInlineToolbarPointerDefault(pointerType: string): boolean {
  return pointerType !== "mouse";
}
