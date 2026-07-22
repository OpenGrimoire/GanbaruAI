export type ChatLayoutVariant =
  | "three_column"
  | "no_inspector"
  | "rail_sheet"
  | "inspector_sheet"
  | "minimum_recovery";

export interface ChatLayoutInput {
  containerWidth: number;
  containerHeight: number;
  fontScale: number;
  railOpen: boolean;
  inspectorOpen: boolean;
  railWidth: number;
  inspectorWidth: number;
  previousVariant?: ChatLayoutVariant;
}

export interface ChatLayoutDecision {
  variant: ChatLayoutVariant;
  railPresentation: "column" | "sheet";
  inspectorPresentation: "closed" | "column" | "sheet";
  activeSurface: "conversation" | "rail" | "inspector";
}

const BASE_CONVERSATION_MIN = 440;
const BASE_MINIMUM_ENTER_WIDTH = 320;
const BASE_MINIMUM_EXIT_WIDTH = 360;
const BASE_MINIMUM_ENTER_HEIGHT = 210;
const BASE_MINIMUM_EXIT_HEIGHT = 240;
const BASE_HYSTERESIS = 24;
const SEPARATOR_WIDTH = 4;

export interface ChatInspectorResizeInput {
  containerWidth: number;
  railVisible: boolean;
  railWidth: number;
  minimum: number;
  maximum: number;
}

/**
 * Bounds a column inspector while reserving the conversation reading width.
 *
 * @param input Current shell, rail, and inspector constraints.
 * @returns The largest inspector width that keeps the conversation usable.
 */
export function chatInspectorResizeMaximum(input: ChatInspectorResizeInput): number {
  const occupiedByRail = input.railVisible ? input.railWidth + SEPARATOR_WIDTH : 0;
  const available = input.containerWidth
    - occupiedByRail
    - BASE_CONVERSATION_MIN
    - SEPARATOR_WIDTH;
  return Math.max(input.minimum, Math.min(input.maximum, available));
}

/**
 * Selects the Chat shell layout from measured fit while preserving the prior
 * layout inside a hysteresis band.
 *
 * @param input Measured shell and panel constraints.
 * @returns Stable panel presentations and the primary visible surface.
 */
export function chatLayoutDecision(input: ChatLayoutInput): ChatLayoutDecision {
  const scale = Math.max(1, Math.min(2, input.fontScale));
  const previous = input.previousVariant;
  const minimum = previous === "minimum_recovery"
    ? input.containerWidth < BASE_MINIMUM_EXIT_WIDTH * scale
      || input.containerHeight < BASE_MINIMUM_EXIT_HEIGHT * scale
    : input.containerWidth < BASE_MINIMUM_ENTER_WIDTH * scale
      || input.containerHeight < BASE_MINIMUM_ENTER_HEIGHT * scale;
  if (minimum) {
    return {
      variant: "minimum_recovery",
      railPresentation: "sheet",
      inspectorPresentation: input.inspectorOpen ? "sheet" : "closed",
      activeSurface: input.inspectorOpen ? "inspector" : input.railOpen ? "rail" : "conversation",
    };
  }

  const conversationMin = BASE_CONVERSATION_MIN * scale;
  const railRequired = input.railWidth + conversationMin + SEPARATOR_WIDTH;
  const railWasColumn = previous === "three_column"
    || previous === "no_inspector"
    || previous === "inspector_sheet";
  const railFits = stableFit(input.containerWidth, railRequired, railWasColumn, scale);
  if (!railFits) {
    return {
      variant: "rail_sheet",
      railPresentation: "sheet",
      inspectorPresentation: input.inspectorOpen ? "sheet" : "closed",
      activeSurface: input.inspectorOpen ? "inspector" : "conversation",
    };
  }

  if (!input.inspectorOpen) {
    return {
      variant: "no_inspector",
      railPresentation: "column",
      inspectorPresentation: "closed",
      activeSurface: "conversation",
    };
  }

  const inspectorRequired = railRequired + input.inspectorWidth + SEPARATOR_WIDTH;
  const inspectorFits = stableFit(
    input.containerWidth,
    inspectorRequired,
    previous === "three_column",
    scale,
  );
  return inspectorFits
    ? {
        variant: "three_column",
        railPresentation: "column",
        inspectorPresentation: "column",
        activeSurface: "conversation",
      }
    : {
        variant: "inspector_sheet",
        railPresentation: "column",
        inspectorPresentation: "sheet",
        activeSurface: "inspector",
      };
}

/**
 * Lists the recovery routes that every layout must keep available.
 *
 * @param decision Current layout decision.
 * @returns Stable primary action identifiers.
 */
export function chatLayoutPrimaryActions(
  decision: ChatLayoutDecision,
): readonly ["threads", "composer", "requests", "stop", "settings", "inspector"] {
  void decision;
  return ["threads", "composer", "requests", "stop", "settings", "inspector"];
}

export interface PanelResizeInput {
  current: number;
  minimum: number;
  maximum: number;
  defaultValue: number;
  step: number;
  direction: "standard" | "reversed";
  key: string;
}

/**
 * Resolves a keyboard separator action to a bounded panel width.
 *
 * @param input Current panel bounds, direction, and pressed key.
 * @returns The next width, or null when the key is not a resize action.
 */
export function panelWidthFromKey(input: PanelResizeInput): number | null {
  const sign = input.direction === "reversed" ? -1 : 1;
  let next: number;
  switch (input.key) {
    case "ArrowLeft": next = input.current - input.step * sign; break;
    case "ArrowRight": next = input.current + input.step * sign; break;
    case "Home": next = input.minimum; break;
    case "End": next = input.maximum; break;
    case "Enter": next = input.defaultValue; break;
    default: return null;
  }
  return Math.max(input.minimum, Math.min(input.maximum, next));
}

/**
 * Selects scroll behavior without bypassing a reduced-motion preference.
 *
 * @param reducedMotion Whether the user requests reduced motion.
 * @returns Immediate or smooth scrolling behavior.
 */
export function chatScrollBehavior(reducedMotion: boolean): ScrollBehavior {
  return reducedMotion ? "auto" : "smooth";
}

/**
 * Preserves both ends of a long label for compact breadcrumbs and paths.
 *
 * @param value Full user-visible label.
 * @param maximumCodePoints Maximum displayed Unicode code points.
 * @returns The original label or a middle-truncated representation.
 */
export function middleTruncate(value: string, maximumCodePoints = 40): string {
  const points = Array.from(value);
  const maximum = Math.max(5, Math.floor(maximumCodePoints));
  if (points.length <= maximum) return value;
  const available = maximum - 1;
  const start = Math.ceil(available / 2);
  const end = Math.floor(available / 2);
  return `${points.slice(0, start).join("")}…${points.slice(points.length - end).join("")}`;
}

function stableFit(
  available: number,
  required: number,
  previouslyFit: boolean,
  scale: number,
): boolean {
  const hysteresis = BASE_HYSTERESIS * scale;
  return previouslyFit
    ? available >= required - hysteresis
    : available >= required + hysteresis;
}
