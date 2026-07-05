interface TextInputPointerGeometry {
  clientX: number;
  inputLeft: number;
  borderLeftWidth: number;
  paddingLeft: number;
  scrollLeft: number;
  text: string;
  measureText: (text: string) => number;
}

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

function textCaretStops(text: string): number[] {
  const stops = [0];
  let index = 0;
  for (const symbol of Array.from(text)) {
    index += symbol.length;
    stops.push(index);
  }
  return stops;
}

export function textInputCaretIndexForPointer(
  geometry: TextInputPointerGeometry,
): number {
  if (geometry.text.length === 0) return 0;
  const textStart = geometry.inputLeft
    + geometry.borderLeftWidth
    + geometry.paddingLeft
    - geometry.scrollLeft;
  const textX = geometry.clientX - textStart;
  if (textX <= 0) return 0;

  const stops = textCaretStops(geometry.text);
  let previousWidth = 0;
  for (let stopIndex = 1; stopIndex < stops.length; stopIndex += 1) {
    const stop = stops[stopIndex] ?? geometry.text.length;
    const nextWidth = geometry.measureText(geometry.text.slice(0, stop));
    const midpoint = previousWidth + (nextWidth - previousWidth) / 2;
    if (textX < midpoint) return stops[stopIndex - 1] ?? 0;
    previousWidth = nextWidth;
  }
  return geometry.text.length;
}

export function moveTextInputCaretToPointer(
  event: PointerEvent & { currentTarget: HTMLInputElement },
): void {
  if (event.button !== 0 || event.detail > 1 || event.shiftKey) return;
  const input = event.currentTarget;
  if (input.disabled || input.readOnly) return;
  event.preventDefault();

  const style = window.getComputedStyle(input);
  const rect = input.getBoundingClientRect();
  const caretIndex = textInputCaretIndexForPointer({
    clientX: event.clientX,
    inputLeft: rect.left,
    borderLeftWidth: numericCssPixel(style.borderLeftWidth),
    paddingLeft: numericCssPixel(style.paddingLeft),
    scrollLeft: input.scrollLeft,
    text: input.value,
    measureText: (text) => measuredTextWidth(text, style.font),
  });

  input.focus({ preventScroll: true });
  input.setSelectionRange(caretIndex, caretIndex);
}
