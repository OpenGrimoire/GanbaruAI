export type ProjectAppIconNode = readonly [string, Readonly<Record<string, string>>];

export const SPORT_SHOE_ICON_NODE = [
  ["path", { d: "M3 15.5h18v-1a3 3 0 0 0-3-3h-1.4a4 4 0 0 1-2.7-1.05L10.2 7a1 1 0 0 0-1.65.28L6.2 12H4a2 2 0 0 0-2 2v.5a1 1 0 0 0 1 1Z" }],
  ["path", { d: "M3 15.5V18a1 1 0 0 0 1 1h16a1 1 0 0 0 1-1v-2.5" }],
  ["path", { d: "m10.5 9 2-1.5" }],
  ["path", { d: "m12 10.5 2-1.5" }],
] as const satisfies readonly ProjectAppIconNode[];

const PROJECT_APP_ICON_NODES: Readonly<Record<string, readonly ProjectAppIconNode[]>> = {
  "sport-shoe": SPORT_SHOE_ICON_NODE,
};

export function projectAppIconNode(slug: string): readonly ProjectAppIconNode[] | null {
  return PROJECT_APP_ICON_NODES[slug] ?? null;
}
