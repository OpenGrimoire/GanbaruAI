import { doomscrolling } from "./doomscrolling";
import { focusAndShortcuts } from "./focus-and-shortcuts";
import { general } from "./general";
import { chatSettings } from "./chat";
import { theme } from "./theme";

export const settings = {
  ...general,
  theme,
  ...focusAndShortcuts,
  chat: chatSettings,
  doomscrolling,
} as const;
