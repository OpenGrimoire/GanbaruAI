import type { ProjectWorkingFolderRead } from "$lib/chat/contracts";

/** Order project working folders with the protected managed folder first. */
export function orderProjectWorkingFolders(
  folders: readonly ProjectWorkingFolderRead[],
): ProjectWorkingFolderRead[] {
  return [...folders].sort((left, right) => {
    const managed = Number(right.workingFolder.kind === "managed")
      - Number(left.workingFolder.kind === "managed");
    return managed
      || left.workingFolder.sortOrder - right.workingFolder.sortOrder
      || left.workingFolder.displayName.localeCompare(right.workingFolder.displayName);
  });
}
