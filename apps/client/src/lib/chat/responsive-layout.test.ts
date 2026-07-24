import { describe, expect, it } from "vitest";
import {
  alignPanelSizeToDevicePixel,
  chatLayoutDecision,
  chatLayoutPrimaryActions,
  chatInspectorResizeMaximum,
  chatScrollBehavior,
  clampPanelSizeToWholePixel,
  middleTruncate,
  panelWidthFromKey,
  preferredPanelWidth,
  type ChatLayoutVariant,
} from "./responsive-layout";

function layout(
  containerWidth: number,
  containerHeight = 700,
  previousVariant?: ChatLayoutVariant,
  fontScale = 1,
) {
  return chatLayoutDecision({
    containerWidth,
    containerHeight,
    fontScale,
    railOpen: true,
    inspectorOpen: true,
    railWidth: 260,
    inspectorWidth: 520,
    previousVariant,
  });
}

describe("Chat responsive layout", () => {
  it("selects all fit-based panel presentations", () => {
    expect(layout(1_400).variant).toBe("three_column");
    expect(layout(1_200).variant).toBe("inspector_sheet");
    expect(layout(950).variant).toBe("inspector_sheet");
    expect(layout(700).variant).toBe("rail_sheet");
    expect(layout(280, 180).variant).toBe("minimum_recovery");
    expect(chatLayoutDecision({
      containerWidth: 900,
      containerHeight: 700,
      fontScale: 1,
      railOpen: true,
      inspectorOpen: false,
      railWidth: 260,
      inspectorWidth: 360,
    }).variant).toBe("no_inspector");
  });

  it("accounts for increased font scale and available height", () => {
    expect(layout(900, 700, undefined, 1.5).variant).toBe("rail_sheet");
    expect(layout(900, 200).variant).toBe("minimum_recovery");
  });

  it("uses hysteresis to avoid panel oscillation at fit thresholds", () => {
    expect(layout(1_230, 700, "three_column").variant).toBe("three_column");
    expect(layout(1_230, 700, "inspector_sheet").variant).toBe("inspector_sheet");
    expect(layout(695, 700, "no_inspector").railPresentation).toBe("column");
    expect(layout(695, 700, "rail_sheet").railPresentation).toBe("sheet");
  });

  it("keeps every primary recovery route in every layout", () => {
    for (const decision of [layout(1_400), layout(950), layout(700), layout(280, 180)]) {
      expect(chatLayoutPrimaryActions(decision)).toEqual([
        "threads",
        "composer",
        "requests",
        "stop",
        "settings",
        "inspector",
      ]);
    }
  });

  it("shows one selected surface at the minimum recovery floor", () => {
    const inspector = layout(280, 180);
    expect(inspector.activeSurface).toBe("inspector");
    const rail = chatLayoutDecision({
      containerWidth: 280,
      containerHeight: 180,
      fontScale: 1,
      railOpen: true,
      inspectorOpen: false,
      railWidth: 260,
      inspectorWidth: 360,
    });
    expect(rail.activeSurface).toBe("rail");
    const conversation = chatLayoutDecision({
      containerWidth: 280,
      containerHeight: 180,
      fontScale: 1,
      railOpen: false,
      inspectorOpen: false,
      railWidth: 260,
      inspectorWidth: 360,
    });
    expect(conversation.activeSurface).toBe("conversation");
  });

  it("middle-truncates long Unicode labels without losing either end", () => {
    expect(middleTruncate("proyecto-非常に長い-workspace-folder", 15)).toBe("proyect…-folder");
    expect(middleTruncate("short", 15)).toBe("short");
  });

  it("bounds long Spanish labels and workspace paths without splitting Unicode", () => {
    const spanish = "Configuración del espacio de trabajo de investigación internacional";
    const path = "proyectos/🚀-lanzamiento/investigación/documentación/decisiones-arquitectónicas.md";
    expect(Array.from(middleTruncate(spanish, 24))).toHaveLength(24);
    expect(middleTruncate(spanish, 24)).toMatch(/^Configuració.*ternacional$/);
    expect(Array.from(middleTruncate(path, 30))).toHaveLength(30);
    expect(middleTruncate(path, 30)).toMatch(/^proyectos\/.*tectónicas\.md$/);
  });

  it("supports keyboard-only separator increments, bounds, and reset", () => {
    const resize = (key: string, direction: "standard" | "reversed" = "standard") => panelWidthFromKey({
      current: 260,
      minimum: 160,
      maximum: 520,
      defaultValue: 260,
      step: 16,
      direction,
      key,
    });
    expect(resize("ArrowLeft")).toBe(244);
    expect(resize("ArrowRight")).toBe(276);
    expect(resize("ArrowLeft", "reversed")).toBe(276);
    expect(resize("Home")).toBe(160);
    expect(resize("End")).toBe(520);
    expect(resize("Enter")).toBe(260);
    expect(resize("Escape")).toBeNull();
  });

  it("uses cached custom panel widths on the first layout", () => {
    expect(preferredPanelWidth(412, 260, 320)).toBe(412);
    expect(preferredPanelWidth(260, 260, 320)).toBe(320);
    expect(preferredPanelWidth(undefined, 260, 320)).toBe(320);
  });

  it("aligns pointer resize dimensions and fractional bounds to whole pixels", () => {
    expect(clampPanelSizeToWholePixel(319.49, 160, 520)).toBe(319);
    expect(clampPanelSizeToWholePixel(319.5, 160, 520)).toBe(320);
    expect(clampPanelSizeToWholePixel(800, 240, 519.75)).toBe(519);
    expect(clampPanelSizeToWholePixel(100, 240.2, 200)).toBe(241);
  });

  it("aligns moving panel edges to physical pixels at fractional display scales", () => {
    const align = (
      value: number,
      direction: "from-start" | "from-end",
      anchor: number,
      devicePixelRatio = 1.25,
    ) => alignPanelSizeToDevicePixel({
      value,
      minimum: 160,
      maximum: 520,
      anchor,
      direction,
      devicePixelRatio,
    });
    expect(align(320, "from-start", 5.625)).toBeCloseTo(319.975);
    expect((5.625 + align(320, "from-start", 5.625)) * 1.25).toBeCloseTo(407);
    expect((994.375 - align(320, "from-end", 994.375)) * 1.25).toBeCloseTo(843);
    expect(align(900, "from-start", 0)).toBe(520);
    expect(align(100, "from-end", 0)).toBe(160);
    expect(align(320.4, "from-start", 0, 0)).toBe(320);
  });

  it("lets the inspector use spare width without shrinking the conversation", () => {
    const maximum = (containerWidth: number, railVisible = true) => chatInspectorResizeMaximum({
      containerWidth,
      railVisible,
      railWidth: 260,
      minimum: 240,
      maximum: 960,
    });
    expect(maximum(1_400)).toBe(700);
    expect(maximum(1_400, false)).toBe(960);
    expect(maximum(2_000)).toBe(960);
    expect(maximum(700)).toBe(240);
    expect(maximum(1_399.75)).toBe(699);
  });

  it("removes smooth scrolling when reduced motion is requested", () => {
    expect(chatScrollBehavior(false)).toBe("smooth");
    expect(chatScrollBehavior(true)).toBe("auto");
  });
});
