export type DismissOnOutsideReason = "outside-pointer" | "escape";

export interface DismissOnOutsideOptions {
  enabled?: boolean;
  onDismiss: (reason: DismissOnOutsideReason, event: Event) => void;
}

function targetIsInsideNode(target: EventTarget | null, node: HTMLElement): boolean {
  return target instanceof Node && node.contains(target);
}

export function dismissOnOutside(node: HTMLElement, options: DismissOnOutsideOptions) {
  let current = options;

  function isEnabled(): boolean {
    return current.enabled !== false;
  }

  function handlePointerDown(event: PointerEvent): void {
    if (!isEnabled() || targetIsInsideNode(event.target, node)) return;
    current.onDismiss("outside-pointer", event);
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (!isEnabled() || event.key !== "Escape") return;
    current.onDismiss("escape", event);
  }

  document.addEventListener("pointerdown", handlePointerDown, true);
  window.addEventListener("keydown", handleKeydown, true);

  return {
    update(nextOptions: DismissOnOutsideOptions) {
      current = nextOptions;
    },
    destroy() {
      document.removeEventListener("pointerdown", handlePointerDown, true);
      window.removeEventListener("keydown", handleKeydown, true);
    },
  };
}
