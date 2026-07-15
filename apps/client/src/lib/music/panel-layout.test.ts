import { describe, expect, it } from "vitest";
import { fittedSidePlaylistPanelHeight } from "./panel-layout";

describe("fittedSidePlaylistPanelHeight", () => {
  it("fits the panel chrome around a 16:9 media column", () => {
    expect(fittedSidePlaylistPanelHeight({
      mediaWidth: 680,
      headerHeight: 42,
      controlsHeight: 76,
      sideBySide: true,
    })).toBe(501);
  });

  it("leaves stacked playlist layouts at their normal responsive height", () => {
    expect(fittedSidePlaylistPanelHeight({
      mediaWidth: 680,
      headerHeight: 42,
      controlsHeight: 76,
      sideBySide: false,
    })).toBeNull();
  });

  it("rejects measurements that cannot produce a usable media surface", () => {
    expect(fittedSidePlaylistPanelHeight({
      mediaWidth: 0,
      headerHeight: 42,
      controlsHeight: 76,
      sideBySide: true,
    })).toBeNull();
  });
});
