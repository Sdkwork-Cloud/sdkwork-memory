#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const targetId = String(process.env.SDKWORK_PACKAGE_TARGET_ID ?? "").trim();

const commands = {
  "linux-x64-standalone-server-tar-gz": [
    "node",
    ["scripts/package-release-artifact.mjs"],
  ],
  "container-x64-cloud-container-oci": [
    "node",
    ["scripts/package-container-oci.mjs"],
  ],
  "web-universal-cloud-browser-zip": [
    "pnpm",
    ["--dir", "apps/sdkwork-memory-pc", "build:browser:cloud"],
  ],
};

const selected = commands[targetId];
if (!selected) {
  throw new Error(`Unsupported SDKWORK_PACKAGE_TARGET_ID: ${targetId || "<empty>"}`);
}

execFileSync(selected[0], selected[1], {
  cwd: root,
  env: process.env,
  stdio: "inherit",
  shell: process.platform === "win32" && selected[0] === "pnpm",
});
