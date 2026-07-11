import { describe, expect, it } from "vitest";
import { shouldRestoreNotesEditorFocusAfterLazyLoad } from "./lazy-editor-focus";

describe("lazy Notes editor focus", () => {
  it("restores focus after a delayed control load when the editor owned it", () => {
    expect(shouldRestoreNotesEditorFocusAfterLazyLoad({
      requestedWhileFocused: true,
      stillOwnsFocus: true,
      compositionActive: false,
    })).toBe(true);
  });

  it("does not steal focus after the user moved away during the load", () => {
    expect(shouldRestoreNotesEditorFocusAfterLazyLoad({
      requestedWhileFocused: true,
      stillOwnsFocus: false,
      compositionActive: false,
    })).toBe(false);
  });

  it("does not interrupt an active IME composition", () => {
    expect(shouldRestoreNotesEditorFocusAfterLazyLoad({
      requestedWhileFocused: true,
      stillOwnsFocus: true,
      compositionActive: true,
    })).toBe(false);
  });
});
