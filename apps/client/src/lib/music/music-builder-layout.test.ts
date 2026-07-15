import { describe, expect, it } from "vitest";
import { projectMusicBuilderLayout } from "./music-builder-layout";

describe("music builder layout projection", () => {
  it("uses the persistent-inspector workspace at the normal panel size", () => {
    expect(projectMusicBuilderLayout({ width: 1000, height: 680 })).toMatchObject({
      mode: "wide",
      inspectorPresentation: "persistent",
      comfortable: true,
    });
  });

  it("moves the inspector to an overlay before the workspace stops fitting", () => {
    expect(projectMusicBuilderLayout({ width: 700, height: 420 })).toMatchObject({
      mode: "medium",
      inspectorPresentation: "overlay",
    });
    expect(projectMusicBuilderLayout({ width: 1000, height: 280 }).mode).toBe("narrow");
  });

  it("keeps a recoverable page projection at the app floor", () => {
    expect(projectMusicBuilderLayout({ width: 280, height: 180 })).toEqual({
      mode: "narrow",
      inspectorPresentation: "page",
      comfortable: false,
    });
  });
});
