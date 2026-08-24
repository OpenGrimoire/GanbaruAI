const FOCUSABLE_SELECTOR = [
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "a[href]",
  "[tabindex]:not([tabindex='-1'])",
].join(",");

function visibleFocusableElements(container: HTMLElement): HTMLElement[] {
  return [...container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)]
    .filter((element) => !element.hidden && element.getAttribute("aria-hidden") !== "true");
}

/** Keep a Tab key event inside a modal container. */
export function trapModalTabKey(container: HTMLElement, event: KeyboardEvent): boolean {
  if (event.key !== "Tab" || container.closest("[inert]")) return false;
  const focusable = visibleFocusableElements(container);
  if (focusable.length === 0) {
    event.preventDefault();
    container.focus();
    return true;
  }

  const first = focusable[0];
  const last = focusable.at(-1);
  if (!first || !last) return false;
  const active = document.activeElement;
  if (event.shiftKey && (active === first || !container.contains(active))) {
    event.preventDefault();
    last.focus();
    return true;
  }
  if (!event.shiftKey && (active === last || !container.contains(active))) {
    event.preventDefault();
    first.focus();
    return true;
  }
  return false;
}

/** Focus a modal, contain Tab navigation, and restore the invoking control. */
export function activateModalFocus(
  container: HTMLElement,
  initialFocus: HTMLElement | null = null,
): () => void {
  const previousFocus = document.activeElement instanceof HTMLElement
    ? document.activeElement
    : null;
  const handleKeydown = (event: KeyboardEvent): void => {
    trapModalTabKey(container, event);
  };
  document.addEventListener("keydown", handleKeydown, true);
  queueMicrotask(() => {
    const target = initialFocus ?? visibleFocusableElements(container)[0] ?? container;
    target.focus();
  });

  return () => {
    document.removeEventListener("keydown", handleKeydown, true);
    if (previousFocus?.isConnected) queueMicrotask(() => previousFocus.focus());
  };
}
