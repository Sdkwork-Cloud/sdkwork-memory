#!/usr/bin/env node
/**
 * Release and supply-chain readiness gate for SDKWork Memory.
 * Follows RELEASE_SPEC.md and OBSERVABILITY_SPEC.md production gates.
 *
 * Default mode reports placeholder gaps as warnings so development can continue.
 * Set SDKWORK_RELEASE_VALIDATION=strict to fail on production-blocking manifest gaps.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const strict = process.env.SDKWORK_RELEASE_VALIDATION !== "development";
const failures = [];
const warnings = [];

function readJson(relativePath) {
  const absolutePath = path.join(repoRoot, relativePath);
  if (!fs.existsSync(absolutePath)) {
    failures.push(`${relativePath} must exist`);
    return null;
  }
  return JSON.parse(fs.readFileSync(absolutePath, "utf8"));
}

function fail(message) {
  failures.push(message);
}

function strictFail(message) {
  if (strict) {
    failures.push(message);
  } else {
    warnings.push(message);
  }
}

function isPlaceholderChecksum(checksum) {
  if (checksum == null) {
    return true;
  }
  if (typeof checksum !== "string" || checksum.length < 16) {
    return true;
  }
  const sample = checksum.slice(0, 8);
  return checksum === sample.repeat(Math.ceil(checksum.length / sample.length)).slice(0, checksum.length);
}

function collectPlaceholderMedia(manifest) {
  const placeholders = [];
  const media = manifest.media ?? {};
  for (const icon of [media.icons?.primary, ...(media.icons?.platform ?? [])].filter(Boolean)) {
    if (icon.metadata?.generatedPlaceholder === true) {
      placeholders.push(`media icon ${icon.id ?? icon.purpose ?? "unknown"}`);
    }
  }
  for (const preview of media.previews ?? []) {
    if (preview.metadata?.generatedPlaceholder === true) {
      placeholders.push(`preview ${preview.id ?? preview.purpose ?? "unknown"}`);
    }
  }
  return placeholders;
}

function main() {
  const manifest = readJson("sdkwork.app.config.json");
  if (!manifest) {
    reportAndExit();
  }

  if (manifest.schemaVersion !== 3 || manifest.kind !== "sdkwork.app") {
    fail("sdkwork.app.config.json must use schemaVersion 3 and kind sdkwork.app");
  }

  if (!fs.existsSync(path.join(repoRoot, "sdkwork.workflow.json"))) {
    fail("sdkwork.workflow.json must exist for release governance");
  }

  if (!fs.existsSync(path.join(repoRoot, "scripts/generate-release-sbom.mjs"))) {
    fail("scripts/generate-release-sbom.mjs must exist when security.sbomRequired is enabled");
  }

  if (!fs.existsSync(path.join(repoRoot, "scripts/package-release-artifact.mjs"))) {
    fail("scripts/package-release-artifact.mjs must exist for workflow package lifecycle");
  }

  const security = manifest.security ?? {};
  if (security.sbomRequired !== true) {
    strictFail("security.sbomRequired must be true for commercial release evidence");
  }
  if (security.checksumRequired !== true) {
    strictFail("security.checksumRequired must be true before production artifact publication");
  }
  if (security.signatureRequired !== true) {
    strictFail("security.signatureRequired must be true before externally distributed packages ship");
  }

  const contractState =
    manifest.artifacts?.installConfig?.metadata?.contractState ?? "unknown";
  if (contractState !== "production-ready") {
    strictFail(`artifacts.installConfig.metadata.contractState must be production-ready (found ${contractState})`);
  }

  const packages = manifest.artifacts?.installConfig?.packages ?? [];
  for (const pkg of packages) {
    if (pkg.enabled === false) {
      continue;
    }
    if (!pkg.checksumAlgorithm) {
      fail(`package ${pkg.id} must declare checksumAlgorithm`);
    }
    if (isPlaceholderChecksum(pkg.checksum)) {
      strictFail(
        `package ${pkg.id} must declare a release-build SHA-256 checksum (run pnpm release:sbom after release build)`,
      );
    }
  }

  for (const placeholder of collectPlaceholderMedia(manifest)) {
    strictFail(`${placeholder} is still a generated placeholder; replace with production media before catalog launch`);
  }

  if (manifest.publish?.status === "ACTIVE") {
    const hasPlaceholderMedia = collectPlaceholderMedia(manifest).length > 0;
    const hasPlaceholderChecksum = packages.some(
      (pkg) => pkg.enabled !== false && isPlaceholderChecksum(pkg.checksum),
    );
    if (hasPlaceholderMedia || hasPlaceholderChecksum) {
      strictFail("publish.status ACTIVE is incompatible with placeholder media or checksums");
    }
  }

  const workflow = readJson("sdkwork.workflow.json");
  if (workflow?.security?.sbomRequired !== true) {
    strictFail("sdkwork.workflow.json security.sbomRequired must be true");
  }

  reportAndExit();
}

function reportAndExit() {
  for (const message of warnings) {
    console.warn(`[release-readiness] warning: ${message}`);
  }
  if (failures.length > 0) {
    for (const message of failures) {
      console.error(`[release-readiness] error: ${message}`);
    }
    console.error(
      `[release-readiness] failed (${failures.length} error(s), ${warnings.length} warning(s))`,
    );
    process.exit(1);
  }
  const mode = strict ? "strict" : "development";
  console.log(`[release-readiness] passed (${mode} mode, ${warnings.length} warning(s))`);
}

main();
