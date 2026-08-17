import { render } from "svelte/server";
import { describe, expect, it } from "vitest";
import ConfirmDialog from "./ConfirmDialog.svelte";

describe("ConfirmDialog", () => {
  it("adds the standard keyboard hints to custom button labels", () => {
    const { body } = render(ConfirmDialog, {
      props: {
        message: "Delete this item?",
        confirmLabel: "Delete",
        cancelLabel: "Keep",
        onConfirm: () => undefined,
        onCancel: () => undefined,
      },
    });

    expect(body).toContain("Keep (Esc)");
    expect(body).toContain("Delete (Enter)");
  });

  it("adds the standard keyboard hints to the default labels", () => {
    const { body } = render(ConfirmDialog, {
      props: {
        message: "Continue?",
        onConfirm: () => undefined,
        onCancel: () => undefined,
      },
    });

    expect(body).toContain("No (Esc)");
    expect(body).toContain("Yes (Enter)");
  });
});
