#!/usr/bin/env node
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, "..");
const checkOnly = process.argv.includes("--check");

const owner = "sdkwork-memory";
const standardVersion = "2026-06-10";

const families = [
  {
    root: "sdks/sdkwork-memory-sdk",
    authority: "sdkwork-memory-open-api",
    input: "openapi/memory-open-api.openapi.json",
    packageName: "@sdkwork/memory-sdk",
    apiPrefix: "/mem/v3/api",
    clientName: "SdkworkMemoryOpenClient",
    forbiddenPathPrefixes: ["/app/v3/api/", "/backend/v3/api/"],
  },
  {
    root: "sdks/sdkwork-memory-app-sdk",
    authority: "sdkwork-memory.app",
    input: "openapi/memory-app-api.openapi.json",
    packageName: "@sdkwork/memory-app-sdk",
    apiPrefix: "/app/v3/api",
    clientName: "SdkworkMemoryAppClient",
    forbiddenPathPrefixes: ["/backend/v3/api/", "/mem/v3/api/"],
  },
  {
    root: "sdks/sdkwork-memory-backend-sdk",
    authority: "sdkwork-memory.backend",
    input: "openapi/memory-backend-api.openapi.json",
    packageName: "@sdkwork/memory-backend-sdk",
    apiPrefix: "/backend/v3/api",
    clientName: "SdkworkMemoryBackendClient",
    forbiddenPathPrefixes: ["/app/v3/api/", "/mem/v3/api/"],
  },
];

function readJson(relativePath) {
  return JSON.parse(readFileSync(path.join(workspaceRoot, relativePath), "utf8"));
}

const failures = [];

for (const family of families) {
  const assembly = readJson(path.join(family.root, ".sdkwork-assembly.json"));
  const manifest = readJson(path.join(family.root, "sdk-manifest.json"));
  const component = readJson(path.join(family.root, "specs/component.spec.json"));

  if (assembly.sdkOwner !== owner) {
    failures.push(`${family.root} assembly sdkOwner must be ${owner}`);
  }
  if (manifest.sdkOwner !== owner) {
    failures.push(`${family.root} manifest sdkOwner must be ${owner}`);
  }
  if (assembly.apiAuthority !== family.authority || manifest.apiAuthority !== family.authority) {
    failures.push(`${family.root} apiAuthority mismatch`);
  }
  if (assembly.generationInputSpec !== family.input || manifest.generationInputSpec !== family.input) {
    failures.push(`${family.root} generationInputSpec mismatch`);
  }
  if (manifest.standardProfile !== "sdkwork-v3") {
    failures.push(`${family.root} must declare standardProfile sdkwork-v3`);
  }
  if (manifest.packageName !== family.packageName) {
    failures.push(`${family.root} packageName mismatch`);
  }
  if (!component.contracts.sdkClients.includes(family.clientName)) {
    failures.push(`${family.root} component spec must declare ${family.clientName}`);
  }

  const openapi = readJson(path.join(family.root, family.input));
  if (openapi["x-sdkwork-owner"] !== owner) {
    failures.push(`${family.root} OpenAPI x-sdkwork-owner mismatch`);
  }
  for (const [routePath, pathItem] of Object.entries(openapi.paths ?? {})) {
    for (const prefix of family.forbiddenPathPrefixes) {
      if (routePath.startsWith(prefix)) {
        failures.push(`${family.root} must not include dependency route ${routePath}`);
      }
    }
  }
}

if (failures.length > 0) {
  console.error(JSON.stringify({ ok: false, mode: checkOnly ? "check" : "validate", failures }, null, 2));
  process.exit(1);
}

console.log(
  JSON.stringify({ ok: true, mode: checkOnly ? "check" : "validate", owner, standardVersion, families: families.length }, null, 2),
);
