import { shell } from "./shell";
import { views } from "./views";
import { configuration } from "./configuration";
import { taskWorkflow } from "./task-workflow";

export const projects = {
  ...shell,
  ...views,
  ...configuration,
  ...taskWorkflow,
} as const;
