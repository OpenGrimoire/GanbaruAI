import { describe, expect, it } from "vitest";
import {
  buildChangedFileTree,
  ChatInspectorSessionState,
  inspectorFocusAction,
  inspectorPresentation,
  inspectorSessionKey,
  splitDiffFits,
} from "./inspector-model";

describe("Chat inspector model", () => {
  it("preserves independent per-thread session state", () => {
    const state = new ChatInspectorSessionState();
    state.update("thread-a", { tab: "terminal", selectedFile: "src/a.ts" });
    state.update("thread-b", { tab: "files" });
    expect(state.read("thread-a").tab).toBe("terminal");
    expect(state.read("thread-a").selectedFile).toBe("src/a.ts");
    expect(state.read("thread-b").tab).toBe("files");
  });

  it("keeps inspector state available for a workspace draft", () => {
    expect(inspectorSessionKey(null, "workspace-a")).toBe("draft:workspace-a");
    expect(inspectorSessionKey("thread-a", "workspace-a")).toBe("thread-a");
    expect(inspectorSessionKey(null, null)).toBeNull();
  });

  it("derives a stable changed-file tree including renames", () => {
    const tree = buildChangedFileTree([
      {
        relativePath: "src/new.ts",
        previousRelativePath: "src/old.ts",
        status: "renamed",
        additions: 1,
        deletions: 1,
        binary: false,
        providerReported: true,
        gitObserved: true,
      },
      {
        relativePath: "README.md",
        previousRelativePath: null,
        status: "modified",
        additions: 2,
        deletions: 0,
        binary: false,
        providerReported: false,
        gitObserved: true,
      },
    ]);
    expect(tree.map((node) => node.name)).toEqual(["README.md", "src"]);
    expect(tree[1].children[0].file?.previousRelativePath).toBe("src/old.ts");
  });

  it("uses a fit-based split diff decision", () => {
    expect(splitDiffFits(900)).toBe(true);
    expect(splitDiffFits(700)).toBe(false);
    expect(splitDiffFits(900, 1.5)).toBe(false);
  });

  it("uses a sheet when a third column does not fit and preserves full maximize", () => {
    expect(inspectorPresentation(1_200, false)).toBe("column");
    expect(inspectorPresentation(700, false)).toBe("sheet");
    expect(inspectorPresentation(1_200, true)).toBe("full");
  });

  it("enters and restores focus only across inspector visibility boundaries", () => {
    expect(inspectorFocusAction(false, true)).toBe("enter");
    expect(inspectorFocusAction(true, false)).toBe("restore");
    expect(inspectorFocusAction(true, true)).toBe("none");
  });
});
