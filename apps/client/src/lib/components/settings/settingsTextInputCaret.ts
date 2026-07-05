interface TextInputPointerGeometry {
  clientX: number;
  inputLeft: number;
  borderLeftWidth: number;
  paddingLeft: number;
  scrollLeft: number;
  textWidth: number;
  tolerancePx?: number;
}

const TEXT_HIT_TOLERANCE_PX = 2;

let textMeasureContext: CanvasRenderingContext2D | null = null;

function numericCssPixel(value: string): number {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

function measuredTextWidth(text: string, font: string): number {
  if (typeof document === "undefined") return text.length * 8;
  if (!textMeasureContext) {
    textMeasureContext = document.createElement("canvas").getContext("2d");
  }
  if (!textMeasureContext) return text.length * 8;
  textMeasureContext.font = font;
  return textMeasureContext.measureText(text).width;
}

export function settingsTextInputPointerMissesText(
  geometry: TextInputPointerGeometry,
): boolean {
  if (geometry.textWidth <= 0) return true;
  const tolerance = geometry.tolerancePx ?? TEXT_HIT_TOLERANCE_PX;
  const textStart = geometry.inputLeft
    + geometry.borderLeftWidth
    + geometry.paddingLeft
    - geometry.scrollLeft;
  const textEnd = textStart + geometry.textWidth;
  return geometry.clientX < textStart - tolerance
    || geometry.clientX > textEnd + tolerance;
}

export function moveCaretToEndWhenSettingsInputTextMissed(
  event: PointerEvent & { currentTarget: HTMLInputElement },
): void {
  if (event.button !== 0 || event.detail > 1) return;
  const input = event.currentTarget;
  if (input.disabled || input.readOnly) return;

  const style = window.getComputedStyle(input);
  const rect = input.getBoundingClientRect();
  const missesText = settingsTextInputPointerMissesText({
    clientX: event.clientX,
    inputLeft: rect.left,
    borderLeftWidth: numericCssPixel(style.borderLeftWidth),
    paddingLeft: numericCssPixel(style.paddingLeft),
    scrollLeft: input.scrollLeft,
    textWidth: measuredTextWidth(input.value, style.font),
  });
  if (!missesText) return;

  window.requestAnimationFrame(() => {
    input.focus();
    const end = input.value.length;
    input.setSelectionRange(end, end);
  });
}
