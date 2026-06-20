#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, "..");
const sdkgen = path.resolve(workspaceRoot, "../sdkwork-sdk-generator/bin/sdkgen.js");

const families = [
  {
    input: "sdks/sdkwork-memory-sdk/openapi/memory-open-api.openapi.json",
    output: "sdks/sdkwork-memory-sdk/sdkwork-memory-sdk-typescript/generated/server-openapi",
    name: "sdkwork-memory-sdk",
    type: "custom",
    packageName: "@sdkwork/memory-sdk",
    apiPrefix: "/mem/v3/api",
    clientName: "SdkworkMemoryOpenClient",
  },
  {
    input: "sdks/sdkwork-memory-app-sdk/openapi/memory-app-api.openapi.json",
    output: "sdks/sdkwork-memory-app-sdk/sdkwork-memory-app-sdk-typescript/generated/server-openapi",
    name: "sdkwork-memory-app-sdk",
    type: "app",
    packageName: "@sdkwork/memory-app-sdk",
    apiPrefix: "/app/v3/api",
    clientName: "SdkworkMemoryAppClient",
  },
  {
    input: "sdks/sdkwork-memory-backend-sdk/openapi/memory-backend-api.openapi.json",
    output: "sdks/sdkwork-memory-backend-sdk/sdkwork-memory-backend-sdk-typescript/generated/server-openapi",
    name: "sdkwork-memory-backend-sdk",
    type: "backend",
    packageName: "@sdkwork/memory-backend-sdk",
    apiPrefix: "/backend/v3/api",
    clientName: "SdkworkMemoryBackendClient",
  },
];

function runGenerate(family) {
  const args = [
    sdkgen,
    "generate",
    "-i",
    path.join(workspaceRoot, family.input),
    "-o",
    path.join(workspaceRoot, family.output),
    "-n",
    family.name,
    "-t",
    family.type,
    "-l",
    "typescript",
    "--package-name",
    family.packageName,
    "--api-prefix",
    family.apiPrefix,
    "--standard-profile",
    "sdkwork-v3",
    "--fixed-sdk-version",
    "0.1.0",
    "--client-name",
    family.clientName,
  ];

  const result = spawnSync("node", args, { stdio: "inherit", cwd: workspaceRoot });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

for (const family of families) {
  console.log(`Generating TypeScript SDK for ${family.name}`);
  runGenerate(family);
}

console.log("Memory SDK generation completed.");
