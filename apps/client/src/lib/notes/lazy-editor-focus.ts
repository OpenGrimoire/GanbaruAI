/**
 * Decides whether an asynchronously loaded editor control may restore focus.
 * Focus is restored only when the same editor owned focus when the request
 * started and no IME composition is active when it resolves.
 */
export function shouldRestoreNotesEditorFocusAfterLazyLoad(input: {
  requestedWhileFocused: boolean;
  stillOwnsFocus: boolean;
  compositionActive: boolean;
}): boolean {
  return input.requestedWhileFocused && input.stillOwnsFocus && !input.compositionActive;
}
