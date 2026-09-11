const ANDROID_SAF_SCHEME = "ganbaru-saf:";

/** Resolve a persisted Android document-tree root into a Media3 locator. */
export function resolveLocalMusicPath(root: string, relativePath: string): string {
  const normalizedRelativePath = relativePath.replaceAll("\\", "/").replace(/^\/+|\/+$/gu, "");
  if (!root.startsWith("content://") || normalizedRelativePath.length === 0) {
    throw new Error("Android local music requires a selected folder and relative file path");
  }
  return `${ANDROID_SAF_SCHEME}${encodeURIComponent(root)}#${encodeURIComponent(normalizedRelativePath)}`;
}
