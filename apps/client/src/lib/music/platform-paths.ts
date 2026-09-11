/** Resolve a persisted local Music root and relative path on desktop. */
export function resolveLocalMusicPath(root: string, relativePath: string): string {
  const separator = root.includes("\\") && !root.includes("/") ? "\\" : "/";
  return `${root.replace(/[\\/]+$/u, "")}${separator}${relativePath.replace(/[\\/]+/gu, separator)}`;
}
