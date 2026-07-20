import assert from "node:assert/strict";
import { generateKeyPairSync } from "node:crypto";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import test from "node:test";

import {
  createSbomProvenanceAndEvidence,
  signReleaseArtifact,
  verifyReleaseEvidence,
} from "./workflow-supply-chain-evidence.mjs";

const root = resolve(".runtime", "tests", "memory-release-evidence");
const artifactRelativePath = "dist/release/demo.bin";

test.beforeEach(() => {
  rmSync(root, { recursive: true, force: true });
  mkdirSync(dirname(resolve(root, artifactRelativePath)), { recursive: true });
  writeFileSync(resolve(root, artifactRelativePath), "immutable memory artifact");
});

test.after(() => rmSync(root, { recursive: true, force: true }));

test("creates and verifies byte-bound signature, SBOM, provenance, and checksum evidence", async () => {
  const { privateKey } = generateKeyPairSync("ed25519");
  const env = {
    SDKWORK_PACKAGE_ARTIFACT_PATH: artifactRelativePath,
    SDKWORK_PACKAGE_ID: "demo-package",
    SDKWORK_PACKAGE_TARGET_ID: "demo-package",
    SDKWORK_PACKAGE_VERSION: "0.1.0",
    SDKWORK_RUNTIME_TARGET: "server",
    SDKWORK_DEPLOYMENT_PROFILE: "standalone",
    SDKWORK_RELEASE_SIGNING_PRIVATE_KEY: privateKey
      .export({ format: "pem", type: "pkcs8" })
      .toString(),
  };
  await signReleaseArtifact({ env, root });
  await createSbomProvenanceAndEvidence({
    env,
    root,
    sourceCommit: "a".repeat(40),
    evidenceWriter: () => {},
    packageInventory: () => [],
  });
  const verified = await verifyReleaseEvidence({
    env,
    root,
    sourceCommit: "a".repeat(40),
  });
  assert.match(verified.digest, /^sha256:[a-f0-9]{64}$/u);
});

test("refuses to create signing evidence without real key material", async () => {
  await assert.rejects(
    signReleaseArtifact({
      env: {
        SDKWORK_PACKAGE_ARTIFACT_PATH: artifactRelativePath,
        SDKWORK_PACKAGE_ID: "demo-package",
      },
      root,
    }),
    /exactly one real release signing private key source/u,
  );
});
