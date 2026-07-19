import { describe, expect, it } from "vitest";
import { projectMusicBuilderLayout } from "./music-builder-layout";

describe("music builder layout projection", () => {
  it("uses three panes only when the contextual panel, list, and inspector fit", () => {
    expect(projectMusicBuilderLayout({ width: 1200, height: 680 })).toMatchObject({
      mode: "wide",
      inspectorPresentation: "persistent",
      contextPanelPresentation: "persistent",
      dockPresentation: "sidebar",
      comfortable: true,
    });
  });

  it("moves the inspector to an overlay before the workspace stops fitting", () => {
    expect(projectMusicBuilderLayout({ width: 700, height: 420 })).toMatchObject({
      mode: "medium",
      inspectorPresentation: "overlay",
      contextPanelPresentation: "persistent",
      dockPresentation: "sidebar",
    });
    expect(projectMusicBuilderLayout({ width: 1000, height: 280 }).mode).toBe("narrow");
  });

  it("keeps a recoverable page projection at the app floor", () => {
    expect(projectMusicBuilderLayout({ width: 280, height: 180 })).toEqual({
      mode: "narrow",
      inspectorPresentation: "page",
      contextPanelPresentation: "sheet",
      dockPresentation: "bottom",
      comfortable: false,
    });
  });
});
