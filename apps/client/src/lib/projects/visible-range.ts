export interface ProjectVisibleRange {
  start: number;
  end: number;
  beforePx: number;
  afterPx: number;
}

/** Calculates a bounded half-open row range with symmetric overscan. */
export function projectVisibleRange(
  itemCount: number,
  scrollTop: number,
  viewportHeight: number,
  rowHeight: number,
  overscan: number,
): ProjectVisibleRange {
  if (itemCount <= 0) return { start: 0, end: 0, beforePx: 0, afterPx: 0 };
  const safeRowHeight = Math.max(1, rowHeight);
  const first = Math.floor(Math.max(0, scrollTop) / safeRowHeight);
  const visibleCount = Math.max(1, Math.ceil(Math.max(0, viewportHeight) / safeRowHeight));
  const start = Math.max(0, first - Math.max(0, overscan));
  const end = Math.min(itemCount, first + visibleCount + Math.max(0, overscan));
  return {
    start,
    end,
    beforePx: start * safeRowHeight,
    afterPx: (itemCount - end) * safeRowHeight,
  };
}
