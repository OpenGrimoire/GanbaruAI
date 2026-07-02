import type { format as enFormat } from "../en/format";
import type { MessageShape } from "../types";

export const format = {
  relativeMinutesNow: "Ahora",
  relativeMinutesFuture: (count: number) => `en ${count} min`,
  relativeMinutesPast: (count: number) => `hace ${count} min`,
} as const satisfies MessageShape<typeof enFormat>;
