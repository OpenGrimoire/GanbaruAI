import type { MessageCatalog } from "../en";
import type { MessageShape } from "../types";
import { benchmark } from "./benchmark";
import { calendar } from "./calendar";
import { common } from "./common";
import { diagnostics } from "./diagnostics";
import { focusDialog, pomodoroOverlay } from "./focus";
import { format } from "./format";
import { music } from "./music";
import { notes } from "./notes";
import { projects } from "./projects";
import { settings } from "./settings";
import { theme } from "./theme";
import { titleBar } from "./title-bar";
import { updates } from "./updates";
import { dataFolderError, language, vaultSetup } from "./vault";
import { window } from "./window";

export const es = {
  common,
  window,
  vaultSetup,
  dataFolderError,
  language,
  focusDialog,
  pomodoroOverlay,
  calendar,
  settings,
  updates,
  diagnostics,
  benchmark,
  notes,
  projects,
  music,
  titleBar,
  format,
  theme,
} as const satisfies MessageShape<MessageCatalog>;
