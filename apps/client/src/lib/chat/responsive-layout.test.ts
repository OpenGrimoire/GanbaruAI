import { describe, expect, it } from "vitest";
import {
  chatLayoutDecision,
  chatLayoutPrimaryActions,
  chatScrollBehavior,
  middleTruncate,
  panelWidthFromKey,
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
    inspectorWidth: 360,
    previousVariant,
  });
}

describe("Chat responsive layout", () => {
  it("selects all fit-based panel presentations", () => {
    expect(layout(1_200).variant).toBe("three_column");
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
    expect(layout(1_080, 700, "three_column").variant).toBe("three_column");
    expect(layout(1_080, 700, "inspector_sheet").variant).toBe("inspector_sheet");
    expect(layout(695, 700, "no_inspector").railPresentation).toBe("column");
    expect(layout(695, 700, "rail_sheet").railPresentation).toBe("sheet");
  });

  it("keeps every primary recovery route in every layout", () => {
    for (const decision of [layout(1_200), layout(950), layout(700), layout(280, 180)]) {
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

  it("removes smooth scrolling when reduced motion is requested", () => {
    expect(chatScrollBehavior(false)).toBe("smooth");
    expect(chatScrollBehavior(true)).toBe("auto");
  });
});
