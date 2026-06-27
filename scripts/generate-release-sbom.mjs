#!/usr/bin/env node
/**
 * Generate release supply-chain evidence for SDKWork Memory.
 * Writes SPDX-style SBOM JSON and SHA-256 checksums for release binaries.
 */
import { createHash } from "node:crypto";
import { execSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const outDir = join(root, "deployments", "artifacts");
mkdirSync(outDir, { recursive: true });

const metadata = JSON.parse(
  execSync("cargo metadata --format-version=1 --no-deps", {
    cwd: root,
    encoding: "utf8",
  }),
);

const packages = metadata.packages.map((pkg) => ({
  name: pkg.name,
  version: pkg.version,
  license: pkg.license ?? null,
  source: pkg.source ?? null,
}));

const sbom = {
  spdxVersion: "SPDX-2.3",
  dataLicense: "CC0-1.0",
  SPDXID: "SPDXRef-DOCUMENT",
  name: "sdkwork-memory-sbom",
  documentNamespace: "https://sdkwork.com/apps/sdkwork-memory/sbom",
  creationInfo: {
    created: new Date().toISOString(),
    creators: ["Tool: scripts/generate-release-sbom.mjs"],
  },
  packages: packages.map((pkg, index) => ({
    ...pkg,
    SPDXID: `SPDXRef-Package-${index + 1}`,
  })),
};

const sbomPath = join(outDir, "sbom.spdx.json");
writeFileSync(sbomPath, `${JSON.stringify(sbom, null, 2)}\n`, "utf8");

const binaryPath = join(root, "target", "release", "sdkwork-memory-standalone-gateway.exe");
const unixReleaseBinary = join(root, "target", "release", "sdkwork-memory-standalone-gateway");
const unixDebugBinary = join(root, "target", "debug", "sdkwork-memory-standalone-gateway");
const winDebugBinary = join(root, "target", "debug", "sdkwork-memory-standalone-gateway.exe");
let checksumSource = null;
let checksumPathLabel = "target/release/sdkwork-memory-standalone-gateway";
for (const candidate of [
  binaryPath,
  unixReleaseBinary,
  winDebugBinary,
  unixDebugBinary,
]) {
  try {
    checksumSource = readFileSync(candidate);
    checksumPathLabel = candidate.replace(`${root}\\`, "").replace(`${root}/`, "");
    break;
  } catch {
    // try next candidate
  }
}
if (!checksumSource) {
  throw new Error(
    "No sdkwork-memory-standalone-gateway binary found; run `cargo build -p sdkwork-memory-standalone-gateway` first",
  );
}

const digest = createHash("sha256").update(checksumSource).digest("hex");
const checksums = {
  generatedAt: new Date().toISOString(),
  artifacts: [
    {
      path: checksumPathLabel,
      algorithm: "SHA-256",
      digest,
    },
  ],
};

const checksumPath = join(outDir, "checksums.json");
writeFileSync(checksumPath, `${JSON.stringify(checksums, null, 2)}\n`, "utf8");

syncReleaseChecksumToAppManifest(digest);

console.log(`Wrote ${sbomPath}`);
console.log(`Wrote ${checksumPath}`);
console.log(`Synced SHA-256 checksum to sdkwork.app.config.json`);

function syncReleaseChecksumToAppManifest(digest) {
  const manifestPath = join(root, "sdkwork.app.config.json");
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  const packages = manifest.artifacts?.installConfig?.packages ?? [];
  let updated = false;
  for (const pkg of packages) {
    if (pkg.id !== "container-x64-server-docker-image" || pkg.enabled === false) {
      continue;
    }
    pkg.checksumAlgorithm = pkg.checksumAlgorithm ?? "SHA-256";
    pkg.checksum = digest;
    updated = true;
  }
  if (!updated) {
    throw new Error(
      "sdkwork.app.config.json is missing enabled container-x64-server-docker-image package for checksum sync",
    );
  }
  writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
}
