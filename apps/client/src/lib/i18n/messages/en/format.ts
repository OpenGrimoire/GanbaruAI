export const format = {
  relativeMinutesNow: "Now",
  relativeMinutesFuture: (count: number) => `in ${count} min`,
  relativeMinutesPast: (count: number) => `${count} min ago`,
} as const;
