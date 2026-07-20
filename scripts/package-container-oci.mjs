#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { mkdirSync, rmSync, statSync } from "node:fs";
import { dirname, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const version = required(process.env.SDKWORK_PACKAGE_VERSION, "SDKWORK_PACKAGE_VERSION");
const sourceRevision = gitValue(["rev-parse", "HEAD"]);
const artifactPath = resolve(
  root,
  process.env.SDKWORK_PACKAGE_ARTIFACT_PATH
    ?? "deployments/artifacts/release/sdkwork-memory-container-x64-cloud.oci.tar",
);

mkdirSync(dirname(artifactPath), { recursive: true });
rmSync(artifactPath, { force: true });

execFileSync(
  "docker",
  [
    "buildx",
    "build",
    "--file",
    "deployments/docker/Dockerfile",
    "--platform",
    "linux/amd64",
    "--build-arg",
    `SDKWORK_SOURCE_REVISION=${sourceRevision}`,
    "--build-arg",
    `SDKWORK_RELEASE_VERSION=${version}`,
    "--tag",
    `registry.sdkwork.com/apps/sdkwork-memory:${version}`,
    "--provenance=mode=max",
    "--sbom=true",
    "--output",
    `type=oci,dest=${artifactPath}`,
    ".",
  ],
  { cwd: root, stdio: "inherit" },
);

const sizeBytes = statSync(artifactPath).size;
if (sizeBytes === 0) {
  throw new Error(`OCI archive is empty: ${artifactPath}`);
}

const entries = execFileSync("tar", ["-tf", artifactPath], {
  cwd: root,
  encoding: "utf8",
}).split(/\r?\n/u);
for (const requiredEntry of ["oci-layout", "index.json"]) {
  if (!entries.includes(requiredEntry)) {
    throw new Error(`OCI archive is missing ${requiredEntry}`);
  }
}
if (!entries.some((entry) => entry.startsWith("blobs/sha256/"))) {
  throw new Error("OCI archive contains no content-addressed blobs");
}

console.log(JSON.stringify({ artifactPath, sizeBytes, sourceRevision, version }, null, 2));

function required(value, label) {
  const text = String(value ?? "").trim();
  if (!text) throw new Error(`${label} is required`);
  return text;
}

function gitValue(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
}
