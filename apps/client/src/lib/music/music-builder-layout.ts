export type MusicBuilderLayoutMode = "wide" | "medium" | "narrow";

export interface MusicBuilderLayoutInput {
  width: number;
  height: number;
}

export interface MusicBuilderLayoutProjection {
  mode: MusicBuilderLayoutMode;
  inspectorPresentation: "persistent" | "overlay" | "page";
  comfortable: boolean;
}

export function projectMusicBuilderLayout(
  input: MusicBuilderLayoutInput,
): MusicBuilderLayoutProjection {
  const width = Math.max(0, input.width);
  const height = Math.max(0, input.height);
  if (width >= 860 && height >= 440) {
    return {
      mode: "wide",
      inspectorPresentation: "persistent",
      comfortable: true,
    };
  }
  if (width >= 560 && height >= 300) {
    return {
      mode: "medium",
      inspectorPresentation: "overlay",
      comfortable: height >= 380,
    };
  }
  return {
    mode: "narrow",
    inspectorPresentation: "page",
    comfortable: width >= 360 && height >= 300,
  };
}
