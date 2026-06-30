export interface DateRangeSelectionInput {
  selectedDate: string;
  startDate?: string;
  endDate?: string;
  fillMissingStartDate?: boolean;
  fillMissingEndDate?: boolean;
}

export interface DateRangeSelection {
  startDate?: string;
  endDate?: string;
}

export interface NormalizedDateRange {
  startDate: string;
  endDate: string;
}

function isIsoDateString(value: string | undefined): value is string {
  return typeof value === "string" && /^\d{4}-\d{2}-\d{2}$/.test(value);
}

/**
 * Selects a new start endpoint and moves the end endpoint only when crossed.
 */
export function selectDateRangeStart(input: DateRangeSelectionInput): DateRangeSelection {
  const startDate = input.selectedDate;
  const endDate = input.endDate;
  if (endDate && startDate > endDate) {
    return { startDate, endDate: startDate };
  }
  if (!endDate && input.fillMissingEndDate) {
    return { startDate, endDate: startDate };
  }
  return { startDate, endDate };
}

/**
 * Selects a new end endpoint and moves the start endpoint only when crossed.
 */
export function selectDateRangeEnd(input: DateRangeSelectionInput): DateRangeSelection {
  const startDate = input.startDate;
  const endDate = input.selectedDate;
  if (startDate && endDate < startDate) {
    return { startDate: endDate, endDate };
  }
  if (!startDate && input.fillMissingStartDate) {
    return { startDate: endDate, endDate };
  }
  return { startDate, endDate };
}

/**
 * Returns a display-ready date range when both endpoints are valid date strings.
 */
export function normalizedDateRange(
  startDate: string | undefined,
  endDate: string | undefined,
): NormalizedDateRange | undefined {
  if (!isIsoDateString(startDate) || !isIsoDateString(endDate)) return undefined;
  return startDate <= endDate
    ? { startDate, endDate }
    : { startDate: endDate, endDate: startDate };
}
