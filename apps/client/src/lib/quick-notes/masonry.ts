export const QUICK_NOTE_MASONRY_MIN_CARD_WIDTH = 210;
export const QUICK_NOTE_MASONRY_GAP = 12;

export interface MasonryPosition {
  left: number;
  top: number;
  width: number;
}

export interface MasonryLayout {
  positions: MasonryPosition[];
  height: number;
  columns: number;
}

export function quickNoteMasonryLayout(
  containerWidth: number,
  heights: readonly number[],
  minimumCardWidth = QUICK_NOTE_MASONRY_MIN_CARD_WIDTH,
  gap = QUICK_NOTE_MASONRY_GAP,
): MasonryLayout {
  const safeWidth = Math.max(0, containerWidth);
  const columns = Math.max(1, Math.floor((safeWidth + gap) / (minimumCardWidth + gap)));
  const cardWidth = columns === 1 ? safeWidth : (safeWidth - gap * (columns - 1)) / columns;
  const columnHeights = Array.from({ length: columns }, () => 0);
  const positions = heights.map((height) => {
    let column = 0;
    for (let index = 1; index < columnHeights.length; index += 1) {
      if ((columnHeights[index] ?? 0) < (columnHeights[column] ?? 0)) column = index;
    }
    const top = columnHeights[column] ?? 0;
    columnHeights[column] = top + Math.max(0, height) + gap;
    return { left: column * (cardWidth + gap), top, width: cardWidth };
  });
  return {
    positions,
    height: Math.max(0, ...columnHeights) - (positions.length > 0 ? gap : 0),
    columns,
  };
}
