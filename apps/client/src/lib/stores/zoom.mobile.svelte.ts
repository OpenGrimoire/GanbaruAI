/** Mobile shells use system accessibility scaling and do not expose WebView zoom. */
export const APP_ZOOM_LEVELS: readonly number[] = Object.freeze([1]);

const noop = (): void => undefined;

/** Return the fixed mobile zoom contract without importing desktop window APIs. */
export function getZoom() {
  return {
    get level(): number {
      return 1;
    },
    get percent(): number {
      return 100;
    },
    get canZoomIn(): boolean {
      return false;
    },
    get canZoomOut(): boolean {
      return false;
    },
    get isDefault(): boolean {
      return true;
    },
    zoomIn: noop,
    zoomOut: noop,
    reset: noop,
    setLevel: noop,
    reapply: noop,
  };
}
