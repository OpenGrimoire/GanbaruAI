import { describe, expect, it } from "vitest";
import { resolveLocalMusicPath } from "./platform-paths.mobile";

describe("Android music paths", () => {
  it("encodes the selected tree and normalized relative path independently", () => {
    expect(resolveLocalMusicPath(
      "content://com.android.externalstorage.documents/tree/primary%3AMusic#root",
      "/Albums\\Björk #1/01 Jóga.flac/",
    )).toBe(
      "ganbaru-saf:content%3A%2F%2Fcom.android.externalstorage.documents%2Ftree%2Fprimary%253AMusic%23root#Albums%2FBj%C3%B6rk%20%231%2F01%20J%C3%B3ga.flac",
    );
  });

  it("rejects incomplete locators before they reach the native boundary", () => {
    expect(() => resolveLocalMusicPath("content://provider/tree/music", "///"))
      .toThrow("relative file path");
    expect(() => resolveLocalMusicPath("/storage/emulated/0/Music", "track.mp3"))
      .toThrow("selected folder");
  });
});
