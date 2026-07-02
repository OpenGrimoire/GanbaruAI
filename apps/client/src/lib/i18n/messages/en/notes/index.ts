import { navigationAndSearch } from "./navigation-and-search";
import { collaboration } from "./collaboration";
import { templatesHistory } from "./templates-history";
import { assets } from "./assets";
import { editor } from "./editor";
import { database } from "./database";
import { advancedBlocks } from "./advanced-blocks";
import { pageActions } from "./page-actions";

export const notes = {
  ...navigationAndSearch,
  ...collaboration,
  ...templatesHistory,
  ...assets,
  ...editor,
  ...database,
  ...advancedBlocks,
  ...pageActions,
} as const;
