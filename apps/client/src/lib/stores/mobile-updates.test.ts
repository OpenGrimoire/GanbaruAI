import { beforeAll, describe, expect, it } from "vitest";
import { vi } from "vitest";

type MobileUpdatesModule = typeof import("./mobile-updates");

let module: MobileUpdatesModule | null = null;

function getModule(): MobileUpdatesModule {
  if (!module) {
    throw new Error("Mobile updates module is not loaded");
  }
  return module;
}

beforeAll(async () => {
  vi.stubGlobal("__GANBARU_AI_BUILD_REF__", "0.1.5+abc123");
  vi.stubGlobal("__GANBARU_AI_GITHUB_REPOSITORY__", "opengrimoire/ganbaru-ai");
  module = await import("./mobile-updates");
});

describe("mobile version parsing", () => {
  it("extracts the build version from a build reference", () => {
    const mobileUpdates = getModule();
    expect(mobileUpdates.parseInstalledVersion("0.1.5+abc123")).toBe("0.1.5");
    expect(mobileUpdates.parseInstalledVersion("1.2.3")).toBe("1.2.3");
  });

  it("rejects an invalid build reference", () => {
    expect(getModule().parseInstalledVersion("build")).toBeNull();
  });

  it("parses a GitHub release tag", () => {
    expect(getModule().parseReleaseTag("app-v0.2.0")).toBe("0.2.0");
    expect(getModule().parseReleaseTag("release-0.2.0")).toBeNull();
  });

  it("parses semver with prerelease and build metadata", () => {
    const parsed = getModule().parseVersion("0.1.0-beta.1+build.45");
    expect(parsed).not.toBeNull();
    expect(parsed?.major).toBe(0);
    expect(parsed?.minor).toBe(1);
    expect(parsed?.patch).toBe(0);
    expect(parsed?.prerelease).toEqual(["beta", "1"]);
  });

  it("compares versions semantically", () => {
    expect(getModule().compareVersions("1.2.4", "1.2.3")).toBeGreaterThan(0);
    expect(getModule().compareVersions("1.2.3", "1.2.4")).toBeLessThan(0);
    expect(getModule().compareVersions("1.2.3", "1.2.3")).toBe(0);
    expect(getModule().compareVersions("1.2.3-alpha", "1.2.3")).toBeLessThan(0);
    expect(getModule().compareVersions("1.2.3-alpha.1", "1.2.3-alpha")).toBeGreaterThan(0);
    expect(getModule().compareVersions("1.2.3-alpha", "1.2.3-alpha.1")).toBeLessThan(0);
  });
});
