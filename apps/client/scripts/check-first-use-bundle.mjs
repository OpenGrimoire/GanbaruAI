import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const clientDir = path.resolve(scriptDir, "..");
const metadataPath = path.join(clientDir, "dist", "first-use-bundle-metadata.json");
const baselinePath = path.join(scriptDir, "first-use-bundle-baseline.json");

async function readJson(filePath) {
  try {
    return JSON.parse(await readFile(filePath, "utf8"));
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`read ${path.relative(clientDir, filePath)}: ${message}`);
  }
}

function requireObject(value, label) {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value;
}

function requireString(value, label) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${label} must be a non-empty string`);
  }
  return value;
}

function requireStringArray(value, label) {
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string")) {
    throw new Error(`${label} must be a string array`);
  }
  return value;
}

function readChunks(metadata) {
  const root = requireObject(metadata, "bundle metadata");
  if (root.schemaVersion !== 1 || !Array.isArray(root.chunks)) {
    throw new Error("bundle metadata has an unsupported schema");
  }
  return root.chunks.map((value, index) => {
    const chunk = requireObject(value, `bundle chunk ${index}`);
    return {
      fileName: requireString(chunk.fileName, `bundle chunk ${index} fileName`),
      isEntry: chunk.isEntry === true,
      modules: requireStringArray(chunk.modules, `bundle chunk ${index} modules`),
    };
  });
}

function readBaseline(value) {
  const root = requireObject(value, "first-use bundle baseline");
  if (root.schemaVersion !== 1 || !Array.isArray(root.routes)) {
    throw new Error("first-use bundle baseline has an unsupported schema");
  }
  const routes = root.routes.map((value, index) => {
    const route = requireObject(value, `baseline route ${index}`);
    const maxSourceModules = route.maxSourceModules;
    if (!Number.isSafeInteger(maxSourceModules) || maxSourceModules < 1) {
      throw new Error(`baseline route ${index} maxSourceModules must be a positive integer`);
    }
    return {
      name: requireString(route.name, `baseline route ${index} name`),
      module: requireString(route.module, `baseline route ${index} module`),
      maxSourceModules,
    };
  });
  return {
    routes,
    forbiddenEntryModules: requireStringArray(
      root.forbiddenEntryModules,
      "baseline forbiddenEntryModules",
    ),
  };
}

const chunks = readChunks(await readJson(metadataPath));
const baseline = readBaseline(await readJson(baselinePath));
const allModules = new Set(chunks.flatMap((chunk) => chunk.modules));
const entryModules = new Set(
  chunks.filter((chunk) => chunk.isEntry).flatMap((chunk) => chunk.modules),
);
const failures = [];
const routes = baseline.routes.map((route) => {
  const owner = chunks.find((chunk) => chunk.modules.includes(route.module));
  if (!owner) {
    failures.push(`${route.name} route module is absent: ${route.module}`);
    return { name: route.name, chunk: null, sourceModules: null };
  }
  const sourceModules = owner.modules.filter((moduleId) => moduleId.startsWith("src/")).length;
  if (sourceModules > route.maxSourceModules) {
    failures.push(
      `${route.name} route chunk has ${sourceModules} source modules, baseline allows ${route.maxSourceModules}`,
    );
  }
  return { name: route.name, chunk: owner.fileName, sourceModules };
});

for (const moduleId of baseline.forbiddenEntryModules) {
  if (!allModules.has(moduleId)) {
    failures.push(`forbidden entry module is absent from all chunks: ${moduleId}`);
  } else if (entryModules.has(moduleId)) {
    failures.push(`forbidden module is present in an entry chunk: ${moduleId}`);
  }
}

if (failures.length > 0) {
  throw new Error(`first-use bundle contract failed:\n${failures.map((failure) => `* ${failure}`).join("\n")}`);
}

console.log(JSON.stringify({ routes, forbiddenEntryModules: baseline.forbiddenEntryModules }, null, 2));
