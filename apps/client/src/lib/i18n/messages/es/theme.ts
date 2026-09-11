import type { theme as enTheme } from "../en/theme";
import type { MessageShape } from "../types";

export const theme = {
  builtInName: {
    light: "Claro predeterminado",
    dark: "Oscuro predeterminado",
  },
} as const satisfies MessageShape<typeof enTheme>;
