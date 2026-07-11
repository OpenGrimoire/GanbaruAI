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
      imports: requireStringArray(chunk.imports, `bundle chunk ${index} imports`),
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
    shell: (() => {
      const shell = requireObject(root.shell, "baseline shell");
      return {
        module: requireString(shell.module, "baseline shell module"),
        forbiddenModules: requireStringArray(
          shell.forbiddenModules,
          "baseline shell forbiddenModules",
        ),
      };
    })(),
    settingsAppearance: (() => {
      const contract = requireObject(root.settingsAppearance, "baseline settingsAppearance");
      return {
        loadedModules: requireStringArray(
          contract.loadedModules,
          "baseline settingsAppearance loadedModules",
        ),
        forbiddenModules: requireStringArray(
          contract.forbiddenModules,
          "baseline settingsAppearance forbiddenModules",
        ),
      };
    })(),
    settingsDetailModules: requireStringArray(
      root.settingsDetailModules,
      "baseline settingsDetailModules",
    ),
    defaultEnglishStartup: (() => {
      const contract = requireObject(
        root.defaultEnglishStartup,
        "baseline defaultEnglishStartup",
      );
      return {
        loadedModules: requireStringArray(
          contract.loadedModules,
          "baseline defaultEnglishStartup loadedModules",
        ),
        forbiddenModulePrefixes: requireStringArray(
          contract.forbiddenModulePrefixes,
          "baseline defaultEnglishStartup forbiddenModulePrefixes",
        ),
      };
    })(),
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
const chunksByFileName = new Map(chunks.map((chunk) => [chunk.fileName, chunk]));

function staticChunkClosure(rootChunks) {
  const visited = new Set();
  const visit = (chunk) => {
    if (!chunk || visited.has(chunk.fileName)) return;
    visited.add(chunk.fileName);
    for (const importedFile of chunk.imports) visit(chunksByFileName.get(importedFile));
  };
  for (const chunk of rootChunks) visit(chunk);
  return chunks.filter((chunk) => visited.has(chunk.fileName));
}
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

const shellChunk = chunks.find((chunk) => chunk.modules.includes(baseline.shell.module));
if (!shellChunk) {
  failures.push(`initial shell module is absent: ${baseline.shell.module}`);
}
for (const moduleId of baseline.shell.forbiddenModules) {
  if (!allModules.has(moduleId)) {
    failures.push(`forbidden shell module is absent from all chunks: ${moduleId}`);
  } else if (shellChunk?.modules.includes(moduleId)) {
    failures.push(`forbidden module is present in the initial shell chunk: ${moduleId}`);
  }
}

const settingsAppearanceRoots = baseline.settingsAppearance.loadedModules.map((moduleId) => {
  const owner = chunks.find((chunk) => chunk.modules.includes(moduleId));
  if (!owner) failures.push(`Settings Appearance loaded module is absent: ${moduleId}`);
  return owner;
}).filter(Boolean);
const settingsAppearanceChunks = staticChunkClosure(settingsAppearanceRoots);
const settingsAppearanceModules = new Set(
  settingsAppearanceChunks.flatMap((chunk) => chunk.modules),
);
for (const moduleId of baseline.settingsAppearance.forbiddenModules) {
  if (!allModules.has(moduleId)) {
    failures.push(`Settings Appearance forbidden module is absent from all chunks: ${moduleId}`);
  } else if (settingsAppearanceModules.has(moduleId)) {
    failures.push(`module is loaded when opening Settings Appearance: ${moduleId}`);
  }
}

const settingsDetailChunks = baseline.settingsDetailModules.map((moduleId) => {
  const owner = chunks.find((chunk) => chunk.modules.includes(moduleId));
  if (!owner) failures.push(`Settings detail module is absent: ${moduleId}`);
  return { module: moduleId, chunk: owner?.fileName ?? null };
});
const settingsDetailChunkNames = settingsDetailChunks
  .map((entry) => entry.chunk)
  .filter((chunk) => chunk !== null);
if (new Set(settingsDetailChunkNames).size !== settingsDetailChunkNames.length) {
  failures.push("Settings detail modules are not emitted in distinct chunks");
}

const defaultEnglishStartupRoots = baseline.defaultEnglishStartup.loadedModules.map(
  (moduleId) => {
    const owner = chunks.find((chunk) => chunk.modules.includes(moduleId));
    if (!owner) failures.push(`default English startup module is absent: ${moduleId}`);
    return owner;
  },
).filter(Boolean);
const defaultEnglishStartupChunks = staticChunkClosure(defaultEnglishStartupRoots);
const defaultEnglishStartupModules = new Set(
  defaultEnglishStartupChunks.flatMap((chunk) => chunk.modules),
);
for (const prefix of baseline.defaultEnglishStartup.forbiddenModulePrefixes) {
  const matchingModules = [...allModules].filter((moduleId) => moduleId.startsWith(prefix));
  if (matchingModules.length === 0) {
    failures.push(`default English startup forbidden prefix matches no modules: ${prefix}`);
    continue;
  }
  for (const moduleId of matchingModules) {
    if (defaultEnglishStartupModules.has(moduleId)) {
      failures.push(`module is loaded during default English startup: ${moduleId}`);
    }
  }
}

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

console.log(JSON.stringify({
  routes,
  shell: {
    chunk: shellChunk?.fileName ?? null,
    forbiddenModules: baseline.shell.forbiddenModules,
  },
  settingsAppearance: {
    chunks: settingsAppearanceChunks.map((chunk) => chunk.fileName),
    sourceModules: [...settingsAppearanceModules]
      .filter((moduleId) => moduleId.startsWith("src/")).length,
    forbiddenModules: baseline.settingsAppearance.forbiddenModules,
  },
  settingsDetailChunks,
  defaultEnglishStartup: {
    chunks: defaultEnglishStartupChunks.map((chunk) => chunk.fileName),
    sourceModules: [...defaultEnglishStartupModules]
      .filter((moduleId) => moduleId.startsWith("src/")).length,
    forbiddenModulePrefixes: baseline.defaultEnglishStartup.forbiddenModulePrefixes,
  },
  forbiddenEntryModules: baseline.forbiddenEntryModules,
}, null, 2));
