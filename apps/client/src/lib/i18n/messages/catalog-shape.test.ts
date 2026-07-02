import { describe, expect, it } from "vitest";

import { en } from "./en";
import { es } from "./es";

type CatalogBranch = Readonly<Record<string, unknown>>;

function isCatalogBranch(value: unknown): value is CatalogBranch {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function catalogShapeIssues(
  reference: unknown,
  candidate: unknown,
  path: readonly string[] = [],
): string[] {
  const location = path.join(".") || "<root>";
  const referenceIsBranch = isCatalogBranch(reference);
  const candidateIsBranch = isCatalogBranch(candidate);

  if (referenceIsBranch !== candidateIsBranch) {
    return [`${location}: expected branch state ${String(referenceIsBranch)}`];
  }

  if (!referenceIsBranch || !candidateIsBranch) {
    const referenceType = typeof reference;
    const candidateType = typeof candidate;
    return referenceType === candidateType
      ? []
      : [`${location}: expected ${referenceType}, received ${candidateType}`];
  }

  const issues: string[] = [];
  for (const key of Object.keys(reference)) {
    if (!Object.hasOwn(candidate, key)) {
      issues.push(`${[...path, key].join(".")}: missing key`);
      continue;
    }
    issues.push(...catalogShapeIssues(reference[key], candidate[key], [...path, key]));
  }

  for (const key of Object.keys(candidate)) {
    if (!Object.hasOwn(reference, key)) {
      issues.push(`${[...path, key].join(".")}: extra key`);
    }
  }

  return issues;
}

describe("message catalogs", () => {
  it("keeps built-in locale shapes aligned", () => {
    expect(catalogShapeIssues(en, es)).toEqual([]);
  });
});
