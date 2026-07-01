import { describe, expect, it } from "vitest";
import { nextNotesFocusRequest } from "./editor-focus";

describe("notes editor focus helpers", () => {
  it("increments the focus request token for each requested block", () => {
    expect(nextNotesFocusRequest({ blockId: "a", requestId: 2 }, "b")).toEqual({
      blockId: "b",
      requestId: 3,
    });
  });

  it("keeps focus clearing observable through the request token", () => {
    expect(nextNotesFocusRequest({ blockId: "a", requestId: 2 }, null)).toEqual({
      blockId: null,
      requestId: 3,
    });
  });
});
