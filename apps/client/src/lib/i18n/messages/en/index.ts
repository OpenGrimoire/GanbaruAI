import { benchmark } from "./benchmark";
import { calendar } from "./calendar";
import { chat } from "./chat";
import { common } from "./common";
import { diagnostics } from "./diagnostics";
import { focusDialog, pomodoroOverlay } from "./focus";
import { format } from "./format";
import { music } from "./music";
import { notes } from "./notes";
import { projects } from "./projects";
import { quickNotes } from "./quick-notes";
import { settings } from "./settings";
import { theme } from "./theme";
import { titleBar } from "./title-bar";
import { updates } from "./updates";
import { dataFolderError, language, vaultSetup } from "./vault";
import { window } from "./window";

export const en = {
  common,
  window,
  vaultSetup,
  dataFolderError,
  language,
  focusDialog,
  pomodoroOverlay,
  calendar,
  chat,
  settings,
  updates,
  diagnostics,
  benchmark,
  notes,
  projects,
  quickNotes,
  music,
  titleBar,
  format,
  theme,
} as const;

export type MessageCatalog = typeof en;
