const FOCUSABLE_SELECTOR = [
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "a[href]",
  "[tabindex]:not([tabindex='-1'])",
].join(", ");

/** Returns visible, interactive descendants in document order. */
export function focusableElements(container: HTMLElement | undefined): HTMLElement[] {
  if (!container) return [];
  return [...container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)]
    .filter((element) => (
      !element.hidden
      && !element.closest("[inert]")
      && element.getClientRects().length > 0
    ));
}

export function firstFocusable(container: HTMLElement | undefined): HTMLElement | undefined {
  return focusableElements(container)[0];
}

/** Keeps Tab focus inside a modal or sheet and reports whether it handled the event. */
export function trapTabFocus(
  event: KeyboardEvent,
  container = event.currentTarget instanceof HTMLElement ? event.currentTarget : undefined,
): boolean {
  if (event.key !== "Tab" || !container) return false;
  const focusable = focusableElements(container);
  const first = focusable[0];
  const last = focusable.at(-1);
  if (!first || !last) {
    event.preventDefault();
    container.focus();
    return true;
  }
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last.focus();
    return true;
  }
  if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
    return true;
  }
  return false;
}
